/// Batched message contain a batch of data and may optionally require processors to flush any processing.
#[derive(Debug)]
pub(super) struct BatchedMessage<T> {
    pub batch: Vec<T>,
    /// `None` means the message does not require a flush.
    /// `Some` contains a list of watchers.
    pub flush: Option<Vec<tokio::sync::oneshot::Sender<()>>>,
}

impl<T> BatchedMessage<T> {
    /// Create a new empty message.
    pub fn empty() -> BatchedMessage<T> {
        BatchedMessage {
            batch: Vec::new(),
            flush: None,
        }
    }

    pub fn requires_flush(&self) -> bool {
        self.flush.is_some()
    }

    /// Mark the message as successfully flushed, consuming it and notifying any interested parties.
    pub fn flushed(self) {
        if let Some(flush) = self.flush {
            for f in flush {
                f.send(());
            }
        }
    }
}
