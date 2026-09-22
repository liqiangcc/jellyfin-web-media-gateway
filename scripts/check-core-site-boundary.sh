#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# These patterns are deliberately separate.  Site behavior/URL/DOM vocabulary
# is forbidden throughout Stable Core.  A small set of site-specific secret
# marker literals is also forbidden in runtime logic, but the shared adapter
# validation function below is allowed to mention them solely to reject them.
SITE_BEHAVIOR_PATTERN='bilibili|youtube|youtu\.be|tiktok|douyin|instagram|facebook|twitter|x\.com|netflix|pornhub|ytcfg|ytInitialData|bpx-player|site_id[[:space:]]*([!=]=)[[:space:]]*[\"]'
SITE_SECRET_MARKER_PATTERN='sessdata|buvid3'

SHARED_SECRET_VALIDATION_FILE="$ROOT_DIR/site-adapter-api/src/lib.rs"

scan_paths() {
    local pattern="$1"
    shift
    local paths=("$@")
    local status
    if command -v rg >/dev/null 2>&1; then
        if rg -n -i --glob '*.rs' "$pattern" "${paths[@]}"; then
            status=0
        else
            status=$?
        fi
    else
        if grep -RInE --include='*.rs' "$pattern" "${paths[@]}"; then
            status=0
        else
            status=$?
        fi
    fi
    if [[ "$status" -eq 0 ]]; then
        echo "Stable Core contains concrete-site vocabulary" >&2
        return 1
    fi
    if [[ "$status" -ne 1 ]]; then
        return "$status"
    fi
}

scan_secret_markers() {
    local filtered_file
    local status=0
    filtered_file="$(mktemp /tmp/core-site-boundary-XXXXXX.rs)"

    # The only permitted occurrence in the shared API is inside this exact
    # generic rejection helper.  Remove that function before scanning; every
    # other occurrence remains fail-closed.
    awk '
        /fn contains_secret_marker\(value: &str\)/ { in_allowed=1; next }
        in_allowed {
            if ($0 ~ /^[[:space:]]*}/) { in_allowed=0 }
            next
        }
        { print }
    ' "$SHARED_SECRET_VALIDATION_FILE" >"$filtered_file"

    scan_paths "$SITE_SECRET_MARKER_PATTERN" \
        "$ROOT_DIR/gateway-core/src" \
        "$ROOT_DIR/display-adapter-api/src" \
        "$filtered_file" || status=$?
    rm -f "$filtered_file"
    return "$status"
}

default_scan() {
    # Keep this list narrow and production-only.  In particular, do not scan
    # docs, tests, fixtures, or plugins: those surfaces are allowed to discuss
    # concrete sites while testing/documenting the boundary.
    scan_paths "$SITE_BEHAVIOR_PATTERN" \
        "$ROOT_DIR/gateway-core/src" \
        "$ROOT_DIR/site-adapter-api/src" \
        "$ROOT_DIR/display-adapter-api/src"
    scan_secret_markers
}

if [[ "${1:-}" == "--self-test" ]]; then
    positive="$ROOT_DIR/scripts/fixtures/core-site-boundary-positive.rs"
    negative="$ROOT_DIR/scripts/fixtures/core-site-boundary-negative.rs"
    secret_positive="$ROOT_DIR/scripts/fixtures/core-site-boundary-secret-marker.rs"

    if scan_paths "$SITE_BEHAVIOR_PATTERN" "$positive" >/dev/null 2>&1; then
        echo "architecture guard failed to detect positive fixture" >&2
        exit 1
    fi
    scan_paths "$SITE_BEHAVIOR_PATTERN" "$negative" >/dev/null

    if scan_paths "$SITE_SECRET_MARKER_PATTERN" "$secret_positive" >/dev/null 2>&1; then
        echo "architecture guard failed to detect secret marker in Core fixture" >&2
        exit 1
    fi
    scan_secret_markers

    # The repository's documentation and plugin implementations are not part
    # of the scan contract even though they necessarily mention real sites.
    if grep -RniE 'bilibili|youtube' "$ROOT_DIR/docs/site-plugin-architecture.md" "$ROOT_DIR/plugins" >/dev/null; then
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
