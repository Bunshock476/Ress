use std::sync::Arc;
use twilight_model::{
    application::{
        command::{Command, CommandOption, CommandOptionType, CommandType},
        interaction::{application_command::CommandOptionValue, Interaction, InteractionData},
    },
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{
    command::CommandBuilder,
    embed::{EmbedBuilder, EmbedFieldBuilder, EmbedFooterBuilder},
    InteractionResponseDataBuilder,
};

use crate::{context::Context, utils::from_millis_to_minutes};

pub const NAME: &str = "queue";

pub fn command() -> Command {
    CommandBuilder::new("queue", "Shows the current queue", CommandType::ChatInput)
        .option(CommandOption {
            autocomplete: Some(false),
            channel_types: None,
            choices: None,
            description: "Page to look".to_owned(),
            description_localizations: None,
            kind: CommandOptionType::Integer,
            max_length: None,
            max_value: None,
            min_length: None,
            min_value: Some(twilight_model::application::command::CommandOptionValue::Integer(1)),
            name: "page".to_owned(),
            name_localizations: None,
            options: None,
            required: Some(false),
        })
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

    // Parse option
    let options = {
        if let Some(InteractionData::ApplicationCommand(data)) = &interaction.data {
            &data.options
        } else {
            unreachable!()
        }
    };

    let page = if options.is_empty() {
        1
    } else {
        if let CommandOptionValue::Integer(i) = options[0].value {
            i as usize
        } else {
            1
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

    let queue = queue_arc.lock().unwrap().current_queue();

    let max_tracks_per_page = 10;

    let num_pages = (queue.len() as f32 / max_tracks_per_page as f32).ceil() as usize;

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
    if page <= 0 || page > num_pages {
        return Ok(InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(
                InteractionResponseDataBuilder::new()
                    .content(format!(
                        "Page out of bounds, use a value between 1 and {}",
                        num_pages
                    ))
                    .build(),
            ),
        });
    }

    let mut embed_builder = EmbedBuilder::new()
        .title("Upcoming tracks")
        .color(0xe04f2e)
        .footer(EmbedFooterBuilder::new(format!(
            "Page {} out of {}",
            page, num_pages
        )));

    if queue.len() > max_tracks_per_page {
        let begin = (page - 1) * max_tracks_per_page;
        let end = begin + max_tracks_per_page;

        let tracks_to_show = if end < queue.len() {
            &queue[begin..end]
        } else {
            &queue[begin..queue.len()]
        };

        for track in tracks_to_show {
            let duration = from_millis_to_minutes(track.info().length);
            let index = queue.iter().position(|t| t == track).unwrap_or(0) + 1;
            embed_builder = embed_builder.field(
                EmbedFieldBuilder::new(
                    "\u{200b}",
                    format!(
                        "**{}: {} - {}**",
                        index,
                        track.info().title.clone().unwrap_or("UNKNOWN".to_owned()),
                        duration
                    ),
                )
                .build(),
            );
        }
    } else {
        for track in &queue {
            let duration = from_millis_to_minutes(track.info().length);
            let index = queue.iter().position(|t| t == track).unwrap_or(0) + 1;
            embed_builder = embed_builder.field(
                EmbedFieldBuilder::new(
                    "\u{200b}",
                    format!(
                        "**{}: {} - {}**",
                        index,
                        track.info().title.clone().unwrap_or("UNKNOWN".to_owned()),
                        duration
                    ),
                )
                .build(),
            );
        }
    }

    Ok(InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(
            InteractionResponseDataBuilder::new()
                .embeds(vec![embed_builder.build()])
                .build(),
        ),
    })
}
