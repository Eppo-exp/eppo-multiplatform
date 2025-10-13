//! Helpers for Python SDK implementation.
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};

/// Similar to [`pyo3::ToPyObject`] but allows the conversion to fail.
pub trait TryToPyObject {
    fn try_to_pyobject(&self, py: Python) -> PyResult<PyObject>;
}

// Implementing on `&T` to allow dtolnay specialization[1] (e.g., for `Option<T>` below).
//
// [1]: https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md
impl<T: ToPyObject> TryToPyObject for &T {
    fn try_to_pyobject(&self, py: Python) -> PyResult<PyObject> {
        Ok(self.to_object(py))
    }
}

impl<T> TryToPyObject for Py<T> {
    fn try_to_pyobject(&self, py: Python) -> PyResult<PyObject> {
        Ok(self.to_object(py))
    }
}

impl<T: TryToPyObject> TryToPyObject for Option<T> {
    fn try_to_pyobject(&self, py: Python) -> PyResult<PyObject> {
        match self {
            Some(it) => it.try_to_pyobject(py),
            None => Ok(py.None()),
        }
    }
}

// Custom implementation for serde_json::Value because the default one serializes `Null` as empty
// tuple `()`.
impl TryToPyObject for serde_json::Value {
    fn try_to_pyobject(&self, py: Python) -> PyResult<PyObject> {
        let obj = match self {
            serde_json::Value::Null => py.None(),
            serde_json::Value::Bool(v) => v.try_to_pyobject(py)?,
            serde_json::Value::Number(n) => serde_pyobject::to_pyobject(py, n)?.unbind(),
            serde_json::Value::String(s) => s.try_to_pyobject(py)?,
            serde_json::Value::Array(values) => {
                let vals = values
                    .iter()
                    .map(|it| it.try_to_pyobject(py))
                    .collect::<Result<Vec<_>, _>>()?;
                PyList::new(py, vals)?.into_any().unbind()
            }
            serde_json::Value::Object(map) => {
                let dict = PyDict::new(py);
                for (key, value) in map {
                    dict.set_item(key, value.try_to_pyobject(py)?)?;
                }
                dict.into_any().unbind()
            }
        };
        Ok(obj)
    }
}
