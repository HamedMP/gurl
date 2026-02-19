# 014 — Watch / Monitor Mode

> **Parent spec:** [001-init](../001-init/spec.md) — Sections 6.2 F2.5
> **Status:** Ready
> **Dependencies:** 003-http-client, 011-response-diffing
> **Estimated effort:** 1 day

---

## Goal

Poll a URL at regular intervals and detect changes. Optionally trigger a webhook or print a diff when something changes.

## Scope

### In scope

- `gurl watch <url> --interval <duration>` — poll at intervals
- `--on-change <action>` — action when change detected:
  - `print` (default): print the diff
  - `webhook:<url>`: POST the diff to a webhook URL
  - `command:<cmd>`: execute a shell command
- `--diff` flag to show what changed (integrates with 011)
- `--ignore-fields` to exclude volatile fields from change detection
- `--count <n>` to limit number of checks
- Structured output: each check emits a JSON line
- Add `watch` subcommand to CLI

### Out of scope

- Persistent monitoring (daemon mode, systemd integration)
- Alerting integrations beyond webhooks (Slack, email, PagerDuty)
- Cron-style scheduling

## Output format

```jsonl
{"check": 1, "timestamp": "...", "status": 200, "changed": false}
{"check": 2, "timestamp": "...", "status": 200, "changed": true, "diff": {...}}
{"check": 3, "timestamp": "...", "status": 503, "changed": true, "diff": {...}}
```

## Acceptance criteria

- [ ] `gurl watch https://httpbin.org/get --interval 5s --count 3` polls 3 times
- [ ] Changes in response body trigger `"changed": true`
- [ ] `--diff` includes the actual diff in output
- [ ] `--ignore-fields "timestamp"` excludes that field from comparison
- [ ] `--on-change webhook:https://...` POSTs the diff payload
- [ ] Ctrl+C cleanly stops monitoring
- [ ] Watch mode works with `--quiet` (just prints diff body)
