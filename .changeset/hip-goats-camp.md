---
"rust-sdk": patch
---

Decrease default jitter from 30s to 3s.

30s jitter was too high and caused configuration interval to vary between 0 to 30s (with average of 15s). The new value makes fetch interval vary between 27 to 30s.
