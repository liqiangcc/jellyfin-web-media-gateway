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
