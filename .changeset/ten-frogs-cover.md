---
"python-sdk": patch
"eppo_core": major
"ruby-sdk": patch
"rust-sdk": patch
---

refactor(core): split poller thread into background thread and configuration poller.

In preparation for doing more work in the background, we're refactoring poller thread into a more generic background thread / background runtime with configuration poller running on top of it.

This changes API of the core but should be invisible for SDKs. The only noticeable difference is that client should be more responsive to graceful shutdown requests.
