use std::future::Future;

use tokio::task::JoinHandle;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

/// `BackgroundRuntime` has two goals:
/// - Allow executing different background tasks concurrently on a single tokio runtime.
/// - Handle graceful shutdown.
///
/// The first goal is achieved by holding a (non-owning) handle to an existing tokio
/// runtime. `BackgroundRuntime` does not actually create/own/drive the tokio runtime and that needs
/// to be done by someone else (see [`BackgroundThread`](super::BackgroundThread) for example). This
/// is to make `BackgroundRuntime` more flexible as not all environments support multi-threading
/// (e.g., wasm) and some may already have a global runtime users might want to reuse.
///
/// The graceful shutdown is achieved by combining [`CancellationToken`] and [`TaskTracker`]. Their
/// usage is described in [tokio tutorial](https://tokio.rs/tokio/topics/shutdown).
///
/// Because graceful shutdown requires cooperation with the task (it needs to watch and react to
/// cancellation token) and most tasks don't actually have any cleanup requirements, we distinguish
/// between "tracked" and "untracked" tasks. Tracked tasks are the tasks that have cleanup
/// requirements (e.g., to ensure files are fsync'ed) and are waited by the runtime before
/// exiting. Untracked tasks are everything else and may get stopped and dropped during runtime
/// shutdown.
///
/// When `BackgroundRuntime` is dropped, all background activities are commanded to stop.
pub struct BackgroundRuntime {
    tokio_runtime: tokio::runtime::Handle,
    /// A cancellation token that gets cancelled when runtime needs to exit.
    cancellation_token: CancellationToken,
    /// A set of tasks that are required to exit before the tokio runtime can be safely
    /// stopped. Rust futures are usually safe to drop, so this is normally not needed. But we may
    /// need this occasionally (e.g., finish writes to disk, etc.)
    watched_tasks: TaskTracker,
}

impl BackgroundRuntime {
    /// Creates a new `BackgroundRuntime` that runs on the given tokio `runtime`.
    ///
    /// The background runtime is active until `stop()` is called.
    #[must_use]
    pub fn new(runtime: tokio::runtime::Handle) -> BackgroundRuntime {
        let cancellation_token = CancellationToken::new();
        let watched_tasks = TaskTracker::new();
        BackgroundRuntime {
            tokio_runtime: runtime,
            cancellation_token,
            watched_tasks,
        }
    }

    #[must_use]
    pub(crate) fn tokio_handle(&self) -> &tokio::runtime::Handle {
        &self.tokio_runtime
    }

    /// Returns a cancellation token that get cancelled when background runtime is going to
    /// shutdown.
    ///
    /// If you want your task to react to shutdown and perform some cleanup, spawn your task with
    /// [`BackgroundThread::spawn_tracked()`] and watch this cancellation token.
    ///
    /// Note: this is a child token of the main cancellation token, so canceling it does not cancel
    /// the whole runtime.
    #[must_use]
    pub(crate) fn cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.child_token()
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.tokio_runtime.block_on(future)
    }

    /// Spawn a task that needs to perform some cleanup on shutdown.
    ///
    /// Most tasks shouldn't need that as Rust futures are usually safe to drop.
    ///
    /// The task must monitor [`BackgroundRuntime::cancellation_token()`] and exit when the token is
    /// cancelled.
    pub(crate) fn spawn_tracked<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tokio_runtime
            .spawn(self.watched_tasks.track_future(future))
    }

    /// Spawn a task that doesn't have any special shutdown requirements.
    ///
    /// When runtime is going to shutdown, this task will not be awaited and will be abandoned.
    ///
    /// If it's not OK to abandon the task, consider using `spawn_tracked()` instead.
    pub(crate) fn spawn_untracked<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tokio_runtime.spawn(future)
    }

    /// Command background activities to stop and exit.
    pub(crate) fn stop(&self) {
        self.watched_tasks.close();
        self.cancellation_token.cancel();
    }

    /// Wait for the background runtime to stop. (It has to be stopped with
    /// [`BackgroundRuntime::stop()`].)
    ///
    /// This is intended to be used by a tokio runtime driver (e.g., [`BackgroundThread`]) to
    /// determine when we're done and tokio runtime can be stopped.
    pub(super) fn wait(&self) -> impl Future {
        let tracker = self.watched_tasks.clone();
        async move { tracker.wait().await }
    }
}

impl Drop for BackgroundRuntime {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        time::Duration,
    };

    use super::*;

    #[test]
    fn test_start_stop() {
        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let background_runtime = BackgroundRuntime::new(tokio_runtime.handle().clone());

        // Without us calling .stop(), .block_on() below would block indefinitely.
        background_runtime.stop();

        tokio_runtime.block_on(background_runtime.wait());

        // Checking that the above works and stops as expected.
    }

    #[test]
    fn test_stops_with_uncooperative_task() {
        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let background_runtime = BackgroundRuntime::new(tokio_runtime.handle().clone());

        // This task is not cooperative and never finishes. It's spawned as "untracked" though, so
        // background runtime doesn't wait for it to finish.
        background_runtime.spawn_untracked(async {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        background_runtime.stop();

        tokio_runtime.block_on(background_runtime.wait());

        // Checking that the above works and stops as expected.
    }

    #[test]
    fn test_waits_for_tracked_task_to_finish() {
        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let background_runtime = BackgroundRuntime::new(tokio_runtime.handle().clone());

        let finished_cleanly = Arc::new(AtomicBool::new(false));

        let cancellation_token = background_runtime.cancellation_token();
        background_runtime.spawn_tracked({
            let finished_cleanly = finished_cleanly.clone();
            async move {
                loop {
                    tokio::select! {
                        _ = cancellation_token.cancelled() => {
                            finished_cleanly.store(true, Ordering::Relaxed);
                            return;
                        },
                        _ = tokio::time::sleep(Duration::from_secs(1)) => {
                            // Continue looping.
                        },


                    }
                }
            }
        });

        background_runtime.stop();

        tokio_runtime.block_on(background_runtime.wait());

        // Asserting that background runtime waited for tracked task to exit cleanly.
        assert!(finished_cleanly.load(Ordering::Relaxed));
    }

    #[test]
    fn test_stops_by_dropping() {
        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let background_runtime = BackgroundRuntime::new(tokio_runtime.handle().clone());

        let thread = std::thread::spawn({
            let wait = background_runtime.wait();
            move || {
                tokio_runtime.block_on(wait);
            }
        });

        drop(background_runtime);

        thread.join().unwrap();

        // Checking that everything stopped and the thread joined.
    }
}
