use tokio::sync::mpsc;

use super::BatchedMessage;

/// Batch messages, so they are at least `min_batch_size` size. Push incomplete batch down if flush
/// is received.
///
/// If uplink is closed, send all buffered data downstream and exit.
///
/// If downlink is closed, just exit.
pub(super) async fn batcher<T>(
    mut uplink: mpsc::Receiver<BatchedMessage<T>>,
    downlink: mpsc::Sender<BatchedMessage<T>>,
    min_batch_size: usize,
) -> Option<()> {
    let mut uplink_alive = true;
    while uplink_alive {
        let mut batch = BatchedMessage::empty();

        while uplink_alive && batch.batch.len() < min_batch_size && batch.flush.is_none() {
            match uplink.recv().await {
                None => {
                    uplink_alive = false;
                }
                Some(BatchedMessage {
                    batch: events,
                    flush,
                }) => {
                    batch.batch.extend(events);
                    batch.flush = flush;
                }
            }
        }

        downlink.send(batch).await.ok()?;
    }
    None
}
