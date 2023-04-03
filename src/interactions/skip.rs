use std::sync::Arc;
use twilight_gateway::ShardId;
use twilight_lavalink::model::Stop;
use twilight_model::application::{
    command::{Command, CommandType},
    interaction::Interaction,
};
use twilight_util::builder::command::CommandBuilder;

use crate::context::Context;

pub const NAME: &str = "skip";

pub fn command() -> Command {
    CommandBuilder::new("skip", "Skips the current track", CommandType::ChatInput).build()
}

pub async fn run(
    interaction: &Interaction,
    ctx: Arc<Context>,
    _shard_id: ShardId,
) -> anyhow::Result<()> {
    tracing::info!("Skip command by {}", interaction.author().unwrap().name);

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

    let queue_arc = match ctx.get_queue(guild_id) {
        Some(arc) => arc,
        None => {
            return ctx
                .send_message_response(interaction, "No tracks queued")
                .await;
        }
    };

    // Workaraound to not await while holding a lock to queue
    let mut empty_queue = false;
    {
        let queue = queue_arc.lock().unwrap();
        if !queue.is_empty() {
            player.send(Stop::from(guild_id))?;
        } else {
            empty_queue = true;
        }
    }

    if empty_queue {
        ctx.send_message_response(interaction, "No more tracks to skip")
            .await
    } else {
        ctx.send_message_response(interaction, "Skipped current track")
            .await
    }
}
