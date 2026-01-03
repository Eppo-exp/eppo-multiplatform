//! Helpers for Python SDK implementation.
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};

// Custom implementation for serde_json::Value because the default one serializes `Null` as empty
// tuple `()`. This one serializes `Null` as python's `None`.
pub fn serde_to_pyobject<'py>(
    value: &serde_json::Value,
    py: Python<'py>,
) -> PyResult<Bound<'py, PyAny>> {
    let obj = match value {
        serde_json::Value::Null => py.None().into_bound(py),
        serde_json::Value::Bool(v) => {
            let Ok(obj) = v.into_pyobject(py); // infallible
            obj.as_any().clone()
        }
        serde_json::Value::Number(n) => serde_pyobject::to_pyobject(py, n)?,
        serde_json::Value::String(s) => {
            let Ok(obj) = s.into_pyobject(py); // infallible
            obj.into_any()
        }
        serde_json::Value::Array(values) => {
            let vals = values
                .iter()
                .map(|it| serde_to_pyobject(it, py))
                .collect::<Result<Vec<_>, _>>()?;
            PyList::new(py, vals)?.into_any()
        }
        serde_json::Value::Object(map) => {
            let dict = PyDict::new(py);
            for (key, value) in map {
                dict.set_item(key, serde_to_pyobject(value, py)?)?;
            }
            dict.into_any()
        }
    };
    Ok(obj)
}
