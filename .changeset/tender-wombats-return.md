---
"eppo_core": major
---

[core] Refactor: make Configuration implementation private.

This allows further evolution of configuration without breaking users.

The change should be invisible to SDKs.
