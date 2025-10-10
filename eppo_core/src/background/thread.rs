use std::sync::Mutex;

use super::runtime::BackgroundRuntime;

/// An owning handle to a background thread running tokio runtime.
///
/// When the handle is dropped, the tokio runtime is commanded to exit and the thread shuts down.
pub struct BackgroundThread {
    join_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
    runtime: BackgroundRuntime<tokio::runtime::Handle>,
}

impl BackgroundThread {
    /// Spawns a new thread and runs a single-threaded background runtime on it. Shuts down the
    /// thread when background runtime completes its shutdown.
    pub fn start() -> std::io::Result<BackgroundThread> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()?;

        let background_runtime = BackgroundRuntime::new(runtime.handle().clone());

        let wait = background_runtime.wait();
        let join_handle = std::thread::Builder::new()
            .name("eppo-background".to_owned())
            .spawn(move || {
                log::info!(target: "eppo", "BackgroundThread: started");
                runtime.block_on(wait);
                log::info!(target: "eppo", "BackgroundThread: exiting");
            })?;

        Ok(BackgroundThread {
            join_handle: Mutex::new(Some(join_handle)),
            runtime: background_runtime,
        })
    }

    pub fn runtime(&self) -> &BackgroundRuntime<tokio::runtime::Handle> {
        &self.runtime
    }

    /// Command the associated background thread to exit (without waiting for it to complete).
    ///
    /// Prefer `shutdown()` if you have the time to wait.
    pub fn kill(&self) {
        self.runtime.stop();
    }

    /// Command background activities to stop and wait for thread to terminate.
    pub fn shutdown(&self) {
        self.runtime.stop();

        let join_handle = {
            // scope to keep mutex lock short
            let Ok(mut join_handle) = self.join_handle.lock() else {
                return;
            };
            join_handle.take()
        };

        if let Some(join_handle) = join_handle {
            let _ = join_handle.join();
        }
    }

    /// Command background activities to stop and wait for thread to terminate.
    #[deprecated]
    pub fn graceful_shutdown(self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_usage() {
        let background_thread = BackgroundThread::start().unwrap();

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        background_thread.runtime().spawn_untracked(async move {
            tx.send(true).unwrap();
        });

        let received = rx.blocking_recv().unwrap();

        assert_eq!(received, true);

        background_thread.shutdown();
    }
}
