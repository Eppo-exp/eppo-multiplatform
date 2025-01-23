use crate::event_ingestion::event::Event;
use crate::event_ingestion::event_delivery::EventDeliveryError::{
    EventPayloadTooLargeError, JsonDeserializationError, JsonSerializationError,
};
use crate::Str;
use log::{debug, info};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;
use uuid::Uuid;

const MAX_EVENT_SERIALIZED_LENGTH: usize = 4096;

#[derive(Clone)]
pub(super) struct EventDelivery {
    sdk_key: Str,
    ingestion_url: Url,
    client: reqwest::Client,
}

#[derive(Debug, PartialEq)]
pub(super) struct EventDeliveryResponse {
    // Events that failed delivery but that may be retried at a later time
    pub retriable_failed_events: HashSet<Uuid>,
    // Events that failed delivery and should not be retried (the failure is final)
    pub non_retriable_failed_events: HashSet<Uuid>,
}

impl EventDeliveryResponse {
    pub fn empty() -> Self {
        EventDeliveryResponse {
            retriable_failed_events: HashSet::new(),
            non_retriable_failed_events: HashSet::new(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.retriable_failed_events.is_empty() && self.non_retriable_failed_events.is_empty()
    }
}

#[derive(serde::Deserialize, Debug)]
pub(super) struct RawEventDeliveryResponse {
    pub failed_events: HashSet<Uuid>,
}

#[derive(thiserror::Error, Debug)]
pub(super) enum EventDeliveryError {
    #[error("Single event payload too large {0} (expected < {max})", max = MAX_EVENT_SERIALIZED_LENGTH)]
    EventPayloadTooLargeError(usize),
    #[error("Failed to serialize events to JSON")]
    JsonSerializationError(serde_json::Error),
    #[error("Failed to parse JSON response")]
    JsonDeserializationError(reqwest::Error),
    #[error("Transient error delivering events")]
    RetriableError(reqwest::Error),
    #[error("Non-retriable error")]
    NonRetriableError(reqwest::Error),
}

#[derive(Debug, Serialize)]
struct IngestionRequestBody<'a> {
    eppo_events: &'a [&'a Event],
}

/// Responsible for delivering event batches to the Eppo ingestion service.
impl EventDelivery {
    pub fn new(sdk_key: Str, ingestion_url: Url) -> Self {
        let client = reqwest::Client::new();
        EventDelivery {
            sdk_key,
            ingestion_url,
            client,
        }
    }

    /// Delivers the provided event batch and returns a Vec with the events that failed to be delivered.
    pub(super) async fn deliver(
        &self,
        events: &[&Event],
    ) -> Result<EventDeliveryResponse, EventDeliveryError> {
        let ingestion_url = self.ingestion_url.clone();
        let sdk_key = &self.sdk_key;
        debug!("Delivering {} events to {}", events.len(), ingestion_url);
        let mut failed_validation_events = HashSet::new();
        let mut events_to_deliver = Vec::new();
        for event in events {
            if !ensure_max_event_size(&event)? {
                failed_validation_events.insert(event.uuid);
            } else {
                events_to_deliver.push(event);
            }
        }
        if events_to_deliver.is_empty() {
            // Short circuit if nothing left to deliver after filtering for event validation
            return Ok(EventDeliveryResponse {
                retriable_failed_events: HashSet::new(),
                non_retriable_failed_events: failed_validation_events,
            });
        }
        let body = IngestionRequestBody {
            eppo_events: events_to_deliver,
        };
        let serialized_body = serde_json::to_vec(&body).map_err(|e| JsonSerializationError(e))?;
        let response = self
            .client
            .post(ingestion_url)
            .header("X-Eppo-Token", sdk_key.as_str())
            .body(serialized_body)
            .send()
            .await
            .map_err(EventDeliveryError::RetriableError)?;
        let response = response.error_for_status().map_err(|err| {
            return if err.status() == Some(StatusCode::UNAUTHORIZED) {
                // This error is not-retriable
                log::warn!(target: "eppo", "client is not authorized. Check your API key");
                EventDeliveryError::NonRetriableError(err)
            } else if err.status() == Some(StatusCode::BAD_REQUEST) {
                // This error is not-retriable
                log::warn!(target: "eppo", "received 400 response delivering events: {:?}", err);
                EventDeliveryError::NonRetriableError(err)
            } else {
                // Other errors **might be** retriable
                log::warn!(target: "eppo", "received non-200 response delivering events: {:?}", err);
                EventDeliveryError::RetriableError(err)
            }
        })?;
        let response = response
            .json::<RawEventDeliveryResponse>()
            .await
            .map_err(|e| JsonDeserializationError(e))?;
        info!(
            "Batch delivered successfully, {} events failed ingestion",
            response.failed_events.len()
        );
        Ok(EventDeliveryResponse {
            retriable_failed_events: response.failed_events,
            non_retriable_failed_events: failed_validation_events,
        })
    }
}

/// Returns whether the provided event's serialized JSON string is over the length limit
fn ensure_max_event_size(event: &Event) -> Result<bool, EventDeliveryError> {
    let serialized_event = serde_json::to_vec(event).map_err(|e| JsonSerializationError(e))?;
    Ok(serialized_event.len() < MAX_EVENT_SERIALIZED_LENGTH)
}

#[cfg(test)]
mod tests {
    use crate::event_ingestion::event::Event;
    use crate::event_ingestion::event_delivery::{
        EventDelivery, EventDeliveryResponse, MAX_EVENT_SERIALIZED_LENGTH,
    };
    use crate::timestamp::now;
    use crate::Str;
    use serde_json::json;
    use std::collections::{HashMap, HashSet};
    use url::Url;
    use uuid::Uuid;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Test that an event over-4096-byte serialized length triggers a EventPayloadTooLargeError.
    #[tokio::test]
    async fn test_deliver_fails_on_large_payload() {
        let client = EventDelivery::new(
            Str::from("foobar"),
            Url::parse("https://example.com").unwrap(),
        );
        // Create an event that will produce a large JSON string.
        // Just repeat "A" enough times that JSON definitely exceeds 4096 bytes.
        let huge_string = "A".repeat(MAX_EVENT_SERIALIZED_LENGTH + 1);
        let large_event = new_test_event(huge_string);
        let events = vec![large_event.clone()];
        let result = client.deliver(events).await;
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        assert_eq!(
            result.unwrap(),
            EventDeliveryResponse {
                non_retriable_failed_events: HashSet::from([large_event.uuid]),
                retriable_failed_events: HashSet::new()
            }
        )
    }

    /// Test that an event serialized size **just** under 4096 bytes succeeds.
    #[tokio::test]
    async fn test_deliver_succeeds_on_small_payload() {
        let response_body = ResponseTemplate::new(200).set_body_json(json!({"failed_events": []}));
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(response_body)
            .mount(&mock_server)
            .await;
        let client = EventDelivery::new(
            Str::from("foobar"),
            Url::parse(mock_server.uri().as_str()).unwrap(),
        );
        let small_event = new_test_event("A".repeat(3500));
        let events = vec![small_event];
        let result = client.deliver(events).await;
        // Should be ok because payload is not over MAX_EVENT_PAYLOAD_SIZE
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        assert_eq!(
            result.unwrap(),
            EventDeliveryResponse::empty()
        )
    }

    fn new_test_event(user_id: String) -> Event {
        let payload: HashMap<&str, String> = HashMap::from([
            ("user_id", user_id),
            ("session_id", "session456".to_string()),
        ]);
        let serialized_payload = serde_json::to_value(payload).expect("Serialization failed");
        Event {
            uuid: Uuid::new_v4(),
            timestamp: now(),
            event_type: "test".to_string(),
            payload: serialized_payload,
        }
    }
}
