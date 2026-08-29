#!/usr/bin/env python3
"""Minimal, data-minimizing browser diagnostics MCP.

This module intentionally depends only on the Python standard library.  It
speaks the small HTTP/WebSocket subset needed by Chrome DevTools over a local
Unix socket and exposes only the fixed MCP tool allowlist below.
"""

from __future__ import annotations

import argparse
import base64
import collections
import hashlib
import ipaddress
import json
import os
import re
import socket
import struct
import sys
import threading
import time
import urllib.parse
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable, Mapping


MCP_PROTOCOL_VERSION = "2025-06-18"
SERVER_NAME = "browser-diag-mcp"
SERVER_VERSION = "0.1.0"
MAX_HTTP_BYTES = 64 * 1024
MAX_CDP_MESSAGE_BYTES = 256 * 1024
MAX_TARGETS = 32
MAX_TARGET_ID_BYTES = 128
MAX_CAPTURE_EVENTS = 256
MAX_REDIRECTS = 8
TARGET_HANDLE_RE = re.compile(r"^target_[0-9a-f]{16}$")
HOST_RE = re.compile(r"^(?=.{1,253}\Z)(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)*[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$", re.ASCII)
VERSION_RE = re.compile(r"\d+(?:\.\d+){0,3}")

TOOL_NAMES = (
    "health",
    "list_targets",
    "open_url",
    "reload",
    "network_capture_start",
    "network_summary",
    "network_capture_stop",
)

STATUS_CLASSES = {"2xx", "3xx", "4xx", "5xx", "network-error", "unknown"}
PROTOCOLS = {"http/1.1", "h2", "h3", "other", "unknown"}
IP_FAMILIES = {"ipv4", "ipv6", "unknown"}
TLS_PROTOCOLS = {"tls1.2", "tls1.3", "other", "unknown"}
DURATION_BUCKETS = {"under-100ms", "100ms-1s", "1s-5s", "5s-30s", "30s-or-more", "unknown"}
TRISTATE = {True, False, "unknown"}
SUMMARY_FIELDS = {
    "status_class",
    "redirect_count",
    "redirect_status_classes",
    "protocol",
    "remote_ip_family",
    "connection_reused",
    "from_disk_cache",
    "from_service_worker",
    "tls_protocol",
    "duration_bucket",
    "browser_product_family",
    "browser_version",
}
PROHIBITED_OUTPUT_TERMS = (
    "authorization",
    "set-cookie",
    "cookie",
    "header",
    "body",
    "dom",
    "websocket",
    "url",
    "token",
    "payload",
    "history",
    "title",
)

class DiagError(Exception):
    """A deliberately coarse error safe to return through MCP."""

    def __init__(self, code: str) -> None:
        super().__init__(code)
        self.code = code


def _object_schema(properties: Mapping[str, Any] | None = None, required: tuple[str, ...] = ()) -> dict[str, Any]:
    schema: dict[str, Any] = {
        "type": "object",
        "properties": dict(properties or {}),
        "additionalProperties": False,
    }
    if required:
        schema["required"] = list(required)
    return schema


def _enum_string(values: tuple[str, ...]) -> dict[str, Any]:
    return {"type": "string", "enum": list(values)}


HEALTH_OUTPUT_SCHEMA = _object_schema(
    {
        "transport_status": _enum_string(("ready", "unavailable")),
        "browser_product_family": _enum_string(("ChromeBeta", "Chromium", "other")),
        "browser_version": {"type": "string", "maxLength": 24},
        "protocol_version_status": _enum_string(("present", "unknown")),
    },
    ("transport_status", "browser_product_family", "browser_version", "protocol_version_status"),
)
TARGETS_OUTPUT_SCHEMA = _object_schema(
    {
        "targets": {
            "type": "array",
            "maxItems": MAX_TARGETS,
            "items": _object_schema(
                {"target_id": {"type": "string", "pattern": TARGET_HANDLE_RE.pattern}, "type": {"const": "page"}},
                ("target_id", "type"),
            ),
        },
        "truncated": {"type": "boolean"},
    },
    ("targets", "truncated"),
)
SUMMARY_OUTPUT_SCHEMA = _object_schema(
    {
        "status_class": _enum_string(tuple(sorted(STATUS_CLASSES))),
        "redirect_count": {"type": "integer", "minimum": 0, "maximum": MAX_REDIRECTS},
        "redirect_status_classes": {"type": "array", "maxItems": MAX_REDIRECTS, "items": _enum_string(tuple(sorted(STATUS_CLASSES)))},
        "protocol": _enum_string(tuple(sorted(PROTOCOLS))),
        "remote_ip_family": _enum_string(tuple(sorted(IP_FAMILIES))),
        "connection_reused": {"enum": [True, False, "unknown"]},
        "from_disk_cache": {"enum": [True, False, "unknown"]},
        "from_service_worker": {"enum": [True, False, "unknown"]},
        "tls_protocol": _enum_string(tuple(sorted(TLS_PROTOCOLS))),
        "duration_bucket": _enum_string(tuple(sorted(DURATION_BUCKETS))),
        "browser_product_family": _enum_string(("ChromeBeta", "Chromium", "other")),
        "browser_version": {"type": "string", "maxLength": 24},
    },
    tuple(sorted(SUMMARY_FIELDS)),
)


TOOL_DEFINITIONS = (
    {
        "name": "health",
        "description": "Check the dedicated diagnostic browser transport.",
        "inputSchema": _object_schema(),
        "outputSchema": HEALTH_OUTPUT_SCHEMA,
    },
    {
        "name": "list_targets",
        "description": "List bounded opaque page targets from the dedicated browser.",
        "inputSchema": _object_schema(),
        "outputSchema": TARGETS_OUTPUT_SCHEMA,
    },
    {
        "name": "open_url",
        "description": "Navigate a dedicated-browser target to an allowlisted HTTPS host.",
        "inputSchema": _object_schema(
            {"target_id": {"type": "string", "pattern": TARGET_HANDLE_RE.pattern}, "url": {"type": "string", "maxLength": 2048}},
            ("target_id", "url"),
        ),
        "outputSchema": _object_schema({"navigation_status": _enum_string(("accepted", "rejected"))}, ("navigation_status",)),
    },
    {
        "name": "reload",
        "description": "Reload a dedicated-browser page target.",
        "inputSchema": _object_schema({"target_id": {"type": "string", "pattern": TARGET_HANDLE_RE.pattern}}, ("target_id",)),
        "outputSchema": _object_schema({"reload_status": {"const": "accepted"}}, ("reload_status",)),
    },
    {
        "name": "network_capture_start",
        "description": "Start bounded top-level navigation metadata capture.",
        "inputSchema": _object_schema({"target_id": {"type": "string", "pattern": TARGET_HANDLE_RE.pattern}}, ("target_id",)),
        "outputSchema": _object_schema({"capture_status": {"const": "active"}}, ("capture_status",)),
    },
    {
        "name": "network_summary",
        "description": "Return only allowlisted coarse navigation metadata.",
        "inputSchema": _object_schema({"target_id": {"type": "string", "pattern": TARGET_HANDLE_RE.pattern}}, ("target_id",)),
        "outputSchema": SUMMARY_OUTPUT_SCHEMA,
    },
    {
        "name": "network_capture_stop",
        "description": "Stop capture and discard its bounded event state.",
        "inputSchema": _object_schema({"target_id": {"type": "string", "pattern": TARGET_HANDLE_RE.pattern}}, ("target_id",)),
        "outputSchema": _object_schema({"capture_status": {"const": "stopped"}}, ("capture_status",)),
    },
)


def parse_allowed_hosts(values: list[str]) -> frozenset[str]:
    hosts: set[str] = set()
    for value in values:
        for item in value.split(","):
            host = item.strip().lower().rstrip(".")
            if not host or not HOST_RE.fullmatch(host):
                raise ValueError("allowed hosts must be exact DNS host names")
            hosts.add(host)
    if not hosts:
        raise ValueError("at least one allowed host is required")
    return frozenset(hosts)


def validate_navigation_url(value: Any, allowed_hosts: frozenset[str]) -> str:
    if not isinstance(value, str) or len(value) > 2048 or any(ord(char) < 0x20 for char in value):
        raise DiagError("INVALID_NAVIGATION")
    try:
        parsed = urllib.parse.urlsplit(value)
        port = parsed.port
    except ValueError as exc:
        raise DiagError("INVALID_NAVIGATION") from exc
    host = (parsed.hostname or "").lower().rstrip(".")
    if parsed.scheme != "https" or parsed.username is not None or parsed.password is not None:
        raise DiagError("HTTPS_REQUIRED")
    if port not in (None, 443) or host not in allowed_hosts:
        raise DiagError("HOST_NOT_ALLOWED")
    return value


def validate_target_handle(value: Any) -> str:
    if not isinstance(value, str) or len(value) > 32 or not TARGET_HANDLE_RE.fullmatch(value):
        raise DiagError("INVALID_TARGET")
    return value


def status_class(value: Any) -> str:
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        code = int(value)
        if 200 <= code <= 599:
            return f"{code // 100}xx"
    return "unknown"


def protocol_class(value: Any) -> str:
    if not isinstance(value, str):
        return "unknown"
    lowered = value.lower()
    if lowered in {"http/1.1", "h2", "h3"}:
        return lowered
    return "other" if lowered else "unknown"


def tls_class(value: Any) -> str:
    if not isinstance(value, str) or not value:
        return "unknown"
    compact = value.lower().replace(" ", "").replace("_", "").replace("v", "")
    if compact in {"tls1.2", "tls12"}:
        return "tls1.2"
    if compact in {"tls1.3", "tls13"}:
        return "tls1.3"
    return "other"


def ip_family(value: Any) -> str:
    if not isinstance(value, str):
        return "unknown"
    candidate = value.strip("[]")
    try:
        return "ipv4" if ipaddress.ip_address(candidate).version == 4 else "ipv6"
    except ValueError:
        return "unknown"


def tristate(value: Any) -> bool | str:
    return value if isinstance(value, bool) else "unknown"


def duration_bucket(seconds: float | None) -> str:
    if seconds is None or seconds < 0:
        return "unknown"
    if seconds < 0.1:
        return "under-100ms"
    if seconds < 1:
        return "100ms-1s"
    if seconds < 5:
        return "1s-5s"
    if seconds < 30:
        return "5s-30s"
    return "30s-or-more"


def sanitize_browser_version(product: Any) -> str:
    if not isinstance(product, str):
        return "unknown"
    match = VERSION_RE.search(product)
    return match.group(0)[:24] if match else "unknown"


class UnixSocketTransport:
    """Factory for the shared AF_UNIX stream accepted by Issue #119."""

    def __init__(self, path: Path, timeout: float = 3.0) -> None:
        if not path.is_absolute() or len(os.fsencode(path)) > 100:
            raise ValueError("CDP Unix socket path must be absolute and bounded")
        self.path = path
        self.timeout = timeout

    def connect(self) -> socket.socket:
        client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        client.settimeout(self.timeout)
        try:
            client.connect(str(self.path))
        except Exception:
            client.close()
            raise
        return client


def _recv_until(client: socket.socket, marker: bytes, limit: int) -> tuple[bytes, bytes]:
    data = bytearray()
    while marker not in data:
        chunk = client.recv(min(4096, limit - len(data)))
        if not chunk:
            raise DiagError("TRANSPORT_UNAVAILABLE")
        data.extend(chunk)
        if len(data) >= limit:
            raise DiagError("TRANSPORT_RESPONSE_TOO_LARGE")
    head, rest = bytes(data).split(marker, 1)
    return head, rest


def _http_get_json(transport: UnixSocketTransport, path: str) -> Any:
    client = transport.connect()
    try:
        request = f"GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nAccept: application/json\r\n\r\n"
        client.sendall(request.encode("ascii"))
        head, body = _recv_until(client, b"\r\n\r\n", 16 * 1024)
        lines = head.split(b"\r\n")
        if not lines or not lines[0].startswith(b"HTTP/1.1 200"):
            raise DiagError("TRANSPORT_UNAVAILABLE")
        length: int | None = None
        for line in lines[1:]:
            name, separator, raw_value = line.partition(b":")
            if separator and name.lower() == b"content-length":
                try:
                    length = int(raw_value.strip())
                except ValueError as exc:
                    raise DiagError("TRANSPORT_INVALID_RESPONSE") from exc
        if length is None or length < 0 or length > MAX_HTTP_BYTES:
            raise DiagError("TRANSPORT_INVALID_RESPONSE")
        while len(body) < length:
            chunk = client.recv(min(4096, length - len(body)))
            if not chunk:
                raise DiagError("TRANSPORT_INVALID_RESPONSE")
            body += chunk
        try:
            return json.loads(body[:length])
        except (UnicodeDecodeError, json.JSONDecodeError) as exc:
            raise DiagError("TRANSPORT_INVALID_RESPONSE") from exc
    finally:
        client.close()


class WebSocketConnection:
    """Small RFC 6455 client sufficient for local Chrome CDP traffic."""

    def __init__(self, transport: UnixSocketTransport, endpoint: str) -> None:
        parsed = urllib.parse.urlsplit(endpoint)
        if parsed.scheme != "ws" or parsed.hostname not in {"localhost", "127.0.0.1"} or not parsed.path.startswith("/devtools/"):
            raise DiagError("TARGET_UNAVAILABLE")
        self._socket = transport.connect()
        self._write_lock = threading.Lock()
        key = base64.b64encode(os.urandom(16)).decode("ascii")
        path = parsed.path + (f"?{parsed.query}" if parsed.query else "")
        request = (
            f"GET {path} HTTP/1.1\r\nHost: localhost\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n"
            f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\nOrigin: http://localhost\r\n\r\n"
        )
        self._socket.sendall(request.encode("ascii"))
        head, rest = _recv_until(self._socket, b"\r\n\r\n", 16 * 1024)
        if rest or not head.startswith(b"HTTP/1.1 101"):
            self.close()
            raise DiagError("TARGET_UNAVAILABLE")
        expected = base64.b64encode(hashlib.sha1((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode("ascii")).digest())
        accept = None
        for line in head.split(b"\r\n")[1:]:
            name, separator, value = line.partition(b":")
            if separator and name.lower() == b"sec-websocket-accept":
                accept = value.strip()
        if accept != expected:
            self.close()
            raise DiagError("TARGET_UNAVAILABLE")
        self._socket.settimeout(None)

    def send_text(self, payload: str) -> None:
        raw = payload.encode("utf-8")
        if len(raw) > MAX_CDP_MESSAGE_BYTES:
            raise DiagError("COMMAND_TOO_LARGE")
        mask = os.urandom(4)
        length = len(raw)
        header = bytearray([0x81])
        if length < 126:
            header.append(0x80 | length)
        elif length <= 0xFFFF:
            header.append(0x80 | 126)
            header.extend(struct.pack("!H", length))
        else:
            header.append(0x80 | 127)
            header.extend(struct.pack("!Q", length))
        header.extend(mask)
        masked = bytes(value ^ mask[index % 4] for index, value in enumerate(raw))
        with self._write_lock:
            self._socket.sendall(header + masked)

    def receive_text(self) -> str | None:
        while True:
            first = self._read_exact(2)
            opcode = first[0] & 0x0F
            masked = bool(first[1] & 0x80)
            length = first[1] & 0x7F
            if length == 126:
                length = struct.unpack("!H", self._read_exact(2))[0]
            elif length == 127:
                length = struct.unpack("!Q", self._read_exact(8))[0]
            if length > MAX_CDP_MESSAGE_BYTES:
                raise DiagError("TRANSPORT_RESPONSE_TOO_LARGE")
            mask = self._read_exact(4) if masked else b""
            payload = self._read_exact(length)
            if masked:
                payload = bytes(value ^ mask[index % 4] for index, value in enumerate(payload))
            if opcode == 0x8:
                return None
            if opcode == 0x9:
                self._send_control(0xA, payload)
                continue
            if opcode != 0x1:
                continue
            try:
                return payload.decode("utf-8")
            except UnicodeDecodeError as exc:
                raise DiagError("TRANSPORT_INVALID_RESPONSE") from exc

    def _read_exact(self, length: int) -> bytes:
        data = bytearray()
        while len(data) < length:
            chunk = self._socket.recv(length - len(data))
            if not chunk:
                raise DiagError("TARGET_UNAVAILABLE")
            data.extend(chunk)
        return bytes(data)

    def _send_control(self, opcode: int, payload: bytes) -> None:
        if len(payload) > 125:
            return
        mask = os.urandom(4)
        masked = bytes(value ^ mask[index % 4] for index, value in enumerate(payload))
        with self._write_lock:
            self._socket.sendall(bytes([0x80 | opcode, 0x80 | len(payload)]) + mask + masked)

    def close(self) -> None:
        try:
            self._socket.shutdown(socket.SHUT_RDWR)
        except OSError:
            pass
        self._socket.close()


@dataclass
class NavigationRecord:
    request_id: str = ""
    started_at: float | None = None
    finished_at: float | None = None
    status_class: str = "unknown"
    redirect_status_classes: list[str] = field(default_factory=list)
    protocol: str = "unknown"
    remote_ip_family: str = "unknown"
    connection_reused: bool | str = "unknown"
    from_disk_cache: bool | str = "unknown"
    from_service_worker: bool | str = "unknown"
    tls_protocol: str = "unknown"

    def summary(self, browser_family: str, browser_version: str) -> dict[str, Any]:
        elapsed = None
        if self.started_at is not None and self.finished_at is not None:
            elapsed = max(0.0, self.finished_at - self.started_at)
        return {
            "status_class": self.status_class,
            "redirect_count": min(len(self.redirect_status_classes), MAX_REDIRECTS),
            "redirect_status_classes": self.redirect_status_classes[:MAX_REDIRECTS],
            "protocol": self.protocol,
            "remote_ip_family": self.remote_ip_family,
            "connection_reused": self.connection_reused,
            "from_disk_cache": self.from_disk_cache,
            "from_service_worker": self.from_service_worker,
            "tls_protocol": self.tls_protocol,
            "duration_bucket": duration_bucket(elapsed),
            "browser_product_family": browser_family,
            "browser_version": browser_version,
        }


class CaptureState:
    """Reduces raw CDP events immediately into a bounded navigation record deque."""

    def __init__(self, max_events: int = MAX_CAPTURE_EVENTS) -> None:
        if max_events < 1 or max_events > MAX_CAPTURE_EVENTS:
            raise ValueError("capture bound is invalid")
        self.active = False
        self.events: collections.deque[tuple[str, str]] = collections.deque(maxlen=max_events)
        self.navigation = NavigationRecord()
        self._lock = threading.Lock()

    def start(self) -> None:
        with self._lock:
            self.active = True
            self.events.clear()
            self.navigation = NavigationRecord()

    def stop(self) -> None:
        with self._lock:
            self.active = False
            self.events.clear()
            self.navigation = NavigationRecord()

    def accept(self, message: Mapping[str, Any]) -> None:
        method = message.get("method")
        params = message.get("params")
        if not isinstance(method, str) or not isinstance(params, Mapping):
            return
        with self._lock:
            if not self.active:
                return
            self._accept_locked(method, params)

    def _accept_locked(self, method: str, params: Mapping[str, Any]) -> None:
        if method == "Network.requestWillBeSent" and params.get("type") == "Document":
            request_id = params.get("requestId")
            if not isinstance(request_id, str) or len(request_id) > MAX_TARGET_ID_BYTES:
                return
            timestamp = params.get("timestamp")
            if self.navigation.request_id != request_id:
                self.navigation = NavigationRecord(request_id=request_id, started_at=float(timestamp) if isinstance(timestamp, (int, float)) else None)
            redirect = params.get("redirectResponse")
            if isinstance(redirect, Mapping) and len(self.navigation.redirect_status_classes) < MAX_REDIRECTS:
                self.navigation.redirect_status_classes.append(status_class(redirect.get("status")))
            self.events.append(("request", self.navigation.status_class))
        elif method == "Network.responseReceived" and params.get("type") == "Document" and params.get("requestId") == self.navigation.request_id:
            response = params.get("response")
            if not isinstance(response, Mapping):
                return
            self.navigation.status_class = status_class(response.get("status"))
            self.navigation.protocol = protocol_class(response.get("protocol"))
            self.navigation.remote_ip_family = ip_family(response.get("remoteIPAddress"))
            self.navigation.connection_reused = tristate(response.get("connectionReused"))
            self.navigation.from_disk_cache = tristate(response.get("fromDiskCache"))
            self.navigation.from_service_worker = tristate(response.get("fromServiceWorker"))
            security = response.get("securityDetails")
            if isinstance(security, Mapping):
                self.navigation.tls_protocol = tls_class(security.get("protocol"))
            self.events.append(("response", self.navigation.status_class))
        elif method == "Network.loadingFinished" and params.get("requestId") == self.navigation.request_id:
            timestamp = params.get("timestamp")
            self.navigation.finished_at = float(timestamp) if isinstance(timestamp, (int, float)) else None
            self.events.append(("finished", self.navigation.status_class))
        elif method == "Network.loadingFailed" and params.get("requestId") == self.navigation.request_id:
            self.navigation.status_class = "network-error"
            timestamp = params.get("timestamp")
            self.navigation.finished_at = float(timestamp) if isinstance(timestamp, (int, float)) else None
            self.events.append(("failed", "network-error"))

    def summary(self, browser_family: str, browser_version: str) -> dict[str, Any]:
        with self._lock:
            if not self.active:
                raise DiagError("CAPTURE_NOT_ACTIVE")
            return self.navigation.summary(browser_family, browser_version)


class CdpSession:
    def __init__(self, websocket: WebSocketConnection, capture: CaptureState) -> None:
        self.websocket = websocket
        self.capture = capture
        self._condition = threading.Condition()
        self._next_id = 1
        self._responses: dict[int, Mapping[str, Any]] = {}
        self._closed = False
        self._reader = threading.Thread(target=self._reader_loop, name="browser-diag-cdp", daemon=True)
        self._reader.start()

    def command(self, method: str, params: Mapping[str, Any] | None = None, timeout: float = 3.0) -> Mapping[str, Any]:
        with self._condition:
            if self._closed:
                raise DiagError("TARGET_UNAVAILABLE")
            command_id = self._next_id
            self._next_id += 1
        payload = {"id": command_id, "method": method, "params": dict(params or {})}
        self.websocket.send_text(json.dumps(payload, separators=(",", ":")))
        deadline = time.monotonic() + timeout
        with self._condition:
            while command_id not in self._responses and not self._closed:
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise DiagError("COMMAND_TIMEOUT")
                self._condition.wait(remaining)
            response = self._responses.pop(command_id, None)
        if response is None:
            raise DiagError("TARGET_UNAVAILABLE")
        if "error" in response:
            raise DiagError("COMMAND_REJECTED")
        result = response.get("result", {})
        return result if isinstance(result, Mapping) else {}

    def _reader_loop(self) -> None:
        try:
            while True:
                text = self.websocket.receive_text()
                if text is None:
                    break
                try:
                    message = json.loads(text)
                except json.JSONDecodeError:
                    continue
                if not isinstance(message, Mapping):
                    continue
                response_id = message.get("id")
                if isinstance(response_id, int):
                    with self._condition:
                        if len(self._responses) < 32:
                            self._responses[response_id] = message
                        self._condition.notify_all()
                else:
                    self.capture.accept(message)
        except (OSError, DiagError):
            pass
        finally:
            with self._condition:
                self._closed = True
                self._condition.notify_all()

    def close(self) -> None:
        self.websocket.close()
        with self._condition:
            self._closed = True
            self._condition.notify_all()
        self._reader.join(timeout=1)


@dataclass(frozen=True)
class Target:
    raw_id: str
    handle: str
    kind: str
    endpoint: str


class BrowserDiagService:
    def __init__(
        self,
        transport: UnixSocketTransport,
        allowed_hosts: frozenset[str],
        browser_family: str,
        websocket_factory: Callable[[UnixSocketTransport, str], WebSocketConnection] = WebSocketConnection,
    ) -> None:
        self.transport = transport
        self.allowed_hosts = allowed_hosts
        self.browser_family = browser_family
        self.websocket_factory = websocket_factory
        self._salt = os.urandom(32)
        self._targets: dict[str, Target] = {}
        self._sessions: dict[str, CdpSession] = {}
        self.browser_version = "unknown"

    def health(self) -> dict[str, Any]:
        try:
            value = _http_get_json(self.transport, "/json/version")
            if not isinstance(value, Mapping):
                raise DiagError("TRANSPORT_INVALID_RESPONSE")
            self.browser_version = sanitize_browser_version(value.get("Browser"))
            protocol_present = isinstance(value.get("Protocol-Version"), str) and bool(value.get("Protocol-Version"))
            return {
                "transport_status": "ready",
                "browser_product_family": self.browser_family,
                "browser_version": self.browser_version,
                "protocol_version_status": "present" if protocol_present else "unknown",
            }
        except (OSError, DiagError):
            return {
                "transport_status": "unavailable",
                "browser_product_family": self.browser_family,
                "browser_version": "unknown",
                "protocol_version_status": "unknown",
            }

    def list_targets(self) -> dict[str, Any]:
        value = _http_get_json(self.transport, "/json/list")
        if not isinstance(value, list):
            raise DiagError("TRANSPORT_INVALID_RESPONSE")
        targets: list[dict[str, str]] = []
        fresh: dict[str, Target] = {}
        for item in value[:MAX_TARGETS]:
            if not isinstance(item, Mapping) or item.get("type") != "page":
                continue
            raw_id, endpoint = item.get("id"), item.get("webSocketDebuggerUrl")
            if not isinstance(raw_id, str) or not isinstance(endpoint, str):
                continue
            if not raw_id or len(raw_id.encode("utf-8")) > MAX_TARGET_ID_BYTES:
                continue
            handle = "target_" + hashlib.sha256(self._salt + raw_id.encode("utf-8")).hexdigest()[:16]
            target = Target(raw_id=raw_id, handle=handle, kind="page", endpoint=endpoint)
            fresh[handle] = target
            targets.append({"target_id": handle, "type": "page"})
        stale = set(self._sessions) - set(fresh)
        for handle in stale:
            self._sessions.pop(handle).close()
        self._targets = fresh
        return {"targets": targets, "truncated": len(value) > MAX_TARGETS}

    def _target(self, arguments: Mapping[str, Any]) -> Target:
        handle = validate_target_handle(arguments.get("target_id"))
        target = self._targets.get(handle)
        if target is None:
            raise DiagError("TARGET_NOT_FOUND")
        return target

    def _session(self, target: Target) -> CdpSession:
        session = self._sessions.get(target.handle)
        if session is None:
            session = CdpSession(self.websocket_factory(self.transport, target.endpoint), CaptureState())
            self._sessions[target.handle] = session
        return session

    def open_url(self, arguments: Mapping[str, Any]) -> dict[str, str]:
        target = self._target(arguments)
        navigation = validate_navigation_url(arguments.get("url"), self.allowed_hosts)
        result = self._session(target).command("Page.navigate", {"url": navigation})
        return {"navigation_status": "rejected" if result.get("errorText") else "accepted"}

    def reload(self, arguments: Mapping[str, Any]) -> dict[str, str]:
        target = self._target(arguments)
        self._session(target).command("Page.reload", {"ignoreCache": False})
        return {"reload_status": "accepted"}

    def network_capture_start(self, arguments: Mapping[str, Any]) -> dict[str, str]:
        target = self._target(arguments)
        session = self._session(target)
        session.capture.start()
        try:
            session.command("Network.enable")
        except DiagError:
            session.capture.stop()
            raise
        return {"capture_status": "active"}

    def network_summary(self, arguments: Mapping[str, Any]) -> dict[str, Any]:
        target = self._target(arguments)
        session = self._sessions.get(target.handle)
        if session is None:
            raise DiagError("CAPTURE_NOT_ACTIVE")
        result = session.capture.summary(self.browser_family, self.browser_version)
        validate_summary(result)
        return result

    def network_capture_stop(self, arguments: Mapping[str, Any]) -> dict[str, str]:
        target = self._target(arguments)
        session = self._sessions.get(target.handle)
        if session is None or not session.capture.active:
            raise DiagError("CAPTURE_NOT_ACTIVE")
        try:
            session.command("Network.disable")
        finally:
            session.capture.stop()
        return {"capture_status": "stopped"}

    def call(self, name: str, arguments: Any) -> dict[str, Any]:
        if name not in TOOL_NAMES:
            raise DiagError("TOOL_NOT_ALLOWED")
        if not isinstance(arguments, Mapping):
            raise DiagError("INVALID_ARGUMENTS")
        expected = {
            "health": set(),
            "list_targets": set(),
            "open_url": {"target_id", "url"},
            "reload": {"target_id"},
            "network_capture_start": {"target_id"},
            "network_summary": {"target_id"},
            "network_capture_stop": {"target_id"},
        }[name]
        if set(arguments) != expected:
            raise DiagError("INVALID_ARGUMENTS")
        result = getattr(self, name)(arguments) if expected else getattr(self, name)()
        assert_safe_output(result)
        return result

    def close(self) -> None:
        for session in list(self._sessions.values()):
            session.close()
        self._sessions.clear()
        self._targets.clear()


def validate_summary(value: Mapping[str, Any]) -> None:
    if set(value) != SUMMARY_FIELDS:
        raise DiagError("UNSAFE_OUTPUT")
    if value["status_class"] not in STATUS_CLASSES or value["protocol"] not in PROTOCOLS:
        raise DiagError("UNSAFE_OUTPUT")
    if value["remote_ip_family"] not in IP_FAMILIES or value["tls_protocol"] not in TLS_PROTOCOLS:
        raise DiagError("UNSAFE_OUTPUT")
    if value["duration_bucket"] not in DURATION_BUCKETS:
        raise DiagError("UNSAFE_OUTPUT")
    if any(value[field] not in TRISTATE for field in ("connection_reused", "from_disk_cache", "from_service_worker")):
        raise DiagError("UNSAFE_OUTPUT")
    redirects = value["redirect_status_classes"]
    if not isinstance(redirects, list) or len(redirects) > MAX_REDIRECTS or any(item not in STATUS_CLASSES for item in redirects):
        raise DiagError("UNSAFE_OUTPUT")
    if value["redirect_count"] != len(redirects):
        raise DiagError("UNSAFE_OUTPUT")
    if value["browser_product_family"] not in {"ChromeBeta", "Chromium", "other"}:
        raise DiagError("UNSAFE_OUTPUT")
    if not isinstance(value["browser_version"], str) or len(value["browser_version"]) > 24:
        raise DiagError("UNSAFE_OUTPUT")


def assert_safe_output(value: Any) -> None:
    def visit(item: Any) -> None:
        if isinstance(item, Mapping):
            for key, child in item.items():
                lowered = str(key).lower()
                if any(term in lowered for term in PROHIBITED_OUTPUT_TERMS):
                    raise DiagError("UNSAFE_OUTPUT")
                visit(child)
        elif isinstance(item, list):
            if len(item) > MAX_CAPTURE_EVENTS:
                raise DiagError("UNSAFE_OUTPUT")
            for child in item:
                visit(child)
        elif isinstance(item, str):
            lowered = item.lower()
            if len(item) > 128 or "://" in lowered or any(
                marker in lowered for marker in ("cookie", "authorization", "set-cookie", "bearer ", "token=")
            ):
                raise DiagError("UNSAFE_OUTPUT")

    visit(value)


class McpServer:
    def __init__(self, service: BrowserDiagService) -> None:
        self.service = service

    def handle(self, request: Any) -> dict[str, Any] | None:
        if not isinstance(request, Mapping) or request.get("jsonrpc") != "2.0":
            return self._error(request.get("id") if isinstance(request, Mapping) else None, -32600, "Invalid Request")
        request_id = request.get("id")
        method = request.get("method")
        if method == "notifications/initialized" or method == "notifications/cancelled":
            return None
        if method == "initialize":
            return self._result(
                request_id,
                {
                    "protocolVersion": MCP_PROTOCOL_VERSION,
                    "capabilities": {"tools": {"listChanged": False}},
                    "serverInfo": {"name": SERVER_NAME, "version": SERVER_VERSION},
                },
            )
        if method == "ping":
            return self._result(request_id, {})
        if method == "tools/list":
            return self._result(request_id, {"tools": list(TOOL_DEFINITIONS)})
        if method == "tools/call":
            params = request.get("params")
            if not isinstance(params, Mapping) or not isinstance(params.get("name"), str):
                return self._error(request_id, -32602, "Invalid params")
            try:
                value = self.service.call(params["name"], params.get("arguments", {}))
                text = json.dumps(value, sort_keys=True, separators=(",", ":"))
                return self._result(request_id, {"content": [{"type": "text", "text": text}], "structuredContent": value, "isError": False})
            except DiagError as exc:
                value = {"error": exc.code}
                text = json.dumps(value, separators=(",", ":"))
                return self._result(request_id, {"content": [{"type": "text", "text": text}], "structuredContent": value, "isError": True})
            except Exception:
                value = {"error": "INTERNAL_ERROR"}
                text = json.dumps(value, separators=(",", ":"))
                return self._result(request_id, {"content": [{"type": "text", "text": text}], "structuredContent": value, "isError": True})
        return self._error(request_id, -32601, "Method not found")

    @staticmethod
    def _result(request_id: Any, value: Any) -> dict[str, Any]:
        return {"jsonrpc": "2.0", "id": request_id, "result": value}

    @staticmethod
    def _error(request_id: Any, code: int, message: str) -> dict[str, Any]:
        return {"jsonrpc": "2.0", "id": request_id, "error": {"code": code, "message": message}}

    def run(self) -> None:
        try:
            for line in sys.stdin.buffer:
                if len(line) > MAX_HTTP_BYTES:
                    response = self._error(None, -32700, "Parse error")
                else:
                    try:
                        response = self.handle(json.loads(line))
                    except (UnicodeDecodeError, json.JSONDecodeError):
                        response = self._error(None, -32700, "Parse error")
                if response is not None:
                    encoded = json.dumps(response, separators=(",", ":"), ensure_ascii=True).encode("utf-8") + b"\n"
                    sys.stdout.buffer.write(encoded)
                    sys.stdout.buffer.flush()
        finally:
            self.service.close()


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Bounded browser diagnostics MCP over a local CDP Unix socket")
    parser.add_argument("--cdp-unix-socket", type=Path, required=True, help="absolute path to the local shared CDP relay socket")
    parser.add_argument("--allow-host", action="append", required=True, help="exact HTTPS DNS host; repeat for each allowed host")
    parser.add_argument("--browser-family", choices=("ChromeBeta", "Chromium", "other"), required=True)
    parser.add_argument("--timeout-seconds", type=float, default=3.0)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if not 0.1 <= args.timeout_seconds <= 10:
        raise SystemExit("timeout must be between 0.1 and 10 seconds")
    try:
        hosts = parse_allowed_hosts(args.allow_host)
        transport = UnixSocketTransport(args.cdp_unix_socket, args.timeout_seconds)
    except ValueError as exc:
        raise SystemExit(str(exc)) from exc
    McpServer(BrowserDiagService(transport, hosts, args.browser_family)).run()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
