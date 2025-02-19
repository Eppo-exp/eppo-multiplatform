use eppo_core::{
    configuration_fetcher::{ConfigurationFetcher, ConfigurationFetcherConfig},
    configuration_poller::{start_configuration_poller, ConfigurationPollerConfig},
    configuration_store::ConfigurationStore,
    eval::{Evaluator, EvaluatorConfig},
    eval::eval_details::EvaluationResultWithDetails,
    {Str, AttributeValue},
    ufc::{VariationType, Assignment, AssignmentValue},
    SdkMetadata,
    background::BackgroundThread,
    events::AssignmentEvent,
};

use rustler::{Encoder, Env, NifResult, NifStruct, ResourceArc, Term, Error};
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
fn shutdown() -> Result<(), String> {
    if let Ok(mut instance) = CLIENT_INSTANCE.write() {
        if let Some(client) = instance.take() {
            drop(client);
        }
    }
    Ok(())
}

fn get_assignment_inner(
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

fn get_assignment_details_inner(
    flag_key: String,
    subject_key: String,
    eppo_attributes: Arc<HashMap<Str, AttributeValue>>,
    expected_type: VariationType,
) -> Result<(
    EvaluationResultWithDetails<AssignmentValue>,
    Option<AssignmentEvent>,
), String> {
    let client = get_instance()?;

    let assignment_with_details = client.evaluator.get_assignment_details(
        &Str::new(flag_key),
        &Str::new(subject_key),
        &eppo_attributes,
        Some(expected_type)
    );

    Ok(assignment_with_details)
}

pub fn convert_attributes(subject_attributes: Term) -> NifResult<Arc<HashMap<Str, AttributeValue>>> {
    // Obtain an iterator over the map's key-value pairs.
    let map: HashMap<String, Term> = subject_attributes.decode()?;
    let mut attributes = HashMap::with_capacity(map.len());

    for (key, value_term) in map {
        // Try to decode the value as one of the supported types.
        let attr_value = if let Ok(b) = value_term.decode::<bool>() {
            // Booleans are stored as categorical attributes.
            AttributeValue::categorical(b)
        } else if let Ok(i) = value_term.decode::<i64>() {
            // Integers are converted to f64 and stored as numeric.
            AttributeValue::numeric(i as f64)
        } else if let Ok(f) = value_term.decode::<f64>() {
            AttributeValue::numeric(f)
        } else if let Ok(s) = value_term.decode::<String>() {
            // Strings are stored as categorical attributes.
            AttributeValue::categorical(s)
        } else {
            // If none of the supported types matched, return a BadArg error.
            return Err(Error::BadArg);
        };

        // Insert the converted key and attribute value into the HashMap.
        // Here we assume that `Str` implements conversion from String.
        attributes.insert(key.into(), attr_value);
    }
    Ok(Arc::new(attributes))
}

fn convert_value_term<'a>(env: Env<'a>, value: AssignmentValue) -> NifResult<Term<'a>> {
    match value {
        AssignmentValue::String(s) => Ok(s.encode(env)),
        AssignmentValue::Integer(i) => Ok(i.encode(env)),
        AssignmentValue::Numeric(n) => Ok(n.encode(env)),
        AssignmentValue::Boolean(b) => Ok(b.encode(env)),
        AssignmentValue::Json { raw, .. } => Ok(raw.encode(env)),
    }
}

fn convert_event_term<'a>(env: Env<'a>, event: Option<AssignmentEvent>) -> NifResult<Term<'a>> {
    if let Some(event) = event {
        let json_value = serde_json::to_value(&event)
            .map_err(|e| rustler::Error::Term(Box::new(format!("Failed to serialize event: {:?}", e))))?;
        Ok(json_value.to_string().encode(env))
    } else {
        Ok(atom::nil().encode(env))
    }
}

#[rustler::nif]
fn get_assignment<'a>(
    env: Env<'a>,
    flag_key: String,
    subject_key: String,
    subject_attributes: Term<'a>,
    expected_type: VariationType,
) -> NifResult<Term<'a>> {
    let eppo_attributes = convert_attributes(subject_attributes)?;
    let assignment = get_assignment_inner(flag_key, subject_key, eppo_attributes, expected_type);
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
    flag_key: String,
    subject_key: String,
    subject_attributes: Term<'a>,
    expected_type: VariationType,
) -> NifResult<Term<'a>> {
    let eppo_attributes = convert_attributes(subject_attributes)?;
    let assignment_with_details =
        get_assignment_details_inner(flag_key, subject_key, eppo_attributes, expected_type);
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

// Update atoms module
mod atoms {
    rustler::atoms! {
        nil
    }
}

rustler::init!("Elixir.SdkCore"); 
