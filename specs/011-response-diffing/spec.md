# 011 — Response Diffing

> **Parent spec:** [001-init](../001-init/spec.md) — Sections 6.2 F2.3
> **Status:** Ready
> **Dependencies:** 003-http-client
> **Estimated effort:** 1 day

---

## Goal

Compare two HTTP responses structurally. Useful for detecting API changes, monitoring content updates, and regression testing.

## Scope

### In scope

- `gurl diff <url_a> <url_b>` — fetch both URLs and diff responses
- `--save-snapshot <name>` — save a response for later comparison
- `gurl diff <url> --against <snapshot>` — diff against saved snapshot
- `--ignore-fields <fields>` — ignore fields in comparison (timestamps, request IDs)
- Structural diff for JSON responses (field-level changes)
- Text diff for markdown/text content
- Header diff
- Status code comparison
- Snapshot storage in `~/.local/share/gurl/snapshots/`
- Add `diff` subcommand to CLI

### Out of scope

- Visual/HTML diff rendering
- Continuous monitoring (see 014)
- Webhook notifications on diff

## Diff output format

```json
{
  "gurl": "0.1.0",
  "diff": {
    "status": { "a": 200, "b": 200, "changed": false },
    "headers": {
      "added": ["x-new-header"],
      "removed": [],
      "changed": {
        "content-length": { "a": "1234", "b": "1256" }
      }
    },
    "content": {
      "type": "json",
      "changes": [
        { "path": "$.data.count", "a": 42, "b": 43, "type": "changed" },
        { "path": "$.data.new_field", "b": "hello", "type": "added" }
      ]
    },
    "summary": {
      "total_changes": 3,
      "added": 1,
      "removed": 0,
      "changed": 2
    }
  }
}
```

## Dependencies (crates)

- `similar` for text diffing
- `serde_json` for JSON structural comparison

## Acceptance criteria

- [ ] `gurl diff https://httpbin.org/get https://httpbin.org/get` shows no content changes (headers may differ)
- [ ] `gurl get https://httpbin.org/get --save-snapshot test1` saves snapshot
- [ ] `gurl diff https://httpbin.org/get --against test1` compares against snapshot
- [ ] `--ignore-fields "timestamp,request_id"` excludes those fields from diff
- [ ] JSON diffs show field-level changes with JSONPath
- [ ] Text/markdown diffs show line-level changes
- [ ] Diff output includes summary counts
