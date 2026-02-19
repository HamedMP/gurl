# 013 — Schema Validation

> **Parent spec:** [001-init](../001-init/spec.md) — Sections 6.2 F2.2
> **Status:** Ready
> **Dependencies:** 003-http-client, 005-content-detection
> **Estimated effort:** 1 day

---

## Goal

Validate API responses against JSON Schema or OpenAPI specs. Agents need to verify that APIs return expected data shapes before processing.

## Scope

### In scope

- `--schema <json-schema>` flag: validate response body against inline JSON Schema
- `--schema @schema.json` flag: validate against schema from file
- `--openapi <spec-file>` flag: validate response against OpenAPI spec for the matched path
- Validation result in response envelope
- Clear error messages listing which fields failed validation
- CSS selector extraction: `--selector <css>` to extract specific elements from HTML

### Out of scope

- LLM-based extraction (`--extract` with a schema prompt)
- Schema generation from responses
- OpenAPI spec generation

## Validation output

When `--schema` is provided, add a `validation` field to the envelope:

```json
{
  "content": { ... },
  "validation": {
    "valid": false,
    "errors": [
      {
        "path": "/items/0/name",
        "message": "missing field: name is required",
        "schema_path": "/properties/items/items/required"
      }
    ]
  }
}
```

When valid: `{ "validation": { "valid": true, "errors": [] } }`

## Selector output

```bash
gurl get https://example.com --selector "h1"
```

```json
{
  "content": {
    "type": "extracted",
    "original_type": "text/html",
    "body": "Example Domain",
    "matches": [
      { "selector": "h1", "text": "Example Domain", "html": "<h1>Example Domain</h1>" }
    ]
  }
}
```

## Dependencies (crates)

- `jsonschema` for JSON Schema validation
- `scraper` for CSS selector extraction

## Acceptance criteria

- [ ] `gurl get https://httpbin.org/get --schema '{"type":"object","required":["url"]}'` shows valid
- [ ] Invalid schema match shows clear error with field path
- [ ] `--schema @file.json` reads schema from file
- [ ] `gurl get https://example.com --selector "h1"` extracts heading text
- [ ] `--selector` supports multiple selectors (comma-separated or repeatable flag)
- [ ] Schema validation works with JSON responses only (error message if non-JSON)
