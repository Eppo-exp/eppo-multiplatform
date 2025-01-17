use crate::event_ingestion::queued_event::{QueuedEvent, QueuedEventStatus};
use linked_hash_set::LinkedHashSet;
use log::warn;
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

    /// Marks the provided events as failed and increments their attempts and adds it to the failed queue.
    fn enqueue_failed_events_for_retry(&self, failed_events: Vec<QueuedEvent>);
}

#[derive(Debug, Clone)]
pub struct VecEventQueueConfig {
    pub batch_size: usize,
    pub max_queue_size: usize,
    pub max_retries: u8,
}

/// A simple event queue that stores events in a vector
#[derive(Debug, Clone)]
pub struct VecEventQueue {
    config: VecEventQueueConfig,
    pending_event_queue: Arc<Mutex<LinkedHashSet<QueuedEvent>>>,
    failed_event_queue: Arc<Mutex<LinkedHashSet<QueuedEvent>>>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum QueueError {
    #[error("Event queue is full")]
    QueueFull,
    #[error("Event queue is locked")]
    QueueLocked,
}

// batch size of zero means each event will be delivered individually, thus effectively disabling batching.
const MIN_BATCH_SIZE: usize = 0;
const MAX_BATCH_SIZE: usize = 10_000;

impl VecEventQueue {
    pub fn new(config: VecEventQueueConfig) -> Self {
        let clamped_batch_size = config.batch_size.clamp(MIN_BATCH_SIZE, MAX_BATCH_SIZE);
        VecEventQueue {
            config: VecEventQueueConfig {
                // clamp batch size between min and max
                batch_size: clamped_batch_size,
                max_queue_size: config.max_queue_size,
                max_retries: config.max_retries,
            },
            pending_event_queue: Arc::new(Mutex::new(LinkedHashSet::with_capacity(
                clamped_batch_size,
            ))),
            failed_event_queue: Arc::new(Mutex::new(LinkedHashSet::with_capacity(
                config.max_queue_size,
            ))),
        }
    }

    fn len(&self) -> usize {
        self.pending_event_queue.lock().unwrap().len()
            + self.failed_event_queue.lock().unwrap().len()
    }
}

impl EventQueue for VecEventQueue {
    /// Pushes an event to the end of the queue. If max_queue_size is reached, returns an error instead.
    fn push(&self, event: QueuedEvent) -> Result<(), QueueError> {
        if self.len() + 1 > self.config.max_queue_size {
            return Err(QueueError::QueueFull);
        }
        let mut queue = self
            .pending_event_queue
            .lock()
            .map_err(|_| QueueError::QueueLocked)?;
        queue.insert(event);
        Ok(())
    }

    /// Returns up to `batch_size` events from the queue with the provided `status`.
    fn next_batch(&self, status: QueuedEventStatus) -> Vec<QueuedEvent> {
        let mut queue = if status == QueuedEventStatus::Pending {
            self.pending_event_queue.lock().unwrap()
        } else {
            self.failed_event_queue.lock().unwrap()
        };
        let mut batch = Vec::new();
        for _ in 0..self.config.batch_size {
            if let Some(event) = queue.pop_front() {
                batch.push(event);
            } else {
                break;
            }
        }
        batch
    }

    fn is_batch_full(&self) -> bool {
        self.pending_event_queue.lock().unwrap().len() >= self.config.batch_size
    }

    fn enqueue_failed_events_for_retry(&self, failed_event_uuids: Vec<QueuedEvent>) {
        let mut failed_event_queue = self.failed_event_queue.lock().unwrap();
        for mut failed_event in failed_event_uuids {
            failed_event.status = QueuedEventStatus::Failed;
            if failed_event.attempts >= self.config.max_retries {
                // do not re-add to the queue if max retries is reached and simply drop the event
                warn!(
                    "Event with UUID {} has reached max retries and will not be requeued",
                    failed_event.event.uuid
                );
                continue;
            }
            failed_event.attempts += 1;
            failed_event_queue.insert(failed_event);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::event_ingestion::event::Event;
    use crate::event_ingestion::queued_event::{QueuedEvent, QueuedEventStatus};
    use crate::event_ingestion::vec_event_queue::{
        EventQueue, QueueError, VecEventQueue, VecEventQueueConfig, MAX_BATCH_SIZE,
    };
    use crate::timestamp::now;
    use linked_hash_set::LinkedHashSet;
    use std::sync::{Arc, Mutex};

    #[test]
    fn new_should_clamp_batch_size() {
        let queue = VecEventQueue::new(VecEventQueueConfig {
            batch_size: 300_001,
            max_queue_size: 20,
            max_retries: 3,
        });
        assert_eq!(queue.config.batch_size, MAX_BATCH_SIZE);
    }

    #[test]
    fn test_next_pending_batch() {
        let queue = VecEventQueue::new(VecEventQueueConfig {
            batch_size: 10,
            max_queue_size: 20,
            max_retries: 3,
        });
        assert_eq!(queue.config.batch_size, 10);
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
    fn test_updated_attempts_count() {
        let queue = VecEventQueue::new(VecEventQueueConfig {
            batch_size: 10,
            max_queue_size: 20,
            max_retries: 2,
        });
        let event = QueuedEvent::new(Event {
            uuid: uuid::Uuid::new_v4(),
            timestamp: now(),
            event_type: "test".to_string(),
            payload: serde_json::json!({"key": "value"}),
        });
        queue.push(event.clone()).expect("should not fail");
        let batch = queue.next_batch(QueuedEventStatus::Pending);
        queue.push(event.clone()).expect("should not fail");
        queue.enqueue_failed_events_for_retry(batch.clone());
        let failed_events = queue.next_batch(QueuedEventStatus::Failed);
        queue.enqueue_failed_events_for_retry(failed_events);
        let failed_events = queue.next_batch(QueuedEventStatus::Failed);
        assert_eq!(failed_events.len(), 1);
        assert_eq!(failed_events[0].event.uuid, event.event.uuid);
        assert_eq!(failed_events[0].status, QueuedEventStatus::Failed);
        assert_eq!(failed_events[0].attempts, 2);
        // failing a third time should not requeue since that exceeds max_retries == 2
        queue.enqueue_failed_events_for_retry(failed_events);
        // event is removed from failed queue
        let failed_events = queue.next_batch(QueuedEventStatus::Failed);
        assert_eq!(failed_events.len(), 0);
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
            config: VecEventQueueConfig {
                batch_size: 10,
                max_queue_size: 10,
                max_retries: 3,
            },
            pending_event_queue: Arc::new(Mutex::new(
                vec![queued_event_a.clone(), queued_event_c.clone()]
                    .into_iter()
                    .collect::<LinkedHashSet<QueuedEvent>>(),
            )),
            failed_event_queue: Arc::new(Mutex::new(
                vec![queued_event_b.clone()]
                    .into_iter()
                    .collect::<LinkedHashSet<QueuedEvent>>(),
            )),
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
        let queue = VecEventQueue::new(VecEventQueueConfig {
            batch_size: 10,
            max_queue_size: 2,
            max_retries: 3,
        });
        queue
            .push(QueuedEvent::new(Event {
                uuid: uuid::Uuid::new_v4(),
                timestamp: now(),
                event_type: "test".to_string(),
                payload: serde_json::json!({"key": "value"}),
            }))
            .expect("should not fail");
        queue
            .push(QueuedEvent::new(Event {
                uuid: uuid::Uuid::new_v4(),
                timestamp: now(),
                event_type: "test".to_string(),
                payload: serde_json::json!({"key": "value"}),
            }))
            .expect("should not fail");
        assert_eq!(
            queue
                .push(QueuedEvent::new(Event {
                    uuid: uuid::Uuid::new_v4(),
                    timestamp: now(),
                    event_type: "test".to_string(),
                    payload: serde_json::json!({"key": "value"}),
                }))
                .expect_err("should fail"),
            QueueError::QueueFull
        );
    }

    #[test]
    fn test_enqueue_failed_events_for_retry() {
        let queue = VecEventQueue::new(VecEventQueueConfig {
            batch_size: 10,
            max_queue_size: 20,
            max_retries: 3,
        });
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
        let queued_event_a = QueuedEvent::new(event_a.clone());
        let queued_event_b = QueuedEvent::new(event_b.clone());
        queue.push(queued_event_a.clone()).expect("should not fail");
        let batch = queue.next_batch(QueuedEventStatus::Pending);
        queue.push(queued_event_b.clone()).expect("should not fail");
        queue.enqueue_failed_events_for_retry(batch);
        let failed_events = queue.failed_event_queue.lock().unwrap();
        assert_eq!(failed_events.len(), 1);
        assert_eq!(
            failed_events.front().unwrap().event.uuid,
            queued_event_a.event.uuid
        );
        assert_eq!(
            failed_events.front().unwrap().status,
            QueuedEventStatus::Failed
        );
        assert_eq!(failed_events.front().unwrap().attempts, 1);
        let pending_events = queue.pending_event_queue.lock().unwrap();
        assert_eq!(pending_events.len(), 1);
    }
}
