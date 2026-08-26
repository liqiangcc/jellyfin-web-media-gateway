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

# This setup is isolated and user-writable. It deliberately uses --target
# instead of requiring python3-venv or any package manager. It is not
# extractor traffic: the real extraction below still gets every HTTP(S)
# request through R008Broker.
site_dir="$work_dir/site-packages"
setup_log="$work_dir/setup.log"
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

unset HTTP_PROXY HTTPS_PROXY ALL_PROXY http_proxy https_proxy all_proxy
"$python_bin" -m pip install --target "$site_dir" \
  --disable-pip-version-check --no-cache-dir --no-deps \
  'yt-dlp @ git+https://github.com/yt-dlp/yt-dlp.git@3a08beaf031ab68f966401ead017ac81fe8486cf' \
  >"$setup_log" 2>&1 || {
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

PYTHONPATH="$site_dir" "$python_bin" - "$site_dir" >"$setup_log" 2>&1 <<'PY' || {
import json
import pathlib
import sys

site_dir = pathlib.Path(sys.argv[1])
import yt_dlp
from importlib import metadata

assert yt_dlp.version.__version__ == "2026.08.19"
dist = metadata.distribution("yt-dlp")
direct_url = json.loads((pathlib.Path(dist._path) / "direct_url.json").read_text())
assert direct_url["vcs_info"]["commit_id"] == "3a08beaf031ab68f966401ead017ac81fe8486cf"
assert str(site_dir) in str(pathlib.Path(dist._path).parent)
PY
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
}

cd -- "$repo_root"
set +e
output=$(YTDLP_PREP_PYTHON="$python_bin" \
  YTDLP_PREP_PYTHONPATH="$site_dir" \
  cargo run --quiet -p generic-ytdlp --features runtime-prep \
    --bin generic-ytdlp-real-smoke -- "$1" 2>"$setup_log")
status=$?
set -e
if [[ -n "$output" ]]; then
  printf '%s\n' "$output"
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
    'process_error: HARNESS_EXECUTION'
fi
exit "$status"
