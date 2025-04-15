---
"eppo_core": minor
"ruby-sdk": minor
---

Fix casing of `evaluationDetails`.

In Ruby SDK v3.4.0, the name of `evaluationDetails` was inadvertently changed to `evaluation_details`. This was a bug that caused backward incompatibility in a minor release.

This release fixes the casing back to `evaluationDetails`.
