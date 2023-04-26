use std::sync::Arc;
use twilight_gateway::ShardId;
use twilight_lavalink::model::Stop;
use twilight_model::application::{
    command::{Command, CommandType},
    interaction::Interaction,
};
use twilight_util::builder::command::CommandBuilder;

use crate::context::Context;

pub const NAME: &str = "stop";

pub fn command() -> Command {
    CommandBuilder::new("stop", "Stop and clears the queue", CommandType::ChatInput).build()
}

pub async fn run(
    interaction: &Interaction,
    ctx: Arc<Context>,
    _shard_id: ShardId,
) -> anyhow::Result<()> {
    tracing::debug!(
        "Stop command by {}",
        interaction
            .author()
            .ok_or(anyhow::anyhow!("No author found"))?
            .name
    );

    let guild_id = interaction.guild_id.expect("Valid guild id");

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
    player.send(Stop::from(guild_id))?;
    
    // Clear queue
    ctx.get_queue(guild_id)
        .ok_or(anyhow::anyhow!("No queue found for guild id {}", guild_id))?
        .lock()
        .unwrap()
        .clear();

    ctx.send_message_response(interaction, "Stopped current queue")
        .await
}
