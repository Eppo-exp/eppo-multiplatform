use std::{cell::RefCell, str::FromStr, sync::Arc, time::Duration};

use crate::{configuration::Configuration, SDK_METADATA};
use eppo_core::{
    background::BackgroundThread,
    configuration_fetcher::{ConfigurationFetcher, ConfigurationFetcherConfig},
    configuration_poller::{
        start_configuration_poller, ConfigurationPoller, ConfigurationPollerConfig,
    },
    configuration_store::ConfigurationStore,
    eval::{Evaluator, EvaluatorConfig},
    event_ingestion::{EventIngestion, EventIngestionConfig},
    ufc::VariationType,
    Attributes, ContextAttributes, SdkKey,
};
use magnus::{error::Result, exception, prelude::*, Error, IntoValue, Ruby, TryConvert, Value};

#[derive(Debug)]
#[magnus::wrap(class = "EppoClient::Core::Config", size, free_immediately)]
pub struct Config {
    api_key: String,
    base_url: String,
    poll_interval: Option<Duration>,
    poll_jitter: Duration,
    log_level: Option<log::LevelFilter>,
    event_ingestion_config: Option<EventIngestionConfig>,
}

impl TryConvert for Config {
    // `val` is expected to be of type EppoClient::Config.
    fn try_convert(val: Value) -> Result<Self> {
        let sdk_key = String::try_convert(val.funcall("api_key", ())?)?;
        let base_url = String::try_convert(val.funcall("base_url", ())?)?;
        let poll_interval_seconds =
            Option::<u64>::try_convert(val.funcall("poll_interval_seconds", ())?)?;
        let poll_jitter_seconds = u64::try_convert(val.funcall("poll_jitter_seconds", ())?)?;
        let log_level = {
            let s = Option::<String>::try_convert(val.funcall("log_level", ())?)?;
            s.map(|s| {
                log::LevelFilter::from_str(&s)
                    .map_err(|err| Error::new(exception::runtime_error(), err.to_string()))
            })
            .transpose()?
        };

        let event_ingestion_config = EventIngestionConfig::new(SdkKey::new(sdk_key.clone().into()));
        Ok(Config {
            api_key: sdk_key,
            base_url,
            poll_interval: poll_interval_seconds.map(Duration::from_secs),
            poll_jitter: Duration::from_secs(poll_jitter_seconds),
            log_level,
            event_ingestion_config,
        })
    }
}

#[magnus::wrap(class = "EventIngestion")]
struct MutEventIngestion(RefCell<EventIngestion>);

impl MutEventIngestion {
    pub fn new(event_ingestion: EventIngestion) -> Self {
        Self(RefCell::new(event_ingestion))
    }
}

#[magnus::wrap(class = "EppoClient::Core::Client")]
pub struct Client {
    configuration_store: Arc<ConfigurationStore>,
    evaluator: Evaluator,

    // Magnus only allows sharing aliased references (&T) through the API, so we need to use RefCell
    // to get interior mutability.
    //
    // This should be safe as Ruby only uses a single OS thread, and `Client` lives in the Ruby
    // world.
    background_thread: RefCell<Option<BackgroundThread>>,
    configuration_poller: Option<ConfigurationPoller>,
    event_ingestion: Option<MutEventIngestion>,
}


impl Client {
    pub fn new(config: Config) -> Client {
        // Initialize logger
        {
            let mut builder = env_logger::Builder::from_env(
                env_logger::Env::new()
                    .filter_or("EPPO_LOG", "eppo=info")
                    .write_style("EPPO_LOG_STYLE"),
            );

            if let Some(log_level) = config.log_level {
                builder.filter_module("eppo", log_level);
            }

            // Logger can only be set once, so we ignore the initialization error here if client is
            // re-initialized.
            let _ = builder.try_init();
        };

        let configuration_store = Arc::new(ConfigurationStore::new());

        let evaluator = Evaluator::new(EvaluatorConfig {
            configuration_store: configuration_store.clone(),
            sdk_metadata: SDK_METADATA,
        });

        let background_thread =
            BackgroundThread::start().expect("should be able to start background thread");

        let configuration_poller = if let Some(poll_interval) = config.poll_interval {
            let poller = start_configuration_poller(
                background_thread.runtime(),
                ConfigurationFetcher::new(ConfigurationFetcherConfig {
                    base_url: config.base_url,
                    api_key: config.api_key,
                    sdk_metadata: SDK_METADATA,
                }),
                configuration_store.clone(),
                ConfigurationPollerConfig {
                    interval: poll_interval,
                    jitter: config.poll_jitter,
                },
            );
            Some(poller)
        } else {
            None
        };

        let event_ingestion = config
            .event_ingestion_config
            .map(|config| MutEventIngestion::new(config.spawn(background_thread.runtime())));

        Client {
            configuration_store,
            evaluator,
            background_thread: RefCell::new(Some(background_thread)),
            configuration_poller,
            event_ingestion,
        }
    }

    pub fn get_assignment(
        ruby: &Ruby,
        rb_self: &Self,
        flag_key: String,
        subject_key: String,
        subject_attributes: Value,
        expected_type: Value,
    ) -> Result<Value> {
        let expected_type: VariationType = serde_magnus::deserialize(expected_type)?;
        let subject_attributes: Attributes = serde_magnus::deserialize(subject_attributes)?;

        let result = rb_self
            .evaluator
            .get_assignment(
                &flag_key,
                &subject_key.into(),
                &Arc::new(subject_attributes),
                Some(expected_type),
            )
            // TODO: maybe expose possible errors individually.
            .map_err(|err| Error::new(exception::runtime_error(), err.to_string()))?;

        Ok(result.into_value_with(&ruby))
    }

    pub fn get_assignment_details(
        &self,
        flag_key: String,
        subject_key: String,
        subject_attributes: Value,
        expected_type: Value,
    ) -> Result<Value> {
        let ruby = Ruby::get_with(subject_attributes);

        let expected_type: VariationType = serde_magnus::deserialize(expected_type)?;
        let subject_attributes: Attributes = serde_magnus::deserialize(subject_attributes)?;

        let result = self.evaluator.get_assignment_details(
            &flag_key,
            &subject_key.into(),
            &Arc::new(subject_attributes),
            Some(expected_type),
        );

        Ok(result.into_value_with(&ruby))
    }

    pub fn get_bandit_action(
        &self,
        flag_key: String,
        subject_key: String,
        subject_attributes: Value,
        actions: Value,
        default_variation: String,
    ) -> Result<Value> {
        let subject_attributes = serde_magnus::deserialize::<_, ContextAttributes>(
            subject_attributes,
        )
        .map_err(|err| {
            Error::new(
                exception::runtime_error(),
                format!("Unexpected value for subject_attributes: {err}"),
            )
        })?;
        let actions = serde_magnus::deserialize(actions)?;

        let result = self.evaluator.get_bandit_action(
            &flag_key,
            &subject_key.into(),
            &subject_attributes,
            &actions,
            &default_variation.into(),
        );

        serde_magnus::serialize(&result)
    }

    pub fn get_bandit_action_details(
        &self,
        flag_key: String,
        subject_key: String,
        subject_attributes: Value,
        actions: Value,
        default_variation: String,
    ) -> Result<Value> {
        let subject_attributes = serde_magnus::deserialize::<_, ContextAttributes>(
            subject_attributes,
        )
        .map_err(|err| {
            Error::new(
                exception::runtime_error(),
                format!("Unexpected value for subject_attributes: {err}"),
            )
        })?;
        let actions = serde_magnus::deserialize(actions)?;

        let result = self.evaluator.get_bandit_action_details(
            &flag_key,
            &subject_key.into(),
            &subject_attributes,
            &actions,
            &default_variation.into(),
        );

        serde_magnus::serialize(&result)
    }

    pub fn get_configuration(&self) -> Option<Configuration> {
        self.configuration_store
            .get_configuration()
            .map(|it| it.into())
    }

    pub fn set_configuration(&self, configuration: &Configuration) {
        self.configuration_store
            .set_configuration(configuration.clone().into())
    }

    pub fn shutdown(&self) {
        if let Some(thread) = self.background_thread.take() {
            thread.graceful_shutdown();
        }
    }

    pub fn set_context(&self, key: String, value: Value) -> Result<()> {
        let Some(event_ingestion) = &self.event_ingestion else {
            // Event ingestion is disabled, do nothing.
            return Ok(());
        };
        let value: serde_json::Value = serde_magnus::deserialize(value).map_err(|err| {
            Error::new(
                exception::runtime_error(),
                format!("Unexpected value for payload: {err}"),
            )
        })?;

        event_ingestion.0.borrow_mut().attach_context(key, value);

        Ok(())
    }

    pub fn track(&self, event_type: String, payload: Value) -> Result<()> {
        let Some(event_ingestion) = &self.event_ingestion else {
            // Event ingestion is disabled, do nothing.
            return Ok(());
        };

        let payload: serde_json::Value = serde_magnus::deserialize(payload).map_err(|err| {
            Error::new(
                exception::runtime_error(),
                format!("Unexpected value for payload: {err}"),
            )
        })?;

        event_ingestion.0.borrow().track(event_type, payload);

        Ok(())
    }
}
