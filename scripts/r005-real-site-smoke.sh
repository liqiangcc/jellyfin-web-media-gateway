#!/usr/bin/env bash
set -euo pipefail

sample_url='https://www.bilibili.com/video/BV14V411W7r5/'
printf 'utc=%s\nsite=bilibili\nsample=BV14V411W7r5\n' \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
printf 'candidate_sha=%s\n' "$(git rev-parse HEAD)"
printf 'sample_url=%s\n' "$sample_url"

# The example uses gateway-core's central EgressPolicy and address-pinned
# client. It never accepts cookies, authorization headers, or signed URLs.
cargo run -q -p bilibili --example real_site_smoke
