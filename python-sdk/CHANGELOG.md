# python-sdk

## 4.4.1

### Patch Changes

- [#382](https://github.com/Eppo-exp/eppo-multiplatform/pull/382) [`78515fb`](https://github.com/Eppo-exp/eppo-multiplatform/commit/78515fb0e706ed574acdabae75678e4e2bf10062) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Update pyo3 to 0.27.2. Support python 3.14.

- [#394](https://github.com/Eppo-exp/eppo-multiplatform/pull/394) [`aa6130d`](https://github.com/Eppo-exp/eppo-multiplatform/commit/aa6130d19693b6318c826e450ec09bc455460b86) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Added experimental support for CityHash-based hashing in bandit evaluation via the `EPPO_EXPERIMENTAL_BANDITS_CITYHASH` environment variable (set to `"1"`, `"true"`, or `"TRUE"` to enable). This provides significant performance improvements over the default MD5 implementation, especially when evaluating bandits with many actions.

  **Warning**: This feature is experimental and unstable. Enabling CityHash will produce different bandit evaluation results compared to the default MD5 implementation and other Eppo SDKs. Do not enable this if you need consistent results across multiple SDKs, services, or for historical data comparisons.

- [#391](https://github.com/Eppo-exp/eppo-multiplatform/pull/391) [`415a90f`](https://github.com/Eppo-exp/eppo-multiplatform/commit/415a90f188cae978e98a8f944502ef7662bd7861) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Use faster md5 implementation.

- [#392](https://github.com/Eppo-exp/eppo-multiplatform/pull/392) [`8995232`](https://github.com/Eppo-exp/eppo-multiplatform/commit/89952327ca6d5c863e7f06ce4f9903ce72e3223f) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Improve bandit evaluation performance.

- [#393](https://github.com/Eppo-exp/eppo-multiplatform/pull/393) [`21b48a2`](https://github.com/Eppo-exp/eppo-multiplatform/commit/21b48a2fdebb29c402670282935d8f958edc75c1) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - perf: use faster hash function for internal hashmaps (ahash).

## 4.4.0

### Minor Changes

- [#338](https://github.com/Eppo-exp/eppo-multiplatform/pull/338) [`3211582`](https://github.com/Eppo-exp/eppo-multiplatform/commit/3211582621f173482c93295ffb25cdf40e9fe324) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Add shutdown() method for graceful client shutdown.

### Patch Changes

- [#344](https://github.com/Eppo-exp/eppo-multiplatform/pull/344) [`084fe1f`](https://github.com/Eppo-exp/eppo-multiplatform/commit/084fe1f4d2a261bec8276ac3794b3f0d140ec728) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - fix(python): serialize null values as None instead of empty tuple.

## 4.3.1

### Patch Changes

- [#245](https://github.com/Eppo-exp/eppo-multiplatform/pull/245) [`2ba0c55`](https://github.com/Eppo-exp/eppo-multiplatform/commit/2ba0c55d6281f2a716f7b8e49c3427397741cf8d) Thanks [@dependabot](https://github.com/apps/dependabot)! - chore(deps): update serde-pyobject requirement from 0.5.0 to 0.6.0

## 4.3.0

### Minor Changes

- [#197](https://github.com/Eppo-exp/eppo-multiplatform/pull/197) [`a4da91f`](https://github.com/Eppo-exp/eppo-multiplatform/commit/a4da91f1a962708924063f3f076d3064441c2f76) Thanks [@rasendubi](https://github.com/rasendubi)! - Change TLS implementation from openssl to rustls.

## 4.2.5

### Patch Changes

- [#212](https://github.com/Eppo-exp/eppo-multiplatform/pull/212) [`095c5f5`](https://github.com/Eppo-exp/eppo-multiplatform/commit/095c5f54b48a8d41bae53125507a9939ae5ce9ec) Thanks [@bennettandrews](https://github.com/bennettandrews)! - Fix `AttributeValue` serialization, so `Null` attributes are properly serialized as None instead of unit value.

- [#213](https://github.com/Eppo-exp/eppo-multiplatform/pull/213) [`9ea7865`](https://github.com/Eppo-exp/eppo-multiplatform/commit/9ea78657dbbfe8fb733dd67fb71357872db9f8b2) Thanks [@rasendubi](https://github.com/rasendubi)! - Bump Minimum Supported Rust Version (MSRV) to 1.80.0.

## 4.2.4

### Patch Changes

- [#185](https://github.com/Eppo-exp/eppo-multiplatform/pull/185) [`1623ee2`](https://github.com/Eppo-exp/eppo-multiplatform/commit/1623ee215be5f07075f25a7c7413697082fd90cc) Thanks [@dependabot](https://github.com/apps/dependabot)! - [core] update rand requirement from 0.8.5 to 0.9.0

- [#168](https://github.com/Eppo-exp/eppo-multiplatform/pull/168) [`9d40446`](https://github.com/Eppo-exp/eppo-multiplatform/commit/9d40446c2346ac0869566699100baf69287da560) Thanks [@rasendubi](https://github.com/rasendubi)! - refactor(core): split poller thread into background thread and configuration poller.

  In preparation for doing more work in the background, we're refactoring poller thread into a more generic background thread / background runtime with configuration poller running on top of it.

  This changes API of the core but should be invisible for SDKs. The only noticeable difference is that client should be more responsive to graceful shutdown requests.

## 4.2.3

### Patch Changes

- [#171](https://github.com/Eppo-exp/eppo-multiplatform/pull/171) [`d4ac73f`](https://github.com/Eppo-exp/eppo-multiplatform/commit/d4ac73fa44627f78c0a325689e8263e120131443) Thanks [@rasendubi](https://github.com/rasendubi)! - Update pyo3 dependencies, enable support cpython-3.13.

## 4.2.2

### Patch Changes

- Updated dependencies [[`aa0ca89`](https://github.com/Eppo-exp/eppo-multiplatform/commit/aa0ca8912bab269613d3da25c06f81b1f19ffb36)]:
  - eppo_core@7.0.2

## 4.2.1

### Patch Changes

- Updated dependencies [[`82d05ae`](https://github.com/Eppo-exp/eppo-multiplatform/commit/82d05aea0263639be56ba5667500f6940b4832ab)]:
  - eppo_core@7.0.1

## 4.2.0

### Minor Changes

- [#136](https://github.com/Eppo-exp/eppo-multiplatform/pull/136) [`74d42bf`](https://github.com/Eppo-exp/eppo-multiplatform/commit/74d42bf1afab1509b87711f0d62e730c8b51e996) Thanks [@rasendubi](https://github.com/rasendubi)! - Preserve types for numeric and boolean attributes.

  Previously, when using numeric and boolean attributes as context attributes, they were converted to strings. Now, the internal type is correctly preserved throughout evaluation to logging.

### Patch Changes

- Updated dependencies [[`3a18f95`](https://github.com/Eppo-exp/eppo-multiplatform/commit/3a18f95f0aa25030aeba6676b76e20862a5fcead)]:
  - eppo_core@7.0.0
