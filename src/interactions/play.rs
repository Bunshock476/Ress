use std::sync::Arc;

use hyper::{Body, Request};
use twilight_lavalink::{
    http::{LoadType, LoadedTracks},
    model::Play,
};
use twilight_model::{
    application::{
        command::{Command, CommandOption, CommandOptionType, CommandType},
        interaction::{application_command::CommandOptionValue, Interaction, InteractionData},
    },
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{
    command::CommandBuilder, embed::EmbedBuilder, InteractionResponseDataBuilder,
};

use crate::interactions::errors::NoAuthorFound;
use crate::{context::Context, interactions::errors::InvalidGuildId};

pub const NAME: &str = "play";

pub fn command() -> Command {
    CommandBuilder::new(
        "play",
        "Play a track from link or search for it on youtube",
        CommandType::ChatInput,
    )
    .option(CommandOption {
        autocomplete: Some(false),
        channel_types: None,
        choices: None,
        description: "Link of track or search query to play".to_owned(),
        description_localizations: None,
        kind: CommandOptionType::String,
        max_length: None,
        max_value: None,
        min_length: None,
        min_value: None,
        name: "link-or-query".to_owned(),
        name_localizations: None,
        options: None,
        required: Some(true),
    })
    .build()
}

pub async fn run(
    interaction: &Interaction,
    ctx: Arc<Context>,
) -> anyhow::Result<InteractionResponse> {
    let guild_id = interaction.guild_id.ok_or(InvalidGuildId {})?;

    let author = interaction.author().ok_or(NoAuthorFound {})?;

    tracing::info!("Play command by {}", author.name);

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

    let options = {
        if let Some(InteractionData::ApplicationCommand(data)) = &interaction.data {
            &data.options
        } else {
            unreachable!()
        }
    };

    let q = match &options[0].value {
        CommandOptionValue::String(n) => n.clone(),
        _ => "".to_string(),
    };

    let query: String;

    if q.is_empty() {
        return Ok(InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(
                InteractionResponseDataBuilder::new()
                    .content("Error: query is empty")
                    .build(),
            ),
        });
    } else if q.starts_with("http") {
        query = q.to_string();
    } else {
        query = format!("ytsearch:{}", q);
    }

    let player = ctx.lavalink.player(guild_id).await?;

    let (parts, body) = twilight_lavalink::http::load_track(
        player.node().config().address,
        query,
        &player.node().config().authorization,
    )?
    .into_parts();

    let req = Request::from_parts(parts, Body::from(body));
    let res = ctx.hyper_client.request(req).await?;
    let res_bytes = hyper::body::to_bytes(res.into_body()).await?;

    let loaded = serde_json::from_slice::<LoadedTracks>(&res_bytes)?;

    let channel_id = interaction
        .channel_id
        .expect("Interaction from valid channel");

    let mut embed_builder = EmbedBuilder::new().color(0xe04f2e);

    match loaded.load_type {
        LoadType::LoadFailed => embed_builder = embed_builder.title("Failed to load track"),
        LoadType::NoMatches => {
            embed_builder = embed_builder.title("No results found");
        }
        LoadType::PlaylistLoaded => {
            let queue_arc = ctx.get_or_create_queue(guild_id);
            let queue = queue_arc.lock().unwrap();
            for track in loaded.tracks {
                queue.push(crate::track::Track::new(track.clone(), channel_id));
            }

            let first = queue.peek()?;

            embed_builder = embed_builder.title("Loaded playlist").description(format!(
                "**{}**",
                loaded.playlist_info.name.unwrap_or("<Unknown>".to_string())
            ));

            player.send(Play::from((guild_id, &first.track())))?;
        }
        LoadType::SearchResult | LoadType::TrackLoaded => {
            let track = loaded.tracks.first().unwrap();
            let title = track.info.title.clone().unwrap_or("<Unknown>".to_string());
            let author = track.info.author.clone().unwrap_or("<Unknown>".to_string());

            let queue_arc = ctx.get_or_create_queue(guild_id);
            let queue = queue_arc.lock().unwrap();
            queue.push(crate::track::Track::new(track.clone(), channel_id));

            embed_builder = embed_builder
                .title("Track queued")
                .description(format!("**{title}** by **{author}**"));
            player.send(Play::from((guild_id, &track.track)))?;
        }
        _ => todo!(),
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
