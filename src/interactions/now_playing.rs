use std::sync::Arc;
use twilight_model::{
    application::{
        command::{Command, CommandType},
        interaction::Interaction,
    },
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{
    command::CommandBuilder,
    embed::{EmbedBuilder, EmbedFieldBuilder},
    InteractionResponseDataBuilder,
};

use crate::{context::Context, utils::from_millis_to_minutes};

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
) -> anyhow::Result<InteractionResponse> {
    tracing::debug!("Queue command by {}", interaction.author().unwrap().name);

    let guild_id = interaction.guild_id.expect("Valid guild id");

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

    let queue_arc = match ctx.get_queue(guild_id) {
        Some(arc) => arc,
        None => {
            return Ok(InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(
                    InteractionResponseDataBuilder::new()
                        .content(format!("No tracks queued"))
                        .build(),
                ),
            })
        }
    };

    let player = ctx.lavalink.player(guild_id).await?;

    let queue = queue_arc.lock().unwrap();

    if queue.is_empty() {
        return Ok(InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(
                InteractionResponseDataBuilder::new()
                    .content("The queue is empty")
                    .build(),
            ),
        });
    }

    let track = queue.peek()?;
    let title = track.info().title.clone().unwrap_or("<UNKNOWN>".to_owned());
    let duration = from_millis_to_minutes(track.info().length - player.position() as u64);
    let author = track
        .info()
        .author
        .clone()
        .unwrap_or("<UNKNOWN>".to_owned());

    let embed_builder = EmbedBuilder::new()
        .title("Now playing")
        .color(0xe04f2e)
        .field(EmbedFieldBuilder::new("\u{200b}", format!("**{} by {}**", title, author)).build())
        .description(format!("Remaining time: {}", duration));

    Ok(InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(
            InteractionResponseDataBuilder::new()
                .embeds(vec![embed_builder.build()])
                .build(),
        ),
    })
}
