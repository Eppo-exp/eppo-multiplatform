# eppo_core

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
