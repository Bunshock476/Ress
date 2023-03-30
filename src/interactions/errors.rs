use std::{error::Error, fmt::Display};

use twilight_gateway::ShardId;

#[derive(Debug)]
pub struct NoAuthorFound {}

impl Display for NoAuthorFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No author found for this interaction")
    }
}

impl Error for NoAuthorFound {}

#[derive(Debug)]
pub struct InvalidGuildId {}

impl Display for InvalidGuildId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No guild id found for interaction")
    }
}

impl Error for InvalidGuildId {}

#[derive(Debug)]
pub struct NoMessageSenderForShard {
    pub shard_id: ShardId,
}

impl Display for NoMessageSenderForShard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No message sender found for shard id {}", self.shard_id)
    }
}

impl Error for NoMessageSenderForShard {}
