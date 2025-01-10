use crate::events::event::Event;
use log::{info, warn};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::{Duration, Instant};
use crate::events::event_delivery::EventDelivery;

#[derive(Debug)]
pub enum EventDispatcherCommand {
    Event(Event),
    Flush,
}

// batch size of one means each event will be delivered individually, thus effectively disabling batching.
const MIN_BATCH_SIZE: usize = 1;
const MAX_BATCH_SIZE: usize = 10_000;

#[derive(Debug, Clone)]
pub struct EventDispatcherConfig {
    pub sdk_key: String,
    pub ingestion_url: String,
    pub delivery_interval: Duration,
    pub retry_interval: Duration,
    pub max_retry_delay: Duration,
    pub max_retries: Option<u32>,
    pub batch_size: usize,
}

/// EventDispatcher is responsible for batching events and delivering them to the ingestion service
/// via [`EventDelivery`].
pub struct EventDispatcher {
    config: EventDispatcherConfig,
    tx: UnboundedSender<EventDispatcherCommand>,
}

impl EventDispatcher {
    pub fn new(config: EventDispatcherConfig, tx: UnboundedSender<EventDispatcherCommand>) -> Self {
        EventDispatcher { config, tx }
    }

    /// Enqueues an event in the batch event processor and starts delivery if needed.
    pub fn dispatch(&self, event: Event) -> Result<(), &str> {
        self.send(EventDispatcherCommand::Event(event))
    }

    pub fn send(&self, command: EventDispatcherCommand) -> Result<(), &str> {
        match self.tx.send(command) {
            Ok(_) => Ok(()),
            Err(_) => Err("receiver should not be closed before all senders are closed"),
        }
    }

    async fn event_dispatcher(&self, rx: &mut UnboundedReceiver<EventDispatcherCommand>) {
        let config = self.config.clone();
        let batch_size = config.batch_size;
        let event_delivery = EventDelivery::new(
            config.sdk_key.clone(),
            config.ingestion_url.clone()
        );
        loop {
            let mut batch_queue: Vec<Event> = Vec::with_capacity(batch_size);

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
                Some(EventDispatcherCommand::Event(event)) => batch_queue.push(event),
                Some(EventDispatcherCommand::Flush) => {
                    // No buffered events yet, nothing to flush.
                    continue;
                }
            }

            // short-circuit for batch size of 1
            if batch_queue.len() < batch_size {
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
                                    batch_queue.push(event);
                                    if batch_queue.len() >= batch_size {
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
            }

            // Send `batch` events.
            tokio::spawn({
                let event_delivery = event_delivery.clone();
                async move {
                    // Spawning a new task, so the main task can continue batching events and respond to
                    // commands. At this point, batch_queue is guaranteed to have at least one event.
                    let result = event_delivery.deliver(batch_queue).await;
                    match result {
                        Ok(response) => {
                            if !response.failed_events.is_empty() {
                                // TODO: Enqueue events for retry
                                warn!("Failed to deliver {} events", response.failed_events.len());
                            }
                        }
                        Err(err) => {
                            // TODO: Handle failure to deliver events
                            warn!("Failed to deliver events: {}", err);
                        }
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::*;
    use crate::timestamp::now;
    use serde::Serialize;
    use serde_json::json;
    use tokio::sync::mpsc::unbounded_channel;
    use tokio::sync::Mutex;
    use tokio::time::Duration;
    use uuid::Uuid;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::http::Method;
    use wiremock::matchers::{method, path, body_json};

    #[derive(Debug, Clone, Serialize)]
    struct LoginPayload {
        pub user_id: String,
        pub session_id: String,
    }

    #[tokio::test]
    async fn test_dispatch_starts_delivery() {
        let payload = LoginPayload {
            user_id: "user123".to_string(),
            session_id: "session456".to_string(),
        };
        let serialized_payload = serde_json::to_value(payload).expect("Serialization failed");
        let event = Event {
            uuid: Uuid::new_v4(),
            timestamp: now(),
            event_type: "test".to_string(),
            payload: serialized_payload,
        };
        let mock_server = MockServer::start().await;
        let mut eppo_events = Vec::new();
        eppo_events.push(serde_json::to_value(event.clone()).unwrap());
        let expected_body = json!({"eppo_events": eppo_events });
        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_json(&expected_body))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;
        let config = EventDispatcherConfig {
            sdk_key: "test-sdk-key".to_string(),
            ingestion_url: mock_server.uri(),
            delivery_interval: Duration::from_millis(100),
            retry_interval: Duration::from_millis(1000),
            max_retry_delay: Duration::from_millis(5000),
            max_retries: Some(3),
            batch_size: 1,
        };
        let (tx, rx) = unbounded_channel();
        let rx = Arc::new(Mutex::new(rx));
        let dispatcher = EventDispatcher::new(config, tx);
        dispatcher.dispatch(event).unwrap();
        dispatcher
            .send(EventDispatcherCommand::Flush)
            .expect("send should not fail");
        let rx_clone = Arc::clone(&rx);
        tokio::spawn(async move {
            let mut rx = rx_clone.lock().await;
            dispatcher.event_dispatcher(&mut rx).await;
        });
        {
            let mut rx = rx.lock().await; // Acquire the lock for rx
            rx.close();
        }
        // wait some time for the async task to finish
        tokio::time::sleep(Duration::from_millis(100)).await;
        let received_requests = mock_server.received_requests().await.unwrap();
        assert_eq!(received_requests.len(), 1);
        let received_request = &received_requests[0];
        assert_eq!(received_request.method, Method::Post);
        assert_eq!(received_request.url.path(), "/");
        let received_body: serde_json::Value = serde_json::from_slice(&received_request.body)
                .expect("Failed to parse request body");
        assert_eq!(received_body, expected_body);
    }
}
