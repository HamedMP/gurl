#!/usr/bin/env bash
set -euo pipefail

# Benchmark gurl content extraction across real-world programming sites
# Tests: content quality (chars extracted), speed, and token savings vs curl

GURL="${GURL:-./target/release/gurl}"

# Categories of sites developers need
declare -A SITES

# Documentation sites
SITES["python-stdlib"]="https://docs.python.org/3/library/index.html"
SITES["rust-std"]="https://doc.rust-lang.org/std/"
SITES["mdn-js"]="https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference"
SITES["react-ref"]="https://react.dev/reference/react"
SITES["nextjs-docs"]="https://nextjs.org/docs"
SITES["go-docs"]="https://go.dev/doc/"
SITES["fastapi"]="https://fastapi.tiangolo.com/"
SITES["typescript"]="https://www.typescriptlang.org/docs/"

# API references
SITES["stripe-api"]="https://docs.stripe.com/api"
SITES["github-rest"]="https://docs.github.com/en/rest"
SITES["openai-api"]="https://platform.openai.com/docs/api-reference"
SITES["anthropic-api"]="https://docs.anthropic.com/en/api"

# Next.js/RSC sites (client-rendered)
SITES["vercel-nextjs"]="https://vercel.com/docs/frameworks/nextjs"
SITES["vercel-deploy"]="https://vercel.com/docs"
SITES["claude-tools"]="https://platform.claude.com/docs/en/agents-and-tools/tool-use/web-fetch-tool"

# Cloud & infrastructure
SITES["cloudflare-workers"]="https://developers.cloudflare.com/workers/"
SITES["fly-docs"]="https://fly.io/docs/"
SITES["neon-docs"]="https://neon.tech/docs"
SITES["supabase-docs"]="https://supabase.com/docs"

# Database docs
SITES["postgres"]="https://www.postgresql.org/docs/current/"
SITES["redis-cmds"]="https://redis.io/docs/latest/commands/"
SITES["prisma"]="https://www.prisma.io/docs"

# DevOps & tools
SITES["docker-ref"]="https://docs.docker.com/reference/"
SITES["k8s-concepts"]="https://kubernetes.io/docs/concepts/"
SITES["gh-actions"]="https://docs.github.com/en/actions"
SITES["git-docs"]="https://git-scm.com/docs"

# Learning & reference
SITES["http-status"]="https://developer.mozilla.org/en-US/docs/Web/HTTP/Status"
SITES["web-apis"]="https://developer.mozilla.org/en-US/docs/Web/API"

# Web content
SITES["wikipedia-rust"]="https://en.wikipedia.org/wiki/Rust_(programming_language)"
SITES["hackernews"]="https://news.ycombinator.com"

# Data formats
SITES["httpbin-json"]="https://httpbin.org/json"
SITES["httpbin-html"]="https://httpbin.org/html"
SITES["httpbin-txt"]="https://httpbin.org/robots.txt"

# Files
SITES["arxiv-pdf"]="https://arxiv.org/pdf/1706.03762"

estimate_tokens() {
  echo $(( $1 / 4 ))
}

# Sort sites by category for readable output
ORDERED=(
  # Docs
  "python-stdlib" "rust-std" "mdn-js" "react-ref" "nextjs-docs" "go-docs" "fastapi" "typescript"
  # APIs
  "stripe-api" "github-rest" "openai-api" "anthropic-api"
  # Next.js/RSC
  "vercel-nextjs" "vercel-deploy" "claude-tools"
  # Cloud
  "cloudflare-workers" "fly-docs" "neon-docs" "supabase-docs"
  # DB
  "postgres" "redis-cmds" "prisma"
  # DevOps
  "docker-ref" "k8s-concepts" "gh-actions" "git-docs"
  # Reference
  "http-status" "web-apis"
  # Content
  "wikipedia-rust" "hackernews"
  # Data
  "httpbin-json" "httpbin-html" "httpbin-txt"
  # Files
  "arxiv-pdf"
)

printf "\n%-22s %8s %10s %10s %7s  %s\n" \
  "SITE" "gurl_ms" "curl_tok" "gurl_tok" "saved" "STATUS"
printf '%.0s─' {1..85}
echo

pass=0
fail=0
total=0

for name in "${ORDERED[@]}"; do
  url="${SITES[$name]}"
  total=$((total + 1))

  # curl: raw content
  curl_out=$(curl -sL --max-time 15 "$url" 2>/dev/null || true)
  curl_chars=${#curl_out}
  curl_tokens=$(estimate_tokens $curl_chars)

  # gurl: converted content
  gurl_start=$(python3 -c "import time; print(int(time.time()*1000))")
  gurl_out=$($GURL get "$url" --quiet 2>/dev/null || true)
  gurl_end=$(python3 -c "import time; print(int(time.time()*1000))")
  gurl_ms=$(( gurl_end - gurl_start ))
  gurl_chars=${#gurl_out}
  gurl_tokens=$(estimate_tokens $gurl_chars)

  # Token savings
  if [ "$curl_tokens" -gt 0 ]; then
    savings=$(( 100 - (gurl_tokens * 100 / curl_tokens) ))
  else
    savings=0
  fi

  # Quality check
  if [ "$gurl_chars" -gt 200 ]; then
    status="OK"
    pass=$((pass + 1))
  elif [ "$gurl_chars" -gt 0 ]; then
    status="SMALL"
    fail=$((fail + 1))
  else
    status="EMPTY"
    fail=$((fail + 1))
  fi

  printf "%-22s %5dms %10d %10d %5d%%  %s\n" \
    "$name" "$gurl_ms" "$curl_tokens" "$gurl_tokens" "$savings" "$status"
done

echo
printf '%.0s─' {1..85}
echo
printf "Results: %d/%d passed (>200 chars extracted)\n" "$pass" "$total"
echo
echo "curl_tok = raw HTML tokens (what web_fetch costs in context window)"
echo "gurl_tok = clean markdown tokens (what gurl costs)"
echo "saved = token reduction percentage"
