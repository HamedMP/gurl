# 004 — Content Intelligence: HTML to Markdown

> **Parent spec:** [001-init](../001-init/spec.md) — Sections 5.2 (Content Intelligence), 6.1 F1.2
> **Status:** Ready
> **Dependencies:** 003-http-client
> **Estimated effort:** 2-3 days

---

## Goal

When gurl fetches an HTML page, automatically extract the main content and convert it to clean, LLM-friendly markdown. This is the single most important differentiator — if the quality isn't competitive with Jina Reader, the product thesis weakens.

## Scope

### In scope

- Readability-style content extraction (strip nav, ads, sidebars, footers)
- HTML-to-markdown conversion of extracted content
- Metadata extraction: title, author, published date, description, language
- Link extraction (list of links found in content)
- Image extraction (alt text, URL, dimensions if available)
- Word count
- `content.type` set to `"markdown"` when HTML is converted
- `--format markdown` (force markdown output)
- `--format html` (return cleaned HTML without markdown conversion)
- `--format raw` (return original HTML untouched)

### Out of scope

- CSS/XPath selectors (`--selector`) — later spec
- JavaScript rendering — see 015 (cloud tier)
- PDF/DOCX conversion — see 009
- LLM-based extraction (`--extract`) — later

## Architecture

```
gurl-core/src/content/
├── mod.rs              # ContentPipeline: auto-detect and route
├── html/
│   ├── mod.rs
│   ├── readability.rs  # Main content extraction
│   ├── markdown.rs     # HTML -> Markdown conversion
│   └── metadata.rs     # Title, author, date, description
└── detector.rs         # Content-type detection (MIME + heuristics)
```

## Pipeline

```
HTTP Response
  → Content-Type detection (is this HTML?)
  → Readability extraction (find main content)
  → Metadata extraction (title, author, date)
  → HTML-to-Markdown conversion
  → Link & image collection
  → Populate content field in envelope
```

## Content field output

```json
{
  "content": {
    "type": "markdown",
    "original_type": "text/html",
    "title": "Article Title",
    "body": "# Article Title\n\nClean article content...",
    "metadata": {
      "author": "Jane Doe",
      "published": "2026-02-18",
      "description": "An example article",
      "word_count": 1234,
      "language": "en"
    },
    "links": [{ "text": "Related", "url": "https://example.com/related" }],
    "images": [{ "alt": "Hero image", "url": "https://example.com/hero.jpg" }]
  }
}
```

## Quality benchmark

Before considering this spec done, test against these real-world pages and compare output to Jina Reader (`r.jina.ai`):

1. A Wikipedia article (complex tables, references, sidebars)
2. A Medium blog post (paywalls, newsletter prompts)
3. A news article (NYT, BBC, or similar)
4. A documentation page (MDN or Rust docs)
5. A GitHub README page

The bar is **"good enough for an LLM to understand the content."** It doesn't need to be pixel-perfect — it needs to preserve the semantic structure (headings, lists, code blocks, links).

## Crate evaluation

Evaluate these Rust crates before implementing:

| Crate         | Purpose                              | Risk                                   |
| ------------- | ------------------------------------ | -------------------------------------- |
| `readability` | Content extraction                   | Rust port quality unknown — test first |
| `html2md`     | HTML-to-markdown                     | Basic, may miss edge cases             |
| `scraper`     | HTML parsing + CSS selectors         | Well-maintained, good for extraction   |
| `lol_html`    | Streaming HTML rewriter (Cloudflare) | Fast, but lower-level                  |

If the existing Rust crates produce poor quality, consider:

- Porting Mozilla's Readability.js logic directly
- Using `scraper` for DOM manipulation + custom markdown emitter

## Acceptance criteria

- [ ] `gurl get https://en.wikipedia.org/wiki/Rust_(programming_language)` returns clean markdown
- [ ] `gurl get https://example.com` returns markdown with title extracted
- [ ] `gurl get https://httpbin.org/html` converts the HTML to markdown
- [ ] Metadata (title, description) is populated from `<meta>` tags
- [ ] Links within content are preserved as markdown links
- [ ] Code blocks in HTML are preserved as fenced code blocks
- [ ] Tables are converted to markdown tables (best effort)
- [ ] `--format html` returns cleaned HTML
- [ ] `--format raw` returns original untouched HTML
- [ ] JSON API responses (`application/json`) are NOT converted to markdown
- [ ] Binary responses (images, etc.) are NOT converted

## Notes

- This spec is intentionally separate from JSON handling (005) to keep the focus on HTML quality.
- Consider adding a `content.conversion_note` field if the conversion had issues (e.g., "tables simplified", "some images could not be resolved").
