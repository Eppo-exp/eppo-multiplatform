# ruby-sdk

## 3.4.1

### Patch Changes

- Updated dependencies [[`82d05ae`](https://github.com/Eppo-exp/eppo-multiplatform/commit/82d05aea0263639be56ba5667500f6940b4832ab)]:
  - eppo_core@7.0.1

## 3.4.0

### Minor Changes

- [#136](https://github.com/Eppo-exp/eppo-multiplatform/pull/136) [`74d42bf`](https://github.com/Eppo-exp/eppo-multiplatform/commit/74d42bf1afab1509b87711f0d62e730c8b51e996) Thanks [@rasendubi](https://github.com/rasendubi)! - Preserve types for numeric and boolean attributes.

  Previously, when using numeric and boolean attributes as context attributes, they were converted to strings. Now, the internal type is correctly preserved throughout evaluation to logging.

### Patch Changes

- [#135](https://github.com/Eppo-exp/eppo-multiplatform/pull/135) [`712b5d8`](https://github.com/Eppo-exp/eppo-multiplatform/commit/712b5d83f9022d8b855a8a0cc846aad4573a83b3) Thanks [@dependabot](https://github.com/apps/dependabot)! - chore(deps): bump rb_sys from 0.9.103 to 0.9.105.

- [#136](https://github.com/Eppo-exp/eppo-multiplatform/pull/136) [`74d42bf`](https://github.com/Eppo-exp/eppo-multiplatform/commit/74d42bf1afab1509b87711f0d62e730c8b51e996) Thanks [@rasendubi](https://github.com/rasendubi)! - Exit poller thread when PollerThread handle is dropped/disconnected.

  This makes SDK more stable in the environment where the main process is killed without calling shutdown on the poller thread.

  For Rust SDK that is a breaking change as you now need to make sure that `PollerThread` handle is not dropped prematurely.

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
