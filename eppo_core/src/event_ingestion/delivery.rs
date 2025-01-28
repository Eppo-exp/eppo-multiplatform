use std::time::Duration;

use exponential_backoff::Backoff;
use tokio::sync::mpsc;

use super::{event::Event, event_delivery::EventDelivery, BatchedMessage};

#[derive(Debug, PartialEq)]
pub(super) struct DeliveryStatus {
    pub success: Vec<Event>,
    pub failure: Vec<Event>,
    pub retry: Vec<Event>,
}

impl DeliveryStatus {
    pub fn new(success: Vec<Event>, failure: Vec<Event>, retry: Vec<Event>) -> Self {
        DeliveryStatus {
            success,
            failure,
            retry,
        }
    }

    pub fn success(success: Vec<Event>) -> Self {
        DeliveryStatus {
            success,
            failure: Vec::new(),
            retry: Vec::new(),
        }
    }

    pub fn failure(failure: Vec<Event>) -> Self {
        DeliveryStatus {
            success: Vec::new(),
            retry: Vec::new(),
            failure,
        }
    }

    pub fn retry(retry: Vec<Event>) -> Self {
        DeliveryStatus {
            success: Vec::new(),
            retry,
            failure: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DeliveryConfig {
    pub max_retries: u32,
    pub base_retry_delay: Duration,
    pub max_retry_delay: Duration,
}

pub(super) async fn delivery(
    mut uplink: mpsc::Receiver<BatchedMessage<Event>>,
    delivery_status: mpsc::Sender<DeliveryStatus>,
    event_delivery: EventDelivery,
    config: DeliveryConfig,
) -> Option<()> {
    // We use this unbounded channel to loop back messages that need retrying.
    let (retries_tx, mut retries_rx) =
        mpsc::unbounded_channel::<(/* attempts: */ u32, BatchedMessage<Event>)>();

    loop {
        // Randomly select between sending a new batch or retrying an old one.
        let (attempts, msg) = tokio::select! {
            msg = uplink.recv() => (0, msg?),
            msg = retries_rx.recv() => msg?,
        };

        let BatchedMessage { batch, flush } = msg;

        let mut result = event_delivery.deliver(batch).await;

        if attempts >= config.max_retries {
            // Exceeded max retries -> promote retriable errors to permanent ones.
            result.failure.append(&mut result.retry);
        }

        let retry_batch = std::mem::take(&mut result.retry);

        let _ = delivery_status.send(result).await;

        if retry_batch.is_empty() {
            // We finished precessing the whole batch: it's either successfully delivered or failed
            // permanently.
            // TODO: flush
        } else {
            // Delay and re-insert retriable events.
            let retries_tx = retries_tx.clone();
            tokio::spawn(async move {
                wait_exponential_backoff(
                    attempts,
                    config.max_retries,
                    config.base_retry_delay,
                    config.max_retry_delay,
                )
                .await;
                let _ = retries_tx.send((
                    attempts + 1,
                    BatchedMessage {
                        batch: retry_batch,
                        flush,
                    },
                ));
            });
        }
    }
}

async fn wait_exponential_backoff(
    attempts: u32,
    max_retries: u32,
    min_retry_delay: Duration,
    max_retry_delay: Duration,
) {
    // TODO: exponential_backoff iterator interface doesn't really suit our usage. Rewrite backoff
    // calculation.
    let backoff = Backoff::new(max_retries + 1, min_retry_delay, max_retry_delay);
    let delay = backoff
        .iter()
        .skip(attempts as usize)
        .take(1)
        .next()
        .flatten()
        .unwrap_or(max_retry_delay);

    log::debug!(target: "eppo", "retry waiting for {:?}", delay);

    tokio::time::sleep(delay).await;
}
