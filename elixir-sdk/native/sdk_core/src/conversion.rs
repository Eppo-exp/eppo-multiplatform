use rustler::{Encoder, Env, NifResult, Term, SerdeTerm};
use std::collections::HashMap;
use std::sync::Arc;
use eppo_core::{AttributeValue, Str, events::AssignmentEvent, ufc::AssignmentValue};
use rustler::types::atom;


pub fn convert_attributes(subject_attributes: Term) -> NifResult<Arc<HashMap<Str, AttributeValue>>> {
    subject_attributes
        .decode()
        .map(Arc::new)
        .map_err(|e| rustler::Error::Term(Box::new(format!("Failed to decode attributes: {:?}", e))))
}

pub fn convert_value_term<'a>(env: Env<'a>, value: AssignmentValue) -> NifResult<Term<'a>> {
    let term = match value {
        AssignmentValue::String(s) => s.encode(env),
        AssignmentValue::Integer(i) => i.encode(env),
        AssignmentValue::Numeric(n) => n.encode(env),
        AssignmentValue::Boolean(b) => b.encode(env),
        AssignmentValue::Json { raw, .. } => raw.encode(env),
    };
    Ok(term)
}

pub fn convert_event_term<'a>(env: Env<'a>, event: Option<AssignmentEvent>) -> NifResult<Term<'a>> {
    Ok(event
        .map(|e| SerdeTerm(&e).encode(env))
        .unwrap_or_else(|| atom::nil().encode(env)))
}