use crate::timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Event<T : Serialize> {
    pub uuid: uuid::Uuid,
    pub timestamp: Timestamp,
    pub event_type: String,
    pub payload: T,
}

pub type GenericEvent = Event<Value>;
