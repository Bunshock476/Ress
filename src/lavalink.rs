use std::sync::Arc;

use futures::StreamExt;
use twilight_lavalink::{
    model::{IncomingEvent, Play, Stop},
    node::IncomingEvents,
};
use twilight_model::id::{marker::ChannelMarker, Id};
use twilight_util::builder::embed::EmbedBuilder;

use crate::{context::Context, queue::QueueLoopMode};

pub async fn handle_events(mut events: IncomingEvents, ctx: Arc<Context>) -> anyhow::Result<()> {
    
    while let Some(event) = events.next().await {
        match event {
            IncomingEvent::TrackEnd(e) => {
                tracing::debug!("Track end");
                let player = ctx.lavalink.player(e.guild_id).await?;
                let mut channel_id: Option<Id<ChannelMarker>> = None;
                let mut end_of_queue = false;
                {
                    let queue_arc = ctx.get_queue(e.guild_id).ok_or(anyhow::anyhow!(
                        "No queue found for guild id {}",
                        e.guild_id
                    ))?;
                    let queue = queue_arc.lock().unwrap();

                    let next_track = match queue.loop_mode {
                        QueueLoopMode::None => {
                            if queue.is_empty() {
                                None
                            } else if queue.len() == 1 {
                            // Last track in queue played
                                channel_id = Some(queue.peek()?.channel_id);
                                player.send(Stop::from(e.guild_id))?;
                                queue.pop()?;
                                end_of_queue = true;
                                None
                            } else {
                                queue.pop()?;
                                Some(queue.peek()?)
                            }
                        }
                        QueueLoopMode::LoopQueue => {
                            let current_track = queue.peek()?;
                            queue.push(current_track);
                            queue.pop()?;

                            Some(queue.peek()?)
                        }
                        QueueLoopMode::LoopTrack => Some(queue.peek()?),
                    };

                    if let Some(track) = next_track {
                        player.send(Play::from((e.guild_id, track.track())))?;
                    }
                }

                if end_of_queue {
                    tracing::debug!("End of queue");
                    if let Some(id) = channel_id {
                        ctx.http_client
                            .create_message(id)
                            .embeds(&vec![EmbedBuilder::new()
                                .color(0xe04f2e)
                                .title("End of queue")
                                .build()])?
                            .await?;
                    }
                }
            }
            IncomingEvent::TrackStart(start) => {
                tracing::debug!("Track start");
                let mut embed_builder = EmbedBuilder::new().color(0xe04f2e);
                let channel_id: Id<ChannelMarker>;
                {
                    let queue_arc = ctx.get_queue(start.guild_id).ok_or(anyhow::anyhow!(
                        "No queue found for guild id {}",
                        start.guild_id
                    ))?;
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
