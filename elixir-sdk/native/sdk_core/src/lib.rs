mod config;
mod conversion;
mod assignment;

use crate::config::Config;
use crate::conversion::{convert_attributes, convert_value_term, convert_event_term};
use crate::assignment::{get_assignment_inner, get_assignment_details_inner};
use eppo_core::{
    configuration_fetcher::{ConfigurationFetcher, ConfigurationFetcherConfig},
    configuration_poller::{start_configuration_poller, ConfigurationPollerConfig},
    configuration_store::ConfigurationStore,
    eval::{Evaluator, EvaluatorConfig},
    ufc::VariationType,
    SdkMetadata,
    background::BackgroundThread,
};
use std::panic::{RefUnwindSafe, UnwindSafe};

use rustler::{Encoder, Env, NifResult, ResourceArc, Term};
use rustler::types::atom;
use std::sync::Arc;
use std::collections::HashMap;

const SDK_METADATA: SdkMetadata = SdkMetadata {
    name: "elixir",
    version: env!("CARGO_PKG_VERSION"),
};

pub struct EppoClient {
    pub evaluator: Evaluator,
    pub background_thread: BackgroundThread,
}

#[rustler::resource_impl]
impl rustler::Resource for EppoClient {}
impl RefUnwindSafe for EppoClient {}
impl UnwindSafe for EppoClient {}


#[rustler::nif]
fn init(config: Config) -> NifResult<ResourceArc<EppoClient>> {
    config.validate().map_err(|e| rustler::Error::Term(Box::new(e)))?;

    let store = Arc::new(ConfigurationStore::new());
    
    let fetcher_config = ConfigurationFetcherConfig {
        base_url: config.base_url.clone(),
        api_key: config.api_key.clone(),
        sdk_metadata: SDK_METADATA,
    };

    let fetcher = ConfigurationFetcher::new(fetcher_config);

    let background_thread = BackgroundThread::start()
        .map_err(|e| rustler::Error::Term(Box::new(
            format!("Failed to start background thread: {}", e)
        )))?;

    let poller_config = ConfigurationPollerConfig::new()
        .with_interval(std::time::Duration::from_secs(
            config.poll_interval_seconds.unwrap_or(30),
        ))
        .with_jitter(std::time::Duration::from_secs(config.poll_jitter_seconds));

    let _poller = start_configuration_poller(
        background_thread.runtime(),
        fetcher,
        store.clone(),
        poller_config,
    );


    let evaluator = Evaluator::new(EvaluatorConfig {
        configuration_store: store,
        sdk_metadata: SDK_METADATA,
    });

    let client = ResourceArc::new(EppoClient {
        evaluator,
        background_thread,
    });

    Ok(client)
}

#[rustler::nif]
fn get_assignment<'a>(
    env: Env<'a>,
    client: ResourceArc<EppoClient>,
    flag_key: String,
    subject_key: String,
    subject_attributes: Term<'a>,
    expected_type: VariationType,
) -> NifResult<Term<'a>> {
    let eppo_attributes = convert_attributes(subject_attributes)?;
    let assignment = get_assignment_inner(client, flag_key, subject_key, eppo_attributes, expected_type);
    match assignment {
        Ok(Some(assignment)) => {
            let value = convert_value_term(env, assignment.value)?;
            let event = convert_event_term(env, assignment.event)?;
            Ok((value, event).encode(env))
        }
        _ => Err(rustler::Error::Term(Box::new("Failed to get assignment".to_string())))
    }
}

#[rustler::nif]
fn get_assignment_details<'a>(
    env: Env<'a>,
    client: ResourceArc<EppoClient>,
    flag_key: String,
    subject_key: String,
    subject_attributes: Term<'a>,
    expected_type: VariationType,
) -> NifResult<Term<'a>> {
    let eppo_attributes = convert_attributes(subject_attributes)?;
    let assignment_with_details =
        get_assignment_details_inner(client, flag_key, subject_key, eppo_attributes, expected_type);
    match assignment_with_details {
        Ok((evaluation_result, assignment_event)) => {
            // Create a HashMap to store all evaluation result fields
            let mut result_map = HashMap::new();
            
            // Add variation
            result_map.insert("variation".to_string(), match evaluation_result.variation {
                Some(val) => convert_value_term(env, val)?,
                None => atom::nil().encode(env),
            });
            
            // Add action
            result_map.insert("action".to_string(), match evaluation_result.action {
                Some(action) => action.encode(env),
                None => atom::nil().encode(env),
            });
            
            // Add evaluation details
            let json_details = serde_json::to_value(&evaluation_result.evaluation_details)
                .map_err(|e| rustler::Error::Term(Box::new(format!(
                    "Failed to serialize evaluation details: {:?}", e
                ))))?;
            
            result_map.insert("details".to_string(), json_details.to_string().encode(env));

            // Convert the event details
            let event_term = convert_event_term(env, assignment_event)?;
            
            // Return tuple with result map and event
            Ok((result_map, event_term).encode(env))
        }
        Err(err) => Err(rustler::Error::Term(Box::new(err))),
    }
}

rustler::init!("Elixir.EppoSdk.Core"); 
