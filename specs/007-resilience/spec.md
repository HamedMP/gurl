# 007 — Resilience: Retry, Timeout, Backoff

> **Parent spec:** [001-init](../001-init/spec.md) — Sections 5.2 (Resilience Layer), 6.1 F1.4
> **Status:** Ready
> **Dependencies:** 003-http-client
> **Estimated effort:** 1 day

---

## Goal

Built-in retry logic with configurable backoff, request timeouts, and rate limiting. Agents need this out of the box — they shouldn't have to implement retry logic themselves.

## Scope

### In scope

- `--retry <n>`: retry failed requests (default: 0)
- `--retry-backoff <strategy>`: `fixed`, `linear`, `exponential` (default: `exponential`)
- `--retry-delay <duration>`: base delay between retries (default: 1s)
- `--retry-on <conditions>`: retry on specific conditions (default: `5xx,timeout,connection`)
- `--timeout <duration>`: total request timeout (default: 30s)
- `--connect-timeout <duration>`: connection timeout (default: 10s)
- Jitter on backoff to prevent thundering herd
- Retry info in response envelope (attempt count, total retry time)
- Config file defaults for all retry settings

### Out of scope

- Circuit breaker pattern (deferred — adds complexity, low value for CLI usage)
- Per-host rate limiting (deferred)
- Dead letter queue (only relevant for batch/workflow mode)

## Backoff strategies

```
fixed:        delay, delay, delay
linear:       delay, delay*2, delay*3
exponential:  delay, delay*2, delay*4  (with jitter)
```

## Retry conditions

```
5xx          — Retry on 5xx status codes
429          — Retry on 429 Too Many Requests (respect Retry-After header)
timeout      — Retry on request timeout
connection   — Retry on connection refused/reset
```

## Envelope addition

```json
{
  "response": {
    "status": 200,
    "retries": {
      "attempts": 3,
      "total_retry_ms": 7200
    }
  }
}
```

The `retries` field is only present when retries were attempted.

## Acceptance criteria

- [ ] `gurl get https://httpbin.org/status/500 --retry 3` retries 3 times then returns 500
- [ ] `gurl get https://httpbin.org/status/200 --retry 3` succeeds on first try (no unnecessary retries)
- [ ] `gurl get https://httpbin.org/delay/60 --timeout 2s` times out after 2 seconds
- [ ] Exponential backoff with jitter is verifiable in verbose output
- [ ] 429 responses with `Retry-After` header are respected
- [ ] Retry attempts are logged in verbose mode
- [ ] `retries` field appears in envelope when retries occurred
- [ ] Config file defaults work: `defaults.retry: 2` applies when no `--retry` flag given
