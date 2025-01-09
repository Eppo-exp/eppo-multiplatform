use crate::events::event::Event;
use crate::Error;
use log::{debug, info};
use reqwest::{StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct EventDelivery {
    sdk_key: String,
    ingestion_url: String,
    client: reqwest::Client,
}

#[derive(serde::Deserialize)]
pub struct EventDeliveryResponse {
    pub failed_events: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
struct IngestionRequestBody {
    eppo_events: Vec<Event>,
}

/// Responsible for delivering event batches to the Eppo ingestion service.
impl EventDelivery {
    pub fn new(sdk_key: String, ingestion_url: String) -> Self {
        let client = reqwest::Client::new();
        EventDelivery {
            sdk_key,
            ingestion_url,
            client,
        }
    }

    // Delivers the provided event batch and returns a Vec with the events that failed to be delivered.
    pub async fn deliver(self, events: &[Event]) -> Result<EventDeliveryResponse, Error> {
        let ingestion_url = self.ingestion_url;
        let sdk_key = self.sdk_key;
        debug!("Delivering {} events to {}", events.len(), ingestion_url);
        let body = IngestionRequestBody { eppo_events: events.to_vec() };
        let response = self.client.post(ingestion_url)
            .header("X-Eppo-Token", sdk_key)
            .json(&body)
            .send()
            .await?;
        let response = response.error_for_status().map_err(|err| {
            return if err.status() == Some(StatusCode::UNAUTHORIZED) {
                log::warn!(target: "eppo", "client is not authorized. Check your API key");
                Error::Unauthorized
            } else {
                log::warn!(target: "eppo", "received non-200 response while fetching new configuration: {:?}", err);
                Error::from(err)
            }
        })?;
        let response = response.json::<EventDeliveryResponse>().await?;
        info!("Batch delivered successfully, {} events failed ingestion", response.failed_events.len());
        Ok(response)
    }
}
