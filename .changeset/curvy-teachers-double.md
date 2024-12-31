---
"rust-sdk": major
---

Preserve types for numeric and boolean attributes. Allow `Attribute` type to encode attribute kind as well (numeric or categorical).

Previously, when using numeric and boolean attributes as context attributes, they were converted to strings. Now, the internal type is correctly preserved throughout evaluation to logging.

As part of this change, attribute types were reworked and their implementation was made private. We also expose new `NumericAttribute` and `CategoricalAttribute` types, and new constructors on `Attribute`.
