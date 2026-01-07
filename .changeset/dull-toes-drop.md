---
"python-sdk": patch
"eppo_core": patch
"ruby-sdk": patch
"elixir-sdk": patch
"rust-sdk": patch
---

Added experimental support for CityHash-based hashing in bandit evaluation via the `EPPO_EXPERIMENTAL_BANDITS_CITYHASH` environment variable (set to `"1"`, `"true"`, or `"TRUE"` to enable). This provides significant performance improvements over the default MD5 implementation, especially when evaluating bandits with many actions.

**Warning**: This feature is experimental and unstable. Enabling CityHash will produce different bandit evaluation results compared to the default MD5 implementation and other Eppo SDKs. Do not enable this if you need consistent results across multiple SDKs, services, or for historical data comparisons.
