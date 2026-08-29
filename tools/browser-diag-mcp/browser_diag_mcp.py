#!/usr/bin/env python3
"""Bounded, stdlib-only MCP for a dedicated browser CDP socket.

This is deliberately a small diagnostic tool, not a CDP proxy.  The only
transport accepted by this module is a configured filesystem AF_UNIX socket.
The relay and browser lifecycle are owned by the operator/phone harness.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import http.client
import json
import re
import socket
import sys
import time
import urllib.parse
from dataclasses import dataclass, field
from typing import Any, BinaryIO


MAX_JSON_BYTES = 128 * 1024
MAX_TARGETS = 16
MAX_REDIRECTS = 8
MAX_URL_LENGTH = 2048
MAX_HOST_LENGTH = 253
MAX_PROTOCOL_LENGTH = 16
ALLOWED_TOOLS = (
    "health",
    "list_targets",
    "open_url",
    "reload",
    "network_capture_start",
    "network_summary",
    "network_capture_stop",
)
STATUS_CLASSES = ("2xx", "3xx", "4xx", "5xx", "network-error", "unknown")
PROTOCOLS = ("http/1.1", "h2", "h3", "other", "unknown")
TLS_PROTOCOLS = ("TLS 1.2", "TLS 1.3", "other", "unknown")
DURATION_BUCKETS = ("<100ms", "100-499ms", "500-999ms", "1-4s", ">=5s", "unknown")
SAFE_ID = re.compile(r"^[A-Za-z0-9_-]{1,128}$")


class DiagnosticError(Exception):
    """An expected, safe-to-return diagnostic error."""


def _bounded_string(value: Any, limit: int = 128) -> str:
    if not isinstance(value, str):
        return ""
    return value[:limit]


def _status_class(status: Any) -> str:
    if isinstance(status, bool) or not isinstance(status, int) or not 100 <= status <= 599:
        return "unknown"
    return f"{status // 100}xx"


def _protocol(value: Any) -> str:
    if not isinstance(value, str):
        return "unknown"
    value = value.lower()
    if value in ("http/1.1", "h2", "h3"):
        return value
    if value:
        return "other"
    return "unknown"


def _tls_protocol(value: Any) -> str:
    if value in ("TLS 1.2", "TLS 1.3"):
        return value
    if isinstance(value, str) and value:
        return "other"
    return "unknown"


def _ip_family(value: Any) -> str:
    if not isinstance(value, str):
        return "unknown"
    if ":" in value:
        return "ipv6"
    if re.fullmatch(r"[0-9.]+", value):
        return "ipv4"
    return "unknown"


def _duration_bucket(start: float | None, end: float | None) -> str:
    if start is None or end is None or end < start:
        return "unknown"
    milliseconds = (end - start) * 1000
    if milliseconds < 100:
        return "<100ms"
    if milliseconds < 500:
        return "100-499ms"
    if milliseconds < 1000:
        return "500-999ms"
    if milliseconds < 5000:
        return "1-4s"
    return ">=5s"


def _safe_browser_version(browser: Any) -> str:
    """Return only a bounded product version, never a UA/fingerprint string."""
    if not isinstance(browser, str):
        return "unknown"
    match = re.search(r"(?:Chrome|Chromium)/([0-9]+(?:\.[0-9]+){0,3})", browser)
    if not match:
        return "unknown"
    return match.group(1)[:32]


def _read_exact(sock: socket.socket, size: int) -> bytes:
    chunks: list[bytes] = []
    remaining = size
    while remaining:
        chunk = sock.recv(min(65536, remaining))
        if not chunk:
            raise DiagnosticError("transport closed")
        chunks.append(chunk)
        remaining -= len(chunk)
    return b"".join(chunks)


def _http_json(socket_path: str, path: str, timeout: float) -> tuple[int, Any]:
    if not path.startswith("/") or "?" in path or "#" in path:
        raise DiagnosticError("invalid CDP endpoint")
    # AF_UNIX is intentional: no TCP fallback and no proxy-aware HTTP client.
    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    sock.settimeout(timeout)
    try:
        sock.connect(socket_path)
        request = (
            f"GET {path} HTTP/1.1\r\n"
            "Host: localhost\r\n"
            "Connection: close\r\n"
            "\r\n"
        ).encode("ascii")
        sock.sendall(request)
        header = bytearray()
        while b"\r\n\r\n" not in header:
            if len(header) > 16384:
                raise DiagnosticError("CDP headers too large")
            part = sock.recv(4096)
            if not part:
                raise DiagnosticError("transport closed")
            header.extend(part)
        raw_header, body_prefix = bytes(header).split(b"\r\n\r\n", 1)
        lines = raw_header.split(b"\r\n")
        match = re.fullmatch(rb"HTTP/1\.[01] ([0-9]{3})(?: .*)?", lines[0])
        if not match:
            raise DiagnosticError("invalid CDP response")
        status = int(match.group(1))
        headers: dict[bytes, bytes] = {}
        for line in lines[1:]:
            if b":" not in line:
                continue
            name, value = line.split(b":", 1)
            headers[name.lower().strip()] = value.strip()
        length_raw = headers.get(b"content-length")
        if length_raw is None:
            raise DiagnosticError("unbounded CDP response")
        try:
            length = int(length_raw)
        except ValueError as exc:
            raise DiagnosticError("invalid CDP response length") from exc
        if length < 0 or length > MAX_JSON_BYTES:
            raise DiagnosticError("CDP response too large")
        body = body_prefix + _read_exact(sock, max(0, length - len(body_prefix)))
        if len(body) != length:
            raise DiagnosticError("invalid CDP response body")
        try:
            parsed = json.loads(body)
        except (UnicodeDecodeError, json.JSONDecodeError) as exc:
            raise DiagnosticError("invalid CDP JSON") from exc
        return status, parsed
    except (OSError, TimeoutError) as exc:
        raise DiagnosticError("CDP transport unavailable") from exc
    finally:
        sock.close()


class WebSocket:
    """Minimal RFC 6455 client for the one fixed CDP target connection."""

    def __init__(self, sock: socket.socket):
        self.sock = sock
        self.next_message_id = 1

    @classmethod
    def connect(cls, socket_path: str, ws_path: str, timeout: float) -> "WebSocket":
        if not ws_path.startswith("/") or len(ws_path) > 512 or "\r" in ws_path or "\n" in ws_path:
            raise DiagnosticError("invalid CDP target")
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.settimeout(timeout)
        try:
            sock.connect(socket_path)
            key = base64.b64encode(hashlib.sha256(str(time.monotonic_ns()).encode()).digest()[:16]).decode()
            sock.sendall(
                (
                    f"GET {ws_path} HTTP/1.1\r\n"
                    "Host: localhost\r\n"
                    "Upgrade: websocket\r\n"
                    "Connection: Upgrade\r\n"
                    f"Sec-WebSocket-Key: {key}\r\n"
                    "Sec-WebSocket-Version: 13\r\n"
                    "\r\n"
                ).encode("ascii")
            )
            response = bytearray()
            while b"\r\n\r\n" not in response:
                if len(response) > 16384:
                    raise DiagnosticError("CDP handshake too large")
                part = sock.recv(4096)
                if not part:
                    raise DiagnosticError("transport closed")
                response.extend(part)
            first_line = bytes(response).split(b"\r\n", 1)[0]
            if not first_line.startswith(b"HTTP/1.1 101 "):
                raise DiagnosticError("CDP websocket unavailable")
            return cls(sock)
        except (OSError, TimeoutError) as exc:
            sock.close()
            raise DiagnosticError("CDP transport unavailable") from exc

    def _recv_frame(self) -> tuple[int, bytes]:
        first, second = _read_exact(self.sock, 2)
        fin = first & 0x80
        opcode = first & 0x0F
        length = second & 0x7F
        masked = second & 0x80
        if not fin or opcode not in (0, 1, 2, 8, 9, 10):
            raise DiagnosticError("unsupported CDP websocket frame")
        if length == 126:
            length = int.from_bytes(_read_exact(self.sock, 2), "big")
        elif length == 127:
            length = int.from_bytes(_read_exact(self.sock, 8), "big")
        if length > MAX_JSON_BYTES:
            raise DiagnosticError("CDP websocket frame too large")
        mask = _read_exact(self.sock, 4) if masked else b""
        payload = bytearray(_read_exact(self.sock, length))
        if masked:
            for index in range(length):
                payload[index] ^= mask[index % 4]
        return opcode, bytes(payload)

    def send(self, payload: dict[str, Any]) -> None:
        data = json.dumps(payload, separators=(",", ":")).encode("utf-8")
        length = len(data)
        if length > MAX_JSON_BYTES:
            raise DiagnosticError("CDP command too large")
        key = hashlib.sha256(str(time.monotonic_ns()).encode()).digest()[:4]
        masked = bytes(data[index] ^ key[index % 4] for index in range(length))
        if length < 126:
            header = bytes((0x81, 0x80 | length))
        elif length < 65536:
            header = bytes((0x81, 0x80 | 126)) + length.to_bytes(2, "big")
        else:
            header = bytes((0x81, 0x80 | 127)) + length.to_bytes(8, "big")
        self.sock.sendall(header + key + masked)

    def command(self, method: str, params: dict[str, Any] | None = None, timeout: float = 5.0) -> list[dict[str, Any]]:
        message_id = self.next_message_id
        self.next_message_id += 1
        self.send({"id": message_id, "method": method, "params": params or {}})
        events: list[dict[str, Any]] = []
        old_timeout = self.sock.gettimeout()
        self.sock.settimeout(timeout)
        try:
            while True:
                opcode, payload = self._recv_frame()
                if opcode == 9:
                    self._send_control(10, payload)
                    continue
                if opcode == 8:
                    raise DiagnosticError("CDP websocket closed")
                if opcode != 1:
                    continue
                try:
                    message = json.loads(payload)
                except (UnicodeDecodeError, json.JSONDecodeError) as exc:
                    raise DiagnosticError("invalid CDP event") from exc
                if not isinstance(message, dict):
                    continue
                if message.get("id") == message_id:
                    if "error" in message:
                        raise DiagnosticError("CDP command failed")
                    return events
                if "method" in message:
                    events.append(message)
        except (OSError, TimeoutError) as exc:
            raise DiagnosticError("CDP command timed out") from exc
        finally:
            self.sock.settimeout(old_timeout)

    def drain(self, timeout: float, on_event: Any) -> None:
        old_timeout = self.sock.gettimeout()
        self.sock.settimeout(max(0.01, timeout))
        deadline = time.monotonic() + timeout
        try:
            while time.monotonic() < deadline:
                try:
                    opcode, payload = self._recv_frame()
                except socket.timeout:
                    break
                if opcode == 9:
                    self._send_control(10, payload)
                    continue
                if opcode == 8:
                    break
                if opcode != 1:
                    continue
                try:
                    message = json.loads(payload)
                except (UnicodeDecodeError, json.JSONDecodeError):
                    continue
                if isinstance(message, dict) and "method" in message:
                    on_event(message)
        except OSError:
            return
        finally:
            self.sock.settimeout(old_timeout)

    def _send_control(self, opcode: int, payload: bytes) -> None:
        length = len(payload)
        key = hashlib.sha256(str(time.monotonic_ns()).encode()).digest()[:4]
        masked = bytes(payload[index] ^ key[index % 4] for index in range(length))
        self.sock.sendall(bytes((0x80 | opcode, 0x80 | length)) + key + masked)

    def close(self) -> None:
        try:
            self.sock.close()
        except OSError:
            pass


@dataclass
class NavigationCapture:
    active: bool = False
    status_class: str = "unknown"
    protocol: str = "unknown"
    remote_ip_family: str = "unknown"
    connection_reused: bool | None = None
    from_disk_cache: bool | None = None
    from_service_worker: bool | None = None
    tls_protocol: str = "unknown"
    start_time: float | None = None
    end_time: float | None = None
    redirect_status_classes: list[str] = field(default_factory=list)
    requests: dict[str, float] = field(default_factory=dict)

    def reset(self) -> None:
        self.__dict__.update(NavigationCapture().__dict__)

    def event(self, message: dict[str, Any]) -> None:
        if not self.active:
            return
        method = message.get("method")
        params = message.get("params")
        if not isinstance(params, dict):
            return
        if method == "Network.requestWillBeSent" and params.get("type") == "Document":
            request_id = params.get("requestId")
            timestamp = params.get("timestamp")
            if isinstance(request_id, str) and isinstance(timestamp, (int, float)):
                if len(self.requests) < MAX_TARGETS:
                    self.requests[request_id] = float(timestamp)
                if self.start_time is None:
                    self.start_time = float(timestamp)
            redirect = params.get("redirectResponse")
            if isinstance(redirect, dict) and len(self.redirect_status_classes) < MAX_REDIRECTS:
                self.redirect_status_classes.append(_status_class(redirect.get("status")))
        elif method == "Network.responseReceived" and params.get("type") == "Document":
            response = params.get("response")
            if not isinstance(response, dict):
                return
            self.status_class = _status_class(response.get("status"))
            self.protocol = _protocol(response.get("protocol"))
            self.remote_ip_family = _ip_family(response.get("remoteIPAddress"))
            reused = response.get("connectionReused")
            self.connection_reused = reused if isinstance(reused, bool) else None
            disk = response.get("fromDiskCache")
            self.from_disk_cache = disk if isinstance(disk, bool) else None
            worker = response.get("fromServiceWorker")
            self.from_service_worker = worker if isinstance(worker, bool) else None
            details = response.get("securityDetails")
            if isinstance(details, dict):
                self.tls_protocol = _tls_protocol(details.get("protocol"))
        elif method == "Network.loadingFinished":
            request_id = params.get("requestId")
            timestamp = params.get("timestamp")
            if isinstance(request_id, str) and isinstance(timestamp, (int, float)):
                if request_id in self.requests:
                    self.end_time = float(timestamp)

    def output(self, browser_family: str, browser_version: str) -> dict[str, Any]:
        return {
            "capture_active": self.active,
            "navigation": {
                "status_class": self.status_class if self.status_class in STATUS_CLASSES else "unknown",
                "redirect_count": min(len(self.redirect_status_classes), MAX_REDIRECTS),
                "redirect_status_classes": [
                    item if item in STATUS_CLASSES else "unknown" for item in self.redirect_status_classes[:MAX_REDIRECTS]
                ],
                "protocol": self.protocol if self.protocol in PROTOCOLS else "unknown",
                "remote_ip_family": self.remote_ip_family if self.remote_ip_family in ("ipv4", "ipv6", "unknown") else "unknown",
                "connection_reused": self.connection_reused,
                "from_disk_cache": self.from_disk_cache,
                "from_service_worker": self.from_service_worker,
                "tls_protocol": self.tls_protocol if self.tls_protocol in TLS_PROTOCOLS else "unknown",
                "duration_bucket": _duration_bucket(self.start_time, self.end_time),
            },
            "browser": {
                "product_family": browser_family,
                "version": browser_version,
            },
        }


class BrowserDiag:
    def __init__(
        self,
        socket_path: str,
        allowed_hosts: set[str],
        browser_product_family: str = "ChromeBeta",
        timeout: float = 5.0,
    ):
        if not socket_path or not socket_path.startswith("/") or len(socket_path) > 255:
            raise ValueError("socket_path must be a bounded filesystem AF_UNIX path")
        if not allowed_hosts or len(allowed_hosts) > 32:
            raise ValueError("at least one configured host is required")
        normalized = set()
        for host in allowed_hosts:
            host = host.strip().lower().rstrip(".")
            if not host or len(host) > MAX_HOST_LENGTH or ":" in host or "/" in host:
                raise ValueError("invalid configured host")
            normalized.add(host)
        if not SAFE_ID.fullmatch(browser_product_family):
            raise ValueError("invalid browser product family")
        self.socket_path = socket_path
        self.allowed_hosts = frozenset(normalized)
        self.browser_product_family = browser_product_family
        self.timeout = min(max(timeout, 0.2), 30.0)
        self.capture = NavigationCapture()
        self.websocket: WebSocket | None = None
        self.websocket_handle: str | None = None
        self.target_ids: dict[str, str] = {}
        self.browser_version = "unknown"
        self.closed = False

    def _version(self) -> dict[str, Any]:
        status, payload = _http_json(self.socket_path, "/json/version", self.timeout)
        if status < 200 or status >= 300:
            raise DiagnosticError("CDP version health failed")
        browser = payload.get("Browser")
        self.browser_version = _safe_browser_version(browser)
        return payload

    def _targets(self) -> list[dict[str, Any]]:
        status, payload = _http_json(self.socket_path, "/json/list", self.timeout)
        if status < 200 or status >= 300 or not isinstance(payload, list):
            raise DiagnosticError("CDP target health failed")
        return [item for item in payload[:MAX_TARGETS] if isinstance(item, dict)]

    def health(self, _: dict[str, Any]) -> dict[str, Any]:
        version = self._version()
        targets = self._targets()
        protocol_present = isinstance(version.get("Protocol-Version"), str) and bool(version["Protocol-Version"])
        return {
            "status": "ok",
            "version_health": "PASS",
            "target_health": "PASS",
            "target_count": min(len(targets), MAX_TARGETS),
            "protocol_version_present": protocol_present,
            "browser_product_family": self.browser_product_family,
            "browser_version": self.browser_version,
        }

    def list_targets(self, _: dict[str, Any]) -> dict[str, Any]:
        targets = self._targets()
        self.target_ids = {}
        result = []
        for index, target in enumerate(targets, start=1):
            actual = target.get("id")
            target_type = target.get("type")
            ws_url = target.get("webSocketDebuggerUrl")
            if not isinstance(actual, str) or not SAFE_ID.fullmatch(actual):
                continue
            if not isinstance(target_type, str) or target_type not in ("page", "webview", "background_page"):
                continue
            if not isinstance(ws_url, str):
                continue
            parsed = urllib.parse.urlsplit(ws_url)
            if parsed.scheme not in ("ws", "wss") or not parsed.path.startswith("/"):
                continue
            handle = f"target-{index}"
            self.target_ids[handle] = actual
            result.append({"target_id": handle, "type": target_type})
        return {"targets": result[:MAX_TARGETS], "target_count": len(result[:MAX_TARGETS])}

    def _connect_target(self, handle: str) -> None:
        if not SAFE_ID.fullmatch(handle) or handle not in self.target_ids:
            raise DiagnosticError("unknown target")
        actual = self.target_ids[handle]
        target = next((item for item in self._targets() if item.get("id") == actual), None)
        if not isinstance(target, dict):
            raise DiagnosticError("target unavailable")
        ws_url = target.get("webSocketDebuggerUrl")
        if not isinstance(ws_url, str):
            raise DiagnosticError("target unavailable")
        parsed = urllib.parse.urlsplit(ws_url)
        if parsed.scheme not in ("ws", "wss") or not parsed.path.startswith("/"):
            raise DiagnosticError("target unavailable")
        if self.websocket is not None:
            self.websocket.close()
        self.websocket = WebSocket.connect(self.socket_path, parsed.path, self.timeout)
        self.websocket_handle = handle

    def _selected_handle(self, arguments: dict[str, Any]) -> str:
        handle = arguments.get("target_id", "target-1")
        if not isinstance(handle, str) or not SAFE_ID.fullmatch(handle):
            raise DiagnosticError("invalid target")
        if handle not in self.target_ids:
            self.list_targets({})
        if handle not in self.target_ids:
            raise DiagnosticError("unknown target")
        return handle

    def _send(self, method: str, params: dict[str, Any] | None = None) -> None:
        if self.websocket is None:
            self.list_targets({})
            self._connect_target("target-1")
        assert self.websocket is not None
        events = self.websocket.command(method, params, self.timeout)
        for event in events:
            self.capture.event(event)

    def _drain(self, seconds: float = 0.25) -> None:
        if self.websocket is not None:
            self.websocket.drain(seconds, self.capture.event)

    def _validated_url(self, value: Any) -> str:
        if not isinstance(value, str) or len(value) > MAX_URL_LENGTH:
            raise DiagnosticError("HTTPS URL required")
        parsed = urllib.parse.urlsplit(value)
        host = (parsed.hostname or "").lower().rstrip(".")
        if parsed.scheme != "https" or not host or parsed.username or parsed.password or parsed.query or parsed.fragment:
            raise DiagnosticError("HTTPS URL required")
        if parsed.port not in (None, 443):
            raise DiagnosticError("HTTPS URL required")
        if host not in self.allowed_hosts:
            raise DiagnosticError("URL host is not allowlisted")
        return value

    def open_url(self, arguments: dict[str, Any]) -> dict[str, Any]:
        url = self._validated_url(arguments.get("url"))
        handle = self._selected_handle(arguments)
        if self.websocket is None or self.websocket_handle != handle:
            if self.capture.active:
                raise DiagnosticError("capture target mismatch")
            self._connect_target(handle)
        self._send("Page.navigate", {"url": url})
        self._drain()
        return {"navigated": True, "target_id": handle}

    def reload(self, arguments: dict[str, Any]) -> dict[str, Any]:
        handle = self._selected_handle(arguments)
        if self.websocket is None or self.websocket_handle != handle:
            if self.capture.active:
                raise DiagnosticError("capture target mismatch")
            self._connect_target(handle)
        self._send("Page.reload", {"ignoreCache": False})
        self._drain()
        return {"reloaded": True, "target_id": handle}

    def network_capture_start(self, arguments: dict[str, Any]) -> dict[str, Any]:
        if self.capture.active:
            raise DiagnosticError("capture already active")
        handle = self._selected_handle(arguments)
        self.capture.reset()
        last_error: DiagnosticError | None = None
        for attempt in range(2):
            try:
                if self.websocket is None or self.websocket_handle != handle:
                    self._connect_target(handle)
                self._send("Page.enable")
                self._send("Network.enable")
                last_error = None
                break
            except DiagnosticError as exc:
                last_error = exc
                if self.websocket is not None:
                    self.websocket.close()
                    self.websocket = None
                    self.websocket_handle = None
                if attempt == 0:
                    time.sleep(0.2)
        if last_error is not None:
            self.capture.reset()
            raise DiagnosticError("CDP capture setup unavailable") from last_error
        self.capture.active = True
        self._drain(0.1)
        return {"capture_started": True, "target_id": handle}

    def network_summary(self, _: dict[str, Any]) -> dict[str, Any]:
        if not self.capture.active:
            raise DiagnosticError("capture is not active")
        self._drain(0.1)
        return self.capture.output(self.browser_product_family, self.browser_version)

    def network_capture_stop(self, _: dict[str, Any]) -> dict[str, Any]:
        if not self.capture.active:
            raise DiagnosticError("capture is not active")
        try:
            self._drain(0.1)
            if self.websocket is not None:
                self._send("Network.disable")
                self._send("Page.disable")
        finally:
            self.capture.active = False
        return {"capture_stopped": True}

    def call(self, name: str, arguments: dict[str, Any] | None = None) -> dict[str, Any]:
        if name not in ALLOWED_TOOLS:
            raise DiagnosticError("tool is not allowlisted")
        if arguments is not None and not isinstance(arguments, dict):
            raise DiagnosticError("arguments must be an object")
        arguments = arguments or {}
        return {
            "health": self.health,
            "list_targets": self.list_targets,
            "open_url": self.open_url,
            "reload": self.reload,
            "network_capture_start": self.network_capture_start,
            "network_summary": self.network_summary,
            "network_capture_stop": self.network_capture_stop,
        }[name](arguments)

    def close(self) -> None:
        if self.closed:
            return
        self.closed = True
        if self.capture.active and self.websocket is not None:
            try:
                self._send("Network.disable")
                self._send("Page.disable")
            except DiagnosticError:
                pass
        if self.websocket is not None:
            self.websocket.close()
            self.websocket = None
            self.websocket_handle = None


def _tool_descriptions() -> list[dict[str, Any]]:
    return [
        {"name": name, "description": "Bounded dedicated-browser diagnostic operation", "inputSchema": {"type": "object"}}
        for name in ALLOWED_TOOLS
    ]


def serve(mcp: BrowserDiag, input_stream: BinaryIO = sys.stdin.buffer, output_stream: BinaryIO = sys.stdout.buffer) -> None:
    try:
        for line in input_stream:
            if not line.strip():
                continue
            request: Any = None
            try:
                request = json.loads(line)
                if not isinstance(request, dict):
                    raise DiagnosticError("request must be an object")
                method = request.get("method")
                request_id = request.get("id")
                if method == "notifications/initialized":
                    continue
                if method == "initialize":
                    result: Any = {
                        "protocolVersion": "2025-06-18",
                        "capabilities": {"tools": {}},
                        "serverInfo": {"name": "browser-diag-mcp", "version": "1"},
                    }
                elif method == "tools/list":
                    result = {"tools": _tool_descriptions()}
                elif method == "tools/call":
                    params = request.get("params")
                    if not isinstance(params, dict):
                        raise DiagnosticError("missing tool parameters")
                    result = {"content": [{"type": "text", "text": json.dumps(mcp.call(params.get("name"), params.get("arguments")), separators=(",", ":"))}]}
                else:
                    raise DiagnosticError("method is not allowlisted")
                response = {"jsonrpc": "2.0", "id": request_id, "result": result}
            except DiagnosticError as exc:
                response = {"jsonrpc": "2.0", "id": request.get("id") if isinstance(request, dict) else None, "error": {"code": -32000, "message": str(exc)}}
            except (TypeError, ValueError):
                response = {"jsonrpc": "2.0", "id": request.get("id") if isinstance(request, dict) else None, "error": {"code": -32600, "message": "invalid request"}}
            output_stream.write((json.dumps(response, separators=(",", ":")) + "\n").encode("utf-8"))
            output_stream.flush()
    finally:
        mcp.close()


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="bounded browser diagnostic MCP")
    parser.add_argument("--socket", required=True, help="configured filesystem AF_UNIX relay socket")
    parser.add_argument("--allowed-host", action="append", required=True, dest="allowed_hosts")
    parser.add_argument("--browser-product-family", default="ChromeBeta")
    args = parser.parse_args(argv)
    try:
        mcp = BrowserDiag(args.socket, set(args.allowed_hosts), args.browser_product_family)
    except ValueError as exc:
        parser.error(str(exc))
    try:
        serve(mcp)
    except BrokenPipeError:
        return 0
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
