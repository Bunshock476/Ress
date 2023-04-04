use std::sync::Arc;
use twilight_gateway::ShardId;
use twilight_model::application::{
    command::{Command, CommandType},
    interaction::Interaction,
};
use twilight_util::builder::{
    command::CommandBuilder,
    embed::{EmbedBuilder, EmbedFieldBuilder, EmbedFooterBuilder},
};

use crate::{
    context::Context,
    utils::{check_voice_state, from_ms_to_minutes},
};

pub const NAME: &str = "np";

pub fn command() -> Command {
    CommandBuilder::new(
        "np",
        "Shows the current playing track",
        CommandType::ChatInput,
    )
    .build()
}

pub async fn run(
    interaction: &Interaction,
    ctx: Arc<Context>,
    _shard_id: ShardId,
) -> anyhow::Result<()> {
    tracing::debug!(
        "Queue command by {}",
        interaction
            .author()
            .ok_or(anyhow::anyhow!("No author found"))?
            .name
    );

    let guild_id = interaction.guild_id.expect("Valid guild id");

    let bot_id = ctx.http_client.current_user().await?.model().await?.id;

    if !check_voice_state(ctx.clone(), bot_id, guild_id) {
        return ctx
            .send_message_response(interaction, "Im not in a voice channel")
            .await;
    }

    let queue_arc = match ctx.get_queue(guild_id) {
        Some(arc) => arc,
        None => {
            return ctx
                .send_message_response(interaction, "No tracks queued")
                .await;
        }
    };

    let player = ctx.lavalink.player(guild_id).await?;

    let mut embed_builder = EmbedBuilder::new().title("Now playing").color(0xe04f2e);

    let mut empty_queue = false;

    // Workaround to not await while holding a lock to queue
    {
        let queue = queue_arc.lock().unwrap();

        if !queue.is_empty() {
            let track = queue.peek()?;
            let title = track.info().title.clone().unwrap_or("<UNKNOWN>".to_owned());
            let duration = from_ms_to_minutes(track.info().length - player.position() as u64);
            let author = track
                .info()
                .author
                .clone()
                .unwrap_or("<UNKNOWN>".to_owned());

            embed_builder = embed_builder
                .field(
                    EmbedFieldBuilder::new("\u{200b}", format!("**{} by {}**", title, author))
                        .build(),
                )
                .footer(EmbedFooterBuilder::new(format!("Remaining time: {}", duration)).build());
        } else {
            empty_queue = true;
        }
    }

    if empty_queue {
        return ctx
            .send_message_response(interaction, "The queue is empty")
            .await;
    }

    ctx.send_embed_response(interaction, embed_builder.build())
        .await
}
