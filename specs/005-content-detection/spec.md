# 005 — Content Intelligence: JSON & Auto-Detection

> **Parent spec:** [001-init](../001-init/spec.md) — Sections 5.2 (Content Intelligence), 6.1 F1.2
> **Status:** Ready
> **Dependencies:** 003-http-client, 004-html-to-markdown
> **Estimated effort:** 1 day

---

## Goal

Automatically detect response content types and handle JSON responses intelligently. When gurl hits an API that returns JSON, it should parse, format, and present it cleanly in the envelope. When content type is ambiguous, use heuristics to pick the right handler.

## Scope

### In scope

- Content-type auto-detection: MIME type from headers + body sniffing as fallback
- JSON response handling: parse, pretty-print, include in envelope as structured data
- XML response handling: detect and include as-is (conversion to JSON is out of scope for now)
- Plain text handling: include as-is
- Binary detection: flag as binary, don't include body (just metadata)
- Image detection: extract dimensions/format metadata if possible
- The `content.type` field reflects the detected/converted type
- Route to HTML-to-markdown pipeline (from 004) when HTML is detected
- `--format json` flag to force JSON output mode

### Out of scope

- PDF extraction (see 009)
- Schema validation (see 013)
- CSS/XPath selectors

## Detection logic

```
1. Check Content-Type header
   ├── application/json → JSON handler
   ├── text/html → HTML-to-markdown pipeline (004)
   ├── text/plain → Plain text handler
   ├── text/xml, application/xml → XML handler
   ├── application/pdf → PDF handler (009, stub for now)
   ├── image/* → Image metadata handler
   └── other → Binary/raw handler

2. If Content-Type is missing or ambiguous:
   ├── Try JSON parse → if valid, treat as JSON
   ├── Check for HTML markers (<!DOCTYPE, <html) → HTML pipeline
   ├── Check magic bytes for PDF (%PDF) → PDF handler
   └── Default to raw/binary
```

## JSON content field

```json
{
  "content": {
    "type": "json",
    "original_type": "application/json",
    "body": {
      "users": [
        { "id": 1, "name": "Alice" }
      ]
    },
    "metadata": {
      "element_count": 1,
      "top_level_type": "object"
    }
  }
}
```

Note: `body` contains the parsed JSON directly, not a stringified version.

## Plain text content field

```json
{
  "content": {
    "type": "text",
    "original_type": "text/plain",
    "body": "Plain text content here...",
    "metadata": {
      "word_count": 42,
      "line_count": 5
    }
  }
}
```

## Binary content field

```json
{
  "content": {
    "type": "binary",
    "original_type": "image/png",
    "body": null,
    "metadata": {
      "size_bytes": 245832,
      "note": "Binary content not included. Use --output to save to file."
    }
  }
}
```

## Acceptance criteria

- [ ] `gurl get https://httpbin.org/get` returns `content.type: "json"` with parsed body
- [ ] `gurl get https://httpbin.org/robots.txt` returns `content.type: "text"`
- [ ] `gurl get https://httpbin.org/image/png` returns `content.type: "binary"` with metadata
- [ ] `gurl get https://example.com` returns `content.type: "markdown"` (routed to HTML pipeline)
- [ ] Response with no Content-Type header but valid JSON body is detected as JSON
- [ ] `--format json` wrapping always produces the envelope regardless of content type
- [ ] `--output file.json` saves the response body to a file
