#!/usr/bin/env bash
# Narrow token adapter for the official actions/runner config.sh interface.
# PREP tests use only a fake config.sh. Live use requires a separate #131 gate.
set +x
set -u
umask 077

result_json() {
  local result="$1"
  local mode="$2"
  local code="$3"
  printf '{"result":"%s","mode":"%s","exit_code":%s}\n' "$result" "$mode" "$code"
}

fail() {
  local reason="$1"
  printf '{"result":"BLOCKED","reason":"%s"}\n' "$reason"
  exit 2
}

[ "$#" -ge 2 ] || fail "usage"
mode="$1"
config_path="$2"
shift 2

case "$config_path" in
  /*/config.sh) ;;
  *) fail "config-path-rejected" ;;
esac
[ -x "$config_path" ] || fail "config-not-executable"

case "$mode" in
  remove)
    [ "$#" -eq 0 ] || fail "remove-args-rejected"
    ;;
  register)
    [ "$#" -eq 4 ] || fail "register-args-rejected"
    repo_url="$1"
    runner_name="$2"
    custom_labels="$3"
    work_dir="$4"
    case "$repo_url" in
      https://github.com/*/*) ;;
      *) fail "repo-url-rejected" ;;
    esac
    [ "$runner_name" = "ubuntu-arm64-target-phone" ] || fail "runner-name-rejected"
    [ "$work_dir" = "_work" ] || fail "work-dir-rejected"
    [ -n "$custom_labels" ] || fail "labels-rejected"
    case "$custom_labels" in
      *--replace*) fail "replace-rejected" ;;
    esac
    ;;
  *)
    fail "mode-rejected"
    ;;
esac

token=""
cleanup_token() {
  token=""
  unset token 2>/dev/null || true
}
trap cleanup_token EXIT HUP INT TERM

IFS= read -r token || fail "token-missing"
[ -n "$token" ] || fail "token-missing"
[ "${#token}" -le 4096 ] || fail "token-oversized"

child_rc=0
if [ "$mode" = "remove" ]; then
  "$config_path" remove --token "$token" >/dev/null 2>&1 || child_rc=$?
else
  register_args=(
    --unattended
    --url "$repo_url"
    --token "$token"
    --name "$runner_name"
  )
  if [ "$custom_labels" != "-" ]; then
    register_args+=(--labels "$custom_labels")
  fi
  register_args+=(--work "$work_dir")
  "$config_path" "${register_args[@]}" >/dev/null 2>&1 || child_rc=$?
fi

cleanup_token
trap - EXIT HUP INT TERM

if [ "$child_rc" -eq 0 ]; then
  result_json "SUCCESS" "$mode" 0
  exit 0
fi

result_json "CHILD_FAILED" "$mode" "$child_rc"
exit 4
