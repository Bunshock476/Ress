use std::sync::Arc;
use twilight_gateway::ShardId;
use twilight_model::application::{
    command::{Command, CommandType, CommandOption, CommandOptionType, CommandOptionChoice, CommandOptionChoiceValue},
    interaction::{Interaction, InteractionData, application_command::CommandOptionValue},
};
use twilight_util::builder::command::CommandBuilder;

use crate::{context::Context, queue::QueueLoopMode};

// The module is called lup cause loop is a restricted keyword

pub const NAME: &str = "loop";

pub fn command() -> Command {
    CommandBuilder::new("loop", "Sets the loop mode of the queue", CommandType::ChatInput)
        .option(CommandOption {
            autocomplete: Some(false),
            channel_types: None,
            choices: Some(vec![
                CommandOptionChoice { name: "none".to_string(), name_localizations: None, value: CommandOptionChoiceValue::String("none".to_string())},
                CommandOptionChoice { name: "queue".to_string(), name_localizations: None, value: CommandOptionChoiceValue::String("queue".to_string())},
                CommandOptionChoice { name: "track".to_string(), name_localizations: None, value: CommandOptionChoiceValue::String("track".to_string())},
            ]),
            description: "Loop mode".to_owned(),
            description_localizations: None,
            kind: CommandOptionType::String,
            max_length: None,
            max_value: None,
            min_length: None,
            min_value: None,
            name: "mode".to_owned(),
            name_localizations: None,
            options: None,
            required: Some(true),
        }).build()
}

pub async fn run(
    interaction: &Interaction,
    ctx: Arc<Context>,
    _shard_id: ShardId,
) -> anyhow::Result<()> {
    tracing::debug!(
        "Loop command by {}",
        interaction
            .author()
            .ok_or(anyhow::anyhow!("No author found"))?
            .name
    );

    let guild_id = interaction.guild_id.expect("Valid guild id");

    let bot_id = ctx.http_client.current_user().await?.model().await?.id;
    match ctx.cache.voice_state(bot_id, guild_id) {
        Some(vc) => vc,
        None => {
            return ctx
                .send_message_response(interaction, "Im not in a voice channel")
                .await;
        }
    };

    let options = {
        if let Some(InteractionData::ApplicationCommand(data)) = &interaction.data {
            &data.options
        } else {
            unreachable!()
        }
    };

    let mode = match &options[0].value {
        CommandOptionValue::String(m) => m.clone(),
        _ => anyhow::bail!("Option value should have been a string"),
    };

    let queue_arc = match ctx.get_queue(guild_id) {
        Some(arc) => arc,
        None => {
            return ctx
                .send_message_response(interaction, "No tracks queued")
                .await;
        }
    };
    let content: String;
    {
        let mut queue = queue_arc.lock().unwrap();
        
        let new_mode = match mode.as_str() {
            "none" => {
                content = "Not looping".to_string();
                QueueLoopMode::None
            },
            "queue" => {
                content = "Looping the whole queue".to_string();
                QueueLoopMode::LoopQueue
            },
            "track" => {
                content = "Looping the current track".to_string();
                QueueLoopMode::LoopTrack
            },
            _ => {anyhow::bail!("Invalid loop mode");}
        };

        queue.set_loop_mode(new_mode);
    }

    ctx.send_message_response(interaction, content)
        .await
}
