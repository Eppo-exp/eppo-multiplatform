use serde::{Deserialize, Serialize};

use crate::timestamp::Timestamp;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub(super) struct Event {
    pub uuid: uuid::Uuid,
    pub timestamp: Timestamp,
    pub event_type: String,
    pub payload: serde_json::Value,
}
