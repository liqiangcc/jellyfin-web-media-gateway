import base64
import hashlib
import importlib.util
import json
import os
import socket
import struct
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).resolve().parents[1] / "browser_diag_mcp.py"
SPEC = importlib.util.spec_from_file_location("browser_diag_mcp", MODULE_PATH)
assert SPEC and SPEC.loader
diag = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = diag
SPEC.loader.exec_module(diag)


def recv_until(client: socket.socket, marker: bytes, limit: int = 65536) -> bytes:
    data = bytearray()
    while marker not in data:
        chunk = client.recv(4096)
        if not chunk:
            raise EOFError
        data.extend(chunk)
        if len(data) > limit:
            raise ValueError("fixture request too large")
    return bytes(data)


def read_exact(client: socket.socket, length: int) -> bytes:
    data = bytearray()
    while len(data) < length:
        chunk = client.recv(length - len(data))
        if not chunk:
            raise EOFError
        data.extend(chunk)
    return bytes(data)


def read_client_frame(client: socket.socket) -> dict:
    header = read_exact(client, 2)
    length = header[1] & 0x7F
    if length == 126:
        length = struct.unpack("!H", read_exact(client, 2))[0]
    elif length == 127:
        length = struct.unpack("!Q", read_exact(client, 8))[0]
    mask = read_exact(client, 4)
    payload = read_exact(client, length)
    payload = bytes(value ^ mask[index % 4] for index, value in enumerate(payload))
    return json.loads(payload)


def send_server_frame(client: socket.socket, value: dict) -> None:
    payload = json.dumps(value, separators=(",", ":")).encode()
    header = bytearray([0x81])
    if len(payload) < 126:
        header.append(len(payload))
    elif len(payload) <= 65535:
        header.append(126)
        header.extend(struct.pack("!H", len(payload)))
    else:
        header.append(127)
        header.extend(struct.pack("!Q", len(payload)))
    client.sendall(header + payload)


class FakeCdpServer:
    def __init__(self, path: Path) -> None:
        self.path = path
        self.listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.listener.bind(str(path))
        self.listener.listen()
        self.stop = threading.Event()
        self.closed_websockets = 0
        self.commands: list[str] = []
        self.threads: list[threading.Thread] = []
        self.thread = threading.Thread(target=self._accept, daemon=True)
        self.thread.start()

    def _accept(self) -> None:
        while not self.stop.is_set():
            try:
                client, _ = self.listener.accept()
            except OSError:
                return
            thread = threading.Thread(target=self._handle, args=(client,), daemon=True)
            self.threads.append(thread)
            thread.start()

    def _handle(self, client: socket.socket) -> None:
        with client:
            try:
                request = recv_until(client, b"\r\n\r\n")
            except (EOFError, OSError):
                return
            first = request.split(b"\r\n", 1)[0]
            if first == b"GET /json/version HTTP/1.1":
                self._json(client, {"Browser": "Chrome/153.0.8010.18 extra-fingerprint", "Protocol-Version": "1.3", "webSocketDebuggerUrl": "secret"})
                return
            if first == b"GET /json/list HTTP/1.1":
                self._json(
                    client,
                    [
                        {
                            "id": "fixture-page",
                            "type": "page",
                            "title": "PERSONAL TITLE MUST NOT ESCAPE",
                            "url": "https://private.invalid/history?token=secret",
                            "webSocketDebuggerUrl": "ws://localhost/devtools/page/fixture-page",
                        },
                        {
                            "id": "x" * 129,
                            "type": "page",
                            "webSocketDebuggerUrl": "ws://localhost/devtools/page/oversized",
                        },
                        {"id": "worker", "type": "service_worker", "title": "private"},
                    ],
                )
                return
            if first.startswith(b"GET /devtools/page/fixture-page HTTP/1.1"):
                key = None
                for line in request.split(b"\r\n"):
                    if line.lower().startswith(b"sec-websocket-key:"):
                        key = line.split(b":", 1)[1].strip()
                assert key
                accept = base64.b64encode(hashlib.sha1(key + b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11").digest())
                client.sendall(b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: " + accept + b"\r\n\r\n")
                self._websocket(client)

    @staticmethod
    def _json(client: socket.socket, value: object) -> None:
        body = json.dumps(value).encode()
        client.sendall(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: " + str(len(body)).encode() + b"\r\nConnection: close\r\n\r\n" + body)

    def _websocket(self, client: socket.socket) -> None:
        try:
            while True:
                command = read_client_frame(client)
                method = command["method"]
                self.commands.append(method)
                if method == "Page.navigate":
                    events = [
                        {
                            "method": "Network.requestWillBeSent",
                            "params": {
                                "requestId": "nav-1",
                                "type": "Document",
                                "timestamp": 10.0,
                                "request": {"url": "https://example.com/?token=secret", "headers": {"Cookie": "secret"}},
                                "redirectResponse": {"status": 302, "headers": {"Set-Cookie": "secret"}},
                            },
                        },
                        {
                            "method": "Network.responseReceived",
                            "params": {
                                "requestId": "nav-1",
                                "type": "Document",
                                "response": {
                                    "status": 204,
                                    "protocol": "h2",
                                    "remoteIPAddress": "203.0.113.8",
                                    "connectionReused": True,
                                    "fromDiskCache": False,
                                    "fromServiceWorker": False,
                                    "securityDetails": {"protocol": "TLS 1.3", "certificateId": 123},
                                    "headers": {"Authorization": "Bearer secret", "Set-Cookie": "secret"},
                                    "url": "https://example.com/signed?token=secret",
                                },
                            },
                        },
                        {"method": "Network.loadingFinished", "params": {"requestId": "nav-1", "timestamp": 10.7, "encodedDataLength": 999999}},
                    ]
                    for event in events:
                        send_server_frame(client, event)
                send_server_frame(client, {"id": command["id"], "result": {}})
        except (EOFError, OSError):
            self.closed_websockets += 1

    def close(self) -> None:
        self.stop.set()
        self.listener.close()
        try:
            wake = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            wake.connect(str(self.path))
            wake.close()
        except OSError:
            pass
        self.thread.join(timeout=1)
        for thread in self.threads:
            thread.join(timeout=1)


class BrowserDiagTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.socket_path = Path(self.temporary.name) / "cdp.sock"
        self.fake = FakeCdpServer(self.socket_path)
        self.service = diag.BrowserDiagService(
            diag.UnixSocketTransport(self.socket_path), frozenset({"example.com"}), "ChromeBeta"
        )

    def tearDown(self) -> None:
        self.service.close()
        self.fake.close()
        self.temporary.cleanup()

    def get_target(self) -> str:
        listed = self.service.call("list_targets", {})
        self.assertEqual(len(listed["targets"]), 1)
        return listed["targets"][0]["target_id"]

    def test_mcp_schema_and_fixed_tool_allowlist(self) -> None:
        server = diag.McpServer(self.service)
        response = server.handle({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}})
        tools = response["result"]["tools"]
        self.assertEqual(tuple(tool["name"] for tool in tools), diag.TOOL_NAMES)
        for tool in tools:
            self.assertFalse(tool["inputSchema"]["additionalProperties"])
            self.assertFalse(tool["outputSchema"]["additionalProperties"])
        rejected = server.handle(
            {"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "send_cdp_command", "arguments": {"method": "Runtime.evaluate"}}}
        )
        self.assertTrue(rejected["result"]["isError"])
        self.assertEqual(rejected["result"]["structuredContent"], {"error": "TOOL_NOT_ALLOWED"})

    def test_https_exact_host_allowlist_is_immutable(self) -> None:
        self.assertEqual(diag.parse_allowed_hosts(["Example.COM"]), frozenset({"example.com"}))
        self.assertEqual(diag.validate_navigation_url("https://example.com/path?q=not-returned", frozenset({"example.com"})), "https://example.com/path?q=not-returned")
        for value, code in (
            ("http://example.com/", "HTTPS_REQUIRED"),
            ("https://user:pass@example.com/", "HTTPS_REQUIRED"),
            ("https://sub.example.com/", "HOST_NOT_ALLOWED"),
            ("https://example.com:444/", "HOST_NOT_ALLOWED"),
            ("https://127.0.0.1/", "HOST_NOT_ALLOWED"),
        ):
            with self.subTest(value=value), self.assertRaises(diag.DiagError) as caught:
                diag.validate_navigation_url(value, frozenset({"example.com"}))
            self.assertEqual(caught.exception.code, code)
        self.assertNotIn("set_allowed_hosts", diag.TOOL_NAMES)

    def test_target_ids_are_opaque_bounded_and_personal_fields_are_absent(self) -> None:
        result = self.service.call("list_targets", {})
        encoded = json.dumps(result).lower()
        self.assertRegex(result["targets"][0]["target_id"], diag.TARGET_HANDLE_RE)
        self.assertLessEqual(len(result["targets"][0]["target_id"]), 32)
        for prohibited in ("fixture-page", "personal", "private.invalid", "title", "history", "url", "token"):
            self.assertNotIn(prohibited, encoded)
        with self.assertRaises(diag.DiagError):
            diag.validate_target_handle("target_" + "a" * 1000)

    def test_network_summary_has_only_allowed_bounded_fields_and_enums(self) -> None:
        self.assertEqual(self.service.call("health", {})["browser_version"], "153.0.8010.18")
        target = self.get_target()
        self.service.call("network_capture_start", {"target_id": target})
        self.service.call("open_url", {"target_id": target, "url": "https://example.com/neutral?q=signed-secret"})
        summary = self.service.call("network_summary", {"target_id": target})
        self.assertEqual(set(summary), diag.SUMMARY_FIELDS)
        self.assertEqual(summary["status_class"], "2xx")
        self.assertEqual(summary["redirect_status_classes"], ["3xx"])
        self.assertEqual(summary["protocol"], "h2")
        self.assertEqual(summary["remote_ip_family"], "ipv4")
        self.assertEqual(summary["tls_protocol"], "tls1.3")
        self.assertEqual(summary["duration_bucket"], "100ms-1s")
        diag.validate_summary(summary)

    def test_prohibited_keys_and_values_do_not_escape_reduction(self) -> None:
        target = self.get_target()
        self.service.call("network_capture_start", {"target_id": target})
        self.service.call("open_url", {"target_id": target, "url": "https://example.com/?token=secret"})
        outputs = [
            self.service.call("health", {}),
            self.service.call("list_targets", {}),
            self.service.call("network_summary", {"target_id": target}),
        ]
        encoded = json.dumps(outputs).lower()
        for prohibited in ("cookie", "authorization", "set-cookie", "bearer", "signed?", "token=", "headers", "body", "dom", "websocket", "private.invalid"):
            self.assertNotIn(prohibited, encoded)
        with self.assertRaises(diag.DiagError):
            diag.assert_safe_output({"response_headers": {}})
        with self.assertRaises(diag.DiagError):
            diag.assert_safe_output({"page_body": "secret"})
        with self.assertRaises(diag.DiagError):
            diag.assert_safe_output({"safe_key": "https://example.com/?token=secret"})

    def test_event_storage_is_bounded_and_stores_only_reduced_tuples(self) -> None:
        capture = diag.CaptureState(max_events=4)
        capture.start()
        for index in range(20):
            capture.accept(
                {
                    "method": "Network.requestWillBeSent",
                    "params": {
                        "requestId": f"nav-{index}",
                        "type": "Document",
                        "timestamp": index,
                        "request": {"url": "https://private.invalid/?token=secret", "headers": {"Cookie": "secret"}},
                    },
                }
            )
        self.assertEqual(len(capture.events), 4)
        self.assertTrue(all(isinstance(event, tuple) and len(event) == 2 for event in capture.events))
        self.assertNotIn("secret", repr(capture.events).lower())

    def test_capture_lifecycle_stop_clears_state_and_service_close_cleans_session(self) -> None:
        target = self.get_target()
        self.service.call("network_capture_start", {"target_id": target})
        self.service.call("open_url", {"target_id": target, "url": "https://example.com/"})
        self.service.call("reload", {"target_id": target})
        stopped = self.service.call("network_capture_stop", {"target_id": target})
        self.assertEqual(stopped, {"capture_status": "stopped"})
        session = self.service._sessions[target]
        self.assertFalse(session.capture.active)
        self.assertEqual(len(session.capture.events), 0)
        with self.assertRaises(diag.DiagError) as caught:
            self.service.call("network_summary", {"target_id": target})
        self.assertEqual(caught.exception.code, "CAPTURE_NOT_ACTIVE")
        self.assertIn("Page.reload", self.fake.commands)
        self.assertIn("Network.disable", self.fake.commands)
        self.service.close()
        deadline = time.monotonic() + 1
        while self.fake.closed_websockets == 0 and time.monotonic() < deadline:
            time.sleep(0.01)
        self.assertGreaterEqual(self.fake.closed_websockets, 1)


if __name__ == "__main__":
    unittest.main()
