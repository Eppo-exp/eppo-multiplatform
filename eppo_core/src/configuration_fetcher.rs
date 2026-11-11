//! An HTTP client that fetches configuration from the server.
use std::sync::Arc;

use reqwest::{
    header::{HeaderValue, ETAG, IF_NONE_MATCH},
    StatusCode, Url,
};

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
    cached_flags: Option<(HeaderValue, Arc<UniversalFlagConfig>)>,
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
            cached_flags: None,
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

        Ok(Configuration::from_shared_flags(ufc, bandits))
    }

    async fn fetch_ufc_configuration(&mut self) -> Result<Arc<UniversalFlagConfig>> {
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
        let mut request = self.client.get(url);
        if let Some((etag, _)) = &self.cached_flags {
            request = request.header(IF_NONE_MATCH, etag);
        }
        let response = request.send().await?;
        if response.status() == StatusCode::NOT_MODIFIED {
            if let Some((_, flags)) = &self.cached_flags {
                return Ok(Arc::clone(flags));
            }
        }

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

        let etag = response.headers().get(ETAG).cloned();
        let configuration = Arc::new(UniversalFlagConfig::from_json(
            self.config.sdk_metadata,
            response.bytes().await?.into(),
        )?);
        // Browser fetch manages its own HTTP cache. Adding If-None-Match there can trigger a
        // CORS preflight and require server headers that existing clients do not need.
        if !cfg!(target_arch = "wasm32") {
            self.cached_flags = etag.map(|tag| (tag, Arc::clone(&configuration)));
        }

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
    async fn reuses_validated_flags_on_not_modified() {
        use wiremock::matchers::{header, header_exists};

        let server = MockServer::start().await;
        let url = server.uri();
        let body =
            r#"{"createdAt":"2025-01-01T00:00:00Z","environment":{"name":"test"},"flags":{}}"#;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("etag", r#""v1""#)
                    .set_body_raw(body, "application/json"),
            )
            .up_to_n_times(1)
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(header("if-none-match", r#""v1""#))
            .respond_with(ResponseTemplate::new(304))
            .expect(1)
            .mount(&server)
            .await;

        let mut fetcher = ConfigurationFetcher::new(ConfigurationFetcherConfig {
            base_url: url,
            api_key: "test-key".to_owned(),
            sdk_metadata: SdkMetadata {
                name: "test-sdk",
                version: "1.0",
            },
        });
        let first = fetcher.fetch_configuration().await.unwrap();
        let second = fetcher.fetch_configuration().await.unwrap();
        assert!(Arc::ptr_eq(&first.flags, &second.flags));

        server.reset().await;
        Mock::given(header_exists("if-none-match"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .expect(1)
            .mount(&server)
            .await;
        let replacement = fetcher.fetch_configuration().await.unwrap();
        assert!(!Arc::ptr_eq(&second.flags, &replacement.flags));
        assert!(fetcher.cached_flags.is_none());
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
