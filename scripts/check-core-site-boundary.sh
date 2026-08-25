#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# This is deliberately an explicit vocabulary for Stable Core production
# surfaces.  Generic browser/UI words (for example `document.querySelector`)
# are not site knowledge and must not make the guard unusably broad.
DENY_PATTERN='bilibili|youtube|youtu\.be|tiktok|douyin|instagram|facebook|twitter|x\.com|netflix|pornhub|sessdata|buvid3|ytcfg|ytInitialData|bpx-player|site_id[[:space:]]*([!=]=)[[:space:]]*[\"]'

scan_paths() {
    local paths=("$@")
    if rg -n -i --glob '*.rs' "$DENY_PATTERN" "${paths[@]}"; then
        echo "Stable Core contains concrete-site vocabulary" >&2
        return 1
    fi
}

default_scan() {
    # Keep this list narrow and production-only.  In particular, do not scan
    # docs, tests, fixtures, or plugins: those surfaces are allowed to discuss
    # concrete sites while testing/documenting the boundary.
    scan_paths \
        "$ROOT_DIR/gateway-core/src" \
        "$ROOT_DIR/site-adapter-api/src" \
        "$ROOT_DIR/display-adapter-api/src"
}

if [[ "${1:-}" == "--self-test" ]]; then
    positive="$ROOT_DIR/scripts/fixtures/core-site-boundary-positive.rs"
    negative="$ROOT_DIR/scripts/fixtures/core-site-boundary-negative.rs"

    if scan_paths "$positive" >/dev/null 2>&1; then
        echo "architecture guard failed to detect positive fixture" >&2
        exit 1
    fi
    scan_paths "$negative" >/dev/null

    # The repository's documentation and plugin implementations are not part
    # of the scan contract even though they necessarily mention real sites.
    if rg -n -i 'bilibili|youtube' "$ROOT_DIR/docs/site-plugin-architecture.md" "$ROOT_DIR/plugins" >/dev/null; then
        :
    else
        echo "self-test fixtures lost their concrete-site sentinel" >&2
        exit 1
    fi
    echo "core site-boundary guard self-test: PASS"
    exit 0
fi

default_scan
echo "core site-boundary guard: PASS"
