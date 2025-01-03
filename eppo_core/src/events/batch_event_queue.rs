use crate::events::event::GenericEvent;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct BatchEventQueue {
    batch_size: usize,
    event_queue: Arc<Mutex<VecDeque<GenericEvent>>>,
}

const MIN_BATCH_SIZE: usize = 100;
const MAX_BATCH_SIZE: usize = 10_000;

impl BatchEventQueue {
    pub fn new(batch_size: usize) -> Self {
        // clamp batch size between min and max
        BatchEventQueue {
            batch_size: batch_size.clamp(MIN_BATCH_SIZE, MAX_BATCH_SIZE),
            event_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn push(&self, event: GenericEvent) {
        let mut queue = self.event_queue.lock().unwrap();
        queue.push_back(event);
    }

    pub fn next_batch(&self) -> Vec<GenericEvent> {
        let mut queue = self.event_queue.lock().unwrap();
        let mut batch = vec![];
        while let Some(event) = queue.pop_front() {
            batch.push(event);
            if batch.len() >= self.batch_size {
                break;
            }
        }
        batch
    }

    pub fn is_full(&self) -> bool {
        let queue = self.event_queue.lock().unwrap();
        queue.len() >= self.batch_size
    }

    pub fn is_empty(&self) -> bool {
        let queue = self.event_queue.lock().unwrap();
        queue.is_empty()
    }
}
