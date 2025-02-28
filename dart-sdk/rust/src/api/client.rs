use std::{collections::HashMap, sync::Arc, time::Duration};

use flutter_rust_bridge::frb;

use eppo_core::{
    background::BackgroundRuntime,
    configuration_fetcher::{ConfigurationFetcher, ConfigurationFetcherConfig, DEFAULT_BASE_URL},
    configuration_poller::{
        start_configuration_poller, ConfigurationPoller, ConfigurationPollerConfig,
    },
    configuration_store::ConfigurationStore,
    eval::{BanditResult, Evaluator, EvaluatorConfig},
    events::{AssignmentEvent, BanditEvent},
    ufc::{Assignment, AssignmentValue, VariationType},
    AttributeValue, Attributes, SdkMetadata, Str,
};

use crate::runtime::{get_runtime, FlutterRustBridgeRuntime};

#[frb(opaque)]
pub struct CoreClient {
    configuration_store: Arc<ConfigurationStore>,
    background: BackgroundRuntime<FlutterRustBridgeRuntime>,
    poller: ConfigurationPoller,
    evaluator: Evaluator,
}

impl CoreClient {
    #[frb(sync)]
    pub fn new(
        sdk_key: String,
        base_url: Option<String>,
        #[frb(default = "30000")] poll_interval_ms: u64,
        #[frb(default = "3000")] poll_jitter_ms: u64,
    ) -> CoreClient {
        let base_url = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_owned());

        let configuration_store = Arc::new(ConfigurationStore::new());

        let background = BackgroundRuntime::new(get_runtime());

        const SDK_METADATA: SdkMetadata = SdkMetadata {
            name: "dart",
            version: env!("CARGO_PKG_VERSION"),
        };

        let poller = start_configuration_poller(
            &background,
            ConfigurationFetcher::new(ConfigurationFetcherConfig {
                base_url,
                api_key: sdk_key,
                sdk_metadata: SDK_METADATA,
            }),
            configuration_store.clone(),
            ConfigurationPollerConfig {
                interval: Duration::from_millis(poll_interval_ms),
                jitter: Duration::from_millis(poll_jitter_ms),
            },
        );

        let evaluator = Evaluator::new(EvaluatorConfig {
            configuration_store: configuration_store.clone(),
            sdk_metadata: SDK_METADATA,
        });

        CoreClient {
            configuration_store,
            background,
            poller,
            evaluator,
        }
    }

    pub async fn wait_for_initialization(&self) {
        let _result = self.poller.wait_for_configuration().await;
    }

    #[frb(sync, positional)]
    pub fn string_assignment(
        &self,
        flag_key: &str,
        subject_key: Str,
        subject_attributes: HashMap<Str, AttributeValue>,
    ) -> (Option<Str>, Option<AssignmentEvent>) {
        let Ok(Some(Assignment {
            value: AssignmentValue::String(result),
            event,
        })) = self.evaluator.get_assignment(
            flag_key,
            &subject_key,
            &Arc::new(subject_attributes),
            Some(VariationType::String),
        )
        else {
            return (None, None);
        };

        (Some(result), event)
    }

    #[frb(sync, positional)]
    pub fn numeric_assignment(
        &self,
        flag_key: &str,
        subject_key: Str,
        subject_attributes: HashMap<Str, AttributeValue>,
    ) -> (Option<f64>, Option<AssignmentEvent>) {
        let Ok(Some(Assignment {
            value: AssignmentValue::Numeric(result),
            event,
        })) = self.evaluator.get_assignment(
            flag_key,
            &subject_key,
            &Arc::new(subject_attributes),
            Some(VariationType::Numeric),
        )
        else {
            return (None, None);
        };

        (Some(result), event)
    }

    #[frb(sync, positional)]
    pub fn integer_assignment(
        &self,
        flag_key: &str,
        subject_key: Str,
        subject_attributes: HashMap<Str, AttributeValue>,
    ) -> (Option<i64>, Option<AssignmentEvent>) {
        let Ok(Some(Assignment {
            value: AssignmentValue::Integer(result),
            event,
        })) = self.evaluator.get_assignment(
            flag_key,
            &subject_key,
            &Arc::new(subject_attributes),
            Some(VariationType::Integer),
        )
        else {
            return (None, None);
        };

        (Some(result), event)
    }

    #[frb(sync, positional)]
    pub fn boolean_assignment(
        &self,
        flag_key: &str,
        subject_key: Str,
        subject_attributes: HashMap<Str, AttributeValue>,
    ) -> (Option<bool>, Option<AssignmentEvent>) {
        let Ok(Some(Assignment {
            value: AssignmentValue::Boolean(result),
            event,
        })) = self.evaluator.get_assignment(
            flag_key,
            &subject_key,
            &Arc::new(subject_attributes),
            Some(VariationType::Boolean),
        )
        else {
            return (None, None);
        };

        (Some(result), event)
    }

    #[frb(sync, positional)]
    pub fn json_assignment(
        &self,
        flag_key: &str,
        subject_key: Str,
        subject_attributes: HashMap<Str, AttributeValue>,
    ) -> (Option<Str>, Option<AssignmentEvent>) {
        let Ok(Some(Assignment {
            value:
                AssignmentValue::Json {
                    raw: result,
                    parsed: _,
                },
            event,
        })) = self.evaluator.get_assignment(
            flag_key,
            &subject_key,
            &Arc::new(subject_attributes),
            Some(VariationType::Json),
        )
        else {
            return (None, None);
        };

        (Some(result), event)
    }

    #[frb(sync, positional)]
    pub fn bandit_action(
        &self,
        flag_key: &str,
        subject_key: Str,
        subject_attributes: HashMap<Str, AttributeValue>,
        actions: HashMap<Str, HashMap<Str, AttributeValue>>,
        default_variation: Str,
    ) -> BanditResult {
        self.evaluator.get_bandit_action(
            flag_key,
            &subject_key,
            &subject_attributes.into(),
            &actions
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
            &default_variation,
        )
    }
}
