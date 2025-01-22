use super::BatchedMessage;
use crate::event_ingestion::event_delivery::{
    EventDelivery, EventDeliveryError, EventDeliveryResponse,
};
use crate::event_ingestion::queued_event::QueuedEvent;
use log::warn;
use tokio::sync::mpsc;

#[derive(Debug, PartialEq)]
pub(super) struct QueuedBatch {
    pub success: Vec<QueuedEvent>,
    pub failure: Vec<QueuedEvent>,
    pub retry: Vec<QueuedEvent>,
}

impl QueuedBatch {
    pub fn new(
        success: Vec<QueuedEvent>,
        failure: Vec<QueuedEvent>,
        retry: Vec<QueuedEvent>,
    ) -> Self {
        QueuedBatch {
            success,
            failure,
            retry,
        }
    }

    pub fn success(success: Vec<QueuedEvent>) -> Self {
        QueuedBatch {
            success,
            failure: Vec::new(),
            retry: Vec::new(),
        }
    }

    pub fn failure(failure: Vec<QueuedEvent>) -> Self {
        QueuedBatch {
            success: Vec::new(),
            retry: Vec::new(),
            failure,
        }
    }

    pub fn retry(retry: Vec<QueuedEvent>) -> Self {
        QueuedBatch {
            success: Vec::new(),
            retry,
            failure: Vec::new(),
        }
    }
}

pub(super) async fn delivery(
    mut uplink: mpsc::Receiver<BatchedMessage<QueuedEvent>>,
    delivery_status: mpsc::Sender<QueuedBatch>,
    max_retries: u32,
    event_delivery: EventDelivery,
) -> Option<()> {
    loop {
        let BatchedMessage {
            batch,
            flush: _flush,
        } = uplink.recv().await?;
        if batch.is_empty() {
            continue;
        }
        let events_to_deliver = batch
            .clone()
            .into_iter()
            .map(|queued_event| queued_event.event)
            .collect();
        let result = event_delivery.deliver(events_to_deliver).await;
        match result {
            Ok(response) => {
                let delivery_status_data = collect_delivery_response(batch, response, max_retries);
                deliver_status(&delivery_status, delivery_status_data).await;
            }
            Err(err) => {
                match err {
                    EventDeliveryError::RetriableError(_) => {
                        // Retry later
                        deliver_status(&delivery_status, QueuedBatch::failure(batch)).await;
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

fn collect_delivery_response(
    batch: Vec<QueuedEvent>,
    response: EventDeliveryResponse,
    max_retries: u32,
) -> QueuedBatch {
    let failed_event_uuids = response.failed_events;
    if failed_event_uuids.is_empty() {
        return QueuedBatch::success(batch);
    }
    warn!("Failed to deliver {} events", failed_event_uuids.len());
    let mut success = Vec::new();
    let mut failure = Vec::new();
    let mut retry = Vec::new();
    for queued_event in batch {
        if failed_event_uuids.contains(&queued_event.event.uuid) {
            if queued_event.attempts < max_retries {
                // increment failed attempts count and retry
                retry.push(QueuedEvent::new_from_failed(queued_event));
            } else {
                // max retries reached, mark as failed
                failure.push(QueuedEvent::new_from_failed(queued_event));
            }
        } else {
            success.push(queued_event);
        }
    }
    QueuedBatch {
        success,
        failure,
        retry,
    }
}

async fn deliver_status(receiver: &mpsc::Sender<QueuedBatch>, status: QueuedBatch) {
    receiver.send(status).await.unwrap_or_else(|err| {
        warn!("Failed to send delivery status: {}", err);
    });
}
