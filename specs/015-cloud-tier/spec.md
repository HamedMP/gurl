# 015 — Cloud Tier (Cloudflare)

> **Parent spec:** [001-init](../001-init/spec.md) — Sections 6.3, 9.x
> **Status:** Blocked (ship core CLI first)
> **Dependencies:** All core specs (002-009)
> **Estimated effort:** 2-3 weeks

---

## Goal

Deploy gurl's cloud backend on Cloudflare: a URL-prefix API (`gurl.dev/https://example.com`), JavaScript rendering via Browser Rendering, response caching via KV, and the `--cloud` / `--js` CLI flags.

## Scope

### In scope

- Cloudflare Worker: API gateway at `gurl.dev`
- URL-prefix API: `gurl.dev/<target-url>` returns structured content
- Browser Rendering integration for JS-heavy sites
- KV cache for responses (configurable TTL)
- `--cloud` CLI flag to route requests through gurl.dev
- `--js` CLI flag (alias for `--cloud` with JS rendering)
- API key management (D1)
- Rate limiting per API key
- Free tier: 1,000 requests/month
- Request headers: `X-Gurl-Format`, `X-Gurl-JS`, `X-Gurl-Timeout`
- New crate: `gurl-cloud-client` (thin client for the cloud API)

### Out of scope

- Batch crawling via Queues (separate spec)
- R2 archival (separate spec)
- Payment/billing integration (manual for early users)
- Landing page / marketing site

## Architecture

```
gurl CLI --cloud --> gurl.dev (Cloudflare Worker)
                        |
                        ├── Simple fetch: Worker fetches directly
                        ├── JS rendering: Durable Object -> Browser Rendering
                        └── Cache: KV read/write
```

## Cloud config

```yaml
# ~/.config/gurl/config.yaml
cloud:
  api_key: "gurl_xxxxx"
  endpoint: "https://gurl.dev"
```

## This spec is intentionally high-level

The cloud tier is a separate project (TypeScript on Cloudflare Workers). Detailed implementation specs will live in `cloud/` once the core CLI is stable and has real users. Don't build this until the local CLI is proven.

## Acceptance criteria

- [ ] `curl https://gurl.dev/https://example.com` returns markdown content
- [ ] `gurl get https://spa-app.com --js` renders JavaScript and returns content
- [ ] API key authentication works
- [ ] Cached responses are served from KV on repeat requests
- [ ] Rate limiting enforced per API key
- [ ] CLI `--cloud` flag routes through gurl.dev transparently
