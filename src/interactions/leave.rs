use std::sync::Arc;

use twilight_gateway::ShardId;
use twilight_lavalink::model::Destroy;
use twilight_model::{
    application::{
        command::{Command, CommandType},
        interaction::Interaction,
    },
    gateway::payload::outgoing::UpdateVoiceState,
};
use twilight_util::builder::command::CommandBuilder;

use crate::context::Context;

pub const NAME: &str = "leave";

pub fn command() -> Command {
    CommandBuilder::new("leave", "Leave a voice channel", CommandType::ChatInput).build()
}

pub async fn run(
    interaction: &Interaction,
    ctx: Arc<Context>,
    shard_id: ShardId,
) -> anyhow::Result<()> {
    tracing::debug!(
        "Leave command by {}",
        interaction
            .author()
            .ok_or(anyhow::anyhow!("No author found"))?
            .name
    );

    let guild_id = interaction.guild_id.expect("Valid guild id");

    tracing::debug!("Guild id: {}", guild_id);

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
    player.send(Destroy::from(guild_id))?;

    let sender = ctx.shard_senders.get(&shard_id).ok_or(anyhow::anyhow!(
        "No message sender for shard id {}",
        shard_id
    ))?;

    sender.command(&UpdateVoiceState::new(guild_id, None, false, false))?;

    // Clear queue
    ctx.get_queue(guild_id)
        .ok_or(anyhow::anyhow!("No queue found for guild id {}", guild_id))?
        .lock()
        .unwrap()
        .clear();

    ctx.send_message_response(interaction, "Left channel").await
}
