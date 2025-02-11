use std::{sync::Arc, time::Duration};

use serde_json::Value;
use tokio::sync::{mpsc, Mutex};
use url::Url;
use uuid::Uuid;

use crate::{background::BackgroundRuntime, sdk_key::SdkKey};

use super::{
    auto_flusher, batcher,
    delivery::{self, DeliveryConfig},
    event_delivery::EventDelivery,
    BatchedMessage, Event,
};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct EventIngestionConfig {
    pub sdk_key: SdkKey,
    pub ingestion_url: Url,
    pub max_queue_size: usize,
    pub delivery_interval: Duration,
    pub batch_size: usize,
    pub max_retries: u32,
    pub base_retry_delay: Duration,
    pub max_retry_delay: Duration,
}

impl EventIngestionConfig {
    const DEFAULT_MAX_QUEUE_SIZE: usize = 10_000;
    const DEFAULT_DELIVERY_INTERVAL: Duration = Duration::from_secs(10);
    const DEFAULT_BATCH_SIZE: usize = 1_000;
    const DEFAULT_BASE_RETRY_DELAY: Duration = Duration::from_secs(5);
    const DEFAULT_MAX_RETRY_DELAY: Duration = Duration::from_secs(30);
    const DEFAULT_MAX_RETRIES: u32 = 3;

    /// Creates new event ingestion config.
    ///
    /// Returns `None` if `sdk_key` is not suitable for event ingestion.
    pub fn new(sdk_key: SdkKey) -> Option<Self> {
        let ingestion_url = sdk_key.event_ingestion_url()?;
        let config = EventIngestionConfig {
            sdk_key,
            ingestion_url,
            max_queue_size: Self::DEFAULT_MAX_QUEUE_SIZE,
            delivery_interval: Self::DEFAULT_DELIVERY_INTERVAL,
            batch_size: Self::DEFAULT_BATCH_SIZE,
            max_retries: Self::DEFAULT_MAX_RETRIES,
            base_retry_delay: Self::DEFAULT_BASE_RETRY_DELAY,
            max_retry_delay: Self::DEFAULT_MAX_RETRY_DELAY,
        };
        Some(config)
    }

    pub fn spawn(&self, runtime: &BackgroundRuntime) -> EventIngestion {
        EventIngestion::spawn(runtime, self)
    }
}

/// A handle to Event Ingestion subsystem.
pub struct EventIngestion {
    tx: mpsc::Sender<BatchedMessage<Event>>,
    context_sender: mpsc::Sender<(String, Value)>
}

impl EventIngestion {
    /// Starts the event ingestion subsystem on the given background runtime.
    pub fn spawn(runtime: &BackgroundRuntime, config: &EventIngestionConfig) -> EventIngestion {
        let event_delivery = Arc::new(Mutex::new(EventDelivery::new(
            reqwest::Client::new(),
            config.sdk_key.clone(),
            config.ingestion_url.clone(),
        )));

        let event_delivery_clone = Arc::clone(&event_delivery);
        let (context_sender, mut context_rx) = mpsc::channel::<(String, Value)>(100);
        runtime.spawn_untracked(async move {
            while let Some((key, value)) = context_rx.recv().await {
                let mut event_delivery = event_delivery_clone.lock().await;
                if let Err(err) = event_delivery.attach_context(key, value) {
                    log::warn!(target: "eppo", "Failed to attach context: {}", err);
                }
            }
        });

        let event_delivery_clone = Arc::clone(&event_delivery);
        let (input, flusher_uplink) = mpsc::channel(config.max_queue_size);
        let (flusher_downlink, batcher_uplink) = mpsc::channel(1);
        let (batcher_downlink, delivery_uplink) = mpsc::channel(1);
        let (delivery_status_tx, delivery_status_rx) = mpsc::channel(1);

        runtime.spawn_untracked(auto_flusher::auto_flusher(
            flusher_uplink,
            flusher_downlink,
            config.delivery_interval,
        ));
        runtime.spawn_untracked(batcher::batcher(
            batcher_uplink,
            batcher_downlink.clone(),
            config.batch_size,
        ));
        runtime.spawn_untracked(delivery::delivery(
            delivery_uplink,
            delivery_status_tx.clone(),
            event_delivery_clone,
            DeliveryConfig {
                max_retries: config.max_retries,
                base_retry_delay: config.base_retry_delay,
                max_retry_delay: config.max_retry_delay,
            },
        ));

        // For now, nobody is interested in delivery statuses.
        let _ = delivery_status_rx;

        EventIngestion {
            tx: input,
            context_sender,
        }
    }

    pub fn track(&self, event_type: String, payload: serde_json::Value) {
        let event = Event {
            uuid: Uuid::new_v4(),
            timestamp: crate::timestamp::now(),
            event_type,
            payload,
        };

        self.track_event(event);
    }

    /// Attaches a context to be included with all events dispatched by the EventDispatcher.
    /// The context is delivered as a top-level object in the ingestion request payload.
    /// An existing key can be removed by providing a `null` value.
    /// Calling this method with same key multiple times causes only the last value to be used for the
    /// given key.
    ///
    /// @param key - The context entry key.
    /// @param value - The context entry value, must be a string, number, boolean, or null. If value is
    /// an object or an array, will throw an ArgumentError.
    pub fn attach_context(&self, key: String, value: Value) {
        if self.context_sender.try_send((key, value)).is_err() {
            log::warn!(target: "eppo", "Failed to send context update to worker");
        }
    }

    fn track_event(&self, event: Event) {
        let result = self.tx.try_send(BatchedMessage::singleton(event));

        if let Err(err) = result {
            log::warn!(target: "eppo", "Failed to submit event to event ingestion: {}", err);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_ingestion::event::Event;
    use crate::timestamp::now;
    use serde_json::json;
    use tokio::time::Duration;
    use uuid::Uuid;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn init() {
        let _ = env_logger::try_init();
    }

    #[tokio::test]
    async fn test_dispatch_starts_delivery() {
        init();

        let event = new_test_event();

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_json(&json!({
                "context": {},
                "eppo_events": [event.clone()],
            })))
            .and(header("x-eppo-token", "test-sdk-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "failed_events": []
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        run_dispatcher_task(event.clone(), &mock_server.uri()).await;

        mock_server.verify().await;
    }

    #[tokio::test]
    async fn test_dispatch_failed_after_max_retries() {
        init();

        let event = new_test_event();

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_json(&json!({"context": {}, "eppo_events": [event] })))
            .and(header("x-eppo-token", "test-sdk-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "failed_events": [event.uuid],
            })))
            .expect(3) // 1 regular attempt + 2 retries
            .mount(&mock_server)
            .await;

        run_dispatcher_task(event.clone(), &mock_server.uri()).await;

        mock_server.verify().await;
    }

    #[tokio::test]
    async fn test_attach_context() {
        init();

        let event = new_test_event();
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_json(&json!({
                "context": {
                    "string": "value",
                    "number": 42,
                    "boolean": true,
                    "null": null,
                },
                "eppo_events": [event.clone()],
            })))
            .and(header("x-eppo-token", "test-sdk-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "failed_events": []
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let batch_size = 1;
        let config = new_test_event_config(Url::parse(&mock_server.uri()).unwrap(), batch_size);
        let runtime = BackgroundRuntime::new(tokio::runtime::Handle::current());
        let event_ingestion = config.spawn(&runtime);
        event_ingestion.attach_context("string".to_string(), json!("value"));
        event_ingestion.attach_context("number".to_string(), json!(42));
        event_ingestion.attach_context("boolean".to_string(), json!(true));
        event_ingestion.attach_context("null".to_string(), json!(null));
        event_ingestion.track_event(event);
        // wait some time for the async task to finish
        // TODO: use flush instead of sleeping
        tokio::time::sleep(Duration::from_millis(100)).await;

        mock_server.verify().await;
    }

    fn new_test_event() -> Event {
        Event {
            uuid: Uuid::new_v4(),
            timestamp: now(),
            event_type: "test".to_string(),
            payload: serde_json::json!({
                "user_id": "user123",
                "session_id": "session456",
            }),
        }
    }

    fn new_test_event_config(ingestion_url: Url, batch_size: usize) -> EventIngestionConfig {
        EventIngestionConfig {
            sdk_key: SdkKey::new("test-sdk-key".into()),
            ingestion_url,
            max_queue_size: 10,
            delivery_interval: Duration::from_millis(10),
            batch_size,
            max_retries: 2,
            base_retry_delay: Duration::from_millis(1),
            max_retry_delay: Duration::from_millis(10),
        }
    }

    async fn run_dispatcher_task(event: Event, mock_server_uri: &str) {
        let batch_size = 1;
        let config = new_test_event_config(Url::parse(mock_server_uri).unwrap(), batch_size);
        let runtime = BackgroundRuntime::new(tokio::runtime::Handle::current());
        let dispatcher = config.spawn(&runtime);
        dispatcher.track_event(event);

        // wait some time for the async task to finish
        // TODO: use flush instead of sleeping
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
