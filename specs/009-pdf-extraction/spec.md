# 009 — Content Intelligence: PDF Text Extraction

> **Parent spec:** [001-init](../001-init/spec.md) — Sections 5.2 (Content Intelligence), 6.1 F1.2
> **Status:** Ready
> **Dependencies:** 003-http-client, 005-content-detection
> **Estimated effort:** 1-2 days

---

## Goal

When gurl fetches a PDF from a URL, extract the text content and return it as markdown. This replaces the `curl + markitdown` two-step workflow.

## Scope

### In scope

- Detect PDF responses (Content-Type + magic bytes `%PDF`)
- Extract text from PDF using a Rust PDF library
- Basic structural preservation: paragraphs, headings (best effort)
- Return as `content.type: "markdown"` in envelope
- PDF metadata extraction: title, author, page count, creation date
- `--format raw` returns the raw PDF bytes (for saving to file)
- `--output report.pdf` saves the original PDF to disk

### Out of scope

- Table extraction from PDFs (extremely hard — defer to cloud/AI later)
- OCR for scanned PDFs (requires Tesseract or similar — out of scope)
- Image extraction from PDFs
- Complex layout analysis (Docling/MarkItDown territory)
- Local file conversion (`gurl convert file.pdf`) — this is URL-fetched only for now

## Content field

```json
{
  "content": {
    "type": "markdown",
    "original_type": "application/pdf",
    "body": "# Document Title\n\nExtracted text from the PDF...",
    "metadata": {
      "title": "Annual Report 2025",
      "author": "Acme Corp",
      "pages": 42,
      "created": "2025-12-01",
      "word_count": 8500
    }
  }
}
```

## Crate evaluation

| Crate | Purpose | Notes |
|-------|---------|-------|
| `pdf-extract` | Text extraction | Simple, may struggle with complex layouts |
| `lopdf` | Low-level PDF parsing | More control, more work |
| `pdf` | PDF reading | Another option to evaluate |

Pick whichever produces the best text output for typical documents. Quality > features.

## Acceptance criteria

- [ ] `gurl get https://example.com/report.pdf` returns extracted text as markdown
- [ ] PDF metadata (title, author, pages) is populated
- [ ] `--output report.pdf` saves the raw PDF file
- [ ] `--format raw` returns PDF bytes (useful for piping)
- [ ] Password-protected PDFs return a clear error message
- [ ] Corrupted/invalid PDFs return a structured error, not a crash
- [ ] Large PDFs (100+ pages) don't OOM — consider streaming or page limits

## Notes

- PDF text extraction in Rust is "good enough" for simple documents but won't match Docling/MarkItDown on complex layouts. That's acceptable — the value is in the zero-dependency, single-command experience. For complex PDFs, users can use `--cloud` later (spec 015).
- Consider a `content.metadata.extraction_quality` field: `"high"` for text-native PDFs, `"low"` for scanned/image-heavy PDFs where extraction is poor.
