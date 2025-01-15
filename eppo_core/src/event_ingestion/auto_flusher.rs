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
                    downlink.send(BatchedMessage { batch: Vec::new(), flush: Some(Vec::new()) }).await.ok()?;
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
