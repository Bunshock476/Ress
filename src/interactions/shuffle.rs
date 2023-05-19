use crate::context::Context;
use std::sync::Arc;
use twilight_gateway::ShardId;
use twilight_model::application::{
    command::{Command, CommandType},
    interaction::Interaction,
};
use twilight_util::builder::command::CommandBuilder;

pub const NAME: &str = "shuffle";

pub fn command() -> Command {
    CommandBuilder::new("shuffle", "Shuffle the queue", CommandType::ChatInput).build()
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

    tracing::debug!("Shuffle command by {}", author.name);

    let bot_id = ctx.http_client.current_user().await?.model().await?.id;
    match ctx.cache.voice_state(bot_id, guild_id) {
        Some(vc) => vc,
        None => {
            return ctx
                .send_message_response(interaction, "Im not in a voice channel")
                .await;
        }
    };

    let queue_arc = match ctx.get_queue(guild_id) {
        Some(arc) => arc,
        None => {
            return ctx
                .send_message_response(interaction, "No tracks queued")
                .await;
        }
    };

    // Workaround to not await while holding a lock to queue
    let mut empty_queue = false;
    {
        let queue = queue_arc.lock().unwrap();
        if !queue.is_empty() {
            queue.shuffle();
        } else {
            empty_queue = true;
        }
    }

    if empty_queue {
        ctx.send_message_response(interaction, "The queue is empty")
            .await
    } else {
        ctx.send_message_response(interaction, "Shuffled current queue")
            .await
    }
}
