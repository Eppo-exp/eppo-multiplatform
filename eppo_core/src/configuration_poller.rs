use std::{sync::Arc, time::Duration};

use rand::{thread_rng, Rng};
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

#[cfg(not(target_arch = "wasm32"))]
use tokio::time::sleep;
#[cfg(target_arch = "wasm32")]
use wasmtimer::tokio::sleep;

use crate::{
    background::{AsyncRuntime, BackgroundRuntime},
    configuration_fetcher::ConfigurationFetcher,
    configuration_store::ConfigurationStore,
    Error,
};

/// Configuration for [`configuration_poller`].
// Not implementing `Copy` as we may add non-copyable fields in the future.
#[derive(Debug, Clone)]
pub struct ConfigurationPollerConfig {
    /// Interval to wait between requests for configuration.
    ///
    /// Defaults to [`ConfigurationPollerConfig::DEFAULT_POLL_INTERVAL`].
    pub interval: Duration,
    /// Jitter applies a randomized duration to wait between requests for configuration. This helps
    /// to avoid multiple server instances synchronizing and producing spiky network load.
    ///
    /// Defaults to [`ConfigurationPollerConfig::DEFAULT_POLL_JITTER`].
    pub jitter: Duration,
}

impl ConfigurationPollerConfig {
    /// Default value for [`ConfigurationPollerConfig::interval`].
    pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(30);
    /// Default value for [`ConfigurationPollerConfig::jitter`].
    pub const DEFAULT_POLL_JITTER: Duration = Duration::from_secs(3);

    /// Create a new `ConfigurationPollerConfig` using default configuration.
    pub fn new() -> ConfigurationPollerConfig {
        ConfigurationPollerConfig::default()
    }

    /// Update poll interval with `interval`.
    pub fn with_interval(mut self, interval: Duration) -> ConfigurationPollerConfig {
        self.interval = interval;
        self
    }

    /// Update poll interval jitter with `jitter`.
    pub fn with_jitter(mut self, jitter: Duration) -> ConfigurationPollerConfig {
        self.jitter = jitter;
        self
    }
}

impl Default for ConfigurationPollerConfig {
    fn default() -> ConfigurationPollerConfig {
        ConfigurationPollerConfig {
            interval: ConfigurationPollerConfig::DEFAULT_POLL_INTERVAL,
            jitter: ConfigurationPollerConfig::DEFAULT_POLL_JITTER,
        }
    }
}

#[derive(Debug)]
pub struct ConfigurationPoller {
    status: watch::Receiver<Option<Result<(), crate::Error>>>,
    cancellation_token: CancellationToken,
}

impl ConfigurationPoller {
    pub async fn wait_for_configuration(&self) -> Result<(), crate::Error> {
        let mut status_rx = self.status.clone();
        let status = status_rx
            .wait_for(|status| status.is_some())
            .await
            .map_err(|_| Error::PollerThreadPanicked)?;
        status
            .as_ref()
            .cloned()
            .expect("option should always be Some because it's checked in .wait_for()")
    }

    pub fn stop(&self) {
        self.cancellation_token.cancel();
    }
}

pub fn start_configuration_poller<AR: AsyncRuntime>(
    runtime: &BackgroundRuntime<AR>,
    fetcher: ConfigurationFetcher,
    store: Arc<ConfigurationStore>,
    config: ConfigurationPollerConfig,
) -> ConfigurationPoller {
    // Note: even though configuration poller listens to a cancellation token, it doesn't have any
    // special cleanup requirements, so we use `spawn_untracked()`. The cancellation token is used
    // to allow stopping the poller without stopping the rest of background runtime.
    #[cfg(not(target_arch = "wasm32"))]
    let spawn = |f| runtime.spawn_untracked(f);

    // On wasm32, reqwest is non-send, so we can't use normal spawn.
    #[cfg(target_arch = "wasm32")]
    let spawn = wasm_bindgen_futures::spawn_local;

    let (status_tx, status_rx) = watch::channel(None);

    let cancellation_token = runtime.cancellation_token();
    log::info!(target: "eppo", "starting configuration poller");
    spawn({
        let cancellation_token = cancellation_token.clone();
        async move {
            cancellation_token
                .run_until_cancelled(configuration_poller(fetcher, store, config, status_tx))
                .await;
        }
    });

    ConfigurationPoller {
        status: status_rx,
        cancellation_token,
    }
}

/// Polls periodically for `Configuration` using `fetcher` and stores it in a `store`. Additionally,
/// it reports its current status (successfully fetched configuration or error occurred) to
/// `status`.
async fn configuration_poller(
    mut fetcher: ConfigurationFetcher,
    store: Arc<ConfigurationStore>,
    config: ConfigurationPollerConfig,
    // TODO: This option-result is somewhat convoluted. Maybe remodel this with an explicit
    // ConfigurationPollerStatus enum.
    status: watch::Sender<Option<Result<(), crate::Error>>>,
) {
    let update_status = move |next: Result<(), crate::Error>| {
        status.send_if_modified(|value| {
            let update = value.as_ref().is_none()
                || value
                    .as_ref()
                    .is_some_and(|prev| prev.is_ok() != next.is_ok());
            if update {
                *value = Some(next);
            }
            update
        });
    };

    loop {
        match fetcher.fetch_configuration().await {
            Ok(configuration) => {
                store.set_configuration(Arc::new(configuration));
                update_status(Ok(()));
            }
            Err(err @ (Error::Unauthorized | Error::InvalidBaseUrl(_))) => {
                // These errors are not recoverable. Update result and exit the poller.
                update_status(Err(Error::from(err)));
                return;
            }
            _ => {
                // Other errors are retriable.
            }
        }

        let timeout = jitter(config.interval, config.jitter);

        sleep(timeout).await;
    }
}

/// Apply randomized `jitter` to `interval`.
fn jitter(interval: Duration, jitter: Duration) -> Duration {
    Duration::saturating_sub(interval, thread_rng().gen_range(Duration::ZERO..=jitter))
}

#[cfg(test)]
mod jitter_tests {
    use std::time::Duration;

    #[test]
    fn jitter_is_subtractive() {
        let interval = Duration::from_secs(30);
        let jitter = Duration::from_secs(30);

        let result = super::jitter(interval, jitter);

        assert!(result <= interval, "{result:?} must be <= {interval:?}");
    }

    #[test]
    fn jitter_truncates_to_zero() {
        let interval = Duration::ZERO;
        let jitter = Duration::from_secs(30);

        let result = super::jitter(interval, jitter);

        assert_eq!(result, Duration::ZERO);
    }

    #[test]
    fn jitter_works_with_zero_jitter() {
        let interval = Duration::from_secs(30);
        let jitter = Duration::ZERO;

        let result = super::jitter(interval, jitter);

        assert_eq!(result, Duration::from_secs(30));
    }
}
