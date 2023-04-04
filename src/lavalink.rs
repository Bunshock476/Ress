use std::sync::Arc;

use futures::StreamExt;
use twilight_lavalink::{node::IncomingEvents, model::{Stop, Play, IncomingEvent}};
use twilight_model::id::{Id, marker::ChannelMarker};
use twilight_util::builder::embed::EmbedBuilder;

use crate::context::Context;

pub async fn handle_events(
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
                    let queue_arc = ctx.get_queue(e.guild_id).ok_or(anyhow::anyhow!(
                        "No queue found for guild id {}",
                        e.guild_id
                    ))?;
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