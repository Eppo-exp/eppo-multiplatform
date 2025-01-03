---
"rust-sdk": patch
"ruby-sdk": patch
---

Fix serialization format for JSON assignments.

JSON assignments were incorrectly serialized as:
```json
{
  "type": "JSON",
  "value": {
    "raw": "{\"hello\":\"world\"}",
    "parsed": {"hello": "world"}
  }
}
```

For Ruby, this caused incorrect values to be returned.

This has been fixed and the proper format is:
```json
{
  "type": "JSON",
  "value": {"hello": "world"}
}
```
