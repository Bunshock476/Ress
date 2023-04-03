use std::sync::Arc;

use twilight_gateway::ShardId;
use twilight_model::application::{
    command::{Command, CommandOption, CommandOptionType, CommandType},
    interaction::{application_command::CommandOptionValue, Interaction, InteractionData},
};
use twilight_util::builder::command::CommandBuilder;

use crate::context::Context;

pub const NAME: &str = "hello-test";

pub fn command() -> Command {
    CommandBuilder::new("hello-test", "Test command", CommandType::ChatInput)
        .option(CommandOption {
            autocomplete: Some(false),
            channel_types: None,
            choices: None,
            description: "Who to say hello to".to_string(),
            description_localizations: None,
            kind: CommandOptionType::String,
            max_length: None,
            max_value: None,
            min_length: None,
            min_value: None,
            name: "name".to_string(),
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
    let options = {
        if let Some(InteractionData::ApplicationCommand(data)) = &interaction.data {
            &data.options
        } else {
            unreachable!()
        }
    };

    let name = match &options[0].value {
        CommandOptionValue::String(n) => n.clone(),
        _ => "".to_string(),
    };

    ctx.send_message_response(interaction, format!("Hello {}", name))
        .await
}
