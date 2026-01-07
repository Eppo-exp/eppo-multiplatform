#[cfg(feature = "ahash")]
pub use ahash::{HashMap, HashMapExt};

#[cfg(not(feature = "ahash"))]
pub use std::collections::HashMap;
