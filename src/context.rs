use dashmap::DashMap;
use hyper::{client::HttpConnector, Client as HyperClient};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{MessageSender, ShardId};
use twilight_http::{client::InteractionClient, Client as HttpClient};
use twilight_lavalink::Lavalink;
use twilight_model::id::{marker::ApplicationMarker, Id};

use crate::interactions;

pub struct Context {
    pub http_client: HttpClient,
    pub hyper_client: HyperClient<HttpConnector>,
    pub cache: InMemoryCache,
    pub lavalink: Lavalink,
    pub shard_senders: DashMap<ShardId, MessageSender>,
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

        Ok(Self {
            http_client,
            hyper_client: HyperClient::new(),
            cache,
            lavalink,
            shard_senders: DashMap::default(),
        })
    }

    pub async fn app_id(&self) -> anyhow::Result<Id<ApplicationMarker>> {
        Ok(self
            .http_client
            .current_user_application()
            .await?
            .model()
            .await?
            .id)
    }

    pub async fn interaction_client(&self) -> anyhow::Result<InteractionClient> {
        Ok(self.http_client.interaction(self.app_id().await?))
    }

    /// Setup all the slash commands (currently only per guild)
    /// TODO: Add support for global commands
    pub async fn setup_commands(&self) -> anyhow::Result<()> {
        let commands = vec![
            interactions::hello_test::command(),
            interactions::join::command(),
            interactions::leave::command(),
            interactions::play::command(),
            interactions::pause::command(),
            interactions::resume::command(),
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
}
