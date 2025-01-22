use crate::event_ingestion::batched_message::BatchedMessage;
use crate::event_ingestion::delivery::QueuedBatch;
use crate::event_ingestion::queued_event::QueuedEvent;
use exponential_backoff::Backoff;
use std::time::Duration;
use log::warn;
use tokio::sync::mpsc;

/// Retry events that failed to be delivered through `retry_downlink`, forwards remaining events to
/// `delivery_status`.
pub(super) async fn retry(
    mut uplink: mpsc::Receiver<QueuedBatch>,
    retry_downlink: mpsc::Sender<BatchedMessage<QueuedEvent>>,
    delivery_status: mpsc::Sender<QueuedBatch>,
    max_retries: u32,
    min_retry_duration: Duration,
    max_retry_delay: Duration,
) -> Option<()> {
    loop {
        let QueuedBatch {
            retry,
            success,
            failure,
        } = uplink.recv().await?;
        if !retry.is_empty() {
            // take the number of attempts from the first event in the batch to determine the
            // exponential backoff delay
            let attempts = retry[0].attempts as usize;
            if wait_exponential_backoff(attempts, max_retries, min_retry_duration, max_retry_delay).await {
                retry_downlink
                    .send(BatchedMessage::new(retry.clone(), None))
                    .await
                    .ok()?;
            } else {
                continue;
            }
        }
        delivery_status
            .send(QueuedBatch::new(success, failure, retry))
            .await
            .ok()?;
    }
}

async fn wait_exponential_backoff(
    attempts: usize,
    max_retries: u32,
    min_retry_duration: Duration,
    max_retry_delay: Duration,
) -> bool {
    let backoff = Backoff::new(max_retries + 1, min_retry_duration, max_retry_delay);
    let delay = backoff.iter().skip(attempts - 1).take(1).next().flatten();
    if let Some(delay) = delay {
        tokio::time::sleep(delay).await;
        true
    } else {
        warn!("Failed to wait for exponential backoff with {} attempts and {} max_retries", attempts, max_retries);
        false
    }
}
