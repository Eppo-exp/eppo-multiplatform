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

#[cfg(test)]
mod tests {
    use crate::event_ingestion::batched_message::BatchedMessage;
    use crate::event_ingestion::{auto_flusher, batcher};

    #[tokio::test]
    async fn test_auto_flusher_and_batcher_pipeline() {
        use tokio::sync::mpsc;
        use tokio::time::{self, Duration};

        // Define test parameters
        let flush_period = Duration::from_millis(100);
        let min_batch_size = 5;

        // Channels for the pipeline
        let (flusher_uplink_tx, flusher_uplink_rx) = mpsc::channel(10);
        let (flusher_downlink_tx, flusher_downlink_rx) = mpsc::channel(10);
        let (batcher_downlink_tx, mut batcher_downlink_rx) = mpsc::channel(10);

        // Spawn the auto_flusher and batcher
        tokio::spawn(auto_flusher::auto_flusher(
            flusher_uplink_rx,
            flusher_downlink_tx,
            flush_period,
        ));
        tokio::spawn(batcher::batcher(
            flusher_downlink_rx,
            batcher_downlink_tx,
            min_batch_size,
        ));

        // Send some messages to the flusher uplink
        flusher_uplink_tx
            .send(BatchedMessage {
                batch: vec![1, 2, 3],
                flush: None,
            })
            .await
            .unwrap();
        flusher_uplink_tx
            .send(BatchedMessage {
                batch: vec![4],
                flush: None,
            })
            .await
            .unwrap();

        // Verify that the batcher does not output anything until the minimum batch size is met
        assert_eq!(
            batcher_downlink_rx.try_recv(),
            Err(mpsc::error::TryRecvError::Empty)
        );

        // Wait for the auto_flusher to send a flush message
        time::sleep(flush_period * 2).await;

        // Verify that the batcher outputs a batch due to the flush
        assert_eq!(
            batcher_downlink_rx.recv().await,
            Some(BatchedMessage {
                batch: vec![1, 2, 3, 4],
                flush: Some(()),
            })
        );

        // Send additional messages to the flusher
        flusher_uplink_tx
            .send(BatchedMessage {
                batch: vec![5, 6, 7, 8, 9],
                flush: None,
            })
            .await
            .unwrap();

        // Verify that the batcher outputs a batch once the minimum batch size is met
        assert_eq!(
            batcher_downlink_rx.recv().await,
            Some(BatchedMessage {
                batch: vec![5, 6, 7, 8, 9],
                flush: None,
            })
        );

        // Simulate uplink closure
        flusher_uplink_tx
            .send(BatchedMessage {
                batch: vec![10],
                flush: None,
            })
            .await
            .unwrap();
        drop(flusher_uplink_tx);

        // Verify that the batcher flushes the remaining data on uplink closure
        assert_eq!(
            batcher_downlink_rx.recv().await,
            Some(BatchedMessage {
                batch: vec![10],
                flush: None,
            })
        );

        // Verify that the batcher exits cleanly
        assert_eq!(batcher_downlink_rx.recv().await, None);
    }
}
