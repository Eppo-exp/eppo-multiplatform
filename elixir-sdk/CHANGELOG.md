# elixir-sdk

## 0.2.4

### Patch Changes

- [#330](https://github.com/Eppo-exp/eppo-multiplatform/pull/330) [`173b0f5`](https://github.com/Eppo-exp/eppo-multiplatform/commit/173b0f58bc6e3a449d8d0d54381f19c688c0fe3f) Thanks [@dependabot](https://github.com/apps/dependabot)! - chore(deps): bump log from 0.4.25 to 0.4.28

- [#388](https://github.com/Eppo-exp/eppo-multiplatform/pull/388) [`9120912`](https://github.com/Eppo-exp/eppo-multiplatform/commit/9120912cae13e4e7e980c55c544a1e6d1626aeea) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Fix SDK incorrectly returning default values instead of false variants.

- [#394](https://github.com/Eppo-exp/eppo-multiplatform/pull/394) [`aa6130d`](https://github.com/Eppo-exp/eppo-multiplatform/commit/aa6130d19693b6318c826e450ec09bc455460b86) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Added experimental support for CityHash-based hashing in bandit evaluation via the `EPPO_EXPERIMENTAL_BANDITS_CITYHASH` environment variable (set to `"1"`, `"true"`, or `"TRUE"` to enable). This provides significant performance improvements over the default MD5 implementation, especially when evaluating bandits with many actions.

  **Warning**: This feature is experimental and unstable. Enabling CityHash will produce different bandit evaluation results compared to the default MD5 implementation and other Eppo SDKs. Do not enable this if you need consistent results across multiple SDKs, services, or for historical data comparisons.

- [#397](https://github.com/Eppo-exp/eppo-multiplatform/pull/397) [`db85656`](https://github.com/Eppo-exp/eppo-multiplatform/commit/db856561ab07e84ff50748986cf2224fb7a97718) Thanks [@dependabot](https://github.com/apps/dependabot)! - chore(deps): bump tokio from 1.48.0 to 1.49.0

- [#389](https://github.com/Eppo-exp/eppo-multiplatform/pull/389) [`066661a`](https://github.com/Eppo-exp/eppo-multiplatform/commit/066661a54161cd511a144be86eea254faa7dd61b) Thanks [@dependabot](https://github.com/apps/dependabot)! - chore(deps): bump ring from 0.17.9 to 0.17.14

- [#332](https://github.com/Eppo-exp/eppo-multiplatform/pull/332) [`176028b`](https://github.com/Eppo-exp/eppo-multiplatform/commit/176028bd4d046a064076787ccd6cd7760b5388bd) Thanks [@dependabot](https://github.com/apps/dependabot)! - chore(deps): bump serde_json from 1.0.138 to 1.0.145

- [#391](https://github.com/Eppo-exp/eppo-multiplatform/pull/391) [`415a90f`](https://github.com/Eppo-exp/eppo-multiplatform/commit/415a90f188cae978e98a8f944502ef7662bd7861) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Use faster md5 implementation.

- [#350](https://github.com/Eppo-exp/eppo-multiplatform/pull/350) [`91b44b7`](https://github.com/Eppo-exp/eppo-multiplatform/commit/91b44b79d8d301261838a83710e21ae18c00b64a) Thanks [@dependabot](https://github.com/apps/dependabot)! - chore(deps): bump tokio from 1.44.1 to 1.48.0

- [#392](https://github.com/Eppo-exp/eppo-multiplatform/pull/392) [`8995232`](https://github.com/Eppo-exp/eppo-multiplatform/commit/89952327ca6d5c863e7f06ce4f9903ce72e3223f) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Improve bandit evaluation performance.

## 0.2.3

### Patch Changes

- [#259](https://github.com/Eppo-exp/eppo-multiplatform/pull/259) [`d316fd3`](https://github.com/Eppo-exp/eppo-multiplatform/commit/d316fd34e1b4ebf4d058ede3c76b853cb7222799) Thanks [@schmit](https://github.com/schmit)! - Fix cargo configuration

## 0.2.2

### Patch Changes

- [#257](https://github.com/Eppo-exp/eppo-multiplatform/pull/257) [`d805868`](https://github.com/Eppo-exp/eppo-multiplatform/commit/d8058688cdbf3d273d688d8993ee377aab53e267) Thanks [@schmit](https://github.com/schmit)! - Add native/core_sdk to package.files

## 0.2.1

### Patch Changes

- [#254](https://github.com/Eppo-exp/eppo-multiplatform/pull/254) [`c4e15cf`](https://github.com/Eppo-exp/eppo-multiplatform/commit/c4e15cfb2aab4047420315533f9dd4052cddbaf9) Thanks [@schmit](https://github.com/schmit)! - Fix eppo_core import

## 0.2.0

### Minor Changes

- [#249](https://github.com/Eppo-exp/eppo-multiplatform/pull/249) [`d7703e2`](https://github.com/Eppo-exp/eppo-multiplatform/commit/d7703e2c50dd257966ccffda6bfd5a8cbcc7edff) Thanks [@schmit](https://github.com/schmit)! - Add `wait_for_initialization()` method.
