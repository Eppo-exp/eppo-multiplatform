use super::BatchedMessage;
use crate::event_ingestion::event_delivery::{EventDelivery, EventDeliveryError};
use crate::event_ingestion::queued_event::QueuedEvent;
use log::warn;
use tokio::sync::mpsc;

pub(super) struct DeliveryStatus {
    pub success: Vec<QueuedEvent>,
    pub failure: Vec<QueuedEvent>,
}

impl DeliveryStatus {
    fn with_success(success: Vec<QueuedEvent>) -> Self {
        DeliveryStatus {
            success,
            failure: Vec::new(),
        }
    }

    fn with_failure(failure: Vec<QueuedEvent>) -> Self {
        DeliveryStatus {
            success: Vec::new(),
            failure,
        }
    }
}

pub(super) async fn delivery(
    mut uplink: mpsc::Receiver<BatchedMessage<QueuedEvent>>,
    delivery_status: mpsc::Sender<DeliveryStatus>,
    event_delivery: EventDelivery,
) -> Option<()> {
    loop {
        let event_delivery = event_delivery.clone();
        let BatchedMessage {
            batch,
            flush: _flush,
        } = uplink.recv().await?;
        if batch.is_empty() {
            continue;
        }
        let result = event_delivery
            .deliver(
                batch
                    .clone()
                    .into_iter()
                    .map(|queued_event| queued_event.event)
                    .collect(),
            )
            .await;
        match result {
            Ok(response) => {
                let failed_event_uuids = response.failed_events;
                if !failed_event_uuids.is_empty() {
                    warn!("Failed to deliver {} events", failed_event_uuids.len());
                    let mut success = Vec::new();
                    let mut failure = Vec::new();
                    for queued_event in batch {
                        if failed_event_uuids.contains(&queued_event.event.uuid) {
                            failure.push(QueuedEvent {
                                event: queued_event.event,
                                attempts: queued_event.attempts + 1,
                            });
                        } else {
                            success.push(queued_event);
                        }
                    }
                    deliver_status(&delivery_status, DeliveryStatus { success, failure }).await;
                } else {
                    deliver_status(&delivery_status, DeliveryStatus::with_success(batch)).await;
                }
            }
            Err(err) => {
                match err {
                    EventDeliveryError::RetriableError(_) => {
                        // Retry later
                        deliver_status(&delivery_status, DeliveryStatus::with_failure(batch)).await;
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

async fn deliver_status(receiver: &mpsc::Sender<DeliveryStatus>, status: DeliveryStatus) {
    receiver
        .send(status)
        .await
        .unwrap_or_else(|err| {
            warn!("Failed to send delivery status: {}", err);
        });
}
