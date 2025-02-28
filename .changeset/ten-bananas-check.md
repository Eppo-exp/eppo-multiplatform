---
"eppo_core": patch
---

Fix configuration poller running in wasm target.

It was failing because time is not implemented for wasm platform. We use wasmtimer for that now.
