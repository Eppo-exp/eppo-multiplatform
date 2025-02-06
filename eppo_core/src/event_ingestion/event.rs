use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::timestamp::Timestamp;

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub(super) struct Event {
    pub uuid: uuid::Uuid,

    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: Timestamp,

    #[serde(rename = "type")]
    pub event_type: String,

    pub payload: serde_json::Value,
}