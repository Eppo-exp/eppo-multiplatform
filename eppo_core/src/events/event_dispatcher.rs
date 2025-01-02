use crate::events::batch_event_queue::BatchEventQueue;
use crate::events::event::{Event, GenericEvent};
use log::info;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use tokio::time::{interval_at, Duration, Instant};

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
    batch_queue: BatchEventQueue,
    delivery_task_active: Arc<Mutex<bool>>,
}

impl EventDispatcher {
    pub fn new(config: EventDispatcherConfig, batch_queue: BatchEventQueue) -> Self {
        EventDispatcher {
            config,
            batch_queue,
            delivery_task_active: Arc::new(Mutex::new(false)),
        }
    }

    /// Enqueues an event in the batch event processor and starts delivery if needed.
    pub fn dispatch<T : Serialize>(&self, event: Event<T>) {
        // Convert the generic payload into a serde_json::Value
        let serialized_payload = serde_json::to_value(event.payload).expect("Serialization failed");
        // Create a new Event with the serialized payload
        let event_with_value = Event {
            uuid: event.uuid,
            timestamp: event.timestamp,
            event_type: event.event_type,
            payload: serialized_payload,
        };
        self.batch_queue.push(event_with_value);

        // Start the delivery loop if it's not already active
        if !self.is_delivery_timer_active() {
            self.start_delivery_loop();
        }
    }

    fn start_delivery_loop(&self) {
        let active_flag = Arc::clone(&self.delivery_task_active);
        let config = self.config.clone();
        let batch_queue = self.batch_queue.clone();

        // Mark the delivery loop as active
        {
            let mut is_active = active_flag.lock().unwrap();
            *is_active = true;
        }

        tokio::spawn(async move {
            let mut interval = interval_at(
                Instant::now() + config.delivery_interval,
                config.delivery_interval,
            );
            loop {
                interval.tick().await;
                let events_to_process = batch_queue.next_batch();
                if !events_to_process.is_empty() {
                    EventDispatcher::deliver(&config.ingestion_url, &events_to_process).await;
                } else {
                    // If no more events to deliver, break the loop
                    let mut is_active = active_flag.lock().unwrap();
                    *is_active = false;
                    break;
                }
            }
        });
    }

    async fn deliver(ingestion_url: &str, events: &[GenericEvent]) {
        // Simulated HTTP request or delivery logic
        info!(
            "Pretending to deliver {} events to {}",
            events.len(),
            ingestion_url
        );
    }

    fn is_delivery_timer_active(&self) -> bool {
        *self.delivery_task_active.lock().unwrap()
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

        let batch_queue = BatchEventQueue::new(10);
        let dispatcher = EventDispatcher::new(config, batch_queue.clone());

        // Add an event
        dispatcher.dispatch(Event {
            uuid: Uuid::new_v4(),
            timestamp: now(),
            event_type: "test".to_string(),
            payload: LoginPayload {
                user_id: "user123".to_string(),
                session_id: "session456".to_string(),
            },
        });

        // Wait a short time to allow delivery task to execute
        tokio::time::sleep(Duration::from_millis(120)).await;

        // Ensure the batch queue is empty after delivery
        assert_eq!(batch_queue.is_empty(), true);
    }
}
