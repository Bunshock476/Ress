use std::sync::{Arc, Mutex};

use rand::seq::SliceRandom;

use crate::track::Track;

#[derive(Debug)]
pub struct TracksQueue {
    inner: Arc<Mutex<Vec<Track>>>,
}

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

    pub fn pop(&self) -> anyhow::Result<Track> {
        let mut inner = self.inner.lock().unwrap();
        if inner.is_empty() {
            Err(anyhow::anyhow!("Empty queue"))
        } else {
            Ok(inner.remove(0_usize))
        }
    }

    pub fn peek(&self) -> anyhow::Result<Track> {
        let inner = self.inner.lock().unwrap();
        match inner.first() {
            Some(val) => Ok(val.clone()),
            None => Err(anyhow::anyhow!("Empty queue")),
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
