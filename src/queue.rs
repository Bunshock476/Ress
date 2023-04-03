use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};
use twilight_model::id::{marker::GuildMarker, Id};

use rand::seq::SliceRandom;

use crate::track::Track;

#[derive(Debug)]
pub enum TracksQueueError {
    EmptyQueue,
    NoQueueFound(Id<GuildMarker>),
}

impl Display for TracksQueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            TracksQueueError::EmptyQueue => write!(f, "Trying to operate on empty queue"),
            TracksQueueError::NoQueueFound(s) => write!(f, "No queue found for guild id {s}"),
        }
    }
}

impl std::error::Error for TracksQueueError {}

#[derive(Debug)]
pub struct TracksQueue {
    inner: Arc<Mutex<Vec<Track>>>,
}

#[allow(dead_code)]
impl TracksQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn push(&self, track: Track) {
        let mut inner = self.inner.lock().unwrap();
        inner.push(track);
    }

    pub fn pop(&self) -> Result<Track, TracksQueueError> {
        let mut inner = self.inner.lock().unwrap();
        if inner.is_empty() {
            Err(TracksQueueError::EmptyQueue)
        } else {
            Ok(inner.remove(0_usize))
        }
    }

    pub fn peek(&self) -> Result<Track, TracksQueueError> {
        let inner = self.inner.lock().unwrap();
        match inner.first() {
            Some(val) => Ok(val.clone()),
            None => Err(TracksQueueError::EmptyQueue),
        }
    }

    pub fn is_empty(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.is_empty()
    }

    pub fn len(&self) -> usize {
        let inner = self.inner.lock().unwrap();
        inner.len()
    }

    pub fn current_queue(&self) -> Vec<Track> {
        let inner = self.inner.lock().unwrap();

        inner.clone()
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();

        inner.clear();
    }

    pub fn shuffle(&self) {
        let mut inner = self.inner.lock().unwrap();

        inner.shuffle(&mut rand::thread_rng());
    }
}
