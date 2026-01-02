# ruby-sdk

## 3.7.4

### Patch Changes

- [#367](https://github.com/Eppo-exp/eppo-multiplatform/pull/367) [`8a5ffef`](https://github.com/Eppo-exp/eppo-multiplatform/commit/8a5ffef082754076b8dff60232b428652ca42513) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Bump rb_sys to 0.9.105. This is the first version that adds Ruby 3.4 to the list of supported versions.

## 3.7.3

### Patch Changes

- [#359](https://github.com/Eppo-exp/eppo-multiplatform/pull/359) [`9dab003`](https://github.com/Eppo-exp/eppo-multiplatform/commit/9dab00339c429ee85246b65244586ae59e121a05) Thanks [@dd-oleksii](https://github.com/dd-oleksii)! - Fix `get_boolean_assignment_details` debug method incorrectly overriding boolean false values with default.

## 3.7.2

### Patch Changes

- [#287](https://github.com/Eppo-exp/eppo-multiplatform/pull/287) [`6271dd9`](https://github.com/Eppo-exp/eppo-multiplatform/commit/6271dd98731ea37bd092967729ed2c14b1fa8589) Thanks [@rasendubi](https://github.com/rasendubi)! - Add prebuilt libraries for Ruby 3.4.

## 3.7.1

### Patch Changes

- [#274](https://github.com/Eppo-exp/eppo-multiplatform/pull/274) [`161fb42`](https://github.com/Eppo-exp/eppo-multiplatform/commit/161fb422301bd59c57d4a725a661d3b820c6c5ee) Thanks [@rasendubi](https://github.com/rasendubi)! - Bump rb_sys to support Ruby 3.4.

## 3.7.0

### Minor Changes

- [#268](https://github.com/Eppo-exp/eppo-multiplatform/pull/268) [`2ec98c6`](https://github.com/Eppo-exp/eppo-multiplatform/commit/2ec98c6f006d9e86c186fc99a903188d9837d653) Thanks [@rasendubi](https://github.com/rasendubi)! - Fix casing of `evaluationDetails`.

  In Ruby SDK v3.4.0, the name of `evaluationDetails` was inadvertently changed to `evaluation_details`. This was a bug that caused backward incompatibility in a minor release.

  This release fixes the casing back to `evaluationDetails`.

## 3.6.0

### Minor Changes

- [#247](https://github.com/Eppo-exp/eppo-multiplatform/pull/247) [`2e3bf09`](https://github.com/Eppo-exp/eppo-multiplatform/commit/2e3bf093d23a2b63f55f3e5336662489ed689a09) Thanks [@rasendubi](https://github.com/rasendubi)! - Add `wait_for_initialization()` method.

### Known Issues

- Starting with v3.4.0, in result of evaluation details methods, the name of `evaluationDetails` key is spelled as `evaluation_details`. This is fixed in 3.7.0.

## 3.5.1

### Patch Changes

- [#94](https://github.com/Eppo-exp/eppo-multiplatform/pull/94) [`30a0062`](https://github.com/Eppo-exp/eppo-multiplatform/commit/30a0062169f030edb6c7b6280850af7c618aae65) Thanks [@dependabot](https://github.com/apps/dependabot)! - Update magnus from 0.6.4 to 0.7.1

- [#94](https://github.com/Eppo-exp/eppo-multiplatform/pull/94) [`30a0062`](https://github.com/Eppo-exp/eppo-multiplatform/commit/30a0062169f030edb6c7b6280850af7c618aae65) Thanks [@dependabot](https://github.com/apps/dependabot)! - Update serde_magnus from 0.8.1 to 0.9.0

- [#223](https://github.com/Eppo-exp/eppo-multiplatform/pull/223) [`9504e92`](https://github.com/Eppo-exp/eppo-multiplatform/commit/9504e928c37f82147e65fe25aab558cad3bbac2a) Thanks [@dependabot](https://github.com/apps/dependabot)! - chore(deps): bump rb_sys from 0.9.110 to 0.9.111

### Known Issues

- Starting with v3.4.0, in result of evaluation details methods, the name of `evaluationDetails` key is spelled as `evaluation_details`. This is fixed in 3.7.0.

## 3.5.0

### Minor Changes

- [#197](https://github.com/Eppo-exp/eppo-multiplatform/pull/197) [`a4da91f`](https://github.com/Eppo-exp/eppo-multiplatform/commit/a4da91f1a962708924063f3f076d3064441c2f76) Thanks [@rasendubi](https://github.com/rasendubi)! - Change TLS implementation from openssl to rustls.

### Known Issues

- Starting with v3.4.0, in result of evaluation details methods, the name of `evaluationDetails` key is spelled as `evaluation_details`. This is fixed in 3.7.0.

## 3.4.6

### Patch Changes

- [#212](https://github.com/Eppo-exp/eppo-multiplatform/pull/212) [`095c5f5`](https://github.com/Eppo-exp/eppo-multiplatform/commit/095c5f54b48a8d41bae53125507a9939ae5ce9ec) Thanks [@bennettandrews](https://github.com/bennettandrews)! - Fix `AttributeValue` serialization, so `Null` attributes are properly serialized as None instead of unit value.

- [#206](https://github.com/Eppo-exp/eppo-multiplatform/pull/206) [`8c04c92`](https://github.com/Eppo-exp/eppo-multiplatform/commit/8c04c9254dc24660f172614b867c0324d94663bd) Thanks [@dependabot](https://github.com/apps/dependabot)! - chore(deps): bump openssl from 0.10.68 to 0.10.70

- [#213](https://github.com/Eppo-exp/eppo-multiplatform/pull/213) [`9ea7865`](https://github.com/Eppo-exp/eppo-multiplatform/commit/9ea78657dbbfe8fb733dd67fb71357872db9f8b2) Thanks [@rasendubi](https://github.com/rasendubi)! - Bump Minimum Supported Rust Version (MSRV) to 1.80.0.

### Known Issues

- Starting with v3.4.0, in result of evaluation details methods, the name of `evaluationDetails` key is spelled as `evaluation_details`. This is fixed in 3.7.0.

## 3.4.5

### Patch Changes

- [#201](https://github.com/Eppo-exp/eppo-multiplatform/pull/201) [`1d310c7`](https://github.com/Eppo-exp/eppo-multiplatform/commit/1d310c7019dde1aa5a965e064eab15187b064d96) Thanks [@felipecsl](https://github.com/felipecsl)! - [Unstable] Event Ingestion: Fix JSON serialization of Event timestamp field

### Known Issues

- Starting with v3.4.0, in result of evaluation details methods, the name of `evaluationDetails` key is spelled as `evaluation_details`. This is fixed in 3.7.0.

## 3.4.4

### Patch Changes

- [#198](https://github.com/Eppo-exp/eppo-multiplatform/pull/198) [`9c6990e`](https://github.com/Eppo-exp/eppo-multiplatform/commit/9c6990ec77dc3ffe8f1b6384f92fcc24db94916f) Thanks [@felipecsl](https://github.com/felipecsl)! - [unstable] Event Ingestion: Fix JSON serialization of Event type field

### Known Issues

- Starting with v3.4.0, in result of evaluation details methods, the name of `evaluationDetails` key is spelled as `evaluation_details`. This is fixed in 3.7.0.

## 3.4.3

### Patch Changes

- [#185](https://github.com/Eppo-exp/eppo-multiplatform/pull/185) [`1623ee2`](https://github.com/Eppo-exp/eppo-multiplatform/commit/1623ee215be5f07075f25a7c7413697082fd90cc) Thanks [@dependabot](https://github.com/apps/dependabot)! - [core] update rand requirement from 0.8.5 to 0.9.0

- [#168](https://github.com/Eppo-exp/eppo-multiplatform/pull/168) [`9d40446`](https://github.com/Eppo-exp/eppo-multiplatform/commit/9d40446c2346ac0869566699100baf69287da560) Thanks [@rasendubi](https://github.com/rasendubi)! - refactor(core): split poller thread into background thread and configuration poller.

  In preparation for doing more work in the background, we're refactoring poller thread into a more generic background thread / background runtime with configuration poller running on top of it.

  This changes API of the core but should be invisible for SDKs. The only noticeable difference is that client should be more responsive to graceful shutdown requests.

### Known Issues

- Starting with v3.4.0, in result of evaluation details methods, the name of `evaluationDetails` key is spelled as `evaluation_details`. This is fixed in 3.7.0.

## 3.4.2

### Patch Changes

- Updated dependencies [[`aa0ca89`](https://github.com/Eppo-exp/eppo-multiplatform/commit/aa0ca8912bab269613d3da25c06f81b1f19ffb36)]:
  - eppo_core@7.0.2

### Known Issues

- Starting with v3.4.0, in result of evaluation details methods, the name of `evaluationDetails` key is spelled as `evaluation_details`. This is fixed in 3.7.0.

## 3.4.1

### Patch Changes

- Updated dependencies [[`82d05ae`](https://github.com/Eppo-exp/eppo-multiplatform/commit/82d05aea0263639be56ba5667500f6940b4832ab)]:
  - eppo_core@7.0.1

### Known Issues

- Starting with v3.4.0, in result of evaluation details methods, the name of `evaluationDetails` key is spelled as `evaluation_details`. This is fixed in 3.7.0.

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

### Known Issues

- Starting with v3.4.0, in result of evaluation details methods, the name of `evaluationDetails` key is spelled as `evaluation_details`. This is fixed in 3.7.0.
