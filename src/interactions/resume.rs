use std::sync::Arc;
use twilight_lavalink::model::Pause;
use twilight_model::{
    application::{
        command::{Command, CommandType},
        interaction::Interaction,
    },
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{command::CommandBuilder, InteractionResponseDataBuilder};

use crate::interactions::errors::NoAuthorFound;
use crate::{context::Context, interactions::errors::InvalidGuildId};

pub const NAME: &str = "resume";

pub fn command() -> Command {
    CommandBuilder::new("resume", "Resume the current track", CommandType::ChatInput).build()
}

pub async fn run(
    interaction: &Interaction,
    ctx: Arc<Context>,
) -> anyhow::Result<InteractionResponse> {
    let guild_id = interaction.guild_id.ok_or(InvalidGuildId {})?;

    let author = interaction.author().ok_or(NoAuthorFound {})?;

    tracing::info!("Resume command by {}", author.name);

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

    if !player.paused() {
        Ok(InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(
                InteractionResponseDataBuilder::new()
                    .content(format!("Not paused"))
                    .build(),
            ),
        })
    } else {
        player.send(Pause::from((guild_id, false)))?;
        Ok(InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(
                InteractionResponseDataBuilder::new()
                    .content(format!("Resumed track"))
                    .build(),
            ),
        })
    }
}
