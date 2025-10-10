---
"eppo_core": minor
"rust-sdk": minor
---

Add `wait_for_configuration_timeout()` method.

In poor network conditions, `wait_for_configuration()` may block waiting on configuration indefinitely which may be undesired. Add a new `wait_for_configuration_timeout()` which allows specifying a timeout for waiting.
