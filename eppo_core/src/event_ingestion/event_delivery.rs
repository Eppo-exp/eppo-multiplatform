use crate::event_ingestion::event::Event;
use crate::Str;
use log::{debug, info};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;
use uuid::Uuid;

#[derive(Clone)]
pub(super) struct EventDelivery {
    sdk_key: Str,
    ingestion_url: Url,
    client: reqwest::Client,
}

#[derive(serde::Deserialize)]
pub(super) struct EventDeliveryResponse {
    pub failed_events: HashSet<Uuid>,
}

#[derive(thiserror::Error, Debug)]
pub(super) enum EventDeliveryError {
    #[error("Transient error delivering events")]
    RetriableError(reqwest::Error),
    #[error("Non-retriable error")]
    NonRetriableError(reqwest::Error),
}

#[derive(Debug, Serialize, Deserialize)]
struct IngestionRequestBody {
    eppo_events: Vec<Event>,
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
        events: Vec<Event>,
    ) -> Result<EventDeliveryResponse, EventDeliveryError> {
        let ingestion_url = self.ingestion_url.clone();
        let sdk_key = &self.sdk_key;
        debug!("Delivering {} events to {}", events.len(), ingestion_url);
        let body = IngestionRequestBody {
            eppo_events: events,
        };
        let response = self
            .client
            .post(ingestion_url)
            .header("X-Eppo-Token", sdk_key.as_str())
            .json(&body)
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
            .json::<EventDeliveryResponse>()
            .await
            .map_err(EventDeliveryError::NonRetriableError)?;
        info!(
            "Batch delivered successfully, {} events failed ingestion",
            response.failed_events.len()
        );
        Ok(response)
    }
}
