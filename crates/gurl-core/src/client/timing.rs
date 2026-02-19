// Timing utilities for HTTP requests.
// Currently we only measure total_ms via Instant.
// Granular timing (DNS, TLS, connect) requires hyper connection-level hooks,
// which we'll add when needed. reqwest doesn't expose these directly.
