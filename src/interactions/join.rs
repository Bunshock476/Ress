use std::sync::Arc;

use twilight_gateway::ShardId;
use twilight_model::{
    application::{
        command::{Command, CommandType},
        interaction::Interaction,
    },
    gateway::payload::outgoing::UpdateVoiceState,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{command::CommandBuilder, InteractionResponseDataBuilder};

use crate::interactions::errors::NoAuthorFound;
use crate::{
    context::Context,
    interactions::errors::{InvalidGuildId, NoMessageSenderForShard},
};

pub const NAME: &str = "join";

pub fn command() -> Command {
    CommandBuilder::new("join", "Join a voice channel", CommandType::ChatInput).build()
}

pub async fn run(
    interaction: &Interaction,
    ctx: Arc<Context>,
    shard_id: ShardId,
) -> anyhow::Result<InteractionResponse> {
    let guild_id = interaction.guild_id.ok_or(InvalidGuildId {})?;

    let author = interaction.author().ok_or(NoAuthorFound {})?;

    tracing::info!("Join command by {}", author.name);

    let vc = match ctx.cache.voice_state(author.id, guild_id) {
        Some(vc) => vc,
        None => {
            return Ok(InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(
                    InteractionResponseDataBuilder::new()
                        .content("You need to be in a voice channel to use this command")
                        .build(),
                ),
            });
        }
    };

    let channel_id = vc.channel_id();

    let sender = ctx
        .shard_senders
        .get(&shard_id)
        .ok_or(NoMessageSenderForShard { shard_id })?;

    sender.command(&UpdateVoiceState::new(guild_id, channel_id, false, false))?;

    Ok(InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(
            InteractionResponseDataBuilder::new()
                .content(format!("Joined <#{channel_id}>"))
                .build(),
        ),
    })
}