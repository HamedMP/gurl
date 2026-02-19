# 003 — Core HTTP Client & Structured Output

> **Parent spec:** [001-init](../001-init/spec.md) — Sections 5.2 (HTTP Client Layer), 5.3 (Response Envelope), 6.1 F1.1
> **Status:** Ready
> **Dependencies:** 002-scaffolding
> **Estimated effort:** 1-2 days

---

## Goal

Implement the core HTTP client wrapper around `reqwest` that makes requests and returns the structured JSON response envelope. This is the foundation everything else builds on.

## Scope

### In scope

- HTTP client wrapper in `gurl-core` using `reqwest`
- All standard HTTP methods: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS
- Request headers (`-H` flag)
- Request body: `--data`, `--json`, `--form`
- Structured response envelope (spec 001 section 5.3)
- Timing data collection (DNS, connect, TLS, first byte, total)
- TLS info extraction (version, cipher, cert issuer)
- Basic redirect following (default 10, `--no-redirect`, `--max-redirects`)
- `--raw` mode (no envelope, just response body — curl-compatible)
- `--verbose` flag (include timing + TLS in output)
- Wire up CLI commands (`get`, `post`, etc.) to the actual client
- User-Agent header: `gurl/0.1.0`

### Out of scope

- Content intelligence / conversion (see 004, 005)
- Retry logic (see 007)
- Streaming / WebSocket / SSE (see 010)
- Proxy support (later)
- Cookie jar persistence (later)

## Response envelope

Direct from spec 001 section 5.3. For this spec, the `content` field is simple:

```json
{
  "gurl": "0.1.0",
  "request": {
    "method": "GET",
    "url": "https://httpbin.org/get",
    "timestamp": "2026-02-19T12:00:00Z"
  },
  "response": {
    "status": 200,
    "status_text": "OK",
    "headers": { ... },
    "timing": {
      "dns_ms": 12,
      "connect_ms": 45,
      "tls_ms": 38,
      "first_byte_ms": 123,
      "total_ms": 456
    },
    "tls": {
      "version": "TLSv1.3",
      "cipher": "TLS_AES_256_GCM_SHA384",
      "cert_issuer": "Let's Encrypt"
    }
  },
  "content": {
    "type": "raw",
    "original_type": "application/json",
    "body": "..."
  }
}
```

Content intelligence (HTML-to-markdown, JSON parsing, etc.) will extend the `content` field in later specs.

## Architecture

```
gurl-core/src/
├── client/
│   ├── mod.rs          # GurlClient struct
│   ├── request.rs      # Request builder
│   ├── response.rs     # Response envelope types
│   └── timing.rs       # Timing measurement
├── output/
│   ├── mod.rs
│   ├── envelope.rs     # JSON envelope serialization
│   └── raw.rs          # Raw output mode
└── lib.rs
```

## Key types

```rust
pub struct GurlClient { ... }

pub struct GurlRequest {
    pub method: Method,
    pub url: Url,
    pub headers: HeaderMap,
    pub body: Option<Body>,
}

pub struct GurlResponse {
    pub gurl_version: String,
    pub request: RequestMeta,
    pub response: ResponseMeta,
    pub content: Content,
}

pub struct Timing {
    pub dns_ms: Option<u64>,
    pub connect_ms: Option<u64>,
    pub tls_ms: Option<u64>,
    pub first_byte_ms: Option<u64>,
    pub total_ms: u64,
}
```

## Acceptance criteria

- [ ] `gurl get https://httpbin.org/get` returns valid JSON envelope
- [ ] `gurl post https://httpbin.org/post --json '{"foo":"bar"}'` sends JSON body
- [ ] `gurl get https://httpbin.org/get -H "X-Custom: test"` sends custom header
- [ ] `gurl get https://httpbin.org/get --raw` returns just the response body
- [ ] `gurl get https://httpbin.org/status/301` follows redirect
- [ ] `gurl get https://httpbin.org/status/301 --no-redirect` returns 301
- [ ] Timing data is populated in the envelope
- [ ] `gurl get https://example.com -v` shows TLS info
- [ ] Error responses (4xx, 5xx) still return the envelope (not a crash)
- [ ] Non-existent hosts return a structured error, not a panic

## Dependencies (crates)

- `reqwest` (with `rustls-tls`, `json`, `cookies` features)
- `serde` + `serde_json`
- `chrono` (timestamps)
- `url`
- `tokio` (async runtime)

## Notes

- Timing granularity depends on what `reqwest` exposes. We may only get `total_ms` reliably — that's fine for MVP. Granular timing (DNS, TLS) can be improved later with `hyper` connection hooks.
- TLS info extraction is best-effort. `rustls` doesn't expose all fields easily. Start with what's available, mark others as `null`.
