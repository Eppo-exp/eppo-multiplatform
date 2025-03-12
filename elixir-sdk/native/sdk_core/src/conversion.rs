use rustler::{Encoder, Env, NifResult, Term, SerdeTerm, types::map::MapIterator};
use std::collections::HashMap;
use std::sync::Arc;
use eppo_core::{AttributeValue, Str, events::AssignmentEvent, ufc::AssignmentValue};
use rustler::types::atom;


pub fn convert_attributes(subject_attributes: Term) -> NifResult<Arc<HashMap<Str, AttributeValue>>> {
    // Directly decode the Term into our target type
    let attributes: HashMap<Str, AttributeValue> = subject_attributes.decode()?;
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
