use std::sync::Arc;

use twilight_gateway::ShardId;
use twilight_model::application::interaction::{Interaction, InteractionData, InteractionType};

use crate::context::Context;

pub mod join;
pub mod leave;
pub mod lup;
pub mod now_playing;
pub mod pause;
pub mod play;
pub mod queue;
pub mod resume;
pub mod shuffle;
pub mod skip;
pub mod stop;

pub async fn handle_interaction(
    ctx: Arc<Context>,
    interaction: Interaction,
    shard_id: ShardId,
) -> anyhow::Result<()> {
    match interaction.kind {
        InteractionType::ApplicationCommand => {
            if let Some(interaction_data) = &interaction.data {
                let command_data = match interaction_data {
                    InteractionData::ApplicationCommand(cd) => cd,
                    _ => anyhow::bail!("Invalid type of data passed to application command"),
                };
                match command_data.name.as_str() {
                    join::NAME => {
                        join::run(&interaction, ctx.clone(), shard_id).await?;
                    }
                    leave::NAME => {
                        leave::run(&interaction, ctx.clone(), shard_id).await?;
                    }
                    play::NAME => {
                        play::run(&interaction, ctx.clone(), shard_id).await?;
                    }
                    pause::NAME => {
                        pause::run(&interaction, ctx.clone(), shard_id).await?;
                    }
                    resume::NAME => {
                        resume::run(&interaction, ctx.clone(), shard_id).await?;
                    }
                    stop::NAME => {
                        stop::run(&interaction, ctx.clone(), shard_id).await?;
                    }
                    skip::NAME => {
                        skip::run(&interaction, ctx.clone(), shard_id).await?;
                    }
                    shuffle::NAME => {
                        shuffle::run(&interaction, ctx.clone(), shard_id).await?;
                    }
                    queue::NAME => {
                        queue::run(&interaction, ctx.clone(), shard_id).await?;
                    }
                    now_playing::NAME => {
                        now_playing::run(&interaction, ctx.clone(), shard_id).await?;
                    }
                    lup::NAME => {
                        lup::run(&interaction, ctx.clone(), shard_id).await?;
                    }
                    _ => anyhow::bail!("Invalid command"),
                };
            }
        }
        _ => todo!("Handle other interaction types"),
    }

    Ok(())
}
