use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::events::event::Event;

#[derive(Debug, Clone)]
pub struct BatchEventQueue {
    batch_size: usize,
    event_queue: Arc<Mutex<VecDeque<Event>>>,
}

// batch size of zero means each event will be delivered individually, thus effectively disabling batching.
const MIN_BATCH_SIZE: usize = 0;
const MAX_BATCH_SIZE: usize = 10_000;

impl BatchEventQueue {
    pub fn new(batch_size: usize) -> Self {
        // clamp batch size between min and max
        BatchEventQueue {
            batch_size: batch_size.clamp(MIN_BATCH_SIZE, MAX_BATCH_SIZE),
            event_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn push(&self, event: Event) {
        let mut queue = self.event_queue.lock().unwrap();
        queue.push_back(event);
    }

    pub fn next_batch(&self) -> Vec<Event> {
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
