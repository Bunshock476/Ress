use crate::context::Context;
use std::sync::Arc;
use twilight_gateway::ShardId;
use twilight_model::{
    application::{
        command::{Command, CommandType},
        interaction::Interaction,
    },
    gateway::payload::outgoing::UpdateVoiceState,
};
use twilight_util::builder::command::CommandBuilder;

pub const NAME: &str = "join";

pub fn command() -> Command {
    CommandBuilder::new("join", "Join a voice channel", CommandType::ChatInput).build()
}

pub async fn run(
    interaction: &Interaction,
    ctx: Arc<Context>,
    shard_id: ShardId,
) -> anyhow::Result<()> {
    let guild_id = interaction
        .guild_id
        .ok_or(anyhow::anyhow!("Invalid guild id"))?;

    let author = interaction
        .author()
        .ok_or(anyhow::anyhow!("No author found"))?;

    tracing::debug!("Join command by {}", author.name);

    let voice_channel = match ctx.cache.voice_state(author.id, guild_id) {
        Some(vc) => vc,
        None => {
            return ctx
                .send_message_response(
                    interaction,
                    "You need to be in a voice channel to use this command",
                )
                .await;
        }
    };

    let channel_id = voice_channel.channel_id();

    let sender = ctx.shard_senders.get(&shard_id).ok_or(anyhow::anyhow!(
        "No message sender for shard id {}",
        shard_id
    ))?;

    sender.command(&UpdateVoiceState::new(guild_id, channel_id, false, false))?;

    ctx.send_message_response(interaction, format!("Joined <#{}>", channel_id))
        .await
}
