use rustler::{Encoder, Env, NifResult, Term, SerdeTerm, types::map::MapIterator};
use std::collections::HashMap;
use std::sync::Arc;
use eppo_core::{AttributeValue, Str, events::AssignmentEvent, ufc::AssignmentValue};
use rustler::types::atom;


pub fn convert_attributes(subject_attributes: Term) -> NifResult<Arc<HashMap<Str, AttributeValue>>> {
    // Obtain an iterator over the map's key-value pairs.
    let mut attributes = HashMap::new();

    // Decode the Term into a MapIterator
    let iterator: MapIterator = subject_attributes.decode()?;
    for (key, value_term) in iterator {
        // Try to decode the value as one of the supported types.
        let attr_value = if let Ok(b) = value_term.decode::<bool>() {
            // Booleans are stored as categorical attributes.
            AttributeValue::categorical(b)
        } else if let Ok(i) = value_term.decode::<i64>() {
            // Integers are converted to f64 and stored as numeric.
            AttributeValue::numeric(i as f64)
        } else if let Ok(f) = value_term.decode::<f64>() {
            AttributeValue::numeric(f)
        } else if let Ok(s) = value_term.decode::<&str>() {
            // Strings are stored as categorical attributes.
            AttributeValue::categorical(s)
        } else {
            // If none of the supported types matched, return a null attribute.
            AttributeValue::null()
        };

        let key_str: &str = key.decode()?;
        attributes.insert(Str::new(key_str), attr_value);
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
        Ok(SerdeTerm(&event).encode(env))
    } else {
        Ok(atom::nil().encode(env))
    }
}
