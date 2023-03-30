use std::sync::Arc;
use twilight_lavalink::model::Stop;
use twilight_model::{
    application::{
        command::{Command, CommandType},
        interaction::Interaction,
    },
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{command::CommandBuilder, InteractionResponseDataBuilder};

use crate::{context::Context, queue::TracksQueueError};

pub const NAME: &str = "stop";

pub fn command() -> Command {
    CommandBuilder::new("stop", "Stop and clears the queue", CommandType::ChatInput).build()
}

pub async fn run(
    interaction: &Interaction,
    ctx: Arc<Context>,
) -> anyhow::Result<InteractionResponse> {
    tracing::info!("Stop command by {}", interaction.author().unwrap().name);

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
    player.send(Stop::from(guild_id))?;

    // Clear queue
    ctx.get_queue(guild_id)
        .ok_or(TracksQueueError::NoQueueFound(guild_id))?
        .lock()
        .unwrap()
        .clear();

    Ok(InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(
            InteractionResponseDataBuilder::new()
                .content(format!("Stopped current tracks"))
                .build(),
        ),
    })
}
