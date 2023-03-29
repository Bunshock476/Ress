use futures::StreamExt;
use twilight_gateway::{
    stream::{self, ShardEventStream},
    Config, Event, Intents,
};
use twilight_http::client::InteractionClient;
use twilight_model::application::interaction::{Interaction, InteractionData, InteractionType};

mod context;
mod interactions;

use context::Context;

// TODO: remove the dependencie on anyhow
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize log tracer
    tracing_subscriber::fmt::init();

    // Load environment variables from .env file
    dotenv::dotenv().expect(".env file");

    // Initialize configurations
    let token = std::env::var("DISCORD_TOKEN")?;
    let intents = Intents::GUILD_MESSAGES
        | Intents::MESSAGE_CONTENT
        | Intents::GUILDS
        | Intents::GUILD_VOICE_STATES;

    let config = Config::new(token.clone(), intents);

    // Bot context for sharing data across tasks and accessing twilight clients and general setup
    let ctx = Context::new(token).await?;

    // Initialize the bot slash commands
    ctx.setup_commands().await?;

    // Initialize shards (currently spawning discord's recommended number of shards, could be only one for small bots)
    let mut shards =
        stream::create_recommended(&ctx.http_client, config, |_, builder| builder.build())
            .await?
            .collect::<Vec<_>>();

    // Stream of shard events
    let mut stream = ShardEventStream::new(shards.iter_mut());

    // Initialize the loop to handle events
    while let Some((_shard, e)) = stream.next().await {
        let event = match e {
            Ok(ev) => ev,
            Err(err) => {
                tracing::error!("Failed to receive event. Error: {err}");

                if err.is_fatal() {
                    break;
                }

                continue;
            }
        };

        match event {
            Event::InteractionCreate(interaction) => {
                interaction_handler(&ctx.interaction_client().await?, interaction.0).await?
            }
            _ => {}
        }
    }

    Ok(())
}

async fn interaction_handler(
    interaction_client: &InteractionClient<'_>,
    interaction: Interaction,
) -> anyhow::Result<()> {
    match interaction.kind {
        InteractionType::ApplicationCommand => {
            if let Some(interaction_data) = &interaction.data {
                match interaction_data {
                    InteractionData::ApplicationCommand(command_data) => {
                        // Run the command and etrieve a response from the command caller
                        let response = match command_data.name.as_str() {
                            interactions::hello_test::NAME => {
                                interactions::hello_test::run(&interaction).await?
                            }
                            _ => todo!("Custom error for non-existent commands"),
                        };

                        interaction_client
                            .create_response(interaction.id, &interaction.token, &response)
                            .await?;
                    }
                    _ => todo!("Handle other interaction types"),
                }
            }
        }
        _ => {}
    }

    Ok(())
}
