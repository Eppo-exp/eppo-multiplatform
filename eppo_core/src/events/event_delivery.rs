use crate::events::event::Event;
use crate::Error;
use log::info;
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};

pub struct EventDelivery {
    sdk_key: String,
    ingestion_url: String,
    client: reqwest::Client,
}

#[derive(serde::Deserialize)]
struct EventDeliveryResponse {
    failed_events: Vec<String>,
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
    pub async fn deliver(self, events: &[Event]) -> Result<Vec<String>, Error> {
        let ingestion_url = self.ingestion_url;
        let sdk_key = self.sdk_key;
        debug!("Delivering {} events to {}", events.len(), ingestion_url);
        let body = IngestionRequestBody { eppo_events: events.to_vec() };
        let serialized_body = serde_json::to_string(&body).expect("Failed to serialize body");
        let response = self.client.request(Method::POST, ingestion_url)
            .header("Content-Type", "application/json")
            .header("X-Eppo-Token", sdk_key)
            .body(serialized_body)
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
        Ok(response.failed_events)
    }
}
