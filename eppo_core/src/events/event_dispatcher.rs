use crate::events::event::Event;
use crate::events::event_delivery::EventDelivery;
use crate::events::queued_event::{QueuedEvent, QueuedEventStatus};
use crate::events::vec_event_queue::{EventQueue, QueueError};
use log::warn;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::{Duration, Instant};
use url::Url;

#[derive(Debug)]
pub enum EventDispatcherCommand {
    Event,
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
}

#[derive(thiserror::Error, Debug)]
pub enum DispatcherError {
    #[error("Queue error: {0}")]
    QueueError(QueueError),
    #[error("Receiver should not be closed before all senders are closed")]
    EventDeliveryError,
}

/// EventDispatcher is responsible for batching events and delivering them to the ingestion service
/// via [`EventDelivery`].
pub struct EventDispatcher<T> {
    config: EventDispatcherConfig,
    event_queue: T,
    tx: UnboundedSender<EventDispatcherCommand>,
}

impl<T: EventQueue + Clone + Send + 'static> EventDispatcher<T> {
    pub fn new(
        config: EventDispatcherConfig,
        event_queue: T,
        tx: UnboundedSender<EventDispatcherCommand>,
    ) -> Self {
        EventDispatcher {
            config,
            tx,
            event_queue,
        }
    }

    /// Enqueues an event in the batch event processor and starts delivery if needed.
    pub fn dispatch(&self, event: Event) -> Result<(), DispatcherError> {
        self.event_queue
            .push(QueuedEvent::new(event))
            .map_err(DispatcherError::QueueError)?;
        self.send(EventDispatcherCommand::Event)
    }

    pub fn send(&self, command: EventDispatcherCommand) -> Result<(), DispatcherError> {
        match self.tx.send(command) {
            Ok(_) => Ok(()),
            Err(_) => Err(DispatcherError::EventDeliveryError),
        }
    }

    async fn event_dispatcher(&self, rx: &mut UnboundedReceiver<EventDispatcherCommand>) {
        let config = self.config.clone();
        let event_queue = self.event_queue.clone();
        let ingestion_url = Url::parse(config.ingestion_url.as_str())
            .expect("Failed to create EventDelivery. invalid ingestion URL");
        let event_delivery = EventDelivery::new(config.sdk_key.into(), ingestion_url);
        loop {
            let delivery_deadline = Instant::now() + config.delivery_interval;
            let retry_deadline = Instant::now() + config.retry_interval;
            // Loop until we have enough events to send or reached a delivery/retry deadline.
            tokio::select! {
                _ = tokio::time::sleep_until(delivery_deadline) => {
                    // delivery deadline reached -> send whatever we have queued up
                    spawn_event_delivery(
                        event_queue.next_batch(QueuedEventStatus::Pending),
                        event_delivery.clone(),
                        event_queue.clone()
                    );
                },
                _ = tokio::time::sleep_until(retry_deadline) => {
                    // retry deadline reached -> retry failed events
                    // TODO: retry with exponential backoff + jitter
                    spawn_event_delivery(
                        event_queue.next_batch(QueuedEventStatus::Retry),
                        event_delivery.clone(),
                        event_queue.clone()
                    );
                },
                command = rx.recv() => {
                    match command {
                        None => {
                            return; // channel closed
                        },
                        Some(EventDispatcherCommand::Event) => {
                            if event_queue.is_batch_full() {
                                // Event queue batch is full, deliver it
                                spawn_event_delivery(
                                    event_queue.next_batch(QueuedEventStatus::Pending),
                                    event_delivery.clone(),
                                    event_queue.clone()
                                );
                            } // else loop to get more events
                        },
                    }
                }
            }
        }
    }
}

fn spawn_event_delivery<T: EventQueue + Clone + Send + 'static>(
    batch: Vec<QueuedEvent>,
    event_delivery: EventDelivery,
    queue: T,
) {
    if batch.is_empty() {
        // nothing to deliver
        return;
    }
    // Send events
    tokio::spawn({
        async move {
            // Spawning a new task, so the main task can continue batching events and respond to
            // commands. At this point, batch is guaranteed to have at least one event.
            let events = batch
                .iter()
                .map(|queued_event| queued_event.clone().event)
                .collect();
            let result = event_delivery.deliver(events).await;
            match result {
                Ok(response) => {
                    let failed_event_uuids = response.failed_events;
                    if !failed_event_uuids.is_empty() {
                        warn!("Failed to deliver {} events", failed_event_uuids.len());
                        let failed_events = batch
                            .into_iter()
                            .filter(|queued_event| {
                                failed_event_uuids.contains(&queued_event.event.uuid)
                            })
                            .collect();
                        queue.mark_events_as_failed(failed_events);
                    }
                }
                Err(err) => {
                    warn!("Failed to deliver events: {}", err);
                    // In this case there is no point in retrying delivery since the error is
                    // non-retriable.
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::vec_event_queue::VecEventQueue;
    use crate::timestamp::now;
    use serde::Serialize;
    use serde_json::json;
    use std::sync::Arc;
    use tokio::sync::mpsc::unbounded_channel;
    use tokio::sync::Mutex;
    use tokio::time::Duration;
    use uuid::Uuid;
    use wiremock::http::Method;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
        let config = new_test_event_config(mock_server.uri());
        let (tx, rx) = unbounded_channel();
        let rx = Arc::new(Mutex::new(rx));
        let dispatcher = EventDispatcher::new(config, VecEventQueue::new(1, 10), tx);
        dispatcher.dispatch(event).unwrap();
        let rx_clone = Arc::clone(&rx);
        tokio::spawn(async move {
            let mut rx = rx_clone.lock().await;
            dispatcher.event_dispatcher(&mut rx).await;
        });
        {
            let mut rx = rx.lock().await;
            rx.close();
        }
        // wait some time for the async task to finish
        tokio::time::sleep(Duration::from_millis(100)).await;
        let received_requests = mock_server.received_requests().await.unwrap();
        assert_eq!(received_requests.len(), 1);
        let received_request = &received_requests[0];
        assert_eq!(received_request.method, Method::POST);
        assert_eq!(received_request.url.path(), "/");
        let received_body: serde_json::Value =
            serde_json::from_slice(&received_request.body).expect("Failed to parse request body");
        assert_eq!(received_body, expected_body);
    }

    #[tokio::test]
    async fn test_dispatch_failed() {
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
        let response_body =
            ResponseTemplate::new(200).set_body_json(json!({"failed_events": [event.uuid]}));
        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_json(&expected_body))
            .respond_with(response_body)
            .mount(&mock_server)
            .await;
        let config = new_test_event_config(mock_server.uri());
        let (tx, rx) = unbounded_channel();
        let rx = Arc::new(Mutex::new(rx));
        let dispatcher = EventDispatcher::new(config, VecEventQueue::new(1, 10), tx);
        let queue = dispatcher.event_queue.clone();
        dispatcher.dispatch(event.clone()).unwrap();
        let rx_clone = Arc::clone(&rx);
        tokio::spawn(async move {
            let mut rx = rx_clone.lock().await;
            dispatcher.event_dispatcher(&mut rx).await;
        });
        {
            let mut rx = rx.lock().await;
            rx.close();
        }
        // wait some time for the async task to finish
        tokio::time::sleep(Duration::from_millis(100)).await;
        let received_requests = mock_server.received_requests().await.unwrap();
        assert_eq!(received_requests.len(), 1);
        let failed_events = queue.next_batch(QueuedEventStatus::Failed);
        // assert failed event was moved to failed queue
        assert_eq!(
            failed_events,
            vec![QueuedEvent {
                event,
                attempts: 1,
                status: QueuedEventStatus::Failed
            }]
        );
        let pending_events = queue.next_batch(QueuedEventStatus::Pending);
        // assert no more pending events
        assert_eq!(pending_events, vec![]);
    }

    fn new_test_event_config(ingestion_url: String) -> EventDispatcherConfig {
        EventDispatcherConfig {
            sdk_key: "test-sdk-key".to_string(),
            ingestion_url,
            delivery_interval: Duration::from_millis(100),
            retry_interval: Duration::from_millis(1000),
            max_retry_delay: Duration::from_millis(5000),
            max_retries: Some(3),
        }
    }
}
