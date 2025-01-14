use crate::events::queued_event::{QueuedEvent, QueuedEventStatus};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

pub trait EventQueue {
    /// Pushes an event to the end of the queue. Returns an error if the queue is full or the event
    /// is duplicated (an identical event already exists).
    fn push(&self, event: QueuedEvent) -> Result<(), QueueError>;

    /// Returns the next batch of events with the provided status.
    /// and with status Pending. Events are returned in FIFO order.
    fn next_batch(&self, status: QueuedEventStatus) -> Vec<QueuedEvent>;

    /// Returns whether the queue contains enough Pending events for delivering *at least* one batch.
    fn is_batch_full(&self) -> bool;
}

/// A simple event queue that stores events in a vector
#[derive(Debug, Clone)]
pub struct VecEventQueue {
    batch_size: usize,
    max_queue_size: usize,
    event_queue: Arc<Mutex<HashMap<QueuedEventStatus, VecDeque<QueuedEvent>>>>,
}

#[derive(Debug, PartialEq)]
pub enum QueueError {
    QueueFull,
    QueueLocked,
}

// batch size of zero means each event will be delivered individually, thus effectively disabling batching.
const MIN_BATCH_SIZE: usize = 0;
const MAX_BATCH_SIZE: usize = 10_000;

impl VecEventQueue {
    pub fn new(batch_size: usize, max_queue_size: usize) -> Self {
        // clamp batch size between min and max
        let clamped_batch_size = batch_size.clamp(MIN_BATCH_SIZE, MAX_BATCH_SIZE);
        VecEventQueue {
            batch_size: clamped_batch_size,
            max_queue_size,
            event_queue: Arc::new(Mutex::new(HashMap::with_capacity(clamped_batch_size))),
        }
    }

    fn len(&self) -> usize {
        self.event_queue
            .lock()
            .unwrap()
            .values()
            .map(|events| events.len())
            .sum()
    }
}

impl EventQueue for VecEventQueue {
    /// Pushes an event to the end of the queue. If max_queue_size is reached, returns an error instead.
    fn push(&self, event: QueuedEvent) -> Result<(), QueueError> {
        if self.len() + 1 > self.max_queue_size {
            return Err(QueueError::QueueFull);
        }
        let mut queue = self
            .event_queue
            .lock()
            .map_err(|_| QueueError::QueueLocked)?;
        let status_set = queue
            .entry(event.status.clone())
            .or_insert_with(VecDeque::new);
        status_set.push_back(event);
        Ok(())
    }

    /// Returns up to `batch_size` pending events for delivery from the queue.
    fn next_batch(&self, status: QueuedEventStatus) -> Vec<QueuedEvent> {
        let mut queue = self.event_queue.lock().unwrap();
        if let Some(events) = queue.get_mut(&status) {
            let mut batch = Vec::new();
            for _ in 0..self.batch_size {
                if let Some(event) = events.pop_front() {
                    batch.push(event);
                } else {
                    break;
                }
            }
            batch
        } else {
            Vec::new()
        }
    }

    fn is_batch_full(&self) -> bool {
        self.event_queue
            .lock()
            .unwrap()
            .entry(QueuedEventStatus::Pending)
            .or_insert_with(VecDeque::new)
            .len() >= self.batch_size
    }
}

#[cfg(test)]
mod tests {
    use crate::events::event::Event;
    use crate::events::queued_event::{QueuedEvent, QueuedEventStatus};
    use crate::events::vec_event_queue::{EventQueue, QueueError, VecEventQueue, MAX_BATCH_SIZE};
    use crate::timestamp::now;
    use std::collections::{HashMap, VecDeque};
    use std::sync::{Arc, Mutex};

    #[test]
    fn new_should_clamp_batch_size() {
        let queue = VecEventQueue::new(300_001, 20);
        assert_eq!(queue.batch_size, MAX_BATCH_SIZE);
    }

    #[test]
    fn test_next_pending_batch() {
        let queue = VecEventQueue::new(10, 20);
        assert_eq!(queue.batch_size, 10);
        assert_eq!(queue.is_batch_full(), false);
        let events = (0..11)
            .map(|i| {
                QueuedEvent::new(Event {
                    uuid: uuid::Uuid::new_v4(),
                    timestamp: now(),
                    event_type: i.to_string(),
                    payload: serde_json::json!({"key": i.to_string()}),
                })
            })
            .collect::<Vec<_>>();
        let cloned_events = events.clone();
        for event in cloned_events {
            queue.push(event).expect("should not fail");
        }
        assert_eq!(queue.is_batch_full(), true);
        assert_eq!(
            queue.next_batch(QueuedEventStatus::Pending),
            events.into_iter().take(10).collect::<Vec<QueuedEvent>>()
        );
        assert_eq!(queue.is_batch_full(), false);
        assert_eq!(queue.next_batch(QueuedEventStatus::Pending).len(), 1);
        assert_eq!(queue.next_batch(QueuedEventStatus::Pending).len(), 0);
    }

    #[test]
    fn test_next_pending_batch_status() {
        let event_a = Event {
            uuid: uuid::Uuid::new_v4(),
            timestamp: now(),
            event_type: "a".to_string(),
            payload: serde_json::json!({"key": "value"}),
        };
        let event_b = Event {
            uuid: uuid::Uuid::new_v4(),
            timestamp: now(),
            event_type: "b".to_string(),
            payload: serde_json::json!({"key": "value"}),
        };
        let event_c = Event {
            uuid: uuid::Uuid::new_v4(),
            timestamp: now(),
            event_type: "C".to_string(),
            payload: serde_json::json!({"key": "value"}),
        };
        let queued_event_a = QueuedEvent {
            event: event_a.clone(),
            attempts: 0,
            status: QueuedEventStatus::Pending,
        };
        let queued_event_b = QueuedEvent {
            event: event_b.clone(),
            attempts: 1,
            status: QueuedEventStatus::Failed,
        };
        let queued_event_c = QueuedEvent {
            event: event_c.clone(),
            attempts: 0,
            status: QueuedEventStatus::Pending,
        };
        let queue = VecEventQueue {
            batch_size: 10,
            max_queue_size: 10,
            event_queue: Arc::new(Mutex::new(HashMap::from([
                (
                    QueuedEventStatus::Pending,
                    VecDeque::from(vec![queued_event_a.clone(), queued_event_c.clone()]),
                ),
                (
                    QueuedEventStatus::Failed,
                    VecDeque::from(vec![queued_event_b.clone()]),
                ),
            ]))),
        };
        assert_eq!(
            queue.next_batch(QueuedEventStatus::Pending),
            vec![queued_event_a, queued_event_c]
        );
        assert_eq!(
            queue.next_batch(QueuedEventStatus::Failed),
            vec![queued_event_b.clone()]
        );
    }

    #[test]
    fn test_max_queue_size_push() {
        let queue = VecEventQueue::new(10, 2);
        let event = QueuedEvent::new(Event {
            uuid: uuid::Uuid::new_v4(),
            timestamp: now(),
            event_type: "test".to_string(),
            payload: serde_json::json!({"key": "value"}),
        });
        queue.push(event.clone()).expect("should not fail");
        queue.push(event.clone()).expect("should not fail");
        assert_eq!(
            queue.push(event.clone()).expect_err("should fail"),
            QueueError::QueueFull
        );
    }
}
