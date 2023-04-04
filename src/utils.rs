use std::sync::Arc;

use twilight_model::id::{
    marker::{GuildMarker, UserMarker},
    Id,
};

use crate::context::Context;

pub fn from_millis_to_minutes(millis: u64) -> String {
    let ms = millis as f64;
    let minutes = (ms / 60000.0).floor() as i32;
    let seconds = ((ms % 60000.0) / 1000.0) as i32;

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
