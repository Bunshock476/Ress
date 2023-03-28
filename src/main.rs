use futures::StreamExt;
use twilight_gateway::{
    stream::{self, ShardEventStream},
    Config, Event, Intents,
};
use twilight_http::{client::InteractionClient, Client};
use twilight_model::{
    application::{
        command::CommandType,
        interaction::{Interaction, InteractionData, InteractionType},
    },
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::Id,
};
use twilight_util::builder::{command::CommandBuilder, InteractionResponseDataBuilder};

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

    // Create http client
    let client = Client::new(token);

    // Initialize shards (currently spawning discord's recommended number of shards, could be only one for small bots)
    let mut shards = stream::create_recommended(&client, config, |_, builder| builder.build())
        .await?
        .collect::<Vec<_>>();

    // Stream of shard events
    let mut stream = ShardEventStream::new(shards.iter_mut());

    // Application/Slash command creation
    // TODO: do it in a function and return a vec of commands
    let test = CommandBuilder::new("hello-test", "Test command", CommandType::ChatInput).build();

    // Inteaction client used to receive and respond to interactions
    let interaction_client =
        client.interaction(client.current_user_application().await?.model().await?.id);

    // Application command registering (doing it per guild as doing it globally can take a couple of minutes)
    interaction_client
        .set_guild_commands(Id::new(std::env::var("TEST_GUILD")?.parse::<u64>()?), &[test])
        .await?;

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
                interaction_handler(&interaction_client, interaction.0).await?
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
                        // TODO: match against a hashmap or vec of commands and handle the response on different module
                        match command_data.name.as_str() {
                            "hello-test" => {
                                let response = InteractionResponse {
                                    kind: InteractionResponseType::ChannelMessageWithSource,
                                    data: Some(
                                        InteractionResponseDataBuilder::new()
                                            .content(format!(
                                                "Hello {}",
                                                interaction.author().unwrap().id
                                            ))
                                            .build(),
                                    ),
                                };

                                interaction_client
                                    .create_response(interaction.id, &interaction.token, &response)
                                    .await?;
                                Ok(())
                            }
                            _ => todo!("Custom error for non-existent commands"),
                        }
                    }
                    _ => todo!(),
                }
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}
