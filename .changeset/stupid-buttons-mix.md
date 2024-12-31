---
"rust-sdk": major
"ruby-sdk": patch
---

Exit poller thread when PollerThread handle is dropped/disconnected.

This makes SDK more stable in the environment where the main process is killed without calling shutdown on the poller thread.

For Rust SDK that is a breaking change as you now need to make sure that `PollerThread` handle is not dropped prematurely.
