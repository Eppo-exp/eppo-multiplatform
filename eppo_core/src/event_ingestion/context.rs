use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Str;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::From)]
#[serde(untagged)]
pub enum ContextValue {
    String(Str),
    Number(f64),
    Boolean(bool),
    Null,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum ContextError {
    #[error("JSON value cannot be an object or an array")]
    InvalidContextValueType,
}

impl ContextValue {
    pub fn try_from_json(value: Value) -> Result<Self, ContextError> {
        match value {
            Value::String(s) => Ok(ContextValue::String(s.into())),
            Value::Number(n) => Ok(ContextValue::Number(n.as_f64().unwrap())), // Safe unwrap since it's always f64 or i64
            Value::Bool(b) => Ok(ContextValue::Boolean(b)),
            Value::Null => Ok(ContextValue::Null),
            _ => Err(ContextError::InvalidContextValueType),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialization() {
        #[derive(Debug, Serialize)]
        struct IngestionRequestBody<'a> {
            context: &'a BTreeMap<String, ContextValue>,
        }

        let mut context = BTreeMap::new();
        context.insert("key1".to_string(), ContextValue::String("value1".into()));
        context.insert("key2".to_string(), ContextValue::Number(42.0));
        context.insert("key3".to_string(), ContextValue::Boolean(true));
        context.insert("key4".to_string(), ContextValue::Null);

        let body = IngestionRequestBody { context: &context };

        let json = serde_json::to_string(&body).unwrap();
        assert_eq!(
            json,
            "{\"context\":{\"key1\":\"value1\",\"key2\":42.0,\"key3\":true,\"key4\":null}}"
        );
    }

    #[test]
    fn test_context_invalid_values() {
        assert_eq!(
            ContextValue::try_from_json(json!({"foo": "bar"})),
            Err(ContextError::InvalidContextValueType)
        );
        assert_eq!(
            ContextValue::try_from_json(json!([1, 2, 3])),
            Err(ContextError::InvalidContextValueType)
        );
    }
}
