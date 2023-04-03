use twilight_lavalink::http::{Track as TwilightTrack, TrackInfo};
use twilight_model::id::{marker::ChannelMarker, Id};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    inner: TwilightTrack,
    pub channel_id: Id<ChannelMarker>,
}

impl Track {
    pub fn new(track: TwilightTrack, channel_id: Id<ChannelMarker>) -> Self {
        Self {
            inner: track,
            channel_id,
        }
    }

    pub fn info(&self) -> &TrackInfo {
        &self.inner.info
    }

    pub fn track(&self) -> String {
        self.inner.track.clone()
    }
}
