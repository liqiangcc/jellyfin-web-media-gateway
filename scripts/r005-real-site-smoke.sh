#!/usr/bin/env bash
set -euo pipefail

sample_url='https://www.bilibili.com/video/BV14V411W7r5/'
page_file="$(mktemp "${TMPDIR:-/tmp}/r005-bilibili.XXXXXX.html")"
trap 'rm -f -- "$page_file"' EXIT

status="$(curl -L --max-time 20 -sS -A 'Mozilla/5.0' -o "$page_file" -w '%{http_code}' "$sample_url" || true)"
printf 'utc=%s\nsite=bilibili\nsample=BV14V411W7r5\nhttp_status=%s\n' \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$status"

if [[ "$status" != 200 ]]; then
  echo 'result=BLOCKED (public page did not return HTTP 200; no challenge bypass attempted)'
  exit 2
fi

cargo run -q -p bilibili --example real_site_smoke -- "$page_file"
