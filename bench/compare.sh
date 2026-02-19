#!/usr/bin/env bash
set -euo pipefail

# Benchmark: gurl vs curl for AI agent consumption
# Compares: speed, output size, and estimated token count

GURL="${GURL:-./target/release/gurl}"
URLS=(
  # Doc sites
  "https://docs.stripe.com/api"
  "https://vercel.com/docs/frameworks/nextjs"
  "https://docs.github.com/en/rest/overview"
  # Web pages
  "https://en.wikipedia.org/wiki/Rust_(programming_language)"
  "https://news.ycombinator.com"
  # Files
  "https://arxiv.org/pdf/1706.03762"
  # API / data
  "https://httpbin.org/json"
  "https://httpbin.org/html"
  # Plain text
  "https://httpbin.org/robots.txt"
)

# Token estimation: ~4 chars per token for English text
estimate_tokens() {
  local chars=$1
  echo $(( chars / 4 ))
}

printf "\n%-55s %8s %8s %10s %10s %7s\n" \
  "URL" "curl" "gurl" "curl_tok" "gurl_tok" "saved"
printf '%.0s─' {1..102}
echo

for url in "${URLS[@]}"; do
  label="${url:0:55}"

  # curl: raw content
  curl_start=$(python3 -c "import time; print(int(time.time()*1000))")
  curl_out=$(curl -sL "$url" 2>/dev/null || true)
  curl_end=$(python3 -c "import time; print(int(time.time()*1000))")
  curl_ms=$(( curl_end - curl_start ))
  curl_chars=${#curl_out}
  curl_tokens=$(estimate_tokens $curl_chars)

  # gurl: converted content (body only)
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

  printf "%-55s %5dms %5dms %10d %10d %5d%%\n" \
    "$label" "$curl_ms" "$gurl_ms" "$curl_tokens" "$gurl_tokens" "$savings"
done

echo
echo "Columns: curl = raw HTML/bytes, gurl = clean markdown"
echo "curl_tok / gurl_tok = estimated LLM tokens (chars/4)"
echo "saved = token reduction when gurl feeds an agent vs raw curl output"
echo
echo "Claude web_fetch costs you ~curl_tok in context window."
echo "gurl costs ~gurl_tok — the difference is wasted tokens (and money)."
