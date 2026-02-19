# gurl — The HTTP Runtime for AI Agents

> **Version:** 0.1.0-draft
> **Date:** 2026-02-19
> **Author:** Hamed (Finna)
> **Status:** Pre-development specification

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Problem Statement](#2-problem-statement)
3. [Market Analysis & Competitive Landscape](#3-market-analysis--competitive-landscape)
4. [Product Vision & Positioning](#4-product-vision--positioning)
5. [Architecture](#5-architecture)
6. [Feature Specification](#6-feature-specification)
7. [User Stories](#7-user-stories)
8. [Technical Specification](#8-technical-specification)
9. [Infrastructure & Deployment](#9-infrastructure--deployment)
10. [Development Plan](#10-development-plan)
11. [Go-to-Market Strategy](#11-go-to-market-strategy)
12. [Risk Analysis](#12-risk-analysis)
13. [Success Metrics](#13-success-metrics)
14. [Appendices](#14-appendices)

---

## 1. Executive Summary

**gurl** is an open-source, Rust-based HTTP client and runtime purpose-built for AI agents. It replaces the fragmented stack of curl + language-specific HTTP libraries + cloud scraping APIs + file converters that every AI agent currently cobbles together.

**Core thesis:** Every AI agent interacts with the web via HTTP. curl was built for humans in terminals. Jina/Firecrawl are cloud-only SaaS with per-request costs. MarkItDown/Docling only convert local files. Nobody has built the **universal, local-first, agent-native HTTP tool** that unifies all of these into one interface with structured output by default.

**Business model:** Open-source core (MIT) + optional hosted cloud tier on Cloudflare infrastructure. The open-source tool drives adoption; the cloud tier monetizes JS rendering, anti-bot bypass, and batch crawling at scale.

---

## 2. Problem Statement

### 2.1 What AI Agents Need from HTTP Today

AI agents (Claude Code, Cursor, Devin, custom agents, MCP-based tools) need to:

1. **Make API requests** and parse structured responses (JSON, XML, GraphQL)
2. **Fetch webpages** and get clean, LLM-friendly content (not raw HTML)
3. **Chain requests** (authenticate → get token → use token → fetch data)
4. **Download and convert files** (PDF, DOCX, etc.) to markdown
5. **Monitor endpoints** for changes
6. **Validate API responses** against schemas
7. **Retry with intelligence** (backoff, circuit breaking)
8. **Stream responses** (SSE, WebSocket — critical for LLM API interactions)

### 2.2 What They Actually Use

| Need | Current Solution | Pain |
|------|-----------------|------|
| API requests | `curl` via shell | Text output, no structured parsing, error-prone flag syntax |
| Web content | Jina Reader / Firecrawl | Per-request cost, data leaves your infra, latency |
| Request chaining | Custom code per language | Not portable across agent frameworks |
| File conversion | MarkItDown / Docling | Separate tool, no HTTP integration |
| Monitoring | Custom scripts | No standard tooling |
| Schema validation | Manual / per-language | Fragmented |
| Retry logic | Custom code | Reimplemented everywhere |
| Streaming | Language-specific | No universal CLI tool handles SSE well |

### 2.3 The Cost Problem

An AI agent making 10,000 web fetches per day:
- **Firecrawl:** ~$333/month (Standard plan) minimum, likely more with extract
- **Jina Reader:** Token-based, scales to hundreds/month at volume
- **gurl (local):** $0 — runs on the agent's own machine
- **gurl cloud (for JS-heavy sites):** Fraction of Firecrawl cost via Cloudflare

### 2.4 The Privacy Problem

Every request through Jina/Firecrawl means your scraping targets, API keys, and returned data flow through a third party. For enterprise, legal, healthcare, and finance use cases, this is unacceptable.

---

## 3. Market Analysis & Competitive Landscape

### 3.1 Direct Competitors

#### Jina AI Reader
- **What:** Cloud API that converts URLs to LLM-friendly markdown. Prepend `r.jina.ai/` to any URL.
- **Strengths:** Dead simple UX, free tier, search endpoint (`s.jina.ai`), ReaderLM-v2 (1.5B model for HTML→markdown), good quality output.
- **Weaknesses:** Cloud-only, per-token pricing at scale, acquired by Elastic (Oct 2025) — future direction uncertain, priorities shifting to enterprise search.
- **Key fact:** Jina is now part of Elastic. The standalone Reader product may get less investment or be absorbed into Elastic's enterprise offerings. This creates a vacuum.

#### Firecrawl
- **What:** YC-backed web data API for AI. Scrape, crawl, search, extract, and an autonomous Agent endpoint.
- **Strengths:** Very well-funded, fast iteration, good developer experience, LLM-optimized output, integrations with LangChain/LlamaIndex.
- **Pricing:** Free for 500 pages, Hobby $16/mo, Standard $83/mo, Growth $333/mo. AI extract billed separately on tokens ($89+/mo).
- **Weaknesses:** Cloud-only (no local/self-hosted), costs add up fast at scale, dual pricing model (credits + tokens) is confusing, data goes through their infra.

#### curl
- **What:** The universal command-line HTTP client. ~149K lines of C, 2,179 test cases, 273 CLI options.
- **Strengths:** Ubiquitous, incredibly robust, supports every protocol imaginable, actively maintained by Daniel Stenberg.
- **Weaknesses:** Human-oriented text output, no structured parsing, no content conversion, arcane flag syntax that even experienced developers look up constantly, not agent-friendly.
- **Key fact:** Daniel Stenberg has explicitly stated curl will remain a general-purpose transfer tool. There is no plan to make it AI/agent-native.

### 3.2 Adjacent Tools (Not Direct Competitors)

#### Microsoft MarkItDown
- **What:** Python tool for converting local files (PDF, DOCX, PPTX, Excel, images, audio) to Markdown. 60K+ GitHub stars.
- **Strengths:** Broad format support, plugin system, MCP server, Microsoft backing.
- **Weaknesses:** File conversion only — no HTTP fetching, no web scraping, no API interaction. Python-only. Basic quality on complex PDFs (loses table structure).
- **Relationship to gurl:** Complementary, not competitive. gurl could integrate MarkItDown (or its Rust equivalent) as the file conversion layer.

#### IBM Docling
- **What:** Python library for document conversion with AI-powered layout analysis and table recognition. LF AI & Data Foundation project.
- **Strengths:** Superior PDF quality vs MarkItDown, OCR support, multiple export formats.
- **Weaknesses:** Resource-heavy (downloads AI models), slow, Python-only, document conversion only.
- **Relationship to gurl:** Another potential conversion backend for gurl's file handling.

#### Marker / MinerU / PyMuPDF4LLM
- **What:** Various open-source PDF-to-Markdown tools with different quality/speed tradeoffs.
- **Relationship to gurl:** Potential backends for PDF conversion within gurl.

#### Hurl
- **What:** Rust CLI that runs HTTP requests from plain text files, powered by libcurl. Good for API testing.
- **Strengths:** Structured test format, assertions, chaining, Rust-based.
- **Weaknesses:** Human-oriented, testing-focused (not agent-focused), wraps libcurl rather than being a native Rust implementation, no content conversion.

#### HTTPie
- **What:** Human-friendly curl alternative with colorized output.
- **Strengths:** Beautiful CLI output, intuitive syntax.
- **Weaknesses:** Python-based (slow), no agent story, no content conversion, no structured output for machines.

### 3.3 Infrastructure Players

#### Cloudflare Browser Rendering
- **What:** Headless Chrome on Cloudflare's edge network. REST API + Puppeteer/Playwright bindings via Workers.
- **Relevance:** The ideal infrastructure backend for gurl's cloud tier. Each data center has warm browser pools, instant session creation via WebSocket, Durable Objects for session persistence.
- **Pricing:** Pay per browser-time with generous free tier.

### 3.4 Competitive Matrix

| Capability | curl | Jina | Firecrawl | MarkItDown | Hurl | **gurl** |
|-----------|------|------|-----------|------------|------|----------|
| HTTP requests | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ |
| Structured JSON output | ❌ | ✅ | ✅ | ❌ | ❌ | ✅ |
| HTML → Markdown | ❌ | ✅ | ✅ | ❌ | ❌ | ✅ |
| File → Markdown | ❌ | Partial | Partial | ✅ | ❌ | ✅ |
| JS rendering | ❌ | ✅ | ✅ | ❌ | ❌ | ✅ (cloud) |
| Request chaining | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |
| Schema validation | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ |
| MCP server | ❌ | ✅ | ✅ | ✅ | ❌ | ✅ |
| Local/offline | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ |
| Free & open source | ✅ | Partial | Partial | ✅ | ✅ | ✅ |
| Self-hostable | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ |
| Agent-native design | ❌ | Partial | Partial | ❌ | ❌ | ✅ |
| Streaming (SSE/WS) | Partial | ✅ | ❌ | ❌ | ❌ | ✅ |
| Response diffing | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ |
| Retry/resilience | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ |

### 3.5 Why Big Players Won't Build This

- **Microsoft** (MarkItDown) — file conversion library, not HTTP tooling. They want developers on Azure, not building standalone CLI tools.
- **IBM** (Docling) — enterprise document AI, not developer tools.
- **Elastic** (Jina) — enterprise search platform. Reader API is a feature, not their core product.
- **Firecrawl** — cloud SaaS business model. An open-source local tool would cannibalize their revenue.
- **curl** — explicitly committed to being a general-purpose transfer tool, not an AI-specific product.
- **Google** — no equivalent tool in this space. Their Document AI is enterprise cloud, not developer tooling.

---

## 4. Product Vision & Positioning

### 4.1 One-liner
**gurl: The HTTP runtime for AI agents. curl for the agentic era.**

### 4.2 Positioning Statement
For AI agent developers who need their agents to interact with the web, gurl is an open-source HTTP client that provides structured, LLM-friendly output from any URL or API. Unlike curl (human-oriented text output), Jina/Firecrawl (cloud-only, per-request cost), or MarkItDown (file conversion only), gurl is local-first, free, agent-native, and handles the entire HTTP lifecycle from request to clean content.

### 4.3 Design Principles

1. **Structured by default.** Every response is a typed JSON object with metadata. No text blob parsing.
2. **Agent-native, not agent-adapted.** MCP server and programmatic API are first-class, not afterthoughts.
3. **Local-first, cloud-optional.** Works offline on your machine. Cloud tier for when you need JS rendering or scale.
4. **Smart about content.** Auto-detects content type and converts to the most useful format. HTML → markdown, JSON → validated/typed, PDF → text.
5. **Composable.** Works as a CLI, a library, an MCP server, or a self-hosted API. Same tool, multiple interfaces.
6. **Resilient by design.** Retries, timeouts, circuit breaking, and backoff are built-in, not bolted on.

### 4.4 Product Tiers

#### Tier 1: gurl Core (Open Source, MIT)
- Rust CLI binary
- All HTTP methods with structured JSON output
- Content intelligence (HTML→MD, JSON validation, file detection)
- MCP server for agent integration
- Request chaining via YAML workflows
- Retry/timeout/backoff logic
- Response diffing
- Installable via cargo, brew, npm, apt, binary download

#### Tier 2: gurl Cloud (Hosted on Cloudflare)
- `gurl.dev/https://example.com` — simple URL-prefix API (like Jina)
- JS rendering for dynamic sites (via CF Browser Rendering)
- Global edge caching
- Anti-bot/proxy rotation
- Batch crawling via Queues
- Free tier: 1,000 requests/month
- Paid: $9/mo (10K), $29/mo (50K), $79/mo (200K)
- Deliberately undercutting Firecrawl by 3-5x

#### Tier 3: gurl Enterprise (Self-hosted)
- Deploy on customer's own Cloudflare account
- Data never leaves their infrastructure
- Volume licensing
- Priority support
- Compliance-friendly (SOC2, GDPR)

---

## 5. Architecture

### 5.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    gurl Interfaces                       │
│  ┌──────────┐  ┌───────────┐  ┌──────────┐  ┌────────┐ │
│  │   CLI    │  │ MCP Server│  │  Library  │  │HTTP API│ │
│  │(terminal)│  │ (agents)  │  │(Rust/Py/JS│  │ (self- │ │
│  │          │  │           │  │  /Go FFI) │  │ hosted)│ │
│  └────┬─────┘  └─────┬─────┘  └────┬──────┘  └───┬────┘ │
│       └───────────────┴─────────────┴─────────────┘      │
│                           │                              │
│  ┌────────────────────────▼──────────────────────────┐   │
│  │              gurl Core Engine (Rust)               │   │
│  │                                                    │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────┐ │   │
│  │  │  HTTP Client  │  │   Content    │  │ Workflow │ │   │
│  │  │  (reqwest)    │  │ Intelligence │  │  Engine  │ │   │
│  │  │              │  │              │  │          │ │   │
│  │  │ - HTTP/1.1   │  │ - HTML→MD    │  │ - YAML   │ │   │
│  │  │ - HTTP/2     │  │ - JSON valid.│  │ - Chain  │ │   │
│  │  │ - HTTP/3     │  │ - PDF→text   │  │ - Capture│ │   │
│  │  │ - WebSocket  │  │ - XML parse  │  │ - Assert │ │   │
│  │  │ - SSE        │  │ - Auto-detect│  │ - Retry  │ │   │
│  │  │ - TLS        │  │ - Readability│  │ - Branch │ │   │
│  │  └──────────────┘  └──────────────┘  └──────────┘ │   │
│  │                                                    │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────┐ │   │
│  │  │  Resilience   │  │   Output     │  │ Response │ │   │
│  │  │              │  │  Formatter   │  │  Differ  │ │   │
│  │  │ - Retry      │  │              │  │          │ │   │
│  │  │ - Backoff    │  │ - JSON       │  │ - Struct │ │   │
│  │  │ - Circuit    │  │ - Markdown   │  │ - Header │ │   │
│  │  │ - Timeout    │  │ - Raw        │  │ - Body   │ │   │
│  │  │ - Rate limit │  │ - Streaming  │  │ - Status │ │   │
│  │  └──────────────┘  └──────────────┘  └──────────┘ │   │
│  └────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘

                    │ (optional, for JS rendering / scale)
                    ▼

┌─────────────────────────────────────────────────────────┐
│              gurl Cloud (Cloudflare)                      │
│                                                          │
│  ┌──────────┐  ┌───────────────┐  ┌──────────────────┐  │
│  │  Workers  │  │   Browser     │  │    Workers AI    │  │
│  │  (API     │  │   Rendering   │  │  (HTML→MD on     │  │
│  │  gateway) │  │   (headless   │  │   edge, optional)│  │
│  │          │  │   Chrome)     │  │                  │  │
│  └──────────┘  └───────────────┘  └──────────────────┘  │
│                                                          │
│  ┌──────────┐  ┌───────────────┐  ┌──────────────────┐  │
│  │    KV    │  │      R2       │  │     Queues       │  │
│  │ (response│  │  (crawl       │  │  (batch          │  │
│  │  cache)  │  │   archives)   │  │   processing)    │  │
│  └──────────┘  └───────────────┘  └──────────────────┘  │
│                                                          │
│  ┌──────────────────┐  ┌───────────────────────────┐    │
│  │ Durable Objects   │  │      D1 (SQLite)          │    │
│  │ (browser session  │  │  (usage tracking,         │    │
│  │  persistence)     │  │   API keys, analytics)    │    │
│  └──────────────────┘  └───────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

### 5.2 Core Engine Components

#### HTTP Client Layer
- Built on `reqwest` (Rust) for async HTTP/1.1 and HTTP/2
- `quinn` for HTTP/3 / QUIC support
- `tokio-tungstenite` for WebSocket
- Native SSE streaming parser
- TLS via `rustls` (no OpenSSL dependency)
- Cookie jar with persistence
- Proxy support (HTTP, SOCKS5)

#### Content Intelligence Layer
- **HTML → Markdown:** `readability` algorithm (extract main content) + `html2md` (convert to markdown). Rule-based, no AI model needed for basic conversion.
- **JSON:** Schema validation via `jsonschema`, auto-type detection, pretty formatting
- **PDF → Text:** Integration with `pdf-extract` or `lopdf` for basic text. For complex PDFs, delegate to MarkItDown/Docling via optional Python bridge or to gurl cloud.
- **XML → Structured:** XPath/CSS selector extraction
- **Content-type auto-detection:** MIME type + magic bytes + URL heuristics
- **Image handling:** Alt-text extraction, dimension metadata, optional base64 inline

#### Workflow Engine
- YAML-based workflow definitions
- Variable capture from responses (JSONPath, regex, header extraction)
- Conditional branching (if status == X, then...)
- Parallel requests
- Loop/pagination support
- Assertion system for testing
- Workflow composition (import other workflows)

#### Resilience Layer
- Configurable retry with exponential backoff + jitter
- Circuit breaker pattern
- Per-host rate limiting
- Request timeout (connect, read, total)
- Automatic redirect following with limit
- Dead letter queue for failed requests in batch mode

#### Output Layer
- **Default:** Structured JSON with metadata envelope
- **Markdown mode:** Clean content suitable for LLMs
- **Raw mode:** Untransformed response body
- **Streaming mode:** Server-Sent Events or chunked output for real-time processing
- **Diff mode:** Structural comparison between two responses

### 5.3 Response Envelope Format

Every gurl response wraps content in a structured envelope:

```json
{
  "gurl": "0.1.0",
  "request": {
    "method": "GET",
    "url": "https://example.com/article",
    "timestamp": "2026-02-19T12:00:00Z"
  },
  "response": {
    "status": 200,
    "status_text": "OK",
    "headers": {
      "content-type": "text/html; charset=utf-8",
      "content-length": "45231"
    },
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
    "type": "markdown",
    "original_type": "text/html",
    "title": "Example Article Title",
    "body": "# Example Article Title\n\nThis is the clean article content...",
    "metadata": {
      "author": "Jane Doe",
      "published": "2026-02-18",
      "description": "An example article",
      "word_count": 1234,
      "language": "en"
    },
    "links": [
      {"text": "Related Article", "url": "https://example.com/related"}
    ],
    "images": [
      {"alt": "Hero image", "url": "https://example.com/hero.jpg", "width": 1200, "height": 630}
    ]
  }
}
```

---

## 6. Feature Specification

### 6.1 Phase 1 — Core (MVP, Weeks 1-2)

#### F1.1: Basic HTTP Client with Structured Output
```bash
# GET with structured JSON output (default)
gurl get https://api.example.com/users
# Returns: { "gurl": "0.1.0", "response": {...}, "content": {...} }

# POST with body
gurl post https://api.example.com/users --json '{"name": "Alice"}'

# All standard HTTP methods
gurl put/patch/delete/head/options <url>

# Headers
gurl get https://api.example.com -H "Authorization: Bearer token123"

# Raw mode (curl-compatible output)
gurl get https://example.com --raw
```

#### F1.2: Content Intelligence (Auto-Convert)
```bash
# HTML page → clean markdown (automatic)
gurl get https://example.com/blog/article
# content.type = "markdown", content.body = "# Article Title\n\n..."

# JSON API → validated, formatted
gurl get https://api.example.com/data
# content.type = "json", content.body = { parsed JSON }

# PDF → extracted text
gurl get https://example.com/report.pdf
# content.type = "markdown", content.body = "extracted text..."

# Force specific output format
gurl get https://example.com --format markdown
gurl get https://example.com --format html
gurl get https://example.com --format raw
```

#### F1.3: MCP Server
```bash
# Start as MCP server for Claude Desktop / Cursor / etc
gurl mcp

# In Claude Desktop config:
# {
#   "mcpServers": {
#     "gurl": {
#       "command": "gurl",
#       "args": ["mcp"]
#     }
#   }
# }
```

MCP tools exposed:
- `gurl_fetch` — fetch a URL, return structured content
- `gurl_api` — make an API request with method/headers/body
- `gurl_search` — search the web (via gurl cloud or configurable backend)
- `gurl_extract` — extract specific data from a URL using CSS/XPath selectors
- `gurl_diff` — compare two URLs or a URL against a previous snapshot

#### F1.4: Retry & Timeout
```bash
# Retry with exponential backoff
gurl get https://api.example.com --retry 3 --retry-backoff exponential

# Timeouts
gurl get https://slow-site.com --timeout 30s --connect-timeout 5s
```

### 6.2 Phase 2 — Power Features (Weeks 3-4)

#### F2.1: Request Chaining (Workflow Engine)
```yaml
# auth-and-fetch.yaml
name: "Authenticated API fetch"
steps:
  - id: login
    method: POST
    url: https://api.example.com/auth/login
    json:
      email: "{{env.API_EMAIL}}"
      password: "{{env.API_PASSWORD}}"
    capture:
      token: jsonpath $.access_token

  - id: fetch_data
    method: GET
    url: https://api.example.com/data
    headers:
      Authorization: "Bearer {{login.token}}"
    assert:
      status: 200
    capture:
      items: jsonpath $.items

  - id: fetch_details
    for_each: "{{fetch_data.items}}"
    method: GET
    url: "https://api.example.com/items/{{item.id}}"
    parallel: 5
```

```bash
gurl chain auth-and-fetch.yaml
```

#### F2.2: Schema Validation
```bash
# Validate response against JSON schema
gurl get https://api.example.com/users \
  --schema '{"type": "array", "items": {"type": "object", "required": ["id", "name"]}}'

# Validate against OpenAPI spec
gurl get https://api.example.com/users --openapi ./api-spec.yaml

# Extract structured data with a schema prompt (requires gurl cloud or local LLM)
gurl get https://example.com/product \
  --extract '{"name": "string", "price": "number", "in_stock": "boolean"}'
```

#### F2.3: Response Diffing
```bash
# Diff two URLs
gurl diff https://api.example.com/v1/data https://api.example.com/v2/data

# Diff against saved snapshot
gurl get https://api.example.com/status --save-snapshot status-baseline
gurl diff https://api.example.com/status --against status-baseline

# Structural diff (ignores timestamps, request IDs, etc.)
gurl diff url1 url2 --ignore-fields "timestamp,request_id,trace_id"
```

#### F2.4: Streaming Support
```bash
# Server-Sent Events
gurl stream https://api.example.com/events --sse

# WebSocket
gurl ws wss://api.example.com/ws --send '{"subscribe": "updates"}'

# LLM API streaming (OpenAI-compatible)
gurl post https://api.openai.com/v1/chat/completions \
  --json '{"model": "gpt-4", "messages": [...], "stream": true}' \
  --stream
```

#### F2.5: Monitoring / Watch Mode
```bash
# Poll every 30s, alert on change
gurl watch https://api.example.com/status --interval 30s --on-change webhook:https://hooks.slack.com/...

# Watch with diff
gurl watch https://example.com/pricing --interval 1h --diff --ignore-fields "timestamp"
```

### 6.3 Phase 3 — Cloud Tier (Weeks 5-8)

#### F3.1: gurl Cloud API
```bash
# Simple URL-prefix API (like Jina's r.jina.ai)
curl https://gurl.dev/https://example.com

# With options via headers
curl https://gurl.dev/https://example.com \
  -H "X-Gurl-Format: markdown" \
  -H "X-Gurl-JS: true" \
  -H "X-Gurl-Timeout: 30"

# Or via the CLI with cloud backend
gurl get https://example.com --cloud  # uses gurl.dev for JS rendering
gurl get https://spa-app.com --js     # alias for --cloud, renders JavaScript
```

#### F3.2: Batch Crawling
```bash
# Crawl entire site
gurl crawl https://docs.example.com --depth 3 --output ./docs/

# Crawl with cloud (handles JS, rate limiting, etc.)
gurl crawl https://spa-app.com --cloud --depth 2 --parallel 10
```

#### F3.3: Search
```bash
# Web search (uses configurable backend: Brave, Serper, SearxNG, or gurl cloud)
gurl search "rust HTTP client library 2026"
# Returns top results with content already extracted
```

---

## 7. User Stories

### 7.1 Agent Developer (Primary Persona)

**"Alex" — Building AI agents with Claude Code / Cursor**

| ID | Story | Priority |
|----|-------|----------|
| U1.1 | As an agent developer, I want to fetch a webpage and get clean markdown so my agent can reason about the content without parsing HTML. | P0 |
| U1.2 | As an agent developer, I want to make API calls and get structured JSON responses so my agent can programmatically use the data. | P0 |
| U1.3 | As an agent developer, I want to add gurl as an MCP server so Claude/Cursor can use it as a tool for web access. | P0 |
| U1.4 | As an agent developer, I want automatic retries with backoff so my agent doesn't fail on transient network errors. | P0 |
| U1.5 | As an agent developer, I want to chain authenticated API requests so my agent can work with protected endpoints. | P1 |
| U1.6 | As an agent developer, I want to validate API responses against a schema so my agent can detect API contract violations. | P1 |
| U1.7 | As an agent developer, I want to fetch JS-rendered pages so my agent can access content on SPAs and dynamic sites. | P1 |
| U1.8 | As an agent developer, I want to monitor an API endpoint and get notified on changes so my agent can react to real-time data. | P2 |
| U1.9 | As an agent developer, I want to diff two API responses structurally so my agent can detect what changed between versions. | P2 |
| U1.10 | As an agent developer, I want streaming SSE/WebSocket support so my agent can interact with real-time APIs and LLM streaming endpoints. | P1 |

### 7.2 AI-Native App Developer

**"Sam" — Building a RAG pipeline / AI-powered product**

| ID | Story | Priority |
|----|-------|----------|
| U2.1 | As a RAG developer, I want to crawl a documentation site and get all pages as markdown so I can build a knowledge base. | P1 |
| U2.2 | As a RAG developer, I want to fetch PDFs from URLs and get text content so I can index documents without separate tools. | P1 |
| U2.3 | As a product developer, I want to self-host gurl's web fetching API so my users' data never leaves my infrastructure. | P2 |
| U2.4 | As a product developer, I want to use gurl as a Rust library in my backend so I can embed HTTP-to-markdown in my service. | P1 |
| U2.5 | As a product developer, I want batch URL processing so I can convert hundreds of URLs in parallel. | P2 |

### 7.3 DevOps / API Testing

**"Jordan" — Testing APIs and monitoring services**

| ID | Story | Priority |
|----|-------|----------|
| U3.1 | As a DevOps engineer, I want to define API test workflows in YAML so I can version and automate my API tests. | P2 |
| U3.2 | As a DevOps engineer, I want to compare API responses before and after deployment so I can detect regressions. | P2 |
| U3.3 | As a DevOps engineer, I want health check monitoring with webhook alerts so I can detect outages automatically. | P3 |

### 7.4 Open Source Contributor

| ID | Story | Priority |
|----|-------|----------|
| U4.1 | As a contributor, I want a well-documented Rust codebase with clear module boundaries so I can contribute features. | P1 |
| U4.2 | As a contributor, I want a comprehensive test suite (unit + integration) so I can verify my changes don't break anything. | P0 |
| U4.3 | As a contributor, I want plugin hooks for custom content converters so I can extend gurl for new file types. | P2 |

---

## 8. Technical Specification

### 8.1 Technology Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| Language | Rust | Performance, memory safety, single binary distribution, WebAssembly compilation |
| HTTP client | reqwest + hyper | Battle-tested, async, HTTP/2 support |
| TLS | rustls | No OpenSSL dependency, memory safe |
| Async runtime | tokio | Standard Rust async runtime |
| HTML → MD | Custom (readability + html2md crates) | Local, no AI model needed |
| JSON schema | jsonschema crate | Fast validation |
| YAML parsing | serde_yaml | Workflow definitions |
| CLI framework | clap | Standard Rust CLI |
| MCP protocol | Custom implementation over stdio/HTTP | MCP spec compliance |
| WebSocket | tokio-tungstenite | Async WebSocket |
| Cloud API | Cloudflare Workers (TypeScript) | Edge deployment, Browser Rendering access |
| Cloud storage | Cloudflare KV + R2 | Cache + archives |
| Cloud DB | Cloudflare D1 | Usage tracking, API keys |

### 8.2 Project Structure

```
gurl/
├── Cargo.toml
├── LICENSE                     # MIT
├── README.md
├── crates/
│   ├── gurl-core/             # Core engine (HTTP + content intelligence)
│   │   ├── src/
│   │   │   ├── client/        # HTTP client wrapper
│   │   │   ├── content/       # Content intelligence (HTML→MD, JSON, PDF)
│   │   │   ├── resilience/    # Retry, backoff, circuit breaker
│   │   │   ├── output/        # Response formatting (JSON envelope, markdown, raw)
│   │   │   ├── diff/          # Response diffing
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── gurl-cli/              # CLI binary
│   │   ├── src/
│   │   │   ├── commands/      # get, post, chain, diff, watch, mcp, etc.
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── gurl-mcp/              # MCP server implementation
│   │   ├── src/
│   │   │   ├── tools/         # MCP tool definitions
│   │   │   ├── transport/     # stdio + HTTP transports
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── gurl-workflow/         # Workflow/chain engine
│   │   ├── src/
│   │   │   ├── parser/        # YAML workflow parser
│   │   │   ├── executor/      # Step execution
│   │   │   ├── capture/       # Variable capture (JSONPath, regex)
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   └── gurl-cloud-client/     # Client for gurl.dev API
│       └── ...
├── cloud/                     # Cloudflare Workers code
│   ├── worker/                # Main API Worker
│   ├── browser/               # Browser rendering Worker
│   └── wrangler.toml
├── tests/
│   ├── integration/           # End-to-end tests
│   ├── fixtures/              # Test HTML, JSON, PDF files
│   └── workflows/             # Test workflow YAML files
├── docs/
│   ├── getting-started.md
│   ├── cli-reference.md
│   ├── mcp-integration.md
│   ├── workflow-syntax.md
│   └── cloud-api.md
└── scripts/
    ├── install.sh             # curl-style installer
    └── benchmark.sh           # Performance benchmarks vs curl/Jina/Firecrawl
```

### 8.3 CLI Interface Design

```
gurl <command> [options] [url]

Commands:
  get        HTTP GET request (default if URL provided)
  post       HTTP POST request
  put        HTTP PUT request
  patch      HTTP PATCH request
  delete     HTTP DELETE request
  head       HTTP HEAD request
  fetch      Smart fetch (auto-detect best method)
  chain      Run a workflow file
  diff       Compare two responses
  watch      Monitor a URL for changes
  stream     Connect to SSE/WebSocket endpoint
  search     Web search
  crawl      Crawl a website
  mcp        Start MCP server
  config     Manage configuration
  version    Show version

Global Options:
  -H, --header <key:value>     Add request header (repeatable)
  -d, --data <body>            Request body (string)
  --json <json>                Request body as JSON (sets Content-Type)
  --form <key=value>           Form data (repeatable)
  -o, --output <file>          Save output to file
  --format <json|markdown|html|raw>  Output format (default: json)
  --raw                        Raw output (no envelope, curl-compatible)
  --timeout <duration>         Total request timeout
  --connect-timeout <duration> Connection timeout
  --retry <n>                  Retry count
  --retry-backoff <strategy>   Backoff strategy (linear, exponential, fixed)
  --proxy <url>                HTTP/SOCKS5 proxy
  --no-redirect                Don't follow redirects
  --max-redirects <n>          Maximum redirects (default: 10)
  --cloud                      Use gurl.dev cloud backend
  --js                         Enable JavaScript rendering (implies --cloud)
  --schema <json-schema>       Validate response against schema
  --extract <schema>           Extract structured data
  --selector <css>             Extract content matching CSS selector
  --verbose, -v                Verbose output (timing, TLS, etc.)
  --quiet, -q                  Minimal output (content body only)
  --config <file>              Config file path
  --api-key <key>              gurl.dev API key
```

### 8.4 Configuration File

```yaml
# ~/.config/gurl/config.yaml
defaults:
  format: json
  timeout: 30s
  connect_timeout: 5s
  retry: 2
  retry_backoff: exponential
  user_agent: "gurl/0.1.0"

cloud:
  api_key: "gurl_xxxxx"
  endpoint: "https://gurl.dev"

search:
  provider: brave  # brave, serper, searxng, gurl-cloud
  api_key: "brave_xxxxx"

content:
  html_to_markdown: true
  extract_metadata: true
  include_links: true
  include_images: true
  max_content_length: 100000  # tokens

proxy:
  http: "http://proxy:8080"
  no_proxy: "localhost,127.0.0.1"
```

---

## 9. Infrastructure & Deployment (Cloud Tier)

### 9.1 Cloudflare Architecture

```yaml
# wrangler.toml
name = "gurl-api"
main = "src/index.ts"
compatibility_date = "2026-02-01"
compatibility_flags = ["nodejs_compat"]

[browser]
binding = "BROWSER"

[[kv_namespaces]]
binding = "CACHE"
id = "xxx"

[[r2_buckets]]
binding = "ARCHIVES"
bucket_name = "gurl-crawl-archives"

[[d1_databases]]
binding = "DB"
database_name = "gurl-usage"
database_id = "xxx"

[[queues.producers]]
binding = "CRAWL_QUEUE"
queue = "gurl-crawl-jobs"

[[queues.consumers]]
queue = "gurl-crawl-jobs"
max_batch_size = 10
max_batch_timeout = 60

[durable_objects]
bindings = [
  { name = "BROWSER_SESSION", class_name = "BrowserSession" }
]
```

### 9.2 Cloud API Flow

```
User Request → Cloudflare Worker (API Gateway)
  │
  ├─→ Static content? → Check KV Cache → Return cached or fetch
  │
  ├─→ JS rendering needed? → Durable Object → Browser Rendering
  │     └─→ Puppeteer renders page → Extract content → Cache in KV → Return
  │
  ├─→ Batch/crawl? → Enqueue to Queues → Process async → Store in R2
  │
  └─→ Simple fetch? → Worker fetches directly → Process → Cache → Return
```

### 9.3 Estimated Cloud Costs (at 100K requests/month)

| Cloudflare Service | Usage | Cost |
|-------------------|-------|------|
| Workers | 100K requests | ~$5/mo |
| Browser Rendering | ~20K (JS-heavy only) | ~$20/mo |
| KV | Cache reads/writes | ~$5/mo |
| R2 | Crawl archives | ~$2/mo |
| D1 | Usage tracking | ~$1/mo |
| **Total infrastructure** | | **~$33/mo** |
| **Revenue at $29/user** | 50K plan | **$29/mo per user** |

At scale, margins improve dramatically because of KV caching (repeat URLs are free).

---

## 10. Development Plan

### 10.1 Phase 1: MVP (Weeks 1-2)

**Goal:** Shippable open-source CLI that agents can use today.

#### Week 1: Core Engine
| Day | Task | Output |
|-----|------|--------|
| 1 | Project scaffolding, Cargo workspace, CI setup | `gurl version` works, builds on Linux/macOS/Windows |
| 2 | HTTP client wrapper (reqwest), structured response envelope | `gurl get https://httpbin.org/get` returns JSON envelope |
| 3 | HTML → Markdown conversion (readability + html2md) | `gurl get https://example.com` returns clean markdown |
| 4 | JSON response handling, auto-detection, pretty formatting | `gurl get https://api.github.com/users/octocat` returns typed JSON |
| 5 | Retry logic, timeout, basic error handling | `gurl get https://httpbin.org/status/500 --retry 3` works |

#### Week 2: Agent Integration
| Day | Task | Output |
|-----|------|--------|
| 6 | MCP server implementation (stdio transport) | `gurl mcp` works with Claude Desktop |
| 7 | MCP tools: gurl_fetch, gurl_api | Agents can fetch URLs and make API calls |
| 8 | CLI polish: all HTTP methods, headers, body, auth | Full CLI interface working |
| 9 | Basic PDF text extraction, content-type detection | PDFs return text content |
| 10 | Testing, README, install script, cargo publish | Ship v0.1.0 |

**Deliverables:**
- `cargo install gurl` works
- `brew install gurl` via tap
- Binary downloads on GitHub Releases
- MCP server for Claude Desktop / Cursor
- README with examples and benchmarks vs curl

### 10.2 Phase 2: Power Features (Weeks 3-4)

| Week | Features |
|------|----------|
| 3 | Workflow engine (YAML chaining, variable capture, assertions), schema validation, response diffing |
| 4 | Streaming (SSE, WebSocket), watch/monitor mode, CSS/XPath selectors, `--extract` with schema |

### 10.3 Phase 3: Cloud Tier (Weeks 5-8)

| Week | Features |
|------|----------|
| 5 | Cloudflare Worker API gateway, basic fetch endpoint, KV caching |
| 6 | Browser Rendering integration for JS-heavy sites |
| 7 | Batch crawling via Queues, R2 archiving |
| 8 | Usage tracking (D1), API key management, free/paid tiers, landing page |

### 10.4 Phase 4: Ecosystem (Weeks 9-12)

| Week | Features |
|------|----------|
| 9 | Python bindings (PyO3), npm package (WASM or napi) |
| 10 | Plugin system for custom content converters |
| 11 | Web search integration (Brave API, SearxNG) |
| 12 | Enterprise self-hosted guide, Terraform/Pulumi for CF deployment |

### 10.5 Testing Strategy

```
Test Pyramid:
                    ┌───────────┐
                    │   E2E     │  Real HTTP against test servers
                    │  (~50)    │  and live sites
                   ┌┴───────────┴┐
                   │ Integration  │  Workflow engine, MCP server,
                   │   (~200)     │  content pipeline tests
                  ┌┴──────────────┴┐
                  │   Unit Tests    │  Each module independently
                  │    (~500+)      │  Content converters, parsers,
                  └─────────────────┘  retry logic, output formatting
```

Key test files to maintain:
- `tests/fixtures/` — curated HTML pages, JSON responses, PDFs for conversion quality testing
- `tests/integration/mcp/` — MCP protocol compliance tests
- `tests/integration/workflows/` — workflow engine end-to-end tests
- `tests/benchmarks/` — performance benchmarks vs curl, jina, firecrawl (run on CI)

---

## 11. Go-to-Market Strategy

### 11.1 Launch Sequence

#### Pre-Launch (Week 1-2)
- Build in public on X/Twitter, dev.to, LinkedIn
- Daily build logs showing progress
- "I'm building curl for AI agents" narrative

#### Launch Day (End of Week 2)
- Hacker News "Show HN: gurl — The HTTP runtime for AI agents, written in Rust"
- Reddit: r/rust, r/LocalLLaMA, r/MachineLearning
- X/Twitter thread with demo GIFs
- Dev.to / Hashnode blog post: "Why I'm replacing curl for my AI agents"

#### Post-Launch (Weeks 3+)
- YouTube demo: "Setting up gurl as an MCP server for Claude"
- Write integration guides: Claude Code, Cursor, Windsurf, custom agents
- Create awesome-gurl list of community plugins and workflows
- GitHub Discussions for community workflows

### 11.2 Distribution Channels

| Channel | Action |
|---------|--------|
| Cargo (crates.io) | `cargo install gurl` |
| Homebrew | `brew install gurl` (custom tap, then official) |
| npm | `npx gurl` (WASM or native binary download) |
| apt/yum | PPA and RPM repos |
| GitHub Releases | Pre-built binaries for Linux/macOS/Windows (x86_64 + arm64) |
| Docker | `docker run ghcr.io/gurl/gurl` |
| GitHub Actions | `uses: gurl/setup-gurl@v1` |

### 11.3 Community Growth Tactics

- Ship a **workflow marketplace** — community-contributed YAML workflows for common tasks (auth flows, API testing, data extraction)
- Create a **gurl playground** — web-based tool to test gurl commands (built with gurl cloud)
- **Weekly "Website of the Week"** — show how gurl handles a notoriously hard-to-scrape site
- Encourage Claude/Cursor users to share their MCP configs

---

## 12. Risk Analysis

### 12.1 Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| HTML→MD quality not matching Jina/Firecrawl | High | Medium | Use readability algorithm (proven), offer cloud fallback with ReaderLM-equivalent model. Quality doesn't need to be 100% — "good enough for agents" is the bar. |
| Rust compile times slow iteration | Medium | High | Workspace with small crates, incremental compilation, focus on integration test speed |
| MCP spec changes | Low | Medium | MCP is stabilizing. Maintain a thin abstraction layer. |
| Cloudflare Browser Rendering limits | Medium | Low | Current limits: 2 browsers/min/account. Use Durable Objects for session reuse. At scale, negotiate enterprise limits. |

### 12.2 Market Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Firecrawl releases a free/local CLI | High | Low | Their business model depends on cloud revenue. A free local tool would cannibalize it. Unlikely. |
| curl adds structured output | High | Very Low | Daniel Stenberg has been clear this won't happen. curl's scope is fixed. |
| Big player (Google, Microsoft, etc.) builds this | Medium | Low | They're focused on cloud platforms, not developer CLI tools. |
| "Good enough" alternatives emerge | Medium | Medium | Move fast, build community, establish as the standard before alternatives gain traction. |
| Jina Reader stays independent/healthy under Elastic | Low | Medium | Even if Jina thrives, it's cloud-only. gurl's local-first story is unaffected. |

### 12.3 Business Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Cloud tier doesn't generate enough revenue | Medium | Medium | Keep burn low (CF costs are minimal). The open-source tool drives consulting/enterprise revenue. |
| Open-source maintainer burnout | High | Medium | Build community early. Accept PRs aggressively. Automate everything. |
| Cloudflare pricing changes | Medium | Low | Architecture is CF-specific but could be ported to Fly.io or AWS Lambda@Edge if needed. |

---

## 13. Success Metrics

### 13.1 North Star Metric
**Monthly Active Installations** (unique machines running gurl per month, tracked via opt-in telemetry)

### 13.2 Phase 1 Goals (Month 1)

| Metric | Target |
|--------|--------|
| GitHub stars | 500+ |
| Cargo installs | 200+ |
| Homebrew installs | 100+ |
| MCP integrations (Claude Desktop configs shared) | 50+ |
| HN front page | Yes |
| Contributors (non-author) | 5+ |

### 13.3 Phase 2 Goals (Month 3)

| Metric | Target |
|--------|--------|
| GitHub stars | 3,000+ |
| Monthly active installations | 1,000+ |
| gurl cloud signups | 200+ |
| Community workflow contributions | 20+ |
| Integration mentions in agent frameworks | 3+ (LangChain, LlamaIndex, CrewAI) |

### 13.4 Phase 3 Goals (Month 6)

| Metric | Target |
|--------|--------|
| GitHub stars | 10,000+ |
| Monthly active installations | 5,000+ |
| gurl cloud paying customers | 50+ |
| Monthly cloud revenue | $2,000+ |
| "Default HTTP tool for agents" mentions | Growing trend |

---

## 14. Appendices

### 14.1 Curl Test Suite Reference

Curl has 2,179 test cases as of end-2025, covering:
- Command-line tool against test servers (all protocols)
- libcurl API behavior
- Unit tests for internal functions
- Valgrind memory leak detection
- "Torture" mode (memory allocation failure testing)

For gurl, we don't need to rewrite curl or pass all curl tests. We focus on HTTP/HTTPS only and test against our own suite. However, curl's test methodology (test servers, protocol verification, memory safety) is a model to follow.

### 14.2 Content Conversion Quality Benchmarks

We should maintain a benchmark suite comparing gurl's HTML→MD conversion against:
- Jina Reader API
- Firecrawl API
- Readability.js (Mozilla)
- Turndown.js

Test pages should include:
- News articles (NYT, BBC, Reuters)
- Documentation sites (MDN, Rust docs)
- Blog platforms (Medium, Substack, Ghost)
- E-commerce product pages
- Wikipedia articles
- Single-page applications (with --cloud)

### 14.3 Existing Rust Crates to Evaluate

| Crate | Purpose | Notes |
|-------|---------|-------|
| reqwest | HTTP client | Standard choice, async, HTTP/2 |
| hyper | Low-level HTTP | reqwest is built on this |
| tokio-tungstenite | WebSocket | Async WebSocket client |
| html2md | HTML to Markdown | Basic conversion |
| readability | Readability algorithm | Content extraction from HTML |
| scraper | HTML parsing + CSS selectors | For --selector feature |
| serde_json | JSON handling | Standard |
| jsonschema | JSON Schema validation | For --schema feature |
| clap | CLI framework | Standard |
| indicatif | Progress bars | For crawl/batch progress |
| lopdf / pdf-extract | PDF text extraction | For basic PDF handling |
| similar | Text diffing | For response diff |

### 14.4 MCP Tool Definitions

```json
{
  "tools": [
    {
      "name": "gurl_fetch",
      "description": "Fetch a URL and return its content as clean, LLM-friendly text. Automatically converts HTML to markdown, handles JSON, PDFs, and other content types.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "url": { "type": "string", "description": "The URL to fetch" },
          "format": { "type": "string", "enum": ["markdown", "json", "html", "raw"], "default": "markdown" },
          "headers": { "type": "object", "description": "Custom request headers" },
          "timeout": { "type": "integer", "description": "Timeout in seconds", "default": 30 },
          "js": { "type": "boolean", "description": "Enable JavaScript rendering (requires gurl cloud)", "default": false },
          "selector": { "type": "string", "description": "CSS selector to extract specific content" }
        },
        "required": ["url"]
      }
    },
    {
      "name": "gurl_api",
      "description": "Make an HTTP API request with full control over method, headers, and body. Returns structured response with status, headers, and parsed body.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "method": { "type": "string", "enum": ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"] },
          "url": { "type": "string" },
          "headers": { "type": "object" },
          "body": { "type": "string", "description": "Request body (string or JSON)" },
          "json": { "type": "object", "description": "JSON body (auto-sets Content-Type)" },
          "schema": { "type": "object", "description": "JSON Schema to validate response against" }
        },
        "required": ["method", "url"]
      }
    },
    {
      "name": "gurl_extract",
      "description": "Extract structured data from a webpage using CSS selectors or a data schema.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "url": { "type": "string" },
          "selectors": {
            "type": "object",
            "description": "Map of field names to CSS selectors",
            "example": { "title": "h1", "price": ".price-tag", "description": ".product-desc" }
          },
          "schema": {
            "type": "object",
            "description": "JSON schema describing desired output structure"
          }
        },
        "required": ["url"]
      }
    },
    {
      "name": "gurl_diff",
      "description": "Compare two URLs or a URL against a previous snapshot to detect changes.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "url_a": { "type": "string" },
          "url_b": { "type": "string", "description": "Second URL or snapshot name" },
          "ignore_fields": { "type": "array", "items": { "type": "string" }, "description": "Fields to ignore in comparison" }
        },
        "required": ["url_a"]
      }
    },
    {
      "name": "gurl_search",
      "description": "Search the web and return results with extracted content.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "query": { "type": "string" },
          "limit": { "type": "integer", "default": 5 },
          "scrape": { "type": "boolean", "description": "Also fetch and convert each result page", "default": false }
        },
        "required": ["query"]
      }
    }
  ]
}
```

### 14.5 Workflow YAML Syntax Reference

```yaml
# Full workflow syntax reference
name: "Workflow name"
description: "What this workflow does"
version: "1.0"

# Environment variables
env:
  BASE_URL: "https://api.example.com"
  # Can reference process env: ${ENV_VAR}

# Workflow steps
steps:
  - id: step_name          # Unique identifier (required)
    method: GET             # HTTP method (required)
    url: "{{env.BASE_URL}}/endpoint"  # URL with template vars (required)

    # Request configuration
    headers:                # Request headers
      Authorization: "Bearer {{prev_step.token}}"
      Content-Type: "application/json"
    json:                   # JSON body (auto-sets Content-Type)
      key: "value"
    body: "raw body"        # Raw body (mutually exclusive with json)
    form:                   # Form data
      field: "value"

    # Response handling
    capture:                # Extract values from response
      token: jsonpath $.access_token
      user_id: jsonpath $.data.id
      csrf: header x-csrf-token
      session: cookie session_id
      title: regex '<title>(.*?)</title>'
    assert:                 # Assertions (fail workflow if not met)
      status: 200           # Exact status
      status_range: [200, 299]  # Status range
      body_contains: "success"
      jsonpath:
        $.data.length: { gt: 0 }
    schema:                 # JSON Schema validation
      type: object
      required: [id, name]

    # Flow control
    retry: 3                # Retry on failure
    retry_backoff: exponential
    timeout: 30s
    delay: 1s               # Wait before this step
    condition: "{{prev_step.status}} == 200"  # Skip if false

    # Iteration
    for_each: "{{prev_step.items}}"  # Loop over array
    as: "item"                        # Loop variable name
    parallel: 5                       # Max concurrent iterations

    # Output
    save: "./output/{{item.id}}.json"  # Save response to file
    format: markdown                    # Output format for this step
```

### 14.6 Comparison: gurl vs Existing Tools (CLI Examples)

```bash
# ---- Fetch a webpage as markdown ----

# curl (doesn't do conversion)
curl -s https://example.com/article | # raw HTML... now what?

# Jina (requires internet, their servers)
curl -s https://r.jina.ai/https://example.com/article

# gurl (local, instant, structured)
gurl get https://example.com/article
# Returns JSON envelope with markdown content


# ---- Authenticated API call ----

# curl
curl -s -H "Authorization: Bearer $TOKEN" https://api.example.com/data | jq .

# gurl (structured, with schema validation)
gurl get https://api.example.com/data \
  -H "Authorization: Bearer $TOKEN" \
  --schema '{"type": "object", "required": ["id", "items"]}'


# ---- Multi-step auth flow ----

# curl (manual, error-prone)
TOKEN=$(curl -s -X POST https://api.example.com/auth \
  -d '{"email":"me@example.com","pass":"xxx"}' | jq -r .token)
curl -s -H "Authorization: Bearer $TOKEN" https://api.example.com/data

# gurl (declarative workflow)
gurl chain auth-flow.yaml


# ---- Fetch a PDF and get text ----

# curl + markitdown (two separate tools)
curl -s https://example.com/report.pdf -o /tmp/report.pdf
markitdown /tmp/report.pdf

# gurl (one command)
gurl get https://example.com/report.pdf


# ---- Monitor an endpoint ----

# curl (need a wrapper script)
while true; do curl -s https://api.example.com/status; sleep 30; done

# gurl (built-in)
gurl watch https://api.example.com/status --interval 30s --on-change notify


# ---- Use as MCP tool in Claude ----

# curl: not possible (no MCP support)
# Jina: separate MCP server package needed
# gurl: built-in
gurl mcp  # Claude Desktop can now use gurl for web access
```

---

> **End of specification. This document should be given to an AI coding agent along with the instruction: "Build gurl according to this spec, starting with Phase 1 (MVP, Weeks 1-2). Begin with project scaffolding and the core HTTP client with structured output."**
