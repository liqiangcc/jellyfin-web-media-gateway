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
  set +e
  if [[ -n "$fixture_pid" ]]; then
    kill "$fixture_pid" 2>/dev/null || true
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
  if curl --silent --fail "$server_url/System/Info/Public" >/dev/null; then
    break
  fi
  sleep 1
done
curl --silent --fail "$server_url/System/Info/Public" >/dev/null

curl --silent --fail --request POST "$server_url/Startup/Configuration" \
  --header 'Content-Type: application/json' \
  --data '{"UICulture":"en-US","MetadataCountryCode":"US","PreferredMetadataLanguage":"en"}' >/dev/null
curl --silent --fail --request POST "$server_url/Startup/User" \
  --data-urlencode "Name=$user_name" --data-urlencode "Password=$user_password" >/dev/null
curl --silent --fail --request POST "$server_url/Startup/Complete" >/dev/null

auth_json="$work_dir/auth.json"
curl --silent --fail --request POST "$server_url/Users/AuthenticateByName" \
  --header 'Content-Type: application/json' \
  --header 'X-Emby-Authorization: MediaBrowser Client="R004", Device="CI", DeviceId="r004-ci", Version="1.0"' \
  --data "{\"Username\":\"$user_name\",\"Pw\":\"$user_password\"}" >"$auth_json"
token="$(jq -r '.AccessToken' "$auth_json")"
user_id="$(jq -r '.User.Id' "$auth_json")"
auth_header="X-Emby-Token: $token"

curl --silent --fail --request POST \
  "$server_url/Library/VirtualFolders?collectionType=movies&refreshLibrary=true&name=R004Smoke&paths=%2Fmedia" \
  --header "$auth_header" >/dev/null

item_id=""
for _ in $(seq 1 90); do
  items="$(curl --silent --fail "$server_url/Items?UserId=$user_id&Recursive=true&IncludeItemTypes=Movie&Fields=Path" \
    --header "$auth_header")"
  item_id="$(jq -r '.Items[] | select(.Path | endswith("R004-smoke.strm")) | .Id' <<<"$items" | head -n 1)"
  if [[ -n "$item_id" && "$item_id" != "null" ]]; then
    break
  fi
  curl --silent --fail --request POST "$server_url/Library/Refresh" \
    --header "$auth_header" >/dev/null || true
  sleep 1
done
test -n "$item_id" && test "$item_id" != null

playback_info="$work_dir/playback-info.json"
curl --silent --fail \
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
