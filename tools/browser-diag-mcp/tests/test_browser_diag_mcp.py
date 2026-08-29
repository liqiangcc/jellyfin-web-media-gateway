import importlib.util
import io
import json
import pathlib
import socket
import sys
import unittest
from unittest import mock


MODULE_PATH = pathlib.Path(__file__).parents[1] / "browser_diag_mcp.py"
SPEC = importlib.util.spec_from_file_location("browser_diag_mcp", MODULE_PATH)
assert SPEC and SPEC.loader
module = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = module
SPEC.loader.exec_module(module)


class CaptureTests(unittest.TestCase):
    def test_network_output_is_bounded_and_redacted(self):
        capture = module.NavigationCapture(active=True)
        capture.event(
            {
                "method": "Network.requestWillBeSent",
                "params": {
                    "requestId": "1",
                    "type": "Document",
                    "timestamp": 10.0,
                    "request": {
                        "url": "https://neutral.example/?cookie=SECRET",
                        "headers": {"Cookie": "secret", "Authorization": "Bearer secret"},
                    },
                    "redirectResponse": {"status": 302, "headers": {"Set-Cookie": "secret"}},
                },
            }
        )
        capture.event(
            {
                "method": "Network.responseReceived",
                "params": {
                    "type": "Document",
                    "response": {
                        "status": 200,
                        "protocol": "h2",
                        "remoteIPAddress": "2001:db8::1",
                        "connectionReused": True,
                        "fromDiskCache": False,
                        "fromServiceWorker": False,
                        "securityDetails": {"protocol": "TLS 1.3", "certificateId": "secret"},
                    },
                },
            }
        )
        capture.event({"method": "Network.loadingFinished", "params": {"requestId": "1", "timestamp": 10.2}})
        output = json.dumps(capture.output("ChromeBeta", "123.0"), sort_keys=True)
        self.assertNotIn("neutral.example", output)
        self.assertNotIn("secret", output.lower())
        self.assertEqual(capture.output("ChromeBeta", "123.0")["navigation"], {
            "status_class": "2xx",
            "redirect_count": 1,
            "redirect_status_classes": ["3xx"],
            "protocol": "h2",
            "remote_ip_family": "ipv6",
            "connection_reused": True,
            "from_disk_cache": False,
            "from_service_worker": False,
            "tls_protocol": "TLS 1.3",
            "duration_bucket": "100-499ms",
        })

    def test_redirect_and_target_bounds(self):
        capture = module.NavigationCapture(active=True)
        for index in range(40):
            capture.event({
                "method": "Network.requestWillBeSent",
                "params": {
                    "requestId": str(index), "type": "Document", "timestamp": float(index),
                    "redirectResponse": {"status": 301},
                },
            })
        output = capture.output("ChromeBeta", "unknown")
        self.assertEqual(output["navigation"]["redirect_count"], module.MAX_REDIRECTS)
        self.assertLessEqual(len(output["navigation"]["redirect_status_classes"]), module.MAX_REDIRECTS)


class AllowlistAndLifecycleTests(unittest.TestCase):
    def make_mcp(self):
        return module.BrowserDiag("/tmp/browser-diag-test.sock", {"example.com"})

    def test_only_task_allowlist_is_exposed(self):
        mcp = self.make_mcp()
        self.assertEqual(module.ALLOWED_TOOLS, (
            "health", "list_targets", "open_url", "reload",
            "network_capture_start", "network_summary", "network_capture_stop",
        ))
        with self.assertRaises(module.DiagnosticError):
            mcp.call("send_cdp_command", {"method": "Runtime.evaluate"})

    def test_https_exact_host_allowlist_and_no_query_or_userinfo(self):
        mcp = self.make_mcp()
        self.assertEqual(mcp._validated_url("https://example.com/"), "https://example.com/")
        for value in (
            "http://example.com/", "https://other.example/", "https://example.com/?x=1",
            "https://user:pass@example.com/", "https://127.0.0.1/", "https://example.com:8443/",
        ):
            with self.subTest(value=value), self.assertRaises(module.DiagnosticError):
                mcp._validated_url(value)

    def test_capture_lifecycle_and_cleanup_are_deterministic(self):
        mcp = self.make_mcp()
        mcp.target_ids = {"target-1": "page-id"}
        fake_ws = mock.Mock()
        mcp.websocket = fake_ws
        mcp.websocket_handle = "target-1"
        with mock.patch.object(mcp, "_send"), mock.patch.object(mcp, "_drain"):
            self.assertEqual(mcp.network_capture_start({"target_id": "target-1"})["capture_started"], True)
            with self.assertRaises(module.DiagnosticError):
                mcp.network_capture_start({"target_id": "target-1"})
            self.assertTrue(mcp.network_summary({})["capture_active"])
            self.assertEqual(mcp.network_capture_stop({})["capture_stopped"], True)
            with self.assertRaises(module.DiagnosticError):
                mcp.network_summary({})
        mcp.close()
        mcp.close()
        fake_ws.close.assert_called_once_with()


    def test_capture_start_failure_resets_state(self):
        mcp = self.make_mcp()
        mcp.target_ids = {"target-1": "page-id"}
        fake_ws = mock.Mock()
        mcp.websocket = fake_ws
        mcp.websocket_handle = "target-1"
        with mock.patch.object(mcp, "_send", side_effect=module.DiagnosticError("not ready")):
            with self.assertRaises(module.DiagnosticError):
                mcp.network_capture_start({"target_id": "target-1"})
        self.assertFalse(mcp.capture.active)
        self.assertIsNone(mcp.websocket)
        fake_ws.close.assert_called_once_with()


    def test_open_url_reuses_active_capture_target(self):
        mcp = self.make_mcp()
        mcp.target_ids = {"target-1": "page-id"}
        fake_ws = mock.Mock()
        mcp.websocket = fake_ws
        mcp.websocket_handle = "target-1"
        mcp.capture.active = True
        with mock.patch.object(mcp, "_connect_target") as reconnect, mock.patch.object(mcp, "_send") as send, mock.patch.object(mcp, "_drain"):
            result = mcp.open_url({"target_id": "target-1", "url": "https://example.com/"})
        self.assertTrue(result["navigated"])
        reconnect.assert_not_called()
        send.assert_called_once_with("Page.navigate", {"url": "https://example.com/"})

    def test_unix_socket_is_the_only_transport(self):
        mcp = self.make_mcp()
        with mock.patch("socket.socket") as socket_factory:
            sock = socket_factory.return_value
            sock.recv.side_effect = [b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}"]
            status, payload = module._http_json(mcp.socket_path, "/json/version", 0.2)
            self.assertEqual((status, payload), (200, {}))
            self.assertEqual(socket_factory.call_args.args[0], socket.AF_UNIX)


class McpStdioTests(unittest.TestCase):
    class FakeDiag:
        def __init__(self):
            self.closed = False

        def call(self, name, arguments):
            if name not in module.ALLOWED_TOOLS:
                raise module.DiagnosticError("tool is not allowlisted")
            if name == "health":
                return {"status": "ok"}
            return {"ok": True}

        def close(self):
            self.closed = True

    def test_stdio_protocol_exact_allowlist_and_notification_silence(self):
        fake = self.FakeDiag()
        messages = [
            {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}},
            {"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}},
            {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
            {"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "health", "arguments": {}}},
            {"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "send_cdp_command", "arguments": {}}},
        ]
        source = io.BytesIO(b"".join((json.dumps(item)+"\n").encode() for item in messages))
        sink = io.BytesIO()
        module.serve(fake, source, sink)
        replies = [json.loads(line) for line in sink.getvalue().splitlines()]
        self.assertEqual([item["id"] for item in replies], [1, 2, 3, 4])
        self.assertEqual(replies[0]["result"]["protocolVersion"], "2025-06-18")
        names = tuple(tool["name"] for tool in replies[1]["result"]["tools"])
        self.assertEqual(names, module.ALLOWED_TOOLS)
        safe = json.loads(replies[2]["result"]["content"][0]["text"])
        self.assertEqual(safe, {"status": "ok"})
        self.assertEqual(replies[3]["error"]["code"], -32000)
        self.assertTrue(fake.closed)


if __name__ == "__main__":
    unittest.main()
