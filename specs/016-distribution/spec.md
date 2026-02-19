# 016 — Distribution & Release

> **Parent spec:** [001-init](../001-init/spec.md) — Sections 10.1, 11.2
> **Status:** Ready (can be done in parallel with feature work)
> **Dependencies:** 002-scaffolding
> **Estimated effort:** 1 day

---

## Goal

Set up the release pipeline so that tagged releases automatically produce binaries for all platforms, publish to crates.io, and provide a Homebrew tap.

## Scope

### In scope

- GitHub Actions release workflow: build binaries on tag push
- Cross-compiled binaries:
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
  - `x86_64-unknown-linux-gnu`
  - `aarch64-unknown-linux-gnu`
  - `x86_64-pc-windows-msvc`
- GitHub Releases with binaries attached
- `cargo publish` to crates.io
- Homebrew tap: `brew install gurl` via custom tap repo
- `install.sh` curl-style installer: `curl -fsSL https://gurl.dev/install | sh`
- Binary naming: `gurl-<version>-<target>.tar.gz`
- SHA256 checksums file

### Out of scope

- npm package (WASM or napi — Phase 4)
- Docker image (later)
- apt/yum repos (later)
- GitHub Actions marketplace action (later)

## Release workflow

```yaml
# Triggered on: push tag v*
# 1. Build binaries for all targets
# 2. Create GitHub Release with binaries
# 3. cargo publish
# 4. Update Homebrew tap
```

## Homebrew tap

Separate repo: `gurl/homebrew-tap`

```ruby
class Gurl < Formula
  desc "The HTTP runtime for AI agents"
  homepage "https://github.com/gurl/gurl"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/gurl/gurl/releases/download/v0.1.0/gurl-0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "..."
    else
      url "https://github.com/gurl/gurl/releases/download/v0.1.0/gurl-0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "..."
    end
  end

  def install
    bin.install "gurl"
  end
end
```

## Acceptance criteria

- [ ] `git tag v0.1.0 && git push --tags` triggers release build
- [ ] GitHub Release has binaries for all 5 targets
- [ ] `cargo install gurl` works from crates.io
- [ ] `brew install gurl/tap/gurl` installs on macOS
- [ ] `install.sh` detects OS/arch and downloads correct binary
- [ ] SHA256 checksums are published with each release
- [ ] Binary name doesn't conflict with existing packages

## Notes

- Check crates.io for `gurl` name availability early. If taken, consider alternatives or reach out to the owner.
- The install script should verify checksums after download.
