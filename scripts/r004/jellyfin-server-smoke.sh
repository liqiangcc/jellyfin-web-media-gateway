#!/usr/bin/env bash
set -euo pipefail

work_dir="$(mktemp -d "${RUNNER_TEMP:-/tmp}/r004-jellyfin.XXXXXX")"
container_name="r004-jellyfin-${RANDOM}"
server_url="http://127.0.0.1:8096"
user_name="r004-smoke"
user_password="r004-smoke-password"
media_url="http://host.docker.internal:8788/r004.mp4"
fixture_pid=""

cleanup() {
  exit_code=$?
  set +e
  if [[ -n "$fixture_pid" ]]; then
    kill "$fixture_pid" 2>/dev/null || true
  fi
  if [[ "$exit_code" -ne 0 ]]; then
    echo 'Jellyfin smoke failed; server diagnostics:' >&2
    docker logs "$container_name" 2>&1 | tail -n 80 >&2 || true
    cat "$work_dir/media-server.log" >&2 2>/dev/null || true
  fi
  docker rm -f "$container_name" >/dev/null 2>&1 || true
  if [[ -d "$work_dir" ]]; then
    docker run --rm --user 0:0 --volume "$work_dir:/cleanup" \
      --entrypoint chown jellyfin/jellyfin:10.11.11 -R "$(id -u):$(id -g)" /cleanup \
      >/dev/null 2>&1 || true
  fi
  rm -rf -- "$work_dir"
}
trap cleanup EXIT

post_until_ready() {
  local endpoint="$1"
  shift
  for _ in $(seq 1 90); do
    if curl --silent --show-error --fail-with-body --request POST \
      "$server_url/$endpoint" "$@" >/dev/null 2>"$work_dir/startup-error.log"; then
      return 0
    fi
    sleep 1
  done
  cat "$work_dir/startup-error.log" >&2
  return 1
}

mkdir -p "$work_dir/media" "$work_dir/config" "$work_dir/cache"
ffmpeg -hide_banner -loglevel error -f lavfi -i testsrc=size=320x180:rate=24 \
  -f lavfi -i sine=frequency=440:sample_rate=44100 -t 5 -c:v libx264 \
  -pix_fmt yuv420p -c:a aac -movflags +faststart "$work_dir/media/r004.mp4"
printf '%s\n' "$media_url" >"$work_dir/media/R004-smoke.strm"
python3 -m http.server 8788 --bind 0.0.0.0 --directory "$work_dir/media" \
  >"$work_dir/media-server.log" 2>&1 &
fixture_pid=$!

docker run --detach --name "$container_name" --add-host host.docker.internal:host-gateway \
  --publish 8096:8096 --volume "$work_dir/config:/config" \
  --volume "$work_dir/cache:/cache" --volume "$work_dir/media:/media" \
  jellyfin/jellyfin:10.11.11 >/dev/null

for _ in $(seq 1 90); do
  if curl --silent --show-error --fail-with-body "$server_url/System/Info/Public" >/dev/null; then
    break
  fi
  sleep 1
done
curl --silent --show-error --fail-with-body "$server_url/System/Info/Public" >/dev/null

for _ in $(seq 1 90); do
  if curl --silent --show-error --fail-with-body "$server_url/Startup/User" \
    >/dev/null 2>"$work_dir/startup-user-get-error.log"; then
    break
  fi
  sleep 1
done
curl --silent --show-error --fail-with-body "$server_url/Startup/User" >/dev/null

post_until_ready 'Startup/Configuration' \
  --header 'Content-Type: application/json' \
  --data '{"UICulture":"en-US","MetadataCountryCode":"US","PreferredMetadataLanguage":"en"}'
post_until_ready 'Startup/User' \
  --header 'Content-Type: application/json' \
  --data "{\"Name\":\"$user_name\",\"Password\":\"$user_password\"}"
post_until_ready 'Startup/Complete'

auth_json="$work_dir/auth.json"
curl --silent --show-error --fail-with-body --request POST "$server_url/Users/AuthenticateByName" \
  --header 'Content-Type: application/json' \
  --header 'X-Emby-Authorization: MediaBrowser Client="R004", Device="CI", DeviceId="r004-ci", Version="1.0"' \
  --data "{\"Username\":\"$user_name\",\"Pw\":\"$user_password\"}" >"$auth_json"
token="$(jq -r '.AccessToken' "$auth_json")"
user_id="$(jq -r '.User.Id' "$auth_json")"
auth_header="X-Emby-Token: $token"

curl --silent --show-error --fail-with-body --request POST \
  "$server_url/Library/VirtualFolders?collectionType=movies&refreshLibrary=true&name=R004Smoke&paths=%2Fmedia" \
  --header "$auth_header" >/dev/null

item_id=""
for _ in $(seq 1 90); do
  items="$(curl --silent --show-error --fail-with-body "$server_url/Items?UserId=$user_id&Recursive=true&IncludeItemTypes=Movie&Fields=Path" \
    --header "$auth_header")"
  item_id="$(jq -r '.Items[] | select(.Path | endswith("R004-smoke.strm")) | .Id' <<<"$items" | head -n 1)"
  if [[ -n "$item_id" && "$item_id" != "null" ]]; then
    break
  fi
  curl --silent --show-error --fail-with-body --request POST "$server_url/Library/Refresh" \
    --header "$auth_header" >/dev/null || true
  sleep 1
done
test -n "$item_id" && test "$item_id" != null

playback_info="$work_dir/playback-info.json"
curl --silent --show-error --fail-with-body \
  "$server_url/Items/$item_id/PlaybackInfo?UserId=$user_id" \
  --header "$auth_header" >"$playback_info"
source_path="$(jq -r '.MediaSources[0].Path' "$playback_info")"
source_id="$(jq -r '.MediaSources[0].Id' "$playback_info")"
test "$source_path" = "$media_url"
test -n "$source_id" && test "$source_id" != null

cat >"$GITHUB_WORKSPACE/r004-jellyfin-server-smoke.txt" <<EOF
server_image=jellyfin/jellyfin:10.11.11
item_representation=strm-library-item
item_id=$item_id
media_source_id=$source_id
playback_info_path_match=PASS
standard_session_play_fields=ItemIds,MediaSourceId,StartPositionTicks
invented_media_url_field=NOT_SENT
android_tv_verdict=NOT_CLAIMED
EOF
cat "$GITHUB_WORKSPACE/r004-jellyfin-server-smoke.txt"
