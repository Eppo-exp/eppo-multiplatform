# rust-sdk

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
