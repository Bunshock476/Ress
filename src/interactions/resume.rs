use std::sync::Arc;
use twilight_gateway::ShardId;
use twilight_lavalink::model::Pause;
use twilight_model::application::{
    command::{Command, CommandType},
    interaction::Interaction,
};
use twilight_util::builder::command::CommandBuilder;

use crate::{context::Context, utils::check_voice_state};

pub const NAME: &str = "resume";

pub fn command() -> Command {
    CommandBuilder::new("resume", "Resume the current track", CommandType::ChatInput).build()
}

pub async fn run(
    interaction: &Interaction,
    ctx: Arc<Context>,
    _shard_id: ShardId,
) -> anyhow::Result<()> {
    let guild_id = interaction
        .guild_id
        .ok_or(anyhow::anyhow!("Invalid guild id"))?;

    let author = interaction
        .author()
        .ok_or(anyhow::anyhow!("No author found"))?;

    tracing::debug!("Resume command by {}", author.name);

    let bot_id = ctx.http_client.current_user().await?.model().await?.id;
    if !check_voice_state(ctx.clone(), bot_id, guild_id) {
        return ctx
            .send_message_response(interaction, "Im not in a voice channel")
            .await;
    }

    let player = ctx.lavalink.player(guild_id).await?;

    let content = if !player.paused() {
        "Not paused".to_owned()
    } else {
        player.send(Pause::from((guild_id, false)))?;
        "Resumed trakc".to_owned()
    };

    ctx.send_message_response(interaction, content).await
}
