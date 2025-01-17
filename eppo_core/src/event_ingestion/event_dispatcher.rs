use crate::event_ingestion::batched_message::BatchedMessage;
use crate::event_ingestion::delivery::DeliveryStatus;
use crate::event_ingestion::event::Event;
use crate::event_ingestion::event_delivery::EventDelivery;
use crate::event_ingestion::queued_event::{QueuedEvent, QueuedEventStatus};
use crate::event_ingestion::vec_event_queue::{EventQueue, QueueError};
use crate::event_ingestion::{auto_flusher, batcher, delivery};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender, UnboundedSender};
use tokio::time::Duration;
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
    pub batch_size: usize,
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
}

impl<T: EventQueue + Clone + Send + 'static> EventDispatcher<T> {
    pub fn new(config: EventDispatcherConfig, event_queue: T) -> Self {
        EventDispatcher {
            config,
            event_queue,
        }
    }

    /// Enqueues an event in the batch event processor and starts delivery if needed.
    pub fn dispatch(&self, event: Event) -> Option<DispatcherError> {
        self.event_queue
            .push(QueuedEvent::new(event))
            .map_err(DispatcherError::QueueError)
            .err()
    }

    fn spawn_event_dispatcher(&self) -> (Sender<BatchedMessage<Event>>, Receiver<DeliveryStatus>) {
        let config = self.config.clone();
        let ingestion_url = Url::parse(config.ingestion_url.as_str())
            .expect("Failed to create EventDelivery. invalid ingestion URL");
        let event_delivery = EventDelivery::new(config.sdk_key.into(), ingestion_url);

        // TODO: Does this make sense for channel size?
        let channel_size = MAX_BATCH_SIZE;
        let (sender, flusher_uplink_rx) = mpsc::channel(channel_size);
        let (flusher_downlink_tx, flusher_downlink_rx) = mpsc::channel(channel_size);
        let (batcher_downlink_tx, batcher_downlink_rx) = mpsc::channel(channel_size);
        let (delivery_downlink_tx, receiver) = mpsc::channel(channel_size);

        // Spawn the auto_flusher, batcher and delivery
        tokio::spawn(auto_flusher::auto_flusher(
            flusher_uplink_rx,
            flusher_downlink_tx,
            config.delivery_interval,
        ));
        tokio::spawn(batcher::batcher(
            flusher_downlink_rx,
            batcher_downlink_tx,
            config.batch_size,
        ));
        tokio::spawn(delivery::delivery(
            batcher_downlink_rx,
            delivery_downlink_tx,
            event_delivery,
        ));

        (sender, receiver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_ingestion::vec_event_queue::{VecEventQueue, VecEventQueueConfig};
    use crate::timestamp::now;
    use serde::Serialize;
    use serde_json::json;
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
        let event = new_test_event();
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
        run_dispatcher_task(event, mock_server.uri()).await;
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
        let event = new_test_event();
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
        let queue = run_dispatcher_task(event.clone(), mock_server.uri()).await;
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

    fn new_test_event() -> Event {
        let payload = LoginPayload {
            user_id: "user123".to_string(),
            session_id: "session456".to_string(),
        };
        let serialized_payload = serde_json::to_value(payload).expect("Serialization failed");
        Event {
            uuid: Uuid::new_v4(),
            timestamp: now(),
            event_type: "test".to_string(),
            payload: serialized_payload,
        }
    }

    fn new_test_event_config(ingestion_url: String, batch_size: usize) -> EventDispatcherConfig {
        EventDispatcherConfig {
            sdk_key: "test-sdk-key".to_string(),
            ingestion_url,
            delivery_interval: Duration::from_millis(100),
            retry_interval: Duration::from_millis(1000),
            max_retry_delay: Duration::from_millis(5000),
            batch_size,
        }
    }

    async fn run_dispatcher_task(event: Event, mock_server_uri: String) -> VecEventQueue {
        let batch_size = 1;
        let config = new_test_event_config(mock_server_uri, batch_size);
        let vec_event_queue_config = VecEventQueueConfig {
            max_retries: 3,
            max_queue_size: 10,
            batch_size,
        };
        let dispatcher = EventDispatcher::new(config, VecEventQueue::new(vec_event_queue_config));
        let queue = dispatcher.event_queue.clone();
        dispatcher.dispatch(event.clone());
        let (tx, rx) = dispatcher.spawn_event_dispatcher();
        tx.send(BatchedMessage {
            batch: vec![event],
            flush: None,
        }).await.unwrap();
        drop(rx);
        // wait some time for the async task to finish
        tokio::time::sleep(Duration::from_millis(100)).await;
        queue
    }
}
