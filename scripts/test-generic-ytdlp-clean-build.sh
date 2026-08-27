#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/.." && pwd)
target_dir=$(mktemp -d "${TMPDIR:-/tmp}/generic-ytdlp-clean-build.XXXXXX")
trap 'rm -rf -- "$target_dir"' EXIT HUP INT TERM

cd -- "$repo_root"

# Reproduce the old clean-target state: Cargo has built only the smoke binary,
# so there must not already be a sibling sandbox to satisfy the runtime check.
CARGO_TARGET_DIR="$target_dir" cargo build --quiet --locked \
  -p generic-ytdlp --features runtime-prep \
  --bin generic-ytdlp-real-smoke
test -x "$target_dir/debug/generic-ytdlp-real-smoke"
test ! -e "$target_dir/debug/ytdlp-sandbox"

assert_sandbox_unavailable() {
  local output status
  set +e
  output=$("$target_dir/debug/generic-ytdlp-real-smoke" \
    'https://no-site-request.invalid/' 2>&1)
  status=$?
  set -e
  test "$status" -eq 75
  grep -q '^result: BLOCKED$' <<<"$output"
  grep -q '^broker_request_count: 0$' <<<"$output"
  grep -q '^process_error: SANDBOX_UNAVAILABLE$' <<<"$output"
}

assert_sandbox_unavailable
mkdir "$target_dir/debug/ytdlp-sandbox"
assert_sandbox_unavailable
rmdir "$target_dir/debug/ytdlp-sandbox"
touch "$target_dir/debug/ytdlp-sandbox"
chmod 600 "$target_dir/debug/ytdlp-sandbox"
assert_sandbox_unavailable
rm "$target_dir/debug/ytdlp-sandbox"

# Exercise the fixed, non-caller-selectable sibling validation against missing,
# invalid and executable artifacts without issuing a site request.
CARGO_TARGET_DIR="$target_dir" cargo test --quiet --locked \
  -p generic-ytdlp --features runtime-prep \
  --bin generic-ytdlp-real-smoke

# The repository-owned closure builds the only accepted sibling name from the
# same checkout. Both artifacts must be regular executable files afterward.
CARGO_TARGET_DIR="$target_dir" cargo build --quiet --locked \
  -p generic-ytdlp --features runtime-prep \
  --bin generic-ytdlp-real-smoke --bin ytdlp-sandbox
test -f "$target_dir/debug/generic-ytdlp-real-smoke"
test -x "$target_dir/debug/generic-ytdlp-real-smoke"
test ! -L "$target_dir/debug/generic-ytdlp-real-smoke"
test -f "$target_dir/debug/ytdlp-sandbox"
test -x "$target_dir/debug/ytdlp-sandbox"
test ! -L "$target_dir/debug/ytdlp-sandbox"

# Run the clean-built pair through the broker-capable worker path without a
# public/site request. R008 must reject loopback before a network side effect;
# reaching that bounded broker result proves the exact sibling was started.
test -d "${YTDLP_SOURCE:?YTDLP_SOURCE must select the prepared frozen runtime}"
set +e
broker_output=$(YTDLP_PREP_PYTHONPATH="$YTDLP_SOURCE" \
  "$target_dir/debug/generic-ytdlp-real-smoke" \
  'http://127.0.0.1/metadata' 2>&1)
broker_status=$?
set -e
test "$broker_status" -ne 0
grep -q '^broker_error_code: BROKER_EGRESS_REJECTED$' <<<"$broker_output"
grep -Eq '^broker_request_count: [1-9][0-9]*$' <<<"$broker_output"
! grep -q '^process_error: SANDBOX_UNAVAILABLE$' <<<"$broker_output"
