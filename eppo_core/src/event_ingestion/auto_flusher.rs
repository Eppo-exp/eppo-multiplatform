use std::time::Duration;

use tokio::{sync::mpsc, time::Instant};

use super::BatchedMessage;

/// Auto-flusher forwards all messages from `uplink` to `downlink` unchanged and inserts extra flush
/// requests if it hasn't seen one within the given `period`. In other words, it makes sure that the
/// channel is flushed at least every `period`.
pub(super) async fn auto_flusher<T>(
    mut uplink: mpsc::Receiver<BatchedMessage<T>>,
    downlink: mpsc::Sender<BatchedMessage<T>>,
    period: Duration,
) -> Option<()> {
    'flushed: loop {
        // Process first message.
        let msg = uplink.recv().await?;
        let flushed = msg.flush.is_some();
        downlink.send(msg).await.ok()?;

        // No need to time if we just flushed.
        if flushed {
            continue;
        }

        let flush_at = Instant::now() + period;
        // loop till we reach flush_at or see a flushed message.
        loop {
            tokio::select! {
                _ =  tokio::time::sleep_until(flush_at) =>  {
                    downlink.send(BatchedMessage { batch: Vec::new(), flush: Some(()) }).await.ok()?;
                    continue 'flushed;
                },
                msg = uplink.recv() => {
                    let msg = msg?;
                    let flushed = msg.flush.is_some();
                    downlink.send(msg).await.ok()?;
                    if flushed {
                        continue 'flushed;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::event_ingestion::auto_flusher;
    use crate::event_ingestion::batched_message::BatchedMessage;
    use tokio::sync::mpsc;
    use tokio::time::Duration;

    #[tokio::test(start_paused = true)]
    async fn test_auto_flusher() {
        let (uplink_tx, uplink_rx) = mpsc::channel(10);
        let (downlink_tx, mut downlink_rx) = mpsc::channel(10);
        let flush_period = Duration::from_millis(100);
        tokio::spawn(auto_flusher::auto_flusher(
            uplink_rx,
            downlink_tx,
            flush_period,
        ));

        uplink_tx
            .send(BatchedMessage {
                batch: vec![1, 2, 3],
                flush: None,
            })
            .await
            .unwrap();
        uplink_tx
            .send(BatchedMessage {
                batch: vec![4, 5, 6],
                flush: None,
            })
            .await
            .unwrap();

        // Verify that the messages are forwarded to downlink
        assert_eq!(
            downlink_rx.recv().await,
            Some(BatchedMessage {
                batch: vec![1, 2, 3],
                flush: None
            })
        );
        assert_eq!(
            downlink_rx.recv().await,
            Some(BatchedMessage {
                batch: vec![4, 5, 6],
                flush: None
            })
        );

        // Wait for the flush period to trigger an auto-flush
        tokio::time::advance(flush_period * 2).await;

        // Verify the auto-flush behavior
        assert_eq!(
            downlink_rx.recv().await,
            Some(BatchedMessage {
                batch: Vec::new(),
                flush: Some(())
            })
        );

        // Send a flushed message explicitly
        uplink_tx
            .send(BatchedMessage {
                batch: vec![],
                flush: Some(()),
            })
            .await
            .unwrap();

        // Verify that the flushed message is forwarded immediately
        assert_eq!(
            downlink_rx.recv().await,
            Some(BatchedMessage {
                batch: vec![],
                flush: Some(())
            })
        );

        // Ensure the loop continues and processes further messages
        uplink_tx
            .send(BatchedMessage {
                batch: vec![7, 8, 9],
                flush: None,
            })
            .await
            .unwrap();
        assert_eq!(
            downlink_rx.recv().await,
            Some(BatchedMessage {
                batch: vec![7, 8, 9],
                flush: None
            })
        );
    }
}
