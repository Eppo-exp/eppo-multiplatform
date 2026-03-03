//! An HTTP client that fetches configuration from the server.
use reqwest::{StatusCode, Url};

use crate::{
    bandits::BanditResponse, ufc::UniversalFlagConfig, Configuration, Error, Result, SdkMetadata,
};

#[derive(Debug, PartialEq, Eq)]
pub struct ConfigurationFetcherConfig {
    pub base_url: String,
    pub api_key: String,
    pub sdk_metadata: SdkMetadata,
}

pub const DEFAULT_BASE_URL: &'static str = "https://fscdn.eppo.cloud/api";

const UFC_ENDPOINT: &'static str = "/flag-config/v1/config";
const BANDIT_ENDPOINT: &'static str = "/flag-config/v1/bandits";

/// A client that fetches Eppo configuration from the server.
pub struct ConfigurationFetcher {
    // Client holds a connection pool internally, so we're reusing the client between requests.
    client: reqwest::Client,
    config: ConfigurationFetcherConfig,
    /// If we receive a 401 Unauthorized error during a request, it means the API key is not
    /// valid. We cache this error so we don't issue additional requests to the server.
    unauthorized: bool,
}

impl ConfigurationFetcher {
    pub fn new(config: ConfigurationFetcherConfig) -> ConfigurationFetcher {
        let builder = reqwest::Client::builder();
        let client = match builder.build() {
            Err(e) => {
                panic!("Reqwest client build failed {:?}", e);
            }
            Ok(client) => client,
        };

        ConfigurationFetcher {
            client,
            config,
            unauthorized: false,
        }
    }

    pub async fn fetch_configuration(&mut self) -> Result<Configuration> {
        if self.unauthorized {
            return Err(Error::Unauthorized);
        }

        let ufc = self.fetch_ufc_configuration().await?;

        let bandits = if ufc.compiled.flag_to_bandit_associations.is_empty() {
            // We don't need bandits configuration if there are no bandits.
            None
        } else {
            Some(self.fetch_bandits_configuration().await?)
        };

        Ok(Configuration::from_server_response(ufc, bandits))
    }

    async fn fetch_ufc_configuration(&mut self) -> Result<UniversalFlagConfig> {
        let url = Url::parse_with_params(
            &format!("{}{}", self.config.base_url, UFC_ENDPOINT),
            &[
                ("apiKey", &*self.config.api_key),
                ("sdkName", self.config.sdk_metadata.name),
                ("sdkVersion", self.config.sdk_metadata.version),
                ("coreVersion", env!("CARGO_PKG_VERSION")),
            ],
        )
        .map_err(|err| {
            log::warn!(target: "eppo", "failed to parse flags configuration URL: {err}");
            Error::InvalidBaseUrl(err)
        })?;

        log::debug!(target: "eppo", "fetching UFC flags configuration");
        let response = self.client.get(url).send().await?;

        let response = response.error_for_status().map_err(|err| {
            if err.status() == Some(StatusCode::UNAUTHORIZED) {
                    log::warn!(target: "eppo", "client is not authorized. Check your API key");
                    self.unauthorized = true;
                    return Error::Unauthorized;
                } else {
                    let err = Error::from(err); // sanitize URL to avoid exposing SDK key
                    log::warn!(target: "eppo", "received non-200 response while fetching new configuration: {err}");
                    return err;

            }
        })?;

        let configuration = UniversalFlagConfig::from_json(
            self.config.sdk_metadata,
            response.bytes().await?.into(),
        )?;

        log::debug!(target: "eppo", "successfully fetched UFC flags configuration");

        Ok(configuration)
    }

    async fn fetch_bandits_configuration(&mut self) -> Result<BanditResponse> {
        let url = Url::parse_with_params(
            &format!("{}{}", self.config.base_url, BANDIT_ENDPOINT),
            &[
                ("apiKey", &*self.config.api_key),
                ("sdkName", self.config.sdk_metadata.name),
                ("sdkVersion", self.config.sdk_metadata.version),
                ("coreVersion", env!("CARGO_PKG_VERSION")),
            ],
        )
        .map_err(|err| {
            log::warn!(target: "eppo", "failed to parse bandits configuration URL: {err}");
            Error::InvalidBaseUrl(err)
        })?;

        log::debug!(target: "eppo", "fetching UFC bandits configuration");
        let response = self.client.get(url).send().await?;

        let response = response.error_for_status().map_err(|err| {
            if err.status() == Some(StatusCode::UNAUTHORIZED) {
                    log::warn!(target: "eppo", "client is not authorized. Check your API key");
                    self.unauthorized = true;
                    return Error::Unauthorized;
                } else {
                    let err = Error::from(err); // sanitize URL to avoid exposing SDK key
                    log::warn!(target: "eppo", "received non-200 response while fetching new configuration: {err}");
                    return err;

            }
        })?;

        let configuration = response.json().await?;

        log::debug!(target: "eppo", "successfully fetched UFC bandits configuration");

        Ok(configuration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error, SdkMetadata};
    use log::{Level, Log, Metadata, Record};
    use std::sync::{Arc, Mutex};
    use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

    // Simple logger that captures log messages
    static CAPTURED_LOGS: std::sync::OnceLock<Arc<Mutex<Vec<String>>>> = std::sync::OnceLock::new();

    struct TestLogger;

    unsafe impl Send for TestLogger {}
    unsafe impl Sync for TestLogger {}

    impl Log for TestLogger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            metadata.target() == "eppo"
        }

        fn log(&self, record: &Record) {
            if self.enabled(record.metadata()) {
                if let Some(logs) = CAPTURED_LOGS.get() {
                    let message = format!("{}", record.args());
                    logs.lock().unwrap().push(message);
                }
            }
        }

        fn flush(&self) {}
    }

    fn setup_test_logger() -> Arc<Mutex<Vec<String>>> {
        let logs = Arc::new(Mutex::new(Vec::new()));
        CAPTURED_LOGS.set(logs.clone()).ok();
        // Try to set logger, ignore error if already set
        let _ = log::set_boxed_logger(Box::new(TestLogger));
        log::set_max_level(log::LevelFilter::Warn);
        logs
    }

    #[tokio::test]
    async fn test_sdk_key_not_exposed_in_error_logs() {
        let logs = setup_test_logger();
        logs.lock().unwrap().clear();

        let test_api_key = "secret-api-key-12345";

        // Create a mock server that returns 500 error
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        // Create ConfigurationFetcher with the test API key pointing to mock server
        let mut fetcher = ConfigurationFetcher::new(ConfigurationFetcherConfig {
            base_url: mock_server.uri(),
            api_key: test_api_key.to_string(),
            sdk_metadata: SdkMetadata {
                name: "test-sdk",
                version: "1.0.0",
            },
        });

        // Attempt to fetch configuration, which will fail and trigger error logging
        let result = fetcher.fetch_configuration().await;

        // Verify the request failed
        assert!(result.is_err(), "Expected configuration fetch to fail");

        // Get captured logs
        let captured_logs = logs.lock().unwrap();
        let all_logs = captured_logs.join(" ");

        // Verify the API key is NOT in any of the log messages
        assert!(
            !all_logs.contains(test_api_key),
            "API key should not appear in log messages. Logs: {}",
            all_logs
        );

        // Also verify the returned error doesn't contain the API key
        if let Err(eppo_error) = result {
            let error_string = format!("{}", eppo_error);
            let error_debug = format!("{:?}", eppo_error);

            assert!(
                !error_string.contains(test_api_key),
                "API key should not appear in error Display: {}",
                error_string
            );
            assert!(
                !error_debug.contains(test_api_key),
                "API key should not appear in error Debug: {}",
                error_debug
            );
        }
    }
}
