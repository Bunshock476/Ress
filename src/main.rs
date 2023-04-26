use std::{net::SocketAddr, str::FromStr, sync::Arc};

use futures::StreamExt;
use twilight_gateway::{
    stream::{self, ShardEventStream},
    Config, Event, Intents, ShardId,
};

mod context;
mod interactions;
mod lavalink;
mod queue;
mod track;
mod utils;

use context::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().expect(".env file");
    // Initialize log tracer
    tracing_subscriber::fmt::init();
    // Setup configurations
    let token = std::env::var("DISCORD_TOKEN")?;
    let intents = Intents::GUILD_MESSAGES
        | Intents::MESSAGE_CONTENT
        | Intents::GUILDS
        | Intents::GUILD_VOICE_STATES;

    let config = Config::new(token.clone(), intents);

    // Bot context for sharing data across tasks, accessing twilight clients and general setup
    let ctx = Arc::new(Context::new(token).await?);

    // Setup lavalink variables to connect to the node
    let lavalink_host = SocketAddr::from_str(&std::env::var("LAVALINK_HOST")?)?;
    let lavalink_secret = std::env::var("LAVALINK_SECRET")?;

    // Connects and adds a node to the lavalink client
    // The handle to the node is not used, but the events are used to check for TrackEnd and TrackStart events
    // Used in the tracks queue
    let (_node, lavalink_events) = ctx.lavalink.add(lavalink_host, lavalink_secret).await?;

    // Initialize the bot slash commands
    ctx.setup_commands().await?;

    // Initialize shards (currently spawning discord's recommended number of shards, could be only one for small bots)
    let mut shards =
        stream::create_recommended(&ctx.http_client, config, |_, builder| builder.build())
            .await?
            .collect::<Vec<_>>();

    // Add shard and it's message sender to a hashmap to allow access from across tasks
    for shard in &shards {
        ctx.add_shard_message_sender(shard.id(), shard.sender());
    }

    // Stream of shard events
    let mut stream = ShardEventStream::new(shards.iter_mut());

    // Separate loop for lavalink events
    tokio::spawn(lavalink::handle_events(lavalink_events, ctx.clone()));

    // Initialize the loop to handle shard events
    while let Some((shard, e)) = stream.next().await {
        let event = match e {
            Ok(ev) => ev,
            Err(err) => {
                tracing::error!("Failed to receive event. Error: {err}");

                if err.is_fatal() {
                    break;
                }

                continue;
            }
        };

        ctx.cache.update(&event);
        ctx.lavalink.process(&event).await?;

        // Spawn task to handle each shard event
        tokio::spawn(handle_shard_stream_event(event, ctx.clone(), shard.id()));
    }

    Ok(())
}

async fn handle_shard_stream_event(
    event: Event,
    ctx: Arc<Context>,
    shard_id: ShardId,
) -> anyhow::Result<()> {
    tracing::trace!("Shard {}, Event: {:?}", shard_id, event);

    match event {
        Event::Ready(r) => {
            tracing::info!(
                "Connected to shard id {} with {} guilds",
                shard_id,
                r.guilds.len()
            )
        }
        Event::InteractionCreate(interaction) => {
            interactions::handle_interaction(ctx.clone(), interaction.0, shard_id).await?
        }
        _ => {}
    }

    Ok(())
}
