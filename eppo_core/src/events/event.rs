use crate::timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub uuid: uuid::Uuid,
    pub timestamp: Timestamp,
    pub event_type: String,
    pub payload: HashMap<String, serde_json::Value>,
}
