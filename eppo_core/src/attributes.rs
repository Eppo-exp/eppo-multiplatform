use std::{borrow::Cow, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{hashmap::HashMap, Str};

mod context_attributes;

pub use context_attributes::ContextAttributes;

/// Type alias for a HashMap representing key-value pairs of attributes.
///
/// Keys are strings representing attribute names.
///
/// # Examples
/// ```
/// # use eppo_core::Attributes;
/// let attributes = [
///     ("age".into(), 30.0.into()),
///     ("is_premium_member".into(), true.into()),
///     ("username".into(), "john_doe".into()),
/// ].into_iter().collect::<Attributes>();
/// ```
pub type Attributes = HashMap<Str, AttributeValue>;

/// Attribute of a subject or action.
///
/// Stores attribute value (string, number, boolean) along with attribute kind (numeric or
/// categorical). Storing kind is helpful to make `Attributes` ↔ `ContextAttributes` conversion
/// isomorphic.
///
/// Note that attribute kind is stripped during serialization, so Attribute → JSON → Attribute
/// conversion is lossy.
#[derive(Debug, Clone, PartialEq, PartialOrd, derive_more::From, Serialize, Deserialize)]
#[from(NumericAttribute, CategoricalAttribute, f64, bool, Str, String, &str, Arc<str>, Arc<String>, Cow<'_, str>)]
pub struct AttributeValue(AttributeValueImpl);
#[derive(Debug, Clone, PartialEq, PartialOrd, derive_more::From, Deserialize)]
#[serde(untagged)]
enum AttributeValueImpl {
    #[from(NumericAttribute, f64)]
    Numeric(NumericAttribute),
    #[from(CategoricalAttribute, Str, bool, String, &str, Arc<str>, Arc<String>, Cow<'_, str>)]
    Categorical(CategoricalAttribute),
    #[from(ignore)]
    Null,
}

impl serde::Serialize for AttributeValueImpl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            AttributeValueImpl::Numeric(numeric_attribute) => {
                numeric_attribute.serialize(serializer)
            }
            AttributeValueImpl::Categorical(categorical_attribute) => {
                categorical_attribute.serialize(serializer)
            }
            AttributeValueImpl::Null => serializer.serialize_none(),
        }
    }
}

impl AttributeValue {
    /// Create a numeric attribute.
    #[inline]
    pub fn numeric(value: impl Into<NumericAttribute>) -> AttributeValue {
        AttributeValue(AttributeValueImpl::Numeric(value.into()))
    }

    /// Create a categorical attribute.
    #[inline]
    pub fn categorical(value: impl Into<CategoricalAttribute>) -> AttributeValue {
        AttributeValue(AttributeValueImpl::Categorical(value.into()))
    }

    #[inline]
    pub const fn null() -> AttributeValue {
        AttributeValue(AttributeValueImpl::Null)
    }

    pub(crate) fn is_null(&self) -> bool {
        self == &AttributeValue(AttributeValueImpl::Null)
    }

    /// Try coercing attribute to a number.
    ///
    /// Number attributes are returned as is. For string attributes, we try to parse them into a
    /// number.
    pub(crate) fn coerce_to_number(&self) -> Option<f64> {
        match self.as_attribute_value()? {
            AttributeValueRef::Number(v) => Some(v),
            AttributeValueRef::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Try coercing attribute to a string.
    ///
    /// String attributes are returned as is. Number and boolean attributes are converted to string.
    pub(crate) fn coerce_to_string(&self) -> Option<Cow<str>> {
        match self.as_attribute_value()? {
            AttributeValueRef::String(s) => Some(Cow::Borrowed(s)),
            AttributeValueRef::Number(v) => Some(Cow::Owned(v.to_string())),
            AttributeValueRef::Boolean(v) => Some(Cow::Borrowed(if v { "true" } else { "false" })),
        }
    }

    pub(crate) fn as_str(&self) -> Option<&Str> {
        match self {
            AttributeValue(AttributeValueImpl::Categorical(CategoricalAttribute(
                CategoricalAttributeImpl::String(s),
            ))) => Some(s),
            _ => None,
        }
    }

    fn as_attribute_value<'a>(&'a self) -> Option<AttributeValueRef<'a>> {
        self.into()
    }
}

/// Numeric attributes are quantitative (e.g., real numbers) and define a scale.
///
/// Not all numbers in programming are numeric attributes. If a number is used to represent an
/// enumeration or on/off values, it is a [categorical attribute](CategoricalAttribute).
#[derive(
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    derive_more::From,
    derive_more::Into,
    Serialize,
    Deserialize,
)]
pub struct NumericAttribute(f64);

impl NumericAttribute {
    pub(crate) fn to_f64(&self) -> f64 {
        self.0
    }
}

/// Categorical attributes are attributes that have a finite set of values that are not directly
/// comparable (i.e., enumeration).
#[derive(Debug, Clone, PartialEq, PartialOrd, derive_more::From, Serialize, Deserialize)]
#[from(Str, f64, bool, String, &str, Arc<str>, Arc<String>, Cow<'_, str>)]
pub struct CategoricalAttribute(CategoricalAttributeImpl);
#[derive(Debug, Clone, PartialEq, PartialOrd, derive_more::From, Serialize, Deserialize)]
#[serde(untagged)]
enum CategoricalAttributeImpl {
    #[from(forward)]
    String(Str),
    #[from]
    Number(f64),
    #[from]
    Boolean(bool),
}

impl CategoricalAttribute {
    pub(crate) fn to_str(&self) -> Cow<str> {
        match self {
            CategoricalAttribute(CategoricalAttributeImpl::String(s)) => Cow::Borrowed(s),
            CategoricalAttribute(CategoricalAttributeImpl::Number(v)) => Cow::Owned(v.to_string()),
            CategoricalAttribute(CategoricalAttributeImpl::Boolean(v)) => {
                Cow::Borrowed(if *v { "true" } else { "false" })
            }
        }
    }
}

/// Enum representing values of an attribute.
///
/// It's a intermediate non-owning representation.
#[derive(Debug, Clone, Copy)]
enum AttributeValueRef<'a> {
    /// A string value.
    String(&'a Str),
    /// A numerical value.
    Number(f64),
    /// A boolean value.
    Boolean(bool),
}

impl<'a> From<&'a AttributeValue> for Option<AttributeValueRef<'a>> {
    fn from(value: &'a AttributeValue) -> Self {
        match value {
            AttributeValue(AttributeValueImpl::Numeric(numeric)) => {
                Some(AttributeValueRef::from(numeric))
            }
            AttributeValue(AttributeValueImpl::Categorical(categorical)) => {
                Some(AttributeValueRef::from(categorical))
            }
            AttributeValue(AttributeValueImpl::Null) => None,
        }
    }
}

impl From<&NumericAttribute> for AttributeValueRef<'_> {
    #[inline]
    fn from(value: &NumericAttribute) -> Self {
        AttributeValueRef::Number(value.0)
    }
}

impl<'a> From<&'a CategoricalAttribute> for AttributeValueRef<'a> {
    fn from(value: &'a CategoricalAttribute) -> Self {
        match value {
            CategoricalAttribute(CategoricalAttributeImpl::String(v)) => {
                AttributeValueRef::String(v)
            }
            CategoricalAttribute(CategoricalAttributeImpl::Number(v)) => {
                AttributeValueRef::Number(*v)
            }
            CategoricalAttribute(CategoricalAttributeImpl::Boolean(v)) => {
                AttributeValueRef::Boolean(*v)
            }
        }
    }
}

#[cfg(feature = "pyo3")]
mod pyo3_impl {
    use pyo3::{exceptions::PyTypeError, prelude::*, types::*};

    use super::*;

    impl<'py> IntoPyObject<'py> for &AttributeValue {
        type Target = PyAny;
        type Output = Bound<'py, Self::Target>;
        type Error = std::convert::Infallible;

        #[inline]
        fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
            match self {
                AttributeValue(AttributeValueImpl::Numeric(numeric)) => {
                    numeric.into_pyobject(py).map(|it| it.into_any())
                }
                AttributeValue(AttributeValueImpl::Categorical(categorical)) => {
                    categorical.into_pyobject(py).map(|it| it.into_any())
                }
                AttributeValue(AttributeValueImpl::Null) => Ok(py.None().into_bound(py).into_any()),
            }
        }
    }

    impl<'py> IntoPyObject<'py> for &NumericAttribute {
        type Target = PyFloat;
        type Output = Bound<'py, Self::Target>;
        type Error = std::convert::Infallible;

        #[inline]
        fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
            self.0.into_pyobject(py)
        }
    }

    impl<'py> IntoPyObject<'py> for &CategoricalAttribute {
        type Target = PyAny;
        type Output = Bound<'py, Self::Target>;
        type Error = std::convert::Infallible;

        #[inline]
        fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
            match self {
                CategoricalAttribute(CategoricalAttributeImpl::String(s)) => {
                    s.into_pyobject(py).map(|it| it.into_any())
                }
                CategoricalAttribute(CategoricalAttributeImpl::Number(v)) => {
                    v.into_pyobject(py).map(|it| it.into_any())
                }
                CategoricalAttribute(CategoricalAttributeImpl::Boolean(v)) => {
                    v.into_pyobject(py).map(|it| it.as_any().clone())
                }
            }
        }
    }

    impl<'a, 'py> FromPyObject<'a, 'py> for AttributeValue {
        type Error = PyErr;

        fn extract(value: Borrowed<'a, 'py, PyAny>) -> PyResult<AttributeValue> {
            if let Ok(s) = value.cast::<PyString>() {
                return Ok(AttributeValue::categorical(s.to_cow()?));
            }
            // In Python, Bool inherits from Int, so it must be checked first here.
            if let Ok(s) = value.cast::<PyBool>() {
                return Ok(AttributeValue::categorical(s.is_true()));
            }
            if let Ok(s) = value.cast::<PyFloat>() {
                return Ok(AttributeValue::numeric(s.value()));
            }
            if let Ok(s) = value.cast::<PyInt>() {
                return Ok(AttributeValue::numeric(s.extract::<f64>()?));
            }
            if value.is_none() {
                return Ok(AttributeValue::null());
            }
            Err(PyTypeError::new_err(
                "invalid type for subject attribute value",
            ))
        }
    }

    impl<'a, 'py> FromPyObject<'a, 'py> for NumericAttribute {
        type Error = PyErr;

        #[inline]
        fn extract(value: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
            f64::extract(value).map(NumericAttribute)
        }
    }

    impl<'a, 'py> FromPyObject<'a, 'py> for CategoricalAttribute {
        type Error = PyErr;
        fn extract(value: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
            if let Ok(s) = value.cast::<PyString>() {
                return Ok(s.to_cow()?.into());
            }
            // In Python, Bool inherits from Int, so it must be checked first here.
            if let Ok(s) = value.cast::<PyBool>() {
                return Ok(s.is_true().into());
            }
            if let Ok(s) = value.cast::<PyFloat>() {
                return Ok(s.value().into());
            }
            if let Ok(s) = value.cast::<PyInt>() {
                return Ok(s.extract::<f64>()?.into());
            }
            Err(PyTypeError::new_err(
                "invalid type for categorical attribute value",
            ))
        }
    }
}

#[cfg(feature = "magnus")]
mod magnus_impl {
    use magnus::{value::ReprValue, RString, Ruby, TryConvert};

    use crate::{AttributeValue, CategoricalAttribute, NumericAttribute};

    use super::{AttributeValueImpl, CategoricalAttributeImpl};

    impl TryConvert for AttributeValue {
        fn try_convert(val: magnus::Value) -> Result<Self, magnus::Error> {
            (NumericAttribute::try_convert(val).map(|it| Self(AttributeValueImpl::Numeric(it))))
                .or_else(|_| {
                    CategoricalAttribute::try_convert(val)
                        .map(|it| Self(AttributeValueImpl::Categorical(it)))
                })
                .or_else(|_|
                // Return null attribute as a fallback
                Ok(Self(AttributeValueImpl::Null)))
        }
    }

    impl TryConvert for NumericAttribute {
        fn try_convert(val: magnus::Value) -> Result<Self, magnus::Error> {
            Ok(Self(TryConvert::try_convert(val)?))
        }
    }

    impl TryConvert for CategoricalAttribute {
        fn try_convert(val: magnus::Value) -> Result<Self, magnus::Error> {
            let ruby = Ruby::get_with(val);
            if let Some(s) = RString::from_value(val) {
                Ok(Self(CategoricalAttributeImpl::String(
                    s.to_string()?.into(),
                )))
            } else if let Ok(v) = f64::try_convert(val) {
                Ok(Self(CategoricalAttributeImpl::Number(v)))
            } else if val.is_kind_of(ruby.class_true_class()) {
                Ok(Self(CategoricalAttributeImpl::Boolean(true)))
            } else if val.is_kind_of(ruby.class_false_class()) {
                Ok(Self(CategoricalAttributeImpl::Boolean(false)))
            } else {
                Err(magnus::Error::new(
                    ruby.exception_type_error(),
                    "CategoricalAttribute must be one of String, Numeric, True, or False",
                ))
            }
        }
    }
}

#[cfg(feature = "rustler")]
mod rustler_impl {
    use super::*;
    use rustler::{Decoder, Encoder, Env, NifResult, Term};

    impl Encoder for AttributeValue {
        fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
            match &self.0 {
                AttributeValueImpl::Numeric(n) => n.0.encode(env),
                AttributeValueImpl::Categorical(c) => match &c.0 {
                    CategoricalAttributeImpl::String(s) => s.encode(env),
                    CategoricalAttributeImpl::Number(n) => n.encode(env),
                    CategoricalAttributeImpl::Boolean(b) => b.encode(env),
                },
                AttributeValueImpl::Null => rustler::types::atom::nil().encode(env),
            }
        }
    }

    impl<'a> Decoder<'a> for AttributeValue {
        fn decode(term: Term<'a>) -> NifResult<Self> {
            if let Ok(b) = bool::decode(term) {
                return Ok(AttributeValue::categorical(b));
            }
            if let Ok(i) = i64::decode(term) {
                return Ok(AttributeValue::numeric(i as f64));
            }
            if let Ok(f) = f64::decode(term) {
                return Ok(AttributeValue::numeric(f));
            }
            if let Ok(s) = String::decode(term) {
                return Ok(AttributeValue::categorical(s));
            }
            Ok(AttributeValue::null())
        }
    }

    impl Encoder for Str {
        fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
            let s: &str = self.as_ref();
            s.encode(env)
        }
    }

    impl<'a> Decoder<'a> for Str {
        fn decode(term: Term<'a>) -> NifResult<Self> {
            let s: &str = term.decode()?;
            Ok(Str::new(s))
        }
    }
}
