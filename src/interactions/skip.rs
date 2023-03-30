use std::sync::Arc;
use twilight_lavalink::model::{Stop};
use twilight_model::{
    application::{
        command::{Command, CommandType},
        interaction::Interaction,
    },
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{command::CommandBuilder, InteractionResponseDataBuilder};

use crate::context::Context;

pub const NAME: &str = "skip";

pub fn command() -> Command {
    CommandBuilder::new("skip", "Skips the current track", CommandType::ChatInput).build()
}

pub async fn run(
    interaction: &Interaction,
    ctx: Arc<Context>,
) -> anyhow::Result<InteractionResponse> {
    tracing::info!("Skip command by {}", interaction.author().unwrap().name);

    let guild_id = interaction.guild_id.expect("Valid guild id");

    let bot_id = ctx.http_client.current_user().await?.model().await?.id;
    match ctx.cache.voice_state(bot_id, guild_id) {
        Some(vc) => vc,
        None => {
            return Ok(InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(
                    InteractionResponseDataBuilder::new()
                        .content("Im not in a voice channel")
                        .build(),
                ),
            });
        }
    };

    let player = ctx.lavalink.player(guild_id).await?;

    let queue_arc = match ctx.get_queue(guild_id) {
        Some(arc) => arc,
        None => {
            return Ok(InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(
                    InteractionResponseDataBuilder::new()
                        .content(format!("No tracks queued"))
                        .build(),
                ),
            })
        }
    };
    {
        let queue = queue_arc.lock().unwrap();
        if queue.is_empty() {
            return Ok(InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(
                    InteractionResponseDataBuilder::new()
                        .content("No more tracks to skip")
                        .build(),
                ),
            });
        }

        player.send(Stop::from(guild_id))?;
    }

    Ok(InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(
            InteractionResponseDataBuilder::new()
                .content(format!("Skipped current track"))
                .build(),
        ),
    })
}
