use rustler::{Encoder, Env, NifResult, Term};
use std::collections::HashMap;
use std::sync::Arc;
use eppo_core::{AttributeValue, Str, events::AssignmentEvent, ufc::AssignmentValue};
use rustler::types::atom;


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
            // If none of the supported types matched, return a null attribute.
            AttributeValue::null()
        };

        // Insert the converted key and attribute value into the HashMap.
        // Here we assume that `Str` implements conversion from String.
        attributes.insert(key.into(), attr_value);
    }
    Ok(Arc::new(attributes))
}

pub fn convert_value_term<'a>(env: Env<'a>, value: AssignmentValue) -> NifResult<Term<'a>> {
    match value {
        AssignmentValue::String(s) => Ok(s.encode(env)),
        AssignmentValue::Integer(i) => Ok(i.encode(env)),
        AssignmentValue::Numeric(n) => Ok(n.encode(env)),
        AssignmentValue::Boolean(b) => Ok(b.encode(env)),
        AssignmentValue::Json { raw, .. } => Ok(raw.encode(env)),
    }
}

pub fn convert_event_term<'a>(env: Env<'a>, event: Option<AssignmentEvent>) -> NifResult<Term<'a>> {
    if let Some(event) = event {
        let json_value = serde_json::to_value(&event)
            .map_err(|e| rustler::Error::Term(Box::new(format!("Failed to serialize event: {:?}", e))))?;
        Ok(json_value.to_string().encode(env))
    } else {
        Ok(atom::nil().encode(env))
    }
}
