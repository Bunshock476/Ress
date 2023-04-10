use std::sync::{Arc, Mutex};

use dashmap::DashMap;
use hyper::{client::HttpConnector, Client as HyperClient};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{MessageSender, ShardId};
use twilight_http::{client::InteractionClient, Client as HttpClient};
use twilight_lavalink::Lavalink;
use twilight_model::{
    application::interaction::Interaction,
    channel::message::Embed,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{
        marker::{ApplicationMarker, GuildMarker},
        Id,
    },
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{interactions, queue::TracksQueue};

pub struct Context {
    pub app_id: Id<ApplicationMarker>,
    pub http_client: HttpClient,
    pub hyper_client: HyperClient<HttpConnector>,
    pub cache: InMemoryCache,
    pub lavalink: Lavalink,
    pub shard_senders: DashMap<ShardId, MessageSender>,
    pub queues: DashMap<Id<GuildMarker>, Arc<Mutex<TracksQueue>>>,
}

impl Context {
    pub async fn new(token: String) -> anyhow::Result<self::Context> {
        // Create http client
        let http_client = HttpClient::new(token);

        let cache = InMemoryCache::builder()
            .resource_types(ResourceType::MESSAGE | ResourceType::VOICE_STATE)
            .build();

        let user_id = http_client.current_user().await?.model().await?.id;

        let lavalink = Lavalink::new(user_id, 1u64);

        let app_id = http_client.current_user_application().await?.model().await?.id;

        Ok(Self {
            app_id,
            http_client,
            hyper_client: HyperClient::new(),
            cache,
            lavalink,
            shard_senders: DashMap::default(),
            queues: DashMap::default(),
        })
    }

    pub async fn interaction_client(&self) -> anyhow::Result<InteractionClient> {
        Ok(self.http_client.interaction(self.app_id))
    }

    /// Setup all the slash commands (currently only per guild)
    /// TODO: Add support for global commands
    pub async fn setup_commands(&self) -> anyhow::Result<()> {
        let commands = vec![
            interactions::join::command(),
            interactions::leave::command(),
            interactions::play::command(),
            interactions::pause::command(),
            interactions::resume::command(),
            interactions::stop::command(),
            interactions::skip::command(),
            interactions::shuffle::command(),
            interactions::queue::command(),
            interactions::now_playing::command(),
            interactions::lup::command(),
        ];
        // Application command registering (doing it per guild as doing it globally can take a couple of minutes)
        self.interaction_client()
            .await?
            .set_guild_commands(
                Id::new(std::env::var("TEST_GUILD")?.parse::<u64>()?),
                &commands,
            )
            .await?;
        Ok(())
    }

    pub fn add_shard_message_sender(&self, shard_id: ShardId, sender: MessageSender) {
        self.shard_senders.insert(shard_id, sender);
    }

    pub fn get_queue(&self, guild_id: Id<GuildMarker>) -> Option<Arc<Mutex<TracksQueue>>> {
        self.queues.get(&guild_id).map(|mapref| Arc::clone(&mapref))
    }

    pub fn get_or_create_queue(&self, guild_id: Id<GuildMarker>) -> Arc<Mutex<TracksQueue>> {
        self.get_queue(guild_id).unwrap_or(
            self.queues
                .entry(guild_id)
                .or_insert(Arc::new(Mutex::new(TracksQueue::new())))
                .clone(),
        )
    }

    pub async fn send_message_response(
        &self,
        interaction: &Interaction,
        content: impl Into<String>,
    ) -> anyhow::Result<()> {
        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(
                InteractionResponseDataBuilder::new()
                    .content(content)
                    .build(),
            ),
        };

        self.interaction_client()
            .await?
            .create_response(interaction.id, &interaction.token, &response)
            .await?;

        Ok(())
    }

    pub async fn send_embed_response(
        &self,
        interaction: &Interaction,
        embed: Embed,
    ) -> anyhow::Result<()> {
        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(
                InteractionResponseDataBuilder::new()
                    .embeds(vec![embed])
                    .build(),
            ),
        };

        self.interaction_client()
            .await?
            .create_response(interaction.id, &interaction.token, &response)
            .await?;

        Ok(())
    }
}
