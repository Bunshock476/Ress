use std::sync::Arc;
use twilight_gateway::ShardId;
use twilight_lavalink::model::Pause;
use twilight_model::application::{
    command::{Command, CommandType},
    interaction::Interaction,
};
use twilight_util::builder::command::CommandBuilder;

use crate::interactions::errors::NoAuthorFound;
use crate::{context::Context, interactions::errors::InvalidGuildId};

pub const NAME: &str = "pause";

pub fn command() -> Command {
    CommandBuilder::new("pause", "Pause the current track", CommandType::ChatInput).build()
}

pub async fn run(
    interaction: &Interaction,
    ctx: Arc<Context>,
    _shard_id: ShardId,
) -> anyhow::Result<()> {
    let guild_id = interaction.guild_id.ok_or(InvalidGuildId {})?;

    let author = interaction.author().ok_or(NoAuthorFound {})?;

    tracing::info!("Pause command by {}", author.name);

    let bot_id = ctx.http_client.current_user().await?.model().await?.id;
    match ctx.cache.voice_state(bot_id, guild_id) {
        Some(vc) => vc,
        None => {
            return ctx
                .send_message_response(interaction, "Im not in a voice channel")
                .await;
        }
    };

    let player = ctx.lavalink.player(guild_id).await?;

    let content = if player.paused() {
        "Already paused".to_owned()
    } else {
        player.send(Pause::from((guild_id, true)))?;
        "Paused track".to_owned()
    };

    ctx.send_message_response(interaction, content).await
}
