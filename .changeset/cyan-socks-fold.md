---
"eppo_core": patch
"python-sdk": patch
---

fix(python): Options that are None are also None in python

None Attributes are now correctly converted to Python's `None` instead of `()` empty tuple.
