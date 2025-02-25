---
"eppo_core": patch
"python-sdk": patch
"ruby-sdk": patch
"rust-sdk": patch
---

Fix `AttributeValue` serialization, so `Null` attributes are properly serialized as None instead of unit value.
