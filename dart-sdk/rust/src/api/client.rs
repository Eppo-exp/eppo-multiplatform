use std::{sync::Arc, time::Duration};

use flutter_rust_bridge::frb;

use eppo_core::{
    background::BackgroundRuntime,
    configuration_fetcher::{ConfigurationFetcher, ConfigurationFetcherConfig},
    configuration_poller::{
        start_configuration_poller, ConfigurationPoller, ConfigurationPollerConfig,
    },
    configuration_store::ConfigurationStore,
    eval::{Evaluator, EvaluatorConfig},
    ufc::{Assignment, AssignmentValue, VariationType},
    Attributes, SdkMetadata, Str,
};

const SDK_METADATA: SdkMetadata = SdkMetadata {
    name: "dart",
    version: env!("CARGO_PKG_VERSION"),
};

#[frb(opaque)]
pub struct EppoClient {
    configuration_store: Arc<ConfigurationStore>,
    background: BackgroundRuntime,
    poller: ConfigurationPoller,
    evaluator: Evaluator,
}

impl EppoClient {
    #[frb(sync)]
    pub fn new(sdk_key: String) -> EppoClient {
        let configuration_store = Arc::new(ConfigurationStore::new());

        let handle = crate::frb_generated::FLUTTER_RUST_BRIDGE_HANDLER
            .async_runtime()
            .0
             .0
            .handle()
            .clone();

        let background = BackgroundRuntime::new(handle);

        let poller = start_configuration_poller(
            &background,
            ConfigurationFetcher::new(ConfigurationFetcherConfig {
                base_url: "https://fscdn.eppo.cloud/api".to_owned(),
                api_key: sdk_key,
                sdk_metadata: SDK_METADATA,
            }),
            configuration_store.clone(),
            ConfigurationPollerConfig {
                interval: Duration::from_secs(30),
                jitter: Duration::from_secs(3),
            },
        );

        let evaluator = Evaluator::new(EvaluatorConfig {
            configuration_store: configuration_store.clone(),
            sdk_metadata: SDK_METADATA,
        });

        EppoClient {
            configuration_store,
            background,
            poller,
            evaluator,
        }
    }

    pub async fn wait_for_configuration(&self) {
        let result = self.poller.wait_for_configuration().await;
    }

    #[frb(sync, positional)]
    pub fn string_assignment(&self, flag_key: &str, subject_key: &str, default: String) -> String {
        let Ok(Some(Assignment {
            value: AssignmentValue::String(result),
            event,
        })) = self.evaluator.get_assignment(
            flag_key,
            &subject_key.into(),
            &Arc::new(Attributes::new()),
            Some(VariationType::String),
        )
        else {
            return default;
        };

        result.as_str().into()
    }

    #[frb(sync, positional)]
    pub fn bool_assignment(&self, flag_key: &str, subject_key: &str, default: bool) -> bool {
        let Ok(Some(Assignment {
            value: AssignmentValue::Boolean(result),
            event,
        })) = self.evaluator.get_assignment(
            flag_key,
            &subject_key.into(),
            &Arc::new(Attributes::new()),
            Some(VariationType::Boolean),
        )
        else {
            return default;
        };

        result
    }
}
