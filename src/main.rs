use std::{net::SocketAddr, str::FromStr, sync::Arc};

use futures::StreamExt;
use twilight_gateway::{
    stream::{self, ShardEventStream},
    Config, Event, Intents, ShardId,
};
use twilight_model::application::interaction::{Interaction, InteractionData, InteractionType};

mod context;
mod interactions;

use context::Context;

// TODO: remove the dependencie on anyhow
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize log tracer
    tracing_subscriber::fmt::init();

    // Load environment variables from .env file
    dotenv::dotenv().expect(".env file");

    // Initialize configurations
    let token = std::env::var("DISCORD_TOKEN")?;
    let intents = Intents::GUILD_MESSAGES
        | Intents::MESSAGE_CONTENT
        | Intents::GUILDS
        | Intents::GUILD_VOICE_STATES;

    let config = Config::new(token.clone(), intents);

    // Bot context for sharing data across tasks and accessing twilight clients and general setup
    let ctx = Arc::new(Context::new(token).await?);

    // Setup lavalink variables to connect to the node
    let lavalink_host = SocketAddr::from_str(&std::env::var("LAVALINK_HOST")?)?;
    let lavalink_secret = std::env::var("LAVALINK_SECRET")?;

    // Connects and adds a node to the lavalink client
    // The handle to the node is not used, but the events are used to check for the TrackEnd event
    // Used in the tracks queue
    let (_node, _lavalink_events) = ctx.lavalink.add(lavalink_host, lavalink_secret).await?;

    // Initialize the bot slash commands
    ctx.setup_commands().await?;

    // Initialize shards (currently spawning discord's recommended number of shards, could be only one for small bots)
    let mut shards =
        stream::create_recommended(&ctx.http_client, config, |_, builder| builder.build())
            .await?
            .collect::<Vec<_>>();

    for shard in &shards {
        ctx.add_shard_message_sender(shard.id(), shard.sender());
    }

    // Stream of shard events
    let mut stream = ShardEventStream::new(shards.iter_mut());

    // Initialize the loop to handle events
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

        tokio::spawn(handle_shard_stream_event(event, ctx.clone(), shard.id()));
    }

    Ok(())
}

async fn handle_shard_stream_event(
    event: Event,
    ctx: Arc<Context>,
    shard_id: ShardId,
) -> anyhow::Result<()> {
    tracing::info!("Shard {shard_id}, Event: {:?}", event.kind());

    match event {
        Event::Ready(_) => tracing::info!("Connected on shard {shard_id}"),
        Event::InteractionCreate(interaction) => {
            handle_interaction(ctx.clone(), interaction.0, shard_id).await?
        }
        _ => {}
    }

    Ok(())
}

async fn handle_interaction(
    ctx: Arc<Context>,
    interaction: Interaction,
    shard_id: ShardId,
) -> anyhow::Result<()> {
    match interaction.kind {
        InteractionType::ApplicationCommand => {
            if let Some(interaction_data) = &interaction.data {
                match interaction_data {
                    InteractionData::ApplicationCommand(command_data) => {
                        // Run the command and etrieve a response from the command caller
                        let response = match command_data.name.as_str() {
                            interactions::hello_test::NAME => {
                                interactions::hello_test::run(&interaction).await?
                            }
                            interactions::join::NAME => {
                                interactions::join::run(&interaction, ctx.clone(), shard_id).await?
                            }
                            interactions::leave::NAME => {
                                interactions::leave::run(&interaction, ctx.clone(), shard_id)
                                    .await?
                            }
                            _ => todo!("Custom error for non-existent commands"),
                        };

                        ctx.interaction_client()
                            .await?
                            .create_response(interaction.id, &interaction.token, &response)
                            .await?;
                    }
                    _ => todo!("Handle other interaction types"),
                }
            }
        }
        _ => {}
    }

    Ok(())
}
