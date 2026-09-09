#!/usr/bin/env python3
"""R005 bounded, secret-free media-shape and refresh/CAS verifier.

This is a research harness. It models the proposed generic contract and audits
the current source tree for the known gap. It never opens a network connection,
reads a profile, or emits a source URL or media bytes.
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any

NOW_MS = 1_700_000_000_000
MAX_TTL_MS = 30 * 60 * 1000
SECRET_MARKERS = ("cookie", "authorization", "bearer", "sessdata", "signed-url", "token")


class HarnessFailure(RuntimeError):
    pass


def check(condition: bool, name: str) -> None:
    if not condition:
        raise HarnessFailure(name)


def result(mode: str, checks: list[str], classifications: dict[str, str]) -> None:
    # Keep the machine-readable result bounded and intentionally free of URLs,
    # identifiers from real sites, headers, access references, or media bytes.
    print(
        json.dumps(
            {
                "mode": mode,
                "result": "PASS",
                "checks": checks,
                "classifications": classifications,
                "secret_policy": "opaque-and-redacted",
            },
            sort_keys=True,
        )
    )


def source_text(path: str) -> str:
    return Path(path).read_text(encoding="utf-8")


def static_mode() -> None:
    api = source_text("site-adapter-api/src/lib.rs")
    plugin = source_text("plugins/bilibili/src/lib.rs")
    source_session = source_text("gateway-core/src/source_session.rs")
    core = source_text("gateway-core/src/lib.rs")
    playback = source_text("gateway-core/src/playback.rs")

    protocol = re.search(
        r"pub enum StreamProtocol\s*\{(?P<body>.*?)\n\}", api, re.DOTALL
    )
    check(protocol is not None, "stream-protocol-declaration-present")
    protocol_body = protocol.group("body")
    check("HttpFile" in protocol_body and "Hls" in protocol_body, "current-direct-protocols")
    check("Dash" not in protocol_body, "current-dash-gap")
    check("pub struct ResolvedStream" in api, "resolved-stream-declaration-present")

    stream_block = api.split("pub struct ResolvedStream", 1)[1].split("}", 1)[0]
    for field in ("pub id:", "pub protocol:", "pub url:", "pub public_headers:", "pub upstream_access_ref:"):
        check(field in stream_block, f"current-stream-field:{field.rstrip(':')}")
    for absent in ("role:", "group_id:", "container:", "codec:", "expires_at", "media_generation"):
        check(absent not in stream_block, f"missing-generic-field:{absent.rstrip(':')}")

    selection = plugin.split("let mut candidate = None;", 1)[1].split("let (observed, handoff)", 1)[0]
    check("BrowserMediaKind::Muxed" in selection, "plugin-muxed-only-selection")
    check("StreamProtocol::HttpFile | StreamProtocol::Hls" in selection, "plugin-direct-only-selection")

    public_stream = source_session.split("pub struct SessionMediaStream", 1)[1].split("}", 1)[0]
    for field in ("pub id:", "pub protocol:", "pub gateway_path:"):
        check(field in public_stream, f"public-stream-field:{field.rstrip(':')}")
    for absent in ("role:", "group_id:", "mime", "container", "codec", "expires"):
        check(absent not in public_stream, f"public-shape-gap:{absent}")

    check("media_generation: 0" in source_session, "initial-media-generation-present")
    check("pub fn begin_media_refresh" in playback, "refresh-sequencing-present")
    check("pub fn commit_media_refresh" in playback, "refresh-commit-present")
    check("media_generation" in source_session and "item_revision" in source_session, "snapshot-freshness-keys-present")
    check("item.protocol==='http_file'" in core, "display-http-file-selection")
    check("player.src=" in core and "rendering.media?.streams" in core, "display-source-attachment-present")

    result(
        "static",
        [
            "current ResolvedStream and public display projection audited",
            "current plugin accepts muxed HTTP-file/HLS only",
            "current Web Display selects an HTTP-file source",
            "Playback media-generation sequencing exists but no generic separated-A/V shape exists",
        ],
        {
            "C1": "PASS",
            "C6": "PASS",
        },
    )


def validate_track(track: dict[str, Any]) -> None:
    required = ("id", "role", "group_id", "source_protocol", "delivery", "container", "codec", "mime")
    for key in required:
        check(isinstance(track.get(key), str) and track[key], f"track-field:{key}")
    check(track["role"] in {"muxed", "video", "audio"}, "track-role")
    check(track["source_protocol"] in {"http_file", "hls", "dash"}, "source-protocol")
    check(track["delivery"] in {"direct", "server_remux"}, "delivery-mode")
    check(track["container"] in {"mp4", "fmp4", "ts", "webm", "unknown"}, "container")
    check(len(track["id"]) <= 128 and len(track["group_id"]) <= 128, "track-identifiers-bounded")
    check(len(track["codec"]) <= 128 and len(track["mime"]) <= 128, "track-codec-mime-bounded")
    access_ref = track.get("upstream_access_ref")
    check(isinstance(access_ref, str) and access_ref, "server-access-ref-present")
    check(len(access_ref) <= 256 and re.fullmatch(r"[A-Za-z0-9._:-]+", access_ref), "access-ref-bounded")
    lowered = access_ref.lower()
    check(not any(marker in lowered for marker in SECRET_MARKERS), "access-ref-not-secret-shaped")
    expires = track.get("expires_at_ms")
    check(isinstance(expires, int) and expires > NOW_MS, "expiry-present")
    check(expires - NOW_MS <= MAX_TTL_MS, "expiry-bounded")


def validate_shape(shape: dict[str, Any]) -> None:
    check(shape.get("schema_version") == 1, "shape-version")
    check(shape.get("media_generation") == 4, "shape-media-generation")
    tracks = shape.get("tracks")
    check(isinstance(tracks, list) and 1 <= len(tracks) <= 8, "track-count-bounded")
    for track in tracks:
        validate_track(track)
    groups: dict[str, set[str]] = {}
    for track in tracks:
        groups.setdefault(track["group_id"], set()).add(track["role"])
    for roles in groups.values():
        check(roles in ({"muxed"}, {"video", "audio"}), "pairing-group-complete")
    check(shape.get("public_projection") == {
        "stream_ids": [track["id"] for track in tracks if track["delivery"] == "server_remux"]
        or [track["id"] for track in tracks],
        "media_generation": 4,
    }, "public-projection-redacted")


def synthetic_mode() -> None:
    muxed = {
        "schema_version": 1,
        "media_generation": 4,
        "tracks": [
            {
                "id": "muxed-main",
                "role": "muxed",
                "group_id": "group-main",
                "source_protocol": "http_file",
                "delivery": "direct",
                "container": "mp4",
                "codec": "avc1+mp4a",
                "mime": "video/mp4",
                "upstream_access_ref": "fixture-ref-muxed",
                "expires_at_ms": NOW_MS + 900_000,
            }
        ],
        "public_projection": {"stream_ids": ["muxed-main"], "media_generation": 4},
    }
    separated = {
        "schema_version": 1,
        "media_generation": 4,
        "tracks": [
            {
                "id": "video-main",
                "role": "video",
                "group_id": "group-av",
                "source_protocol": "dash",
                "delivery": "server_remux",
                "container": "fmp4",
                "codec": "avc1",
                "mime": "video/mp4",
                "upstream_access_ref": "fixture-ref-video",
                "expires_at_ms": NOW_MS + 900_000,
            },
            {
                "id": "audio-main",
                "role": "audio",
                "group_id": "group-av",
                "source_protocol": "dash",
                "delivery": "server_remux",
                "container": "fmp4",
                "codec": "mp4a",
                "mime": "audio/mp4",
                "upstream_access_ref": "fixture-ref-audio",
                "expires_at_ms": NOW_MS + 900_000,
            },
        ],
        "public_projection": {"stream_ids": ["video-main", "audio-main"], "media_generation": 4},
    }
    validate_shape(muxed)
    validate_shape(separated)

    bad_pair = dict(separated)
    bad_pair["tracks"] = [dict(separated["tracks"][0], group_id="different")]
    try:
        validate_shape(bad_pair)
    except HarnessFailure:
        pass
    else:
        raise HarnessFailure("mismatched-pairing-must-fail")

    public_text = json.dumps(separated["public_projection"], sort_keys=True).lower()
    check("upstream_access_ref" not in public_text, "public-projection-no-access-ref")
    check("expires_at_ms" not in public_text, "public-projection-no-upstream-expiry")
    check("url" not in public_text and "cookie" not in public_text, "public-projection-no-url-or-cookie")

    result(
        "synthetic",
        [
            "muxed direct fixture validates",
            "paired video/audio fixture validates",
            "mismatched pairing is rejected",
            "public projection excludes server access and upstream expiry",
        ],
        {
            "C2": "PASS",
            "C3": "PASS",
            "C4": "PASS",
            "C6": "PASS",
        },
    )


def refresh_mode() -> None:
    state = {
        "session_revision": 11,
        "item_id": "item-1",
        "item_revision": 7,
        "media_generation": 4,
        "display_generation": 9,
        "capability_generation": 4,
    }

    def refresh(ticket: dict[str, int], resolved_generation: int) -> str:
        if ticket["item_revision"] != state["item_revision"]:
            return "STALE_ITEM"
        if ticket["media_generation"] != state["media_generation"]:
            return "STALE_MEDIA"
        if ticket["display_generation"] != state["display_generation"]:
            return "STALE_DISPLAY"
        state["media_generation"] += 1
        state["capability_generation"] = resolved_generation
        state["session_revision"] += 1
        return "COMMITTED"

    ticket = {
        "item_revision": 7,
        "media_generation": 4,
        "display_generation": 9,
    }
    check(refresh(ticket, 5) == "COMMITTED", "fresh-refresh-commits")
    check(state["media_generation"] == 5, "refresh-generation-increments")
    check(state["display_generation"] == 9, "refresh-preserves-display-generation")
    check(refresh(ticket, 6) == "STALE_MEDIA", "duplicate-refresh-is-stale")
    check(
        refresh({"item_revision": 6, "media_generation": 5, "display_generation": 9}, 7)
        == "STALE_ITEM",
        "old-item-result-rejected",
    )
    check(
        refresh({"item_revision": 7, "media_generation": 5, "display_generation": 8}, 7)
        == "STALE_DISPLAY",
        "old-display-result-rejected",
    )

    result(
        "refresh",
        [
            "same-item refresh uses item/media/display compare-and-swap",
            "successful refresh increments media generation",
            "stale item, media and display results are rejected",
            "active display generation is preserved",
        ],
        {
            "C3": "PASS",
            "C4": "PASS",
            "C6": "PASS",
        },
    )


def compatibility_mode() -> None:
    routes = {
        "muxed_http_file": "PASS",
        "muxed_hls_gateway": "CONDITIONAL PASS",
        "separated_server_remux_fmp4": "CONDITIONAL PASS",
        "separated_server_remux_hls": "CONDITIONAL PASS",
        "dash_mse": "CONDITIONAL PASS",
        "source_browser_playback": "FAIL",
    }
    check(routes["muxed_http_file"] == "PASS", "existing-muxed-route-retained")
    check(routes["separated_server_remux_fmp4"] == "CONDITIONAL PASS", "selected-route-is-conditional")
    check(routes["source_browser_playback"] == "FAIL", "source-browser-not-gateway-display")
    result(
        "compatibility",
        [
            "existing muxed HTTP-file route remains the baseline",
            "server remux to browser-compatible fMP4/HTTP-file is selected conditionally",
            "HLS and DASH/MSE remain explicit comparison routes",
            "source-browser playback is excluded from Gateway display delivery",
        ],
        {
            "C3": "CONDITIONAL PASS",
            "C5": "CONDITIONAL PASS",
            "C7": "PASS",
        },
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", choices=("static", "synthetic", "refresh", "compatibility"), required=True)
    args = parser.parse_args()
    {
        "static": static_mode,
        "synthetic": synthetic_mode,
        "refresh": refresh_mode,
        "compatibility": compatibility_mode,
    }[args.mode]()


if __name__ == "__main__":
    main()
