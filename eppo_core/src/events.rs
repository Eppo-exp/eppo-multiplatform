use std::{collections::HashMap, sync::Arc};

use serde::Serialize;

use crate::{eval::eval_details::EvaluationDetails, ArcStr, Attributes, SdkMetadata};

/// Events that can be emitted during evaluation of assignment or bandit. They need to be logged to
/// analytics storage and fed back to Eppo for analysis.
#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Events {
    pub assignment: Option<AssignmentEvent>,
    pub bandit: Option<BanditEvent>,
}

/// Represents an event capturing the assignment of a feature flag to a subject and its logging
/// details.
#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentEvent {
    /// The key of the feature flag being assigned.
    pub feature_flag: ArcStr,
    /// The key of the allocation that the subject was assigned to.
    pub allocation: ArcStr,
    /// The key of the experiment associated with the assignment.
    pub experiment: String,
    /// The specific variation assigned to the subject.
    pub variation: ArcStr,
    /// The key identifying the subject receiving the assignment.
    pub subject: ArcStr,
    /// Custom attributes of the subject relevant to the assignment.
    pub subject_attributes: Arc<Attributes>,
    /// The timestamp indicating when the assignment event occurred.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional metadata such as SDK language and version.
    pub meta_data: EventMetaData,
    /// Additional user-defined logging fields for capturing extra information related to the
    /// assignment.
    #[serde(flatten)]
    pub extra_logging: Arc<HashMap<String, String>>,
    /// Evaluation details that could help with debugging the assigment. Only populated when
    /// details-version of the `get_assigment` was called.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluation_details: Option<Arc<EvaluationDetails>>,
}

/// Bandit evaluation event that needs to be logged to analytics storage.
#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BanditEvent {
    pub flag_key: String,
    pub bandit_key: String,
    pub subject: ArcStr,
    pub action: String,
    pub action_probability: f64,
    pub optimality_gap: f64,
    pub model_version: String,
    pub timestamp: String,
    pub subject_numeric_attributes: HashMap<String, f64>,
    pub subject_categorical_attributes: HashMap<String, String>,
    pub action_numeric_attributes: HashMap<String, f64>,
    pub action_categorical_attributes: HashMap<String, String>,
    pub meta_data: EventMetaData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventMetaData {
    pub sdk_name: &'static str,
    pub sdk_version: &'static str,
    pub core_version: &'static str,
}

impl From<SdkMetadata> for EventMetaData {
    fn from(sdk: SdkMetadata) -> EventMetaData {
        (&sdk).into()
    }
}

impl From<&SdkMetadata> for EventMetaData {
    fn from(sdk: &SdkMetadata) -> EventMetaData {
        EventMetaData {
            sdk_name: sdk.name,
            sdk_version: sdk.version,
            core_version: env!("CARGO_PKG_VERSION"),
        }
    }
}

#[cfg(feature = "pyo3")]
mod pyo3_impl {
    use pyo3::{PyObject, PyResult, Python};

    use crate::pyo3::TryToPyObject;

    use super::{AssignmentEvent, BanditEvent};

    impl TryToPyObject for AssignmentEvent {
        fn try_to_pyobject(&self, py: Python) -> PyResult<PyObject> {
            serde_pyobject::to_pyobject(py, self)
                .map(|it| it.unbind())
                .map_err(|err| err.0)
        }
    }

    impl TryToPyObject for BanditEvent {
        fn try_to_pyobject(&self, py: Python) -> PyResult<PyObject> {
            serde_pyobject::to_pyobject(py, self)
                .map(|it| it.unbind())
                .map_err(|err| err.0)
        }
    }
}
