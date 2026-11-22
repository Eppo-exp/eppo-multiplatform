---
"elixir-sdk": patch
---

Update `rustls` 0.23.40 → 0.23.46, `rustls-webpki` 0.103.13 → 0.103.16, `webpki-roots` 1.0.7 → 1.0.10, `tokio` 1.52.3 → 1.53.3 and `url` 2.5.8 → 2.5.9 in `elixir-sdk/Cargo.lock`. The hex package does not include this lockfile, so projects resolve these crates when they compile the NIF.
