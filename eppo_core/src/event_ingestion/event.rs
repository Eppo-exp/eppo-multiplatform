use crate::timestamp::Timestamp;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};
use serde::de::Deserializer;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub(super) struct Event {
    pub uuid: uuid::Uuid,
    #[serde(
        serialize_with = "serialize_millis",
        deserialize_with = "deserialize_millis"
    )]
    pub timestamp: Timestamp,
    #[serde(rename = "type")]
    pub event_type: String,
    pub payload: serde_json::Value,
}

fn serialize_millis<S>(date: &Timestamp, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Convert to i64 representing milliseconds since the Unix epoch
    let millis = date.timestamp_millis();
    serializer.serialize_i64(millis)
}

fn deserialize_millis<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
where
    D: Deserializer<'de>,
{
    let millis = i64::deserialize(deserializer)?;
    // Convert i64 millis back to Timestamp
    let secs = millis / 1000;
    let subsec_nanos = ((millis % 1000) * 1_000_000) as u32;
    let naive = DateTime::from_timestamp(secs, subsec_nanos)
        .ok_or_else(|| serde::de::Error::custom("Invalid timestamp"))?;
    Ok(Timestamp::from(naive))
}