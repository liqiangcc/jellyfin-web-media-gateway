#!/usr/bin/env python3
"""Offline privacy-safe transform for bounded reachability observations.

Raw endpoint values are accepted only through the in-memory API / stdin. The
process emits bounded metadata and a run-local opaque HMAC alias. It performs no
network, DNS, subprocess, or destination-selection work.
"""
from __future__ import annotations

import hashlib
import hmac
import ipaddress
import json
import secrets
import sys
from dataclasses import dataclass
from typing import Any

MAX_INPUT_LINE = 4096
ALIAS_HEX_LEN = 12
UNKNOWN = "unknown"


@dataclass(frozen=True)
class SanitizedObservation:
    family: str
    endpoint_alias: str
    http_version_class: str
    status_class: str
    timing_bucket: str

    def as_dict(self) -> dict[str, str]:
        return {
            "family": self.family,
            "endpoint_alias": self.endpoint_alias,
            "http_version_class": self.http_version_class,
            "status_class": self.status_class,
            "timing_bucket": self.timing_bucket,
        }


class RunSanitizer:
    """One bounded-run sanitizer.

    A fresh process gets a fresh random key. Tests may inject a fixed key so
    alias behavior is deterministic without publishing the runtime key.
    """

    def __init__(self, key: bytes | None = None) -> None:
        self._key = key if key is not None else secrets.token_bytes(32)
        if not isinstance(self._key, bytes) or len(self._key) < 16:
            raise ValueError("invalid run context")

    def sanitize(self, observation: Any) -> SanitizedObservation:
        item = observation if isinstance(observation, dict) else {}
        family, endpoint_alias = self._sanitize_endpoint(item.get("endpoint"))
        return SanitizedObservation(
            family=family,
            endpoint_alias=endpoint_alias,
            http_version_class=_http_version_class(item.get("http_version")),
            status_class=_status_class(item.get("status")),
            timing_bucket=_timing_bucket(item.get("timing_ms")),
        )

    def _sanitize_endpoint(self, raw: Any) -> tuple[str, str]:
        if not isinstance(raw, str) or len(raw) > 128:
            return UNKNOWN, UNKNOWN
        try:
            endpoint = ipaddress.ip_address(raw.strip())
        except ValueError:
            return UNKNOWN, UNKNOWN
        family = "ipv4" if endpoint.version == 4 else "ipv6"
        digest = hmac.new(self._key, endpoint.packed, hashlib.sha256).hexdigest()
        return family, f"ep-{digest[:ALIAS_HEX_LEN]}"


def _http_version_class(raw: Any) -> str:
    value = str(raw).strip().lower() if isinstance(raw, (str, int, float)) else ""
    if value in {"1", "1.0", "1.1", "h1", "http/1", "http/1.0", "http/1.1"}:
        return "h1"
    if value in {"2", "2.0", "h2", "http/2", "http/2.0"}:
        return "h2"
    if value in {"3", "3.0", "h3", "http/3", "http/3.0"}:
        return "h3"
    return UNKNOWN


def _status_class(raw: Any) -> str:
    if isinstance(raw, str) and raw.strip().lower() == "network-error":
        return "network-error"
    try:
        code = int(raw)
    except (TypeError, ValueError):
        return UNKNOWN
    if 200 <= code <= 599:
        return f"{code // 100}xx"
    return UNKNOWN


def _timing_bucket(raw: Any) -> str:
    try:
        value = float(raw)
    except (TypeError, ValueError):
        return UNKNOWN
    if value < 0:
        return UNKNOWN
    if value < 100:
        return "lt100ms"
    if value < 250:
        return "100-249ms"
    if value < 500:
        return "250-499ms"
    if value < 1000:
        return "500-999ms"
    if value < 2000:
        return "1-2s"
    return "ge2s"


def _unknown_record() -> dict[str, str]:
    return SanitizedObservation(UNKNOWN, UNKNOWN, UNKNOWN, UNKNOWN, UNKNOWN).as_dict()


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    if args:
        print("usage: reachability_observation_sanitizer.py < normalized-observations.jsonl", file=sys.stderr)
        return 2

    sanitizer = RunSanitizer()
    for line in sys.stdin:
        if len(line) > MAX_INPUT_LINE:
            print(json.dumps(_unknown_record(), separators=(",", ":")))
            continue
        try:
            payload = json.loads(line)
        except (json.JSONDecodeError, TypeError):
            payload = {}
        record = sanitizer.sanitize(payload).as_dict()
        print(json.dumps(record, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
