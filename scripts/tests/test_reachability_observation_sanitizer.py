import ast
import importlib.util
import json
import pathlib
import subprocess
import sys
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
PATH = ROOT / "scripts" / "reachability_observation_sanitizer.py"
spec = importlib.util.spec_from_file_location("reachability_observation_sanitizer", PATH)
mod = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = mod
spec.loader.exec_module(mod)


def v4() -> str:
    return ".".join(str(x) for x in (192, 0, 2, 10))


def v4_other() -> str:
    return ".".join(str(x) for x in (198, 51, 100, 7))


def v6() -> str:
    return ":".join(("2001", "db8", "0", "0", "0", "0", "0", "1"))


class SanitizerTests(unittest.TestCase):
    KEY_A = b"a" * 32
    KEY_B = b"b" * 32

    def sanitizer(self, key=None):
        return mod.RunSanitizer(key or self.KEY_A)

    def test_ipv4_is_bounded_and_raw_value_is_absent(self):
        raw = v4()
        out = self.sanitizer().sanitize({"endpoint": raw, "status": 200, "http_version": "2", "timing_ms": 120}).as_dict()
        self.assertEqual(out["family"], "ipv4")
        self.assertEqual(out["status_class"], "2xx")
        self.assertEqual(out["http_version_class"], "h2")
        self.assertEqual(out["timing_bucket"], "100-249ms")
        self.assertRegex(out["endpoint_alias"], r"^ep-[0-9a-f]{12}$")
        self.assertNotIn(raw, json.dumps(out))

    def test_ipv6_is_bounded_and_raw_value_is_absent(self):
        raw = v6()
        out = self.sanitizer().sanitize({"endpoint": raw}).as_dict()
        self.assertEqual(out["family"], "ipv6")
        self.assertRegex(out["endpoint_alias"], r"^ep-[0-9a-f]{12}$")
        self.assertNotIn(raw, json.dumps(out))

    def test_same_endpoint_same_run_same_alias(self):
        s = self.sanitizer()
        self.assertEqual(s.sanitize({"endpoint": v4()}).endpoint_alias, s.sanitize({"endpoint": v4()}).endpoint_alias)

    def test_different_endpoints_same_run_different_alias(self):
        s = self.sanitizer()
        self.assertNotEqual(s.sanitize({"endpoint": v4()}).endpoint_alias, s.sanitize({"endpoint": v4_other()}).endpoint_alias)

    def test_same_endpoint_new_run_context_changes_alias(self):
        a = self.sanitizer(self.KEY_A).sanitize({"endpoint": v4()}).endpoint_alias
        b = self.sanitizer(self.KEY_B).sanitize({"endpoint": v4()}).endpoint_alias
        self.assertNotEqual(a, b)

    def test_malformed_endpoint_fails_closed_without_reflection(self):
        sentinel = "ENDPOINT_SENTINEL_NOT_AN_ADDRESS"
        out = self.sanitizer().sanitize({"endpoint": sentinel}).as_dict()
        self.assertEqual(out["family"], "unknown")
        self.assertEqual(out["endpoint_alias"], "unknown")
        self.assertNotIn(sentinel, json.dumps(out))

    def test_extra_sensitive_fields_are_ignored_without_reflection(self):
        sentinel = "SENSITIVE_SENTINEL_VALUE"
        out = self.sanitizer().sanitize({"endpoint": v4(), "headers": sentinel, "body": sentinel, "cookie": sentinel, "authorization": sentinel}).as_dict()
        self.assertNotIn(sentinel, json.dumps(out))
        self.assertEqual(set(out), {"family", "endpoint_alias", "http_version_class", "status_class", "timing_bucket"})

    def test_status_classes_are_bounded(self):
        s = self.sanitizer()
        expected = [(200, "2xx"), (302, "3xx"), (404, "4xx"), (503, "5xx"), ("network-error", "network-error"), (99, "unknown"), ("oops", "unknown")]
        for raw, want in expected:
            with self.subTest(raw=raw):
                self.assertEqual(s.sanitize({"status": raw}).status_class, want)

    def test_http_version_classes_are_bounded(self):
        s = self.sanitizer()
        for raw, want in [("1.1", "h1"), ("2", "h2"), ("HTTP/3", "h3"), ("x", "unknown")]:
            with self.subTest(raw=raw):
                self.assertEqual(s.sanitize({"http_version": raw}).http_version_class, want)

    def test_timing_is_bucketed_not_exact(self):
        s = self.sanitizer()
        cases = [(50, "lt100ms"), (100, "100-249ms"), (300, "250-499ms"), (700, "500-999ms"), (1200, "1-2s"), (2500, "ge2s"), (-1, "unknown")]
        for raw, want in cases:
            with self.subTest(raw=raw):
                self.assertEqual(s.sanitize({"timing_ms": raw}).timing_bucket, want)

    def test_cli_jsonl_does_not_echo_endpoint_or_extra_fields(self):
        endpoint = v4()
        sentinel = "BODY_SENTINEL"
        payload = json.dumps({"endpoint": endpoint, "status": 404, "http_version": "2", "body": sentinel}) + "\n"
        cp = subprocess.run([sys.executable, str(PATH)], input=payload, text=True, capture_output=True, check=False)
        self.assertEqual(cp.returncode, 0)
        self.assertNotIn(endpoint, cp.stdout + cp.stderr)
        self.assertNotIn(sentinel, cp.stdout + cp.stderr)
        self.assertEqual(json.loads(cp.stdout)["status_class"], "4xx")

    def test_cli_rejects_argv_without_echoing_argument(self):
        sentinel = "RAW_ENDPOINT_MUST_NOT_BE_ECHOED"
        cp = subprocess.run([sys.executable, str(PATH), sentinel], text=True, capture_output=True, check=False)
        self.assertEqual(cp.returncode, 2)
        self.assertNotIn(sentinel, cp.stdout + cp.stderr)

    def test_invalid_json_and_oversized_input_are_generic(self):
        invalid = "INVALID_SENTINEL{" + "\n"
        oversized = "X" * (mod.MAX_INPUT_LINE + 20) + "\n"
        cp = subprocess.run([sys.executable, str(PATH)], input=invalid + oversized, text=True, capture_output=True, check=False)
        self.assertEqual(cp.returncode, 0)
        self.assertNotIn("INVALID_SENTINEL", cp.stdout + cp.stderr)
        rows = [json.loads(x) for x in cp.stdout.splitlines()]
        self.assertEqual(rows, [mod._unknown_record(), mod._unknown_record()])

    def test_source_has_no_network_dns_or_subprocess_imports(self):
        tree = ast.parse(PATH.read_text())
        banned = {"socket", "urllib", "http", "requests", "aiohttp", "dns", "subprocess", "ftplib"}
        imported = set()
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                imported.update(alias.name.split(".")[0] for alias in node.names)
            elif isinstance(node, ast.ImportFrom) and node.module:
                imported.add(node.module.split(".")[0])
        self.assertFalse(imported & banned, imported & banned)

    def test_output_schema_and_lengths_are_bounded(self):
        out = self.sanitizer().sanitize({"endpoint": v4(), "status": 599, "http_version": "HTTP/3", "timing_ms": 999999}).as_dict()
        self.assertEqual(set(out), {"family", "endpoint_alias", "http_version_class", "status_class", "timing_bucket"})
        for value in out.values():
            self.assertLessEqual(len(value), 32)


if __name__ == "__main__":
    unittest.main()
