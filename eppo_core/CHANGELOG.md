# eppo_core

## 10.0.0

### Major Changes

- [#382](https://github.com/Eppo-exp/eppo-multiplatform/pull/382) [`78515fb`](https://github.com/Eppo-exp/eppo-multiplatform/commit/78515fb0e706ed574acdabae75678e4e2bf10062) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - [pyo3] Update to pyo3 0.27, remove `TryToPyObject` trait because pyo3's `IntoPyObject` now properly handles errors.

### Minor Changes

- [#393](https://github.com/Eppo-exp/eppo-multiplatform/pull/393) [`21b48a2`](https://github.com/Eppo-exp/eppo-multiplatform/commit/21b48a2fdebb29c402670282935d8f958edc75c1) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - [ahash] Add "ahash" feature flag to use faster hash function for all hashmaps. It is changing the public interface so is disabled by default so as to not cause breakage across SDKs and allow them to update one by one.

- [#380](https://github.com/Eppo-exp/eppo-multiplatform/pull/380) [`6c66f91`](https://github.com/Eppo-exp/eppo-multiplatform/commit/6c66f91ba2b85c46591a3f361c037ce5c2d5e7a7) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - [ruby] support ruby-4.0

### Patch Changes

- [#394](https://github.com/Eppo-exp/eppo-multiplatform/pull/394) [`aa6130d`](https://github.com/Eppo-exp/eppo-multiplatform/commit/aa6130d19693b6318c826e450ec09bc455460b86) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Added experimental support for CityHash-based hashing in bandit evaluation via the `EPPO_EXPERIMENTAL_BANDITS_CITYHASH` environment variable (set to `"1"`, `"true"`, or `"TRUE"` to enable). This provides significant performance improvements over the default MD5 implementation, especially when evaluating bandits with many actions.

  **Warning**: This feature is experimental and unstable. Enabling CityHash will produce different bandit evaluation results compared to the default MD5 implementation and other Eppo SDKs. Do not enable this if you need consistent results across multiple SDKs, services, or for historical data comparisons.

- [#391](https://github.com/Eppo-exp/eppo-multiplatform/pull/391) [`415a90f`](https://github.com/Eppo-exp/eppo-multiplatform/commit/415a90f188cae978e98a8f944502ef7662bd7861) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Use faster md5 implementation.

- [#392](https://github.com/Eppo-exp/eppo-multiplatform/pull/392) [`8995232`](https://github.com/Eppo-exp/eppo-multiplatform/commit/89952327ca6d5c863e7f06ce4f9903ce72e3223f) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Improve bandit evaluation performance.

## 9.3.0

### Minor Changes

- [#338](https://github.com/Eppo-exp/eppo-multiplatform/pull/338) [`3211582`](https://github.com/Eppo-exp/eppo-multiplatform/commit/3211582621f173482c93295ffb25cdf40e9fe324) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Add by-ref graceful shutdown for BackgroundThread.

- [#339](https://github.com/Eppo-exp/eppo-multiplatform/pull/339) [`9a4d2a5`](https://github.com/Eppo-exp/eppo-multiplatform/commit/9a4d2a53b4477c55f3a4b254aef612d8006d8ae0) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Add `wait_for_configuration_timeout()` method.

  In poor network conditions, `wait_for_configuration()` may block waiting on configuration indefinitely which may be undesired. Add a new `wait_for_configuration_timeout()` which allows specifying a timeout for waiting.

### Patch Changes

- [#344](https://github.com/Eppo-exp/eppo-multiplatform/pull/344) [`084fe1f`](https://github.com/Eppo-exp/eppo-multiplatform/commit/084fe1f4d2a261bec8276ac3794b3f0d140ec728) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - fix(python): serialize null values as None instead of empty tuple.

## 9.2.0

### Minor Changes

- [#268](https://github.com/Eppo-exp/eppo-multiplatform/pull/268) [`2ec98c6`](https://github.com/Eppo-exp/eppo-multiplatform/commit/2ec98c6f006d9e86c186fc99a903188d9837d653) Thanks [@rasendubi](https://github.com/rasendubi)! - Fix casing of `evaluationDetails`.

  In Ruby SDK v3.4.0, the name of `evaluationDetails` was inadvertently changed to `evaluation_details`. This was a bug that caused backward incompatibility in a minor release.

  This release fixes the casing back to `evaluationDetails`.

## 9.1.1

### Patch Changes

- [#245](https://github.com/Eppo-exp/eppo-multiplatform/pull/245) [`2ba0c55`](https://github.com/Eppo-exp/eppo-multiplatform/commit/2ba0c55d6281f2a716f7b8e49c3427397741cf8d) Thanks [@dependabot](https://github.com/apps/dependabot)! - chore(deps): update serde-pyobject requirement from 0.5.0 to 0.6.0

## 9.1.0

### Minor Changes

- [#208](https://github.com/Eppo-exp/eppo-multiplatform/pull/208) [`f236e42`](https://github.com/Eppo-exp/eppo-multiplatform/commit/f236e424c01c162fe9a1c01210cb71928b9fab97) Thanks [@schmit](https://github.com/schmit)! - Add rustler (Elixir) `Encoder`/`Decoder` support for core types

### Patch Changes

- [#94](https://github.com/Eppo-exp/eppo-multiplatform/pull/94) [`30a0062`](https://github.com/Eppo-exp/eppo-multiplatform/commit/30a0062169f030edb6c7b6280850af7c618aae65) Thanks [@dependabot](https://github.com/apps/dependabot)! - Update magnus from 0.6.4 to 0.7.1

- [#94](https://github.com/Eppo-exp/eppo-multiplatform/pull/94) [`30a0062`](https://github.com/Eppo-exp/eppo-multiplatform/commit/30a0062169f030edb6c7b6280850af7c618aae65) Thanks [@dependabot](https://github.com/apps/dependabot)! - Update serde_magnus from 0.8.1 to 0.9.0

## 9.0.0

### Major Changes

- [#197](https://github.com/Eppo-exp/eppo-multiplatform/pull/197) [`a4da91f`](https://github.com/Eppo-exp/eppo-multiplatform/commit/a4da91f1a962708924063f3f076d3064441c2f76) Thanks [@rasendubi](https://github.com/rasendubi)! - Make async runtime abstract.

  This introduces an `AsyncRuntime` trait that allows us to abstract over different async runtimes. This is required to support Dart SDK that doesn't use tokio runtime in web build.

### Minor Changes

- [#197](https://github.com/Eppo-exp/eppo-multiplatform/pull/197) [`a4da91f`](https://github.com/Eppo-exp/eppo-multiplatform/commit/a4da91f1a962708924063f3f076d3064441c2f76) Thanks [@rasendubi](https://github.com/rasendubi)! - Change TLS implementation from openssl to rustls.

### Patch Changes

- [#197](https://github.com/Eppo-exp/eppo-multiplatform/pull/197) [`a4da91f`](https://github.com/Eppo-exp/eppo-multiplatform/commit/a4da91f1a962708924063f3f076d3064441c2f76) Thanks [@rasendubi](https://github.com/rasendubi)! - Fix configuration poller running in wasm target.

  It was failing because time is not implemented for wasm platform. We use wasmtimer for that now.

## 8.0.3

### Patch Changes

- [#212](https://github.com/Eppo-exp/eppo-multiplatform/pull/212) [`095c5f5`](https://github.com/Eppo-exp/eppo-multiplatform/commit/095c5f54b48a8d41bae53125507a9939ae5ce9ec) Thanks [@bennettandrews](https://github.com/bennettandrews)! - Fix `AttributeValue` serialization, so `Null` attributes are properly serialized as None instead of unit value.

- [#213](https://github.com/Eppo-exp/eppo-multiplatform/pull/213) [`9ea7865`](https://github.com/Eppo-exp/eppo-multiplatform/commit/9ea78657dbbfe8fb733dd67fb71357872db9f8b2) Thanks [@rasendubi](https://github.com/rasendubi)! - Bump Minimum Supported Rust Version (MSRV) to 1.80.0.

## 8.0.2

### Patch Changes

- [#201](https://github.com/Eppo-exp/eppo-multiplatform/pull/201) [`1d310c7`](https://github.com/Eppo-exp/eppo-multiplatform/commit/1d310c7019dde1aa5a965e064eab15187b064d96) Thanks [@felipecsl](https://github.com/felipecsl)! - [Unstable] Event Ingestion: Fix JSON serialization of Event timestamp field

## 8.0.1

### Patch Changes

- [#198](https://github.com/Eppo-exp/eppo-multiplatform/pull/198) [`9c6990e`](https://github.com/Eppo-exp/eppo-multiplatform/commit/9c6990ec77dc3ffe8f1b6384f92fcc24db94916f) Thanks [@felipecsl](https://github.com/felipecsl)! - [unstable] Event Ingestion: Fix JSON serialization of Event type field

## 8.0.0

### Major Changes

- [#168](https://github.com/Eppo-exp/eppo-multiplatform/pull/168) [`9d40446`](https://github.com/Eppo-exp/eppo-multiplatform/commit/9d40446c2346ac0869566699100baf69287da560) Thanks [@rasendubi](https://github.com/rasendubi)! - refactor(core): split poller thread into background thread and configuration poller.

  In preparation for doing more work in the background, we're refactoring poller thread into a more generic background thread / background runtime with configuration poller running on top of it.

  This changes API of the core but should be invisible for SDKs. The only noticeable difference is that client should be more responsive to graceful shutdown requests.

- [#180](https://github.com/Eppo-exp/eppo-multiplatform/pull/180) [`02a310d`](https://github.com/Eppo-exp/eppo-multiplatform/commit/02a310d4c0196821b29ff8cc4007374c41dfad26) Thanks [@rasendubi](https://github.com/rasendubi)! - [core] Refactor: make Configuration implementation private.

  This allows further evolution of configuration without breaking users.

  The change should be invisible to SDKs.

### Patch Changes

- [#185](https://github.com/Eppo-exp/eppo-multiplatform/pull/185) [`1623ee2`](https://github.com/Eppo-exp/eppo-multiplatform/commit/1623ee215be5f07075f25a7c7413697082fd90cc) Thanks [@dependabot](https://github.com/apps/dependabot)! - [core] update rand requirement from 0.8.5 to 0.9.0

- [#190](https://github.com/Eppo-exp/eppo-multiplatform/pull/190) [`8c44059`](https://github.com/Eppo-exp/eppo-multiplatform/commit/8c44059a5daf54b522db69c85589a6f04cc7b5a5) Thanks [@dependabot](https://github.com/apps/dependabot)! - chore(deps): update derive_more requirement from 1.0.0 to 2.0.0

## 7.0.3

### Patch Changes

- [#171](https://github.com/Eppo-exp/eppo-multiplatform/pull/171) [`d4ac73f`](https://github.com/Eppo-exp/eppo-multiplatform/commit/d4ac73fa44627f78c0a325689e8263e120131443) Thanks [@rasendubi](https://github.com/rasendubi)! - Update pyo3 dependencies, enable support cpython-3.13.

## 7.0.2

### Patch Changes

- [#164](https://github.com/Eppo-exp/eppo-multiplatform/pull/164) [`aa0ca89`](https://github.com/Eppo-exp/eppo-multiplatform/commit/aa0ca8912bab269613d3da25c06f81b1f19ffb36) Thanks [@rasendubi](https://github.com/rasendubi)! - Hide event ingestion under a feature flag.

## 7.0.1

### Patch Changes

- [#160](https://github.com/Eppo-exp/eppo-multiplatform/pull/160) [`82d05ae`](https://github.com/Eppo-exp/eppo-multiplatform/commit/82d05aea0263639be56ba5667500f6940b4832ab) Thanks [@leoromanovsky](https://github.com/leoromanovsky)! - add sync feature to tokio crate

## 7.0.0

### Major Changes

- [#145](https://github.com/Eppo-exp/eppo-multiplatform/pull/145) [`3a18f95`](https://github.com/Eppo-exp/eppo-multiplatform/commit/3a18f95f0aa25030aeba6676b76e20862a5fcead) Thanks [@leoromanovsky](https://github.com/leoromanovsky)! - precomputed bandits response flattened to Map<Str, PrecomputedBandit>
