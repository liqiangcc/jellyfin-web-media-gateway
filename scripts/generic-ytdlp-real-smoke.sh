#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  printf '%s\n' \
    'result: BLOCKED' \
    'plugin: generic-ytdlp' \
    'broker_status_class: n/a' \
    'broker_error_code: n/a' \
    'broker_request_count: 0' \
    'protocol: n/a' \
    'stream_count: 0' \
    'title_length: n/a' \
    'process_error: INVALID_ARGUMENTS'
  exit 64
fi

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/.." && pwd)
work_dir=$(mktemp -d "${TMPDIR:-/tmp}/generic-ytdlp-real-smoke.XXXXXX")
trap 'rm -rf -- "$work_dir"' EXIT HUP INT TERM

setup_log="$work_dir/setup.log"
offline_helper="$script_dir/generic-ytdlp-offline-runtime.py"
python_bin=$(command -v python3 || true)
if [[ -z "$python_bin" || ! -x "$python_bin" ]]; then
  printf '%s\n' \
    'result: BLOCKED' \
    'plugin: generic-ytdlp' \
    'broker_status_class: n/a' \
    'broker_error_code: n/a' \
    'broker_request_count: 0' \
    'protocol: n/a' \
    'stream_count: 0' \
    'title_length: n/a' \
    'process_error: FROZEN_RUNTIME_SETUP'
  exit 75
fi

if [[ -z "${YTDLP_OFFLINE_BUNDLE:-}" ]]; then
  printf '%s\n' \
      'result: BLOCKED' \
      'plugin: generic-ytdlp' \
      'broker_status_class: n/a' \
      'broker_error_code: n/a' \
      'broker_request_count: 0' \
      'protocol: n/a' \
      'stream_count: 0' \
      'title_length: n/a' \
      'process_error: FROZEN_RUNTIME_SETUP'
  exit 75
fi

cache_info=$(
  "$python_bin" "$offline_helper" install "$YTDLP_OFFLINE_BUNDLE" 2>"$setup_log"
) || {
  printf '%s\n' \
    'result: BLOCKED' \
    'plugin: generic-ytdlp' \
    'broker_status_class: n/a' \
    'broker_error_code: n/a' \
    'broker_request_count: 0' \
    'protocol: n/a' \
    'stream_count: 0' \
    'title_length: n/a' \
    'process_error: FROZEN_RUNTIME_SETUP'
  exit 75
}
mapfile -t cache_lines <<<"$cache_info"
if [[ "${#cache_lines[@]}" -ne 3 || ( "${cache_lines[0]}" != "hit" && "${cache_lines[0]}" != "prepared" ) || -z "${cache_lines[1]}" || ! "${cache_lines[2]}" =~ ^[0-9a-f]{64}$ ]]; then
  printf '%s\n' \
    'result: BLOCKED' \
    'plugin: generic-ytdlp' \
    'broker_status_class: n/a' \
    'broker_error_code: n/a' \
    'broker_request_count: 0' \
    'protocol: n/a' \
    'stream_count: 0' \
    'title_length: n/a' \
    'process_error: FROZEN_RUNTIME_VERIFY'
  exit 75
fi
runtime_cache="offline-${cache_lines[0]}"
site_dir="${cache_lines[1]}"

cd -- "$repo_root"
# Setup-only proxy/network state must not enter the extractor process. The
# worker itself also uses env_clear and only receives the inherited R008 fd.
unset HTTP_PROXY HTTPS_PROXY ALL_PROXY http_proxy https_proxy all_proxy
# The smoke binary resolves only a fixed sibling sandbox. Build that sibling
# explicitly from this checkout before cargo starts the smoke binary so a clean
# target directory cannot depend on an artifact left by an earlier command.
cargo build --quiet -p generic-ytdlp --features runtime-prep \
  --bin ytdlp-sandbox 2>"$setup_log"
set +e
output=$(YTDLP_PREP_PYTHONPATH="$site_dir" \
  cargo run --quiet -p generic-ytdlp --features runtime-prep \
  --bin generic-ytdlp-real-smoke -- "$1" 2>"$setup_log")
status=$?
set -e
if [[ -n "$output" ]]; then
  printf '%s\nruntime_cache: %s\n' "$output" "$runtime_cache"
else
  printf '%s\n' \
    'result: FAIL' \
    'plugin: generic-ytdlp' \
    'broker_status_class: n/a' \
    'broker_error_code: n/a' \
    'broker_request_count: 0' \
    'protocol: n/a' \
    'stream_count: 0' \
    'title_length: n/a' \
    'process_error: HARNESS_EXECUTION' \
    "runtime_cache: $runtime_cache"
fi
exit "$status"
