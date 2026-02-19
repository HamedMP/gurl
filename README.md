<p align="center">
  <strong><code>gurl</code></strong>
</p>

<h1 align="center">The HTTP client for AI agents</h1>

<p align="center">
  <strong>Fetch any URL. Get clean markdown. Save 90%+ tokens.</strong>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge" alt="MIT License"></a>
  <a href="https://deepwiki.com/FinnaAI/gurl"><img src="https://img.shields.io/badge/DeepWiki-FinnaAI%2Fgurl-blue.svg?style=for-the-badge&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACwAAAAyCAYAAAAnWDnqAAAAAXNSR0IArs4c6QAAA05JREFUaEPtmUtyEzEQhtWTQyQLHNak2AB7ZnyXZMEjXMGeK/AIi+QuHrMnbChYY7MIh8g01fJoopFb0uhhEqqcbWTp06/uv1saEDv4O3n3dV60RfP947Mm9/SQc0ICFQgzfc4CYZoTPAswgSJCCUJUnAAoRHOAUOcATwbmVLWdGoH//PB8mnKqScAhsD0kYP3j/Yt5LPQe2KvcXmGvRHcDnpxfL2zOYJ1mFwrryWTz0advv1Ut4CJgf5uhDuDj5eUcAUoahrdY/56ebRWeraTjMt/00Sh3UDtjgHtQNHwcRGOC98BJEAEymycmYcWwOprTgcB6VZ5JK5TAJ+fXGLBm3FDAmn6oPPjR4rKCAoJCal2eAiQp2x0vxTPB3ALO2CRkwmDy5WohzBDwSEFKRwPbknEggCPB/imwrycgxX2NzoMCHhPkDwqYMr9tRcP5qNrMZHkVnOjRMWwLCcr8ohBVb1OMjxLwGCvjTikrsBOiA6fNyCrm8V1rP93iVPpwaE+gO0SsWmPiXB+jikdf6SizrT5qKasx5j8ABbHpFTx+vFXp9EnYQmLx02h1QTTrl6eDqxLnGjporxl3NL3agEvXdT0WmEost648sQOYAeJS9Q7bfUVoMGnjo4AZdUMQku50McDcMWcBPvr0SzbTAFDfvJqwLzgxwATnCgnp4wDl6Aa+Ax283gghmj+vj7feE2KBBRMW3FzOpLOADl0Isb5587h/U4gGvkt5v60Z1VLG8BhYjbzRwyQZemwAd6cCR5/XFWLYZRIMpX39AR0tjaGGiGzLVyhse5C9RKC6ai42ppWPKiBagOvaYk8lO7DajerabOZP46Lby5wKjw1HCRx7p9sVMOWGzb/vA1hwiWc6jm3MvQDTogQkiqIhJV0nBQBTU+3okKCFDy9WwferkHjtxib7t3xIUQtHxnIwtx4mpg26/HfwVNVDb4oI9RHmx5WGelRVlrtiw43zboCLaxv46AZeB3IlTkwouebTr1y2NjSpHz68WNFjHvupy3q8TFn3Hos2IAk4Ju5dCo8B3wP7VPr/FGaKiG+T+v+TQqIrOqMTL1VdWV1DdmcbO8KXBz6esmYWYKPwDL5b5FA1a0hwapHiom0r/cKaoqr+27/XcrS5UwSMbQAAAABJRU5ErkJggg==" alt="DeepWiki"></a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2024_Edition-DEA584?style=for-the-badge&logo=rust&logoColor=white" alt="Rust 2024">
  <img src="https://img.shields.io/badge/Tests-52_passing-brightgreen?style=for-the-badge" alt="52 Tests">
  <img src="https://img.shields.io/badge/Sites_Tested-32%2F34-brightgreen?style=for-the-badge" alt="32/34 Sites">
</p>

---

When an AI agent fetches a web page with `curl` or `web_fetch`, it gets raw HTML. A typical docs page is **300K+ tokens** of tags, scripts, and navigation. The agent wastes context window, money, and reasoning on noise.

`gurl` fetches the same URL and returns **clean markdown** in a structured JSON envelope. Same content, 90-100% fewer tokens.

```
curl  https://docs.stripe.com/api  →  279,363 tokens (raw HTML)
gurl  https://docs.stripe.com/api  →    2,814 tokens (clean markdown)  — 99% saved
```

---

## How It Works

```bash
gurl get https://docs.stripe.com/api
```

```json
{
  "url": "https://docs.stripe.com/api",
  "response": { "status": 200, "headers": { "content-type": "text/html; charset=utf-8" } },
  "content": {
    "type": "markdown",
    "original_type": "text/html; charset=utf-8",
    "title": "Stripe API Reference",
    "body": "# Stripe API Reference\n\nThe Stripe API is organized around REST..."
  },
  "timing": { "total_ms": 1139 }
}
```

Every response is a JSON envelope with metadata + clean content. HTML becomes markdown. PDFs become text. JSON stays structured. Agents parse one format, always.

---

## Install

```bash
cargo install --git https://github.com/FinnaAI/gurl
```

Or build from source:

```bash
git clone https://github.com/FinnaAI/gurl.git
cd gurl
cargo build --release
# Binary at ./target/release/gurl
```

---

## Usage

```bash
# Fetch a page — returns structured JSON envelope with markdown content
gurl get https://docs.stripe.com/api

# Body only (for piping to agents)
gurl get https://react.dev/reference/react --quiet

# Raw response (no conversion, actual HTML/bytes)
gurl get https://example.com --raw

# Extract specific fields with dot notation
gurl get https://httpbin.org/json --select content.body

# POST with JSON body
gurl post https://httpbin.org/post --json '{"key": "value"}'

# Form data
gurl post https://httpbin.org/post -F "name=gurl" -F "type=cli"

# Custom headers
gurl get https://api.example.com -H "Authorization: Bearer token"

# Save to file
gurl get https://arxiv.org/pdf/1706.03762 -o paper.md

# Timeout
gurl get https://slow-site.com --timeout 5
```

### Output Modes

| Flag | Output |
|------|--------|
| *(default)* | Full JSON envelope (pretty in terminal, compact when piped) |
| `--quiet` | Body content only |
| `--raw` | Original response bytes, no conversion |
| `--select path` | Extract field via dot notation (`content.body`, `response.status`) |
| `-o file` | Save body to file |

---

## Content Intelligence

gurl auto-detects content type and converts to the optimal format for AI consumption:

| Input | Output | Method |
|-------|--------|--------|
| HTML pages | Markdown | Readability + noise stripping |
| Next.js RSC sites | Markdown | RSC payload extraction |
| Cookie-wall pages | Markdown | Banner removal + article extraction |
| PDF documents | Text with page breaks | pdf-extract |
| JSON / API responses | Structured JSON | Pass-through with formatting |
| CSV / TSV | Markdown tables | Column detection |
| DOCX | Markdown | XML parsing |
| XLSX | Markdown tables | Sheet extraction |
| EPUB | Markdown | Chapter extraction |
| PPTX | Markdown | Slide-by-slide text |
| RSS / Atom | Markdown | Feed item extraction |
| Images (EXIF) | Metadata markdown | EXIF tag extraction |
| Outlook .msg | Markdown | CFB parsing |
| Jupyter notebooks | Markdown | Cell extraction |
| Wikipedia | Markdown | Infobox + article extraction |
| ZIP archives | Content listing | File enumeration |
| Plain text | Pass-through | Charset detection |

### Next.js RSC Extraction

Modern Next.js sites (Vercel docs, etc.) render content client-side via React Server Components. The actual text lives in `self.__next_f.push()` script chunks, not the DOM. `gurl` extracts content directly from the RSC payload — no headless browser needed.

### Noise Stripping

Before extraction, `gurl` removes cookie banners, consent dialogs, navigation, sidebars, footers, modals, and inline scripts. A quality gate rejects results that are predominantly navigation links.

---

## Benchmark: 34 Real-World Sites

```
SITE                    gurl_ms   curl_tok   gurl_tok   saved  STATUS
──────────────────────────────────────────────────────────────────────
python-stdlib            271ms      19531       5558    72%  OK
rust-std                 178ms      13240       7818    41%  OK
mdn-js                    77ms      54735       5976    90%  OK
react-ref                176ms      39001       1175    97%  OK
nextjs-docs              190ms     191117       8184    96%  OK
fastapi                  119ms      40131       8906    78%  OK
stripe-api              1139ms     279363       2814    99%  OK
github-rest               55ms     103150       5106    96%  OK
anthropic-api           1070ms     122772       2212    99%  OK
vercel-nextjs            278ms     467820       3935   100%  OK
claude-tools            1483ms     169138       4124    98%  OK
cloudflare-workers        81ms      61316       1382    98%  OK
neon-docs                247ms      97939       1608    99%  OK
supabase-docs            152ms      69433       1689    98%  OK
redis-cmds               140ms     172909      18101    90%  OK
docker-ref               109ms      73236      16024    79%  OK
k8s-concepts             147ms     120643        752   100%  OK
wikipedia-rust           322ms     146789      24417    84%  OK
arxiv-pdf                316ms     541536       9903    99%  OK
...
Results: 32/34 passed (>200 chars extracted)
```

`curl_tok` is what `curl` or `web_fetch` costs your agent in context window tokens. `gurl_tok` is what `gurl` costs. The difference is wasted money.

Run yourself: `bash bench/sites.sh`

---

## Architecture

```
gurl-cli                    # CLI binary (clap)
  |
  +-- gurl-core             # HTTP client + content pipeline
        |
        +-- markitdown-rs   # Document conversion (Rust port of Microsoft MarkItDown)
              |
              +-- 15 converters (HTML, PDF, DOCX, XLSX, CSV, EPUB, RSS, ...)
              +-- MIME detection (magic bytes + extension)
              +-- Charset detection (encoding_rs)
```

Three crates in a Cargo workspace:

| Crate | Purpose |
|-------|---------|
| `gurl-cli` | CLI interface, output formatting, TTY detection |
| `gurl-core` | HTTP client (reqwest), response envelope, content routing |
| `markitdown-rs` | 15 document converters, content detection, noise stripping |

### markitdown-rs

A standalone Rust port of [Microsoft MarkItDown](https://github.com/microsoft/markitdown) with feature flags for each converter:

```toml
[dependencies]
markitdown-rs = { version = "0.1", features = ["html", "pdf"] }
# or everything:
markitdown-rs = { version = "0.1", features = ["all"] }
```

Available features: `html`, `pdf`, `docx`, `xlsx`, `csv-convert`, `epub`, `rss`, `image`, `outlook`, `ipynb`, `pptx`, `wikipedia`, `zip-convert`

---

## For Agent Developers

gurl is designed as a drop-in tool for AI agents. The structured JSON envelope means your agent always gets:

```python
# Pseudo-code for any agent framework
result = run("gurl get https://docs.stripe.com/api --quiet")
# result is clean markdown, ready for the context window
```

Compared to alternatives:

| Tool | Output | Tokens for Stripe API docs |
|------|--------|---------------------------|
| `curl` | Raw HTML | ~279K |
| Claude `web_fetch` | HTML with cookie noise | ~169K |
| `gurl` | Clean markdown | ~2.8K |

The JSON envelope also includes timing, headers, and content metadata — useful for agents that need to make decisions based on response characteristics.

---

## Development

```bash
# Run tests
cargo test --all-features     # 52 tests

# Build release
cargo build --release

# Run benchmarks
bash bench/compare.sh         # gurl vs curl comparison
bash bench/sites.sh           # 34-site extraction quality test
```

---

## License

MIT

---

*gurl. Fetch anything. Read everything. Waste nothing.*
