use crate::event_ingestion::batched_message::BatchedMessage;
use crate::event_ingestion::delivery::QueuedBatch;
use crate::event_ingestion::event_delivery::EventDelivery;
use crate::event_ingestion::queued_event::QueuedEvent;
use crate::event_ingestion::{auto_flusher, batcher, delivery, retry};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Duration;
use url::Url;

// batch size of one means each event will be delivered individually, thus effectively disabling batching.
const MIN_BATCH_SIZE: usize = 1;
const MAX_BATCH_SIZE: usize = 10_000;

#[derive(Debug, Clone)]
pub(super) struct EventDispatcherConfig {
    pub sdk_key: String,
    pub ingestion_url: String,
    pub delivery_interval: Duration,
    pub retry_interval: Duration,
    pub max_retry_delay: Duration,
    pub max_retries: u32,
    pub batch_size: usize,
    pub max_queue_size: usize,
}

/// EventDispatcher is responsible for batching events and delivering them to the ingestion service
/// via [`EventDelivery`].
pub(super) struct EventDispatcher {
    config: EventDispatcherConfig,
}

impl EventDispatcher {
    pub fn new(config: EventDispatcherConfig) -> Self {
        EventDispatcher { config }
    }

    /// Starts the event dispatcher related tasks and returns a sender and receiver pair.
    /// Use the sender to dispatch events and the receiver to receive delivery statuses.
    fn spawn_event_dispatcher(
        &self,
    ) -> (Sender<BatchedMessage<QueuedEvent>>, Receiver<QueuedBatch>) {
        let config = self.config.clone();
        let ingestion_url = Url::parse(config.ingestion_url.as_str())
            .expect("Failed to create EventDelivery. invalid ingestion URL");
        let event_delivery = EventDelivery::new(config.sdk_key.into(), ingestion_url);

        let channel_size = config.max_queue_size;
        let (sender, flusher_uplink_rx) = mpsc::channel(config.max_queue_size);
        let (flusher_downlink_tx, flusher_downlink_rx) = mpsc::channel(1);
        let (batcher_downlink_tx, batcher_downlink_rx) = mpsc::channel(1);
        let (delivery_downlink_tx, delivery_downlink_rx) = mpsc::channel(1);
        let (retry_downlink_tx, receiver) = mpsc::channel(1);

        // Spawn the auto_flusher, batcher and delivery
        tokio::spawn(auto_flusher::auto_flusher(
            flusher_uplink_rx,
            flusher_downlink_tx,
            config.delivery_interval,
        ));
        tokio::spawn(batcher::batcher(
            flusher_downlink_rx,
            batcher_downlink_tx.clone(),
            config.batch_size,
        ));
        tokio::spawn(delivery::delivery(
            batcher_downlink_rx,
            delivery_downlink_tx,
            config.max_retries,
            event_delivery,
        ));
        tokio::spawn(retry::retry(
            delivery_downlink_rx,
            batcher_downlink_tx,
            retry_downlink_tx,
            config.max_retries as u32,
            config.retry_interval,
            config.max_retry_delay,
        ));

        (sender, receiver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_ingestion::delivery::QueuedBatch;
    use crate::timestamp::now;
    use serde::Serialize;
    use serde_json::json;
    use tokio::time::Duration;
    use uuid::Uuid;
    use wiremock::http::Method;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use crate::event_ingestion::event::Event;

    #[derive(Debug, Clone, Serialize)]
    struct LoginPayload {
        pub user_id: String,
        pub session_id: String,
    }

    fn init() {
        let _ = env_logger::try_init();
    }

    #[tokio::test]
    async fn test_dispatch_starts_delivery() {
        init();
        let event = new_test_event();
        let mock_server = MockServer::start().await;
        let mut eppo_events = Vec::new();
        eppo_events.push(serde_json::to_value(event.clone()).unwrap());
        let expected_body = json!({"eppo_events": eppo_events });
        let response_body = ResponseTemplate::new(200).set_body_json(json!({"failed_events": []}));
        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_json(&expected_body))
            .respond_with(response_body)
            .mount(&mock_server)
            .await;
        let mut rx = run_dispatcher_task(event.clone(), mock_server.uri()).await;
        let delivery_status = rx.recv().await.unwrap();
        let successful_events = delivery_status.success.clone();
        let failed_events = delivery_status.failure.clone();
        drop(delivery_status);
        let received_requests = mock_server.received_requests().await.unwrap();
        assert_eq!(received_requests.len(), 1);
        let received_request = &received_requests[0];
        assert_eq!(received_request.method, Method::POST);
        assert_eq!(received_request.url.path(), "/");
        let received_body: serde_json::Value =
            serde_json::from_slice(&received_request.body).expect("Failed to parse request body");
        assert_eq!(received_body, expected_body);
        assert_eq!(successful_events, vec![QueuedEvent { event, attempts: 0 }]);
        assert_eq!(failed_events.len(), 0);
    }

    #[tokio::test]
    async fn test_dispatch_failed_after_max_retries() {
        init();
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
        let mut rx = run_dispatcher_task(event.clone(), mock_server.uri()).await;
        let delivery_status = rx.recv().await.unwrap();
        assert_eq!(
            delivery_status,
            QueuedBatch::retry(vec![QueuedEvent {
                event: event.clone(),
                attempts: 1
            }])
        );
        let delivery_status = rx.recv().await.unwrap();
        assert_eq!(
            delivery_status,
            QueuedBatch::retry(vec![QueuedEvent {
                event: event.clone(),
                attempts: 2
            }])
        );
        let delivery_status = rx.recv().await.unwrap();
        assert_eq!(
            delivery_status,
            QueuedBatch::failure(vec![QueuedEvent {
                event: event.clone(),
                attempts: 3 // 1 regular attempt + 2 retries
            }]),
        );
        let received_requests = mock_server.received_requests().await.unwrap();
        assert_eq!(received_requests.len(), 3);
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
            batch_size,
            delivery_interval: Duration::from_millis(100),
            retry_interval: Duration::from_millis(1000),
            max_retry_delay: Duration::from_millis(5000),
            max_retries: 2,
            max_queue_size: 10,
        }
    }

    async fn run_dispatcher_task(event: Event, mock_server_uri: String) -> Receiver<QueuedBatch> {
        let batch_size = 1;
        let config = new_test_event_config(mock_server_uri, batch_size);
        let dispatcher = EventDispatcher::new(config);
        let (tx, rx) = dispatcher.spawn_event_dispatcher();
        tx.send(BatchedMessage::new(vec![QueuedEvent::new(event)], None))
            .await
            .unwrap();
        // wait some time for the async task to finish
        tokio::time::sleep(Duration::from_millis(100)).await;
        rx
    }
}
