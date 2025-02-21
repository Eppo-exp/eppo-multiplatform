---
"eppo_core": major
---

Make async runtime abstract.

This introduces an `AsyncRuntime` trait that allows us to abstract over different async runtimes. This is required to support Dart SDK that doesn't use tokio runtime in web build.
