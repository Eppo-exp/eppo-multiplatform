use super::{BatchedMessage, Event};
use crate::event_ingestion::event_delivery::{EventDelivery, EventDeliveryError};
use log::warn;
use tokio::sync::mpsc;

pub(super) struct DeliveryStatus {
    success: Vec<Event>,
    failure: Vec<Event>,
}

pub(super) async fn delivery(
    mut uplink: mpsc::Receiver<BatchedMessage<Event>>,
    delivery_status: mpsc::Sender<DeliveryStatus>,
    event_delivery: EventDelivery,
) -> Option<()> {
    loop {
        let event_delivery = event_delivery.clone();
        match uplink.recv().await {
            None => {
                return None;
            }
            Some(BatchedMessage {
                batch,
                flush: _flush,
            }) => {
                let result = event_delivery.deliver(batch.clone()).await;
                match result {
                    Ok(response) => {
                        let failed_event_uuids = response.failed_events;
                        if !failed_event_uuids.is_empty() {
                            warn!("Failed to deliver {} events", failed_event_uuids.len());
                            let mut success = Vec::new();
                            let mut failure = Vec::new();
                            batch.into_iter().for_each(|queued_event| {
                                if failed_event_uuids.contains(&queued_event.uuid) {
                                    failure.push(queued_event);
                                } else {
                                    success.push(queued_event);
                                }
                            });
                            delivery_status
                                .send(DeliveryStatus { success, failure })
                                .await
                                .ok()?;
                        }
                    }
                    Err(err) => {
                        match err {
                            EventDeliveryError::RetriableError(_) => {
                                // Retry later
                                delivery_status
                                    .send(DeliveryStatus {
                                        failure: batch,
                                        success: Vec::new(),
                                    })
                                    .await
                                    .ok()?;
                            }
                            EventDeliveryError::NonRetriableError(_) => {
                                warn!("Failed to deliver events: {}", err);
                                // In this case there is no point in retrying delivery since the error is
                                // non-retriable.
                            }
                        }
                    }
                }
            }
        }
    }
}
