use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use log::debug;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;
use uuid::Uuid;

use crate::sdk_key::SdkKey;

use super::{delivery::DeliveryStatus, event::Event};

const MAX_EVENT_SERIALIZED_LENGTH: usize = 4096;

#[derive(Clone)]
pub(super) struct EventDelivery {
    sdk_key: SdkKey,
    ingestion_url: Url,
    client: reqwest::Client,
    context: HashMap<String, Value>,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum ContextError {
    #[error("JSON value cannot be an object or an array")]
    InvalidContextValueType,
}

#[derive(thiserror::Error, Debug)]
pub(super) enum EventDeliveryError {
    #[error("Transient error delivering events")]
    RetriableError(#[source] reqwest::Error),
    #[error("Non-retriable error")]
    NonRetriableError(#[source] reqwest::Error),
}

impl From<reqwest::Error> for EventDeliveryError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_builder() || err.is_request() {
            // Issue with request. Most likely a json serialization error.
            EventDeliveryError::NonRetriableError(err)
        } else if err.is_status() {
            match err.status() {
                Some(StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) => {
                    log::warn!(target: "eppo", "client is not authorized. Check your API key");
                    EventDeliveryError::NonRetriableError(err)
                }
                Some(
                    status @ (StatusCode::BAD_REQUEST
                    | StatusCode::NOT_FOUND
                    | StatusCode::METHOD_NOT_ALLOWED
                    | StatusCode::CONFLICT
                    | StatusCode::UNPROCESSABLE_ENTITY),
                ) => {
                    // These errors are not-retriable
                    log::warn!(target: "eppo", "received {status} response delivering events: {:?}", err);
                    EventDeliveryError::NonRetriableError(err)
                }
                Some(status) if status.is_server_error() => {
                    log::warn!(target: "eppo", "received {status} response delivering events: {err:?}");
                    EventDeliveryError::RetriableError(err)
                }
                _ => {
                    // Other error statuses **might be** retriable
                    log::warn!(target: "eppo", "received non-200 response delivering events: {:?}", err);
                    EventDeliveryError::RetriableError(err)
                }
            }
        } else {
            // Failed to get a server response. Most likely, an intermittent network error.
            EventDeliveryError::RetriableError(err)
        }
    }
}

#[derive(Debug, Serialize)]
struct IngestionRequestBody<'a> {
    context: &'a HashMap<String, Value>,
    eppo_events: &'a [Event],
}

#[derive(Debug, Deserialize)]
struct IngestionResponseBody {
    failed_events: HashSet<Uuid>,
}

/// Responsible for delivering event batches to the Eppo ingestion service.
impl EventDelivery {
    pub fn new(client: reqwest::Client, sdk_key: SdkKey, ingestion_url: Url) -> Self {
        EventDelivery {
            sdk_key,
            ingestion_url,
            client,
            context: HashMap::new(),
        }
    }

    /// Delivers the provided event batch and returns delivery status.
    pub(super) async fn deliver(&self, events: Vec<Event>) -> DeliveryStatus {
        let result = self.deliver_inner(&events).await;

        let body = match result {
            Ok(body) => body,
            Err(EventDeliveryError::RetriableError(_)) => return DeliveryStatus::retry(events),
            Err(_) => {
                // Non-retriable error
                return DeliveryStatus::failure(events);
            }
        };

        if body.failed_events.is_empty() {
            // Partial failure is expected to be rather rare, so this branch is an optimization for
            // the more common case (whole-batch success).
            return DeliveryStatus::success(events);
        }

        let mut status = DeliveryStatus::new(
            Vec::with_capacity(events.len() - body.failed_events.len()),
            Vec::new(),
            Vec::with_capacity(body.failed_events.len()),
        );
        for event in events {
            if body.failed_events.contains(&event.uuid) {
                status.retry.push(event);
            } else {
                status.success.push(event);
            }
        }

        status
    }

    pub fn attach_context(&mut self, key: String, value: Value) -> Result<(), ContextError> {
        // ensure value is valid (not object or array)
        return match value {
            Value::Object(_) | Value::Array(_) => Err(ContextError::InvalidContextValueType),
            _ => {
                self.context.insert(key, value);
                Ok(())
            }
        };
    }

    async fn deliver_inner(
        &self,
        events: &[Event],
    ) -> Result<IngestionResponseBody, EventDeliveryError> {
        if events.is_empty() {
            return Ok(IngestionResponseBody {
                failed_events: HashSet::new(),
            });
        }

        let ingestion_url = self.ingestion_url.clone();
        let sdk_key = &self.sdk_key;
        debug!("Delivering {} events to {}", events.len(), ingestion_url);

        let body = IngestionRequestBody {
            context: &self.context,
            eppo_events: events,
        };

        let response = self
            .client
            .post(ingestion_url)
            .header("X-Eppo-Token", sdk_key.as_str())
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<IngestionResponseBody>()
            .await?;

        debug!(
            target: "eppo",
            "Batch delivered successfully, {} events failed ingestion",
            response.failed_events.len()
        );

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_ingestion::event::Event;
    use crate::sdk_key::SdkKey;
    use crate::timestamp::now;
    use serde_json::json;
    use url::Url;
    use uuid::Uuid;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_delivery() {
        let uuid = Uuid::new_v4();
        let timestamp = now();
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .and(header("X-Eppo-Token", "foobar"))
            .and(body_json(&json!({
                "context": {
                    "key1": "value1",
                    "key2": 42,
                    "key3": true,
                    "key4": null,
                },
                "eppo_events": [{
                    "uuid": uuid,
                    "timestamp": timestamp.timestamp_millis(),
                    "type": "test",
                    "payload": {
                        "user_id": "user123",
                        "session_id": "session456",
                    }
                }]
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"failed_events": []})))
            .expect(1)
            .mount(&mock_server)
            .await;

        let mut delivery = EventDelivery::new(
            reqwest::Client::new(),
            SdkKey::new("foobar".into()),
            Url::parse(mock_server.uri().as_str()).unwrap(),
        );

        let event = Event {
            uuid,
            timestamp,
            event_type: "test".to_string(),
            payload: serde_json::json!({
                "user_id": "user123",
                "session_id": "session456",
            }),
        };

        delivery
            .attach_context("key1".to_string(), json!("value1"))
            .unwrap();
        delivery
            .attach_context("key2".to_string(), json!(42))
            .unwrap();
        delivery
            .attach_context("key3".to_string(), json!(true))
            .unwrap();
        delivery
            .attach_context("key4".to_string(), json!(null))
            .unwrap();

        let result = delivery.deliver(vec![event.clone()]).await;

        assert_eq!(result, DeliveryStatus::success(vec![event]));

        mock_server.verify().await;
    }

    #[test]
    fn test_attach_context_valid_values() {
        let mut delivery = EventDelivery::new(
            reqwest::Client::new(),
            SdkKey::new("foobar".into()),
            Url::parse("http://example.com").unwrap(),
        );
        assert!(delivery
            .attach_context("key1".to_string(), json!("value1"))
            .is_ok());
        assert!(delivery
            .attach_context("key2".to_string(), json!(42))
            .is_ok());
        assert!(delivery
            .attach_context("key3".to_string(), json!(true))
            .is_ok());
        assert!(delivery
            .attach_context("key4".to_string(), json!(null))
            .is_ok());
        let ctx = delivery.context;
        assert_eq!(ctx.len(), 4);
        assert_eq!(ctx.get("key1").unwrap(), &json!("value1"));
        assert_eq!(ctx.get("key2").unwrap(), &json!(42));
        assert_eq!(ctx.get("key3").unwrap(), &json!(true));
        assert_eq!(ctx.get("key4").unwrap(), &json!(null));
    }

    #[test]
    fn test_attach_context_invalid_values() {
        let mut delivery = EventDelivery::new(
            reqwest::Client::new(),
            SdkKey::new("foobar".into()),
            Url::parse("http://example.com").unwrap(),
        );
        assert_eq!(
            delivery.attach_context("key1".to_string(), json!({"foo": "bar"})),
            Err(ContextError::InvalidContextValueType)
        );
        assert_eq!(
            delivery.attach_context("key2".to_string(), json!([1, 2, 3])),
            Err(ContextError::InvalidContextValueType)
        );
        assert_eq!(delivery.context.len(), 0);
    }
}
