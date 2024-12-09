use chrono::Utc;
use eppo_core::ufc::UniversalFlagConfig;
use eppo_core::{Attributes, Configuration, SdkMetadata, Str};
use fastly::http::StatusCode;
use fastly::kv_store::KVStoreError;
use fastly::{Error, KVStore, Request, Response};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::sync::Arc;

const KV_STORE_NAME: &str = "edge-assignment-kv-store";
// TODO: Migrate authorization to header `Authorization`.
const SDK_KEY_QUERY_PARAM: &str = "apiKey"; // For legacy reasons this is named `apiKey`

const SDK_NAME: &str = "fastly-edge-assignments";
const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

fn kv_store_key(token_hash: String) -> String {
    format!("ufc-by-sdk-key-token-hash-{}", token_hash)
}

fn token_hash(sdk_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(sdk_key.as_bytes());
    base64_url::encode(&hasher.finalize())
}

// Request
#[derive(Debug, Deserialize)]
struct PrecomputedAssignmentsServiceRequestBody {
    pub subject_key: Str,
    pub subject_attributes: Arc<Attributes>,
    // TODO: Add bandit actions
    // #[serde(rename = "banditActions")]
    // #[serde(skip_serializing_if = "Option::is_none")]
    // bandit_actions: Option<HashMap<String, serde_json::Value>>,
}

pub fn handle_assignments(mut req: Request) -> Result<Response, Error> {
    // Extract the SDK key and generate a token hash matching the pre-defined encoding.
    let Some(token) = req
        .get_query_parameter(SDK_KEY_QUERY_PARAM)
        .filter(|it| !it.is_empty())
    else {
        return Ok(
            Response::from_status(StatusCode::BAD_REQUEST).with_body_text_plain(&format!(
                "Missing required query parameter: {}",
                SDK_KEY_QUERY_PARAM
            )),
        );
    };
    let token_hash = token_hash(token);

    // Deserialize the request body into a struct
    let (subject_key, subject_attributes): (eppo_core::Str, Arc<Attributes>) =
        match serde_json::from_slice::<PrecomputedAssignmentsServiceRequestBody>(
            &req.take_body_bytes(),
        ) {
            Ok(body) => {
                if body.subject_key.is_empty() {
                    return Ok(Response::from_status(StatusCode::BAD_REQUEST)
                        .with_body_text_plain("subject_key is required and cannot be empty"));
                }
                (body.subject_key, body.subject_attributes)
            }
            Err(e) => {
                let error_message = if e.to_string().contains("subject_key") {
                    "subject_key is required in the request body"
                } else {
                    "Invalid request body format"
                };
                return Ok(Response::from_status(StatusCode::BAD_REQUEST)
                    .with_body_text_plain(error_message));
            }
        };

    // Open the KV store
    let kv_store = KVStore::open(KV_STORE_NAME).map(|store| store.expect("KVStore exists"))?;

    let mut kv_store_item = match kv_store.lookup(&kv_store_key(token_hash.clone())) {
        Ok(item) => item,
        Err(e) => {
            let (status, message) = match e {
                KVStoreError::ItemNotFound => {
                    eprintln!("Missing configuration for SDK key: {}", token_hash);

                    // Return unauthorized if the key does not exist.
                    // Our protocol lets the client know that the SDK key has not had a UFC
                    // configuration pre-computed for it in the KV Store.
                    (StatusCode::UNAUTHORIZED, "Invalid SDK key.".to_string())
                }
                _ => {
                    eprintln!("KV Store error: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Unexpected KV Store error".to_string(),
                    )
                }
            };

            return Ok(Response::from_status(status).with_body_text_plain(&message));
        }
    };

    // Parse the response from the KV store
    let kv_store_item_body = kv_store_item.take_body();
    let ufc_config = match UniversalFlagConfig::from_json(
        SdkMetadata {
            name: SDK_NAME,
            version: SDK_VERSION,
        },
        kv_store_item_body.into_bytes(),
    ) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to parse UFC config: {:?}", e);
            return Ok(Response::from_status(StatusCode::INTERNAL_SERVER_ERROR)
                .with_body_text_plain("Invalid configuration format in KV store"));
        }
    };

    let configuration = Configuration::from_server_response(ufc_config, None);
    let precomputed_configuration = eppo_core::eval::get_precomputed_assignments(
        Some(&configuration),
        &subject_key,
        &subject_attributes,
        Utc::now(),
    );

    // Create an HTTP OK response with the assignments
    Response::from_status(StatusCode::OK)
        .with_body_json(&precomputed_configuration.obfuscate())
        .or_else(|e| {
            eprintln!("Failed to serialize response: {:?}", e);
            Ok(Response::from_status(StatusCode::INTERNAL_SERVER_ERROR)
                .with_body_text_plain("Failed to serialize response"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_hash() {
        // Test case with a known SDK key and its expected hash
        let sdk_key = "5qCSVzH1lCI11.ZWg9ZDhlYnhsLmV2ZW50cy5lcHBvLmxvY2FsaG9zdA";
        let expected_hash = "V--77TScV5Etm78nIMTSOdiroOh1__NsupwUwsetEVM";

        let result = token_hash(sdk_key);

        assert_eq!(result, expected_hash);
    }
}
