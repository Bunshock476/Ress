use std::sync::Arc;

use hyper::{Body, Request};
use twilight_gateway::ShardId;
use twilight_lavalink::{
    http::{LoadType, LoadedTracks},
    model::Play,
};
use twilight_model::application::{
    command::{Command, CommandOption, CommandOptionType, CommandType},
    interaction::{application_command::CommandOptionValue, Interaction, InteractionData},
};
use twilight_util::builder::{command::CommandBuilder, embed::EmbedBuilder};

use crate::{context::Context, utils::check_voice_state};

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
    _shard_id: ShardId,
) -> anyhow::Result<()> {
    let guild_id = interaction
        .guild_id
        .ok_or(anyhow::anyhow!("Invalid guild id"))?;

    let author = interaction
        .author()
        .ok_or(anyhow::anyhow!("No author found"))?;

    tracing::debug!("Play command by {}", author.name);

    let bot_id = ctx.http_client.current_user().await?.model().await?.id;
    if !check_voice_state(ctx.clone(), bot_id, guild_id) {
        return ctx
            .send_message_response(interaction, "Im not in a voice channel")
            .await;
    }

    let options = {
        if let Some(InteractionData::ApplicationCommand(data)) = &interaction.data {
            &data.options
        } else {
            unreachable!()
        }
    };

    let q = match &options[0].value {
        CommandOptionValue::String(n) => n.clone(),
        _ => anyhow::bail!("Option value should have been a string"),
    };

    let query = if q.starts_with("http") {
        q.to_string()
    } else {
        format!("ytsearch:{}", q)
    };

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
        .ok_or(anyhow::anyhow!("Invalid channel id"))?;

    let mut embed_builder = EmbedBuilder::new().color(0xe04f2e);

    match loaded.load_type {
        LoadType::LoadFailed => {
            return ctx
                .send_message_response(interaction, "Failed to load track")
                .await
        }
        LoadType::NoMatches => {
            return ctx
                .send_message_response(interaction, "No results found")
                .await;
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
            let track = match loaded.tracks.first() {
                Some(t) => t,
                None => {
                    return ctx
                        .send_message_response(interaction, "Failed to process track")
                        .await
                }
            };
            let title = track.info.title.clone().unwrap_or("<Unknown>".to_string());
            let uri = &track.info.uri;
            let author = track.info.author.clone().unwrap_or("<Unknown>".to_string());

            let queue_arc = ctx.get_or_create_queue(guild_id);
            let queue = queue_arc.lock().unwrap();
            queue.push(crate::track::Track::new(track.clone(), channel_id));

            embed_builder = embed_builder
                .title("Track queued")
                .description(format!("**[{}]({})** \n By **{}**", title, uri, author));
            player.send(Play::from((guild_id, &track.track)))?;
        }
        _ => todo!(),
    }

    ctx.send_embed_response(interaction, embed_builder.build())
        .await
}
