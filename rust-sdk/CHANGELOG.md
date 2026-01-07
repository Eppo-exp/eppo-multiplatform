# rust-sdk

## 5.2.1

### Patch Changes

- [#394](https://github.com/Eppo-exp/eppo-multiplatform/pull/394) [`aa6130d`](https://github.com/Eppo-exp/eppo-multiplatform/commit/aa6130d19693b6318c826e450ec09bc455460b86) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Added experimental support for CityHash-based hashing in bandit evaluation via the `EPPO_EXPERIMENTAL_BANDITS_CITYHASH` environment variable (set to `"1"`, `"true"`, or `"TRUE"` to enable). This provides significant performance improvements over the default MD5 implementation, especially when evaluating bandits with many actions.

  **Warning**: This feature is experimental and unstable. Enabling CityHash will produce different bandit evaluation results compared to the default MD5 implementation and other Eppo SDKs. Do not enable this if you need consistent results across multiple SDKs, services, or for historical data comparisons.

- [#391](https://github.com/Eppo-exp/eppo-multiplatform/pull/391) [`415a90f`](https://github.com/Eppo-exp/eppo-multiplatform/commit/415a90f188cae978e98a8f944502ef7662bd7861) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Use faster md5 implementation.

- [#392](https://github.com/Eppo-exp/eppo-multiplatform/pull/392) [`8995232`](https://github.com/Eppo-exp/eppo-multiplatform/commit/89952327ca6d5c863e7f06ce4f9903ce72e3223f) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Improve bandit evaluation performance.

## 5.2.0

### Minor Changes

- [#339](https://github.com/Eppo-exp/eppo-multiplatform/pull/339) [`9a4d2a5`](https://github.com/Eppo-exp/eppo-multiplatform/commit/9a4d2a53b4477c55f3a4b254aef612d8006d8ae0) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Add `wait_for_configuration_timeout()` method.

  In poor network conditions, `wait_for_configuration()` may block waiting on configuration indefinitely which may be undesired. Add a new `wait_for_configuration_timeout()` which allows specifying a timeout for waiting.

## 5.1.0

### Minor Changes

- [#197](https://github.com/Eppo-exp/eppo-multiplatform/pull/197) [`a4da91f`](https://github.com/Eppo-exp/eppo-multiplatform/commit/a4da91f1a962708924063f3f076d3064441c2f76) Thanks [@rasendubi](https://github.com/rasendubi)! - Change TLS implementation from openssl to rustls.

## 5.0.4

### Patch Changes

- [#212](https://github.com/Eppo-exp/eppo-multiplatform/pull/212) [`095c5f5`](https://github.com/Eppo-exp/eppo-multiplatform/commit/095c5f54b48a8d41bae53125507a9939ae5ce9ec) Thanks [@bennettandrews](https://github.com/bennettandrews)! - Fix `AttributeValue` serialization, so `Null` attributes are properly serialized as None instead of unit value.

- [#213](https://github.com/Eppo-exp/eppo-multiplatform/pull/213) [`9ea7865`](https://github.com/Eppo-exp/eppo-multiplatform/commit/9ea78657dbbfe8fb733dd67fb71357872db9f8b2) Thanks [@rasendubi](https://github.com/rasendubi)! - Bump Minimum Supported Rust Version (MSRV) to 1.80.0.

## 5.0.3

### Patch Changes

- [#185](https://github.com/Eppo-exp/eppo-multiplatform/pull/185) [`1623ee2`](https://github.com/Eppo-exp/eppo-multiplatform/commit/1623ee215be5f07075f25a7c7413697082fd90cc) Thanks [@dependabot](https://github.com/apps/dependabot)! - [core] update rand requirement from 0.8.5 to 0.9.0

- [#168](https://github.com/Eppo-exp/eppo-multiplatform/pull/168) [`9d40446`](https://github.com/Eppo-exp/eppo-multiplatform/commit/9d40446c2346ac0869566699100baf69287da560) Thanks [@rasendubi](https://github.com/rasendubi)! - refactor(core): split poller thread into background thread and configuration poller.

  In preparation for doing more work in the background, we're refactoring poller thread into a more generic background thread / background runtime with configuration poller running on top of it.

  This changes API of the core but should be invisible for SDKs. The only noticeable difference is that client should be more responsive to graceful shutdown requests.

## 5.0.2

### Patch Changes

- Updated dependencies [[`aa0ca89`](https://github.com/Eppo-exp/eppo-multiplatform/commit/aa0ca8912bab269613d3da25c06f81b1f19ffb36)]:
  - eppo_core@7.0.2

## 5.0.1

### Patch Changes

- Updated dependencies [[`82d05ae`](https://github.com/Eppo-exp/eppo-multiplatform/commit/82d05aea0263639be56ba5667500f6940b4832ab)]:
  - eppo_core@7.0.1

## 5.0.0

### Major Changes

- [#136](https://github.com/Eppo-exp/eppo-multiplatform/pull/136) [`74d42bf`](https://github.com/Eppo-exp/eppo-multiplatform/commit/74d42bf1afab1509b87711f0d62e730c8b51e996) Thanks [@rasendubi](https://github.com/rasendubi)! - Preserve types for numeric and boolean attributes. Allow `Attribute` type to encode attribute kind as well (numeric or categorical).

  Previously, when using numeric and boolean attributes as context attributes, they were converted to strings. Now, the internal type is correctly preserved throughout evaluation to logging.

  As part of this change, attribute types were reworked and their implementation was made private. We also expose new `NumericAttribute` and `CategoricalAttribute` types, and new constructors on `Attribute`.

- [#136](https://github.com/Eppo-exp/eppo-multiplatform/pull/136) [`74d42bf`](https://github.com/Eppo-exp/eppo-multiplatform/commit/74d42bf1afab1509b87711f0d62e730c8b51e996) Thanks [@rasendubi](https://github.com/rasendubi)! - Exit poller thread when PollerThread handle is dropped/disconnected.

  This makes SDK more stable in the environment where the main process is killed without calling shutdown on the poller thread.

  For Rust SDK that is a breaking change as you now need to make sure that `PollerThread` handle is not dropped prematurely.

### Patch Changes

- [#136](https://github.com/Eppo-exp/eppo-multiplatform/pull/136) [`74d42bf`](https://github.com/Eppo-exp/eppo-multiplatform/commit/74d42bf1afab1509b87711f0d62e730c8b51e996) Thanks [@rasendubi](https://github.com/rasendubi)! - Fix poller thread crash when zero jitter value is specified.

- [#136](https://github.com/Eppo-exp/eppo-multiplatform/pull/136) [`74d42bf`](https://github.com/Eppo-exp/eppo-multiplatform/commit/74d42bf1afab1509b87711f0d62e730c8b51e996) Thanks [@rasendubi](https://github.com/rasendubi)! - Decrease default jitter from 30s to 3s.

  30s jitter was too high and caused configuration interval to vary between 0 to 30s (with average of 15s). The new value makes fetch interval vary between 27 to 30s.

- [#136](https://github.com/Eppo-exp/eppo-multiplatform/pull/136) [`74d42bf`](https://github.com/Eppo-exp/eppo-multiplatform/commit/74d42bf1afab1509b87711f0d62e730c8b51e996) Thanks [@rasendubi](https://github.com/rasendubi)! - Fix serialization format for JSON assignments.

  JSON assignments were incorrectly serialized as:

  ```json
  {
    "type": "JSON",
    "value": {
      "raw": "{\"hello\":\"world\"}",
      "parsed": { "hello": "world" }
    }
  }
  ```

  For Ruby, this caused incorrect values to be returned.

  This has been fixed and the proper format is:

  ```json
  {
    "type": "JSON",
    "value": { "hello": "world" }
  }
  ```

- Updated dependencies [[`3a18f95`](https://github.com/Eppo-exp/eppo-multiplatform/commit/3a18f95f0aa25030aeba6676b76e20862a5fcead)]:
  - eppo_core@7.0.0
