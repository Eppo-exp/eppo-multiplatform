---
"eppo_core": minor
---

[ahash] Add "ahash" feature flag to use faster hash function for all hashmaps. It is changing the public interface so is disabled by default so as to not cause breakage across SDKs and allow them to update one by one.
