use eppo_core::{
    {Str,AttributeValue},
    events::AssignmentEvent,
    ufc::{Assignment, VariationType, AssignmentValue},
    eval::eval_details::EvaluationResultWithDetails,
};
use std::collections::HashMap;
use std::sync::Arc;
use rustler::ResourceArc;
use crate::EppoClient;


pub fn get_assignment_inner(
    client: ResourceArc<EppoClient>,
    flag_key: String,
    subject_key: String,
    eppo_attributes: Arc<HashMap<Str, AttributeValue>>,
    expected_type: VariationType,
) -> Result<Option<Assignment>, String> {
    // Get assignment
    let assignment = client.evaluator.get_assignment(
        &Str::new(flag_key),
        &Str::new(subject_key),
        &eppo_attributes,
        Some(expected_type)
    ).map_err(|e| format!("Failed to get assignment: {:?}", e))?;

    Ok(assignment)
}

pub type AssignmentDetailsResult = (
    EvaluationResultWithDetails<AssignmentValue>,
    Option<AssignmentEvent>,
);

pub fn get_assignment_details_inner(
    client: ResourceArc<EppoClient>,
    flag_key: String,
    subject_key: String,
    eppo_attributes: Arc<HashMap<Str, AttributeValue>>,
    expected_type: VariationType,
) -> Result<AssignmentDetailsResult, String> {
    let assignment_with_details = client.evaluator.get_assignment_details(
        &Str::new(flag_key),
        &Str::new(subject_key),
        &eppo_attributes,
        Some(expected_type)
    );

    Ok(assignment_with_details)
}