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

pub const DEFAULT_BASE_URL: &str = "https://fscdn.eppo.cloud/api";
const UFC_ENDPOINT: &str = "/flag-config/v1/config";
const BANDIT_ENDPOINT: &str = "/flag-config/v1/bandits";

/// A client that fetches Eppo configuration from the server.
pub struct ConfigurationFetcher {
    // Client holds a connection pool internally, so we're reusing the client between requests.
    client: reqwest::blocking::Client,
    config: ConfigurationFetcherConfig,
    /// If we receive a 401 Unauthorized error during a request, it means the API key is not
    /// valid. We cache this error so we don't issue additional requests to the server.
    unauthorized: bool,
}

impl ConfigurationFetcher {
    pub fn new(config: ConfigurationFetcherConfig) -> ConfigurationFetcher {
        let client = reqwest::blocking::Client::new();

        ConfigurationFetcher {
            client,
            config,
            unauthorized: false,
        }
    }

    pub fn fetch_configuration(&mut self) -> Result<Configuration> {
        if self.unauthorized {
            return Err(Error::Unauthorized);
        }

        let ufc = self.fetch_ufc_configuration()?;

        let bandits = if ufc.compiled.flag_to_bandit_associations.is_empty() {
            // We don't need bandits configuration if there are no bandits.
            None
        } else {
            Some(self.fetch_bandits_configuration()?)
        };

        Ok(Configuration::from_server_response(ufc, bandits))
    }

    fn fetch_ufc_configuration(&mut self) -> Result<UniversalFlagConfig> {
        let url = Url::parse_with_params(
            &format!("{}{}", self.config.base_url, UFC_ENDPOINT),
            &[
                ("apiKey", &*self.config.api_key),
                ("sdkName", self.config.sdk_metadata.name),
                ("sdkVersion", self.config.sdk_metadata.version),
                ("coreVersion", env!("CARGO_PKG_VERSION")),
            ],
        )
        .map_err(Error::InvalidBaseUrl)?;

        log::debug!(target: "eppo", "fetching UFC flags configuration");
        let response = self.client.get(url).send()?;

        let response = response.error_for_status().map_err(|err| {
            if err.status() == Some(StatusCode::UNAUTHORIZED) {
                log::warn!(target: "eppo", "client is not authorized. Check your API key");
                self.unauthorized = true;
                Error::Unauthorized
            } else {
                log::warn!(target: "eppo", "received non-200 response while fetching new configuration: {:?}", err);
                Error::from(err)
            }
        })?;

        let configuration =
            UniversalFlagConfig::from_json(self.config.sdk_metadata, response.bytes()?.into())?;

        log::debug!(target: "eppo", "successfully fetched UFC flags configuration");

        Ok(configuration)
    }

    fn fetch_bandits_configuration(&mut self) -> Result<BanditResponse> {
        let url = Url::parse_with_params(
            &format!("{}{}", self.config.base_url, BANDIT_ENDPOINT),
            &[
                ("apiKey", &*self.config.api_key),
                ("sdkName", self.config.sdk_metadata.name),
                ("sdkVersion", self.config.sdk_metadata.version),
                ("coreVersion", env!("CARGO_PKG_VERSION")),
            ],
        )
        .map_err(|err| Error::InvalidBaseUrl(err))?;

        log::debug!(target: "eppo", "fetching UFC bandits configuration");
        let response = self.client.get(url).send()?;

        let response = response.error_for_status().map_err(|err| {
            if err.status() == Some(StatusCode::UNAUTHORIZED) {
                log::warn!(target: "eppo", "client is not authorized. Check your API key");
                self.unauthorized = true;
                Error::Unauthorized
            } else {
                log::warn!(target: "eppo", "received non-200 response while fetching new configuration: {:?}", err);
                Error::from(err)
            }
        })?;

        let configuration = response.json()?;

        log::debug!(target: "eppo", "successfully fetched UFC bandits configuration");

        Ok(configuration)
    }
}
