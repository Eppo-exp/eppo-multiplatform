use std::future::Future;

use tokio::task::JoinHandle;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

/// `BackgroundRuntime` is a tokio runtime that is used to run background activities with controlled
/// shutdown. It may also be occasionally used to execute async future in sync context (e.g., from
/// user's thread where we don't assume that tokio runtime is available).
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

    pub(crate) fn tokio_handle(&self) -> &tokio::runtime::Handle {
        &self.tokio_runtime
    }

    #[must_use]
    pub(crate) fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
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

    /// Wait for all background activities to stop.
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
