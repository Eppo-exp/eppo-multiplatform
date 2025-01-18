use tokio::sync::mpsc;
use crate::event_ingestion::batched_message::BatchedMessage;
use crate::event_ingestion::delivery::QueuedBatch;
use crate::event_ingestion::queued_event::QueuedEvent;

pub(super) struct FinishedBatch {
    pub success: Vec<QueuedEvent>,
    pub failure: Vec<QueuedEvent>,
}

/// Retry events that failed to be delivered through `retry_downlink`, forwards remaining events to
/// `delivery_status`.
pub(super) async fn retry(
    mut uplink: mpsc::Receiver<QueuedBatch>,
    retry_downlink: mpsc::Sender<BatchedMessage<QueuedEvent>>,
    delivery_status: mpsc::Sender<FinishedBatch>,
) -> Option<()> {
    loop {
        let QueuedBatch { retry, success, failure } = uplink.recv().await?;
        if !retry.is_empty() {
            retry_downlink.send(BatchedMessage::new(retry, None)).await.ok()?;
        }
        // forward remaining events to delivery
        delivery_status.send(FinishedBatch { success, failure }).await.ok()?;
    }
}
