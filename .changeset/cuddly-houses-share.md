---
"python-sdk": minor
"ruby-sdk": minor
---

Preserve types for numeric and boolean attributes.

Previously, when using numeric and boolean attributes as context attributes, they were converted to strings. Now, the internal type is correctly preserved throughout evaluation to logging.
