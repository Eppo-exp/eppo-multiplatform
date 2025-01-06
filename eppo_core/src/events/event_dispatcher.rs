use std::collections::VecDeque;
use crate::events::event::{Event};
use log::info;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::time::{Duration, Instant};

enum EventDispatcherCommand {
    Event(Event),
    Flush,
}

// batch size of zero means each event will be delivered individually, thus effectively disabling batching.
const MIN_BATCH_SIZE: usize = 0;
const MAX_BATCH_SIZE: usize = 10_000;

#[derive(Debug, Clone)]
pub struct EventDispatcherConfig {
    pub sdk_key: String,
    pub ingestion_url: String,
    pub delivery_interval: Duration,
    pub retry_interval: Duration,
    pub max_retry_delay: Duration,
    pub max_retries: Option<u32>,
}

pub struct EventDispatcher {
    config: EventDispatcherConfig,
    batch_size: usize,
    tx: UnboundedSender<EventDispatcherCommand>,
    rx: UnboundedReceiver<EventDispatcherCommand>,
}

impl EventDispatcher {
    pub fn new(config: EventDispatcherConfig, batch_size: usize) -> Self {
        let (tx, rx) = unbounded_channel();
        EventDispatcher {
            config,
            batch_size: batch_size.clamp(MIN_BATCH_SIZE, MAX_BATCH_SIZE),
            tx,
            rx,
        }
    }

    /// Enqueues an event in the batch event processor and starts delivery if needed.
    pub fn dispatch(&self, event: Event) {
        self.tx.send(EventDispatcherCommand::Event(event))
            // TODO: handle/log error instead of panicking
            .expect("receiver should not be closed before all senders are closed")
    }

    async fn event_dispatcher(&self, mut rx: UnboundedReceiver<EventDispatcherCommand>) {
        let batch_size = self.batch_size;
        let config = self.config.clone();
        loop {
            let mut batch_queue: VecDeque<Event> = VecDeque::with_capacity(batch_size);
            let ingestion_url = config.ingestion_url.clone();

            // Wait for the first event in the batch.
            //
            // Optimization: Moved outside the loop below, so we're not woken up on regular intervals
            // unless we have something to send. (This achieves a similar effect as starting/stopping
            // delivery loop.)
            match rx.recv().await {
                None => {
                    // Channel closed, no more messages. Exit the main loop.
                    return;
                }
                Some(EventDispatcherCommand::Event(event)) => batch_queue.push_back(event),
                Some(EventDispatcherCommand::Flush) => {
                    // No buffered events yet, nothing to flush.
                    continue;
                }
            }

            let deadline = Instant::now() + config.delivery_interval;
            // Loop until we have enough events to send or reached deadline.
            loop {
                tokio::select! {
                    _ = tokio::time::sleep_until(deadline) => {
                        // reached deadline -> send everything we have
                        break;
                    },
                    command = rx.recv() => {
                        match command {
                            None => {
                                // channel closed
                                break;
                            },
                            Some(EventDispatcherCommand::Event(event)) => {
                                batch_queue.push_back(event);
                                if batch_queue.len() > batch_size {
                                    // Reached max batch size -> send events immediately
                                    break;
                                } // else loop to get more events
                            },
                            Some(EventDispatcherCommand::Flush) => {
                                break;
                            }
                        }
                    }
                }
            }

            // Send `batch` events.
            tokio::spawn(async move {
                // Spawning a new task, so the main task can continue batching events and respond to
                // commands.
                if !batch_queue.is_empty() {
                    batch_queue.make_contiguous();
                    let (events_to_deliver, _) = batch_queue.as_slices();
                    EventDispatcher::deliver(&ingestion_url, &events_to_deliver).await;
                }
            });
        }
    }

    async fn deliver(ingestion_url: &str, events: &[Event]) {
        // Simulated HTTP request or delivery logic
        info!(
            "Pretending to deliver {} events to {}",
            events.len(),
            ingestion_url
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timestamp::now;
    use tokio::time::Duration;
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize)]
    struct LoginPayload {
        pub user_id: String,
        pub session_id: String,
    }

    #[tokio::test]
    async fn test_dispatch_starts_delivery() {
        let config = EventDispatcherConfig {
            sdk_key: "test-sdk-key".to_string(),
            ingestion_url: "http://example.com".to_string(),
            delivery_interval: Duration::from_millis(100),
            retry_interval: Duration::from_millis(1000),
            max_retry_delay: Duration::from_millis(5000),
            max_retries: Some(3),
        };

        let dispatcher = EventDispatcher::new(config, 10);

        // Add an event
        let payload = LoginPayload {
            user_id: "user123".to_string(),
            session_id: "session456".to_string(),
        };
        let serialized_payload = serde_json::to_value(payload).expect("Serialization failed");
        dispatcher.dispatch(Event {
            uuid: Uuid::new_v4(),
            timestamp: now(),
            event_type: "test".to_string(),
            payload: serialized_payload,
        });

        // TODO: start the dispatcher thread and assert on successful delivery
        // // Wait a short time to allow delivery task to execute
        // tokio::time::sleep(Duration::from_millis(120)).await;
        //
        // // Ensure the batch queue is empty after delivery
        // assert_eq!(batch_queue.is_empty(), true);
    }
}
