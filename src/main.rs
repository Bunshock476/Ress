use std::{net::SocketAddr, str::FromStr, sync::Arc};

use futures::StreamExt;
use twilight_gateway::{
    stream::{self, ShardEventStream},
    Config, Event, Intents, ShardId,
};
use twilight_lavalink::{
    model::{IncomingEvent, Play, Stop},
    node::IncomingEvents,
};
use twilight_model::{
    application::interaction::{Interaction, InteractionData, InteractionType},
    id::{marker::ChannelMarker, Id},
};

mod context;
mod interactions;
mod queue;
mod track;
mod utils;

use context::Context;
use twilight_util::builder::embed::EmbedBuilder;

use crate::queue::TracksQueueError;

// TODO: remove the dependencie on anyhow
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().expect(".env file");

    // Initialize log tracer
    tracing_subscriber::fmt::init();

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
    let (_node, lavalink_events) = ctx.lavalink.add(lavalink_host, lavalink_secret).await?;

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

    // Separate loop for lavalink events
    tokio::spawn(handle_lavalink_events(lavalink_events, ctx.clone()));

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
                        // Run the command and retrieve a response from the command caller
                        match command_data.name.as_str() {
                            interactions::hello_test::NAME => {
                                interactions::hello_test::run(&interaction, ctx.clone(), shard_id)
                                    .await?;
                            }
                            interactions::join::NAME => {
                                interactions::join::run(&interaction, ctx.clone(), shard_id)
                                    .await?;
                            }
                            interactions::leave::NAME => {
                                interactions::leave::run(&interaction, ctx.clone(), shard_id)
                                    .await?;
                            }
                            interactions::play::NAME => {
                                interactions::play::run(&interaction, ctx.clone(), shard_id)
                                    .await?;
                            }
                            interactions::pause::NAME => {
                                interactions::pause::run(&interaction, ctx.clone(), shard_id)
                                    .await?;
                            }
                            interactions::resume::NAME => {
                                interactions::resume::run(&interaction, ctx.clone(), shard_id)
                                    .await?;
                            }
                            interactions::stop::NAME => {
                                interactions::stop::run(&interaction, ctx.clone(), shard_id)
                                    .await?;
                            }
                            interactions::skip::NAME => {
                                interactions::skip::run(&interaction, ctx.clone(), shard_id)
                                    .await?;
                            }
                            interactions::shuffle::NAME => {
                                interactions::shuffle::run(&interaction, ctx.clone(), shard_id)
                                    .await?;
                            }
                            interactions::queue::NAME => {
                                interactions::queue::run(&interaction, ctx.clone(), shard_id)
                                    .await?;
                            }
                            interactions::now_playing::NAME => {
                                interactions::now_playing::run(&interaction, ctx.clone(), shard_id)
                                    .await?;
                            }
                            _ => todo!("Custom error for non-existent commands"),
                        };
                    }
                    _ => todo!("Handle other interaction types"),
                }
            }
        }
        _ => todo!("Handle other interaction types"),
    }

    Ok(())
}

async fn handle_lavalink_events(
    mut events: IncomingEvents,
    ctx: Arc<Context>,
) -> anyhow::Result<()> {
    while let Some(event) = events.next().await {
        match event {
            IncomingEvent::TrackEnd(e) => {
                tracing::info!("Track end");
                let player = ctx.lavalink.player(e.guild_id).await?;
                let channel_id: Id<ChannelMarker>;
                let mut embed_builder = EmbedBuilder::new().color(0xe04f2e);
                {
                    let queue_arc = ctx
                        .get_queue(e.guild_id)
                        .ok_or(TracksQueueError::NoQueueFound(e.guild_id))?;
                    let queue = queue_arc.lock().unwrap();

                    // Last track in queue played
                    if queue.len() == 1 {
                        channel_id = queue.peek()?.channel_id;
                        player.send(Stop::from(e.guild_id))?;
                        queue.pop()?;
                        embed_builder = embed_builder.title("End of queue".to_owned());
                    } else {
                        queue.pop()?;
                        let track = queue.peek()?;
                        player.send(Play::from((e.guild_id, &track.track())))?;
                        continue;
                    }
                }

                ctx.http_client
                    .create_message(channel_id)
                    .embeds(&vec![embed_builder.build()])?
                    .await?;
            }
            IncomingEvent::TrackStart(start) => {
                tracing::info!("Track start");
                let mut embed_builder = EmbedBuilder::new().color(0xe04f2e);
                let channel_id: Id<ChannelMarker>;
                {
                    let queue_arc = ctx
                        .get_queue(start.guild_id)
                        .ok_or(TracksQueueError::NoQueueFound(start.guild_id))?;
                    let queue = queue_arc.lock().unwrap();

                    let track = queue.peek()?;
                    channel_id = track.channel_id;

                    let title = track
                        .info()
                        .title
                        .clone()
                        .unwrap_or("<Unknown>".to_string());
                    let uri = &track.info().uri;
                    let author = track
                        .info()
                        .author
                        .clone()
                        .unwrap_or("<Unknown>".to_string());
                    embed_builder = embed_builder
                        .title("Now playing".to_owned())
                        .description(format!("**[{}]({})** \n By **{}**", title, uri, author))
                }

                ctx.http_client
                    .create_message(channel_id)
                    .embeds(&vec![embed_builder.build()])?
                    .await?;
            }
            _ => {}
        }
    }
    Ok(())
}
