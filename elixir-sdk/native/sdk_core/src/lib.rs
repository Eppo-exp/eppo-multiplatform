use eppo_core::{
    configuration_fetcher::{ConfigurationFetcher, ConfigurationFetcherConfig},
    configuration_poller::{start_configuration_poller, ConfigurationPollerConfig},
    configuration_store::ConfigurationStore,
    eval::{Evaluator, EvaluatorConfig},
    {Str, AttributeValue, Attributes},
    ufc::{VariationType, Assignment},
    SdkMetadata,
    background::BackgroundThread,
};

use rustler::{Encoder, Env, NifResult, NifStruct, ResourceArc, Term, Atom};
use rustler::types::atom;
use std::{sync::{Arc, RwLock}};
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::collections::HashMap;

const SDK_METADATA: SdkMetadata = SdkMetadata {
    name: "elixir",
    version: env!("CARGO_PKG_VERSION"),
};

static CLIENT_INSTANCE: RwLock<Option<ResourceArc<EppoClient>>> = RwLock::new(None);

#[derive(NifStruct)]
#[module = "SdkCore.Config"]
struct Config {
    api_key: String,
    base_url: String,
    is_graceful_mode: bool,
    poll_interval_seconds: Option<u64>,
    poll_jitter_seconds: u64,
}

struct EppoClient {
    evaluator: Evaluator,
    background_thread: BackgroundThread,
}

#[rustler::resource_impl]
impl rustler::Resource for EppoClient {}
impl RefUnwindSafe for EppoClient {}
impl UnwindSafe for EppoClient {}

#[rustler::nif]
fn init(config: Config) -> Result<(), String> {
    // Validate config
    if config.api_key.is_empty() {
        return Err("Invalid value for api_key: cannot be blank".to_string());
    }

    let store = Arc::new(ConfigurationStore::new());
    
    let fetcher_config = ConfigurationFetcherConfig {
        base_url: config.base_url.clone(),
        api_key: config.api_key.clone(),
        sdk_metadata: SDK_METADATA,
    };

    let fetcher = ConfigurationFetcher::new(fetcher_config);

    let background_thread = BackgroundThread::start()
        .map_err(|e| format!("Failed to start background thread: {}", e))?;

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

    // Set global instance
    let mut instance = CLIENT_INSTANCE
        .write()
        .map_err(|e| format!("Failed to acquire write lock: {}", e))?;
    
    if let Some(existing) = instance.take() {
        // Shutdown existing client
        drop(existing);
    }
    
    *instance = Some(client.clone());

    Ok(())
}

// #[rustler::nif]
fn get_instance() -> Result<ResourceArc<EppoClient>, String> {
    let instance = CLIENT_INSTANCE
        .read()
        .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

    match &*instance {
        Some(client) => Ok(client.clone()),
        None => Err("init() must be called before get_instance()".to_string()),
    }
}

#[rustler::nif]
fn shutdown() -> Atom {
    if let Ok(mut instance) = CLIENT_INSTANCE.write() {
        if let Some(client) = instance.take() {
            drop(client);
        }
    }
    atoms::ok()
}

fn get_assignment(
    flag_key: String,
    subject_key: String,
    eppo_attributes: Arc<HashMap<Str, AttributeValue>>,
    expected_type: VariationType,
) -> Result<Option<Assignment>, String> {
    let client = get_instance()?;

    // Get assignment
    let assignment = client.evaluator.get_assignment(
        &Str::new(flag_key),
        &Str::new(subject_key),
        &eppo_attributes,
        Some(expected_type)
    ).map_err(|e| format!("Failed to get assignment: {:?}", e))?;

    Ok(assignment)
}

fn convert_attributes(subject_attributes: Term) -> NifResult<Arc<HashMap<Str, AttributeValue>>> {
    // Convert subject_attributes Term to HashMap
    let attributes: HashMap<String, String> = subject_attributes
        .decode()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Failed to decode subject attributes: {:?}", e))))?;
    
    // Convert attributes to the required format
    Ok(Arc::new(attributes
        .into_iter()
        .map(|(k, v)| (Str::new(k), AttributeValue::categorical(v)))
        .collect::<HashMap<Str, AttributeValue>>()))
}

#[rustler::nif]
fn get_string_assignment<'a>(
    env: Env<'a>,
    flag_key: String,
    subject_key: String,
    subject_attributes: Term<'a>
) -> NifResult<Term<'a>> {
    let eppo_attributes = convert_attributes(subject_attributes)?;
    let assignment = get_assignment(flag_key, subject_key, eppo_attributes, VariationType::String);
    match assignment {
        Ok(Some(assignment)) => match assignment.value.as_str() {
            Some(s) => Ok(s.encode(env)),
            None => Ok(atoms::nil().encode(env))
        },
        _ => Ok(atoms::nil().encode(env))
    }
}

#[rustler::nif]
fn get_boolean_assignment<'a>(
    env: Env<'a>,
    flag_key: String,
    subject_key: String,
    subject_attributes: Term<'a>
) -> NifResult<Term<'a>> {
    let eppo_attributes = convert_attributes(subject_attributes)?;
    let assignment = get_assignment(flag_key, subject_key, eppo_attributes, VariationType::Boolean);
    match assignment {
        Ok(Some(assignment)) => match assignment.value.as_str() {
            Some(s) => Ok(s.encode(env)),
            None => Ok(atoms::nil().encode(env))
        },
        _ => Ok(atoms::nil().encode(env))
    }
}


// Update atoms module
mod atoms {
    rustler::atoms! {
        ok,
        nil
    }
}


rustler::init!("Elixir.SdkCore"); 
