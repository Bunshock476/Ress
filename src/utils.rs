use std::sync::Arc;

use twilight_model::id::{
    marker::{GuildMarker, UserMarker},
    Id,
};

use crate::context::Context;

pub fn from_ms_to_minutes(ms: u64) -> String {
    let minutes = (ms as f64 / 60000.0).floor() as i32;
    let seconds = ((ms as f64 % 60000.0) / 1000.0) as i32;

    if seconds == 60 {
        format!("{}:00", minutes + 1)
    } else if seconds < 10 {
        format!("{}:0{}", minutes, seconds)
    } else {
        format!("{}:{}", minutes, seconds)
    }
}
pub fn check_voice_state(
    ctx: Arc<Context>,
    author_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
) -> bool {
    matches!(ctx.cache.voice_state(author_id, guild_id), Some(_vc))
}
