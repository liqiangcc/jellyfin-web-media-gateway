import ast
import copy
import importlib.util
import json
import pathlib
import subprocess
import sys
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
PATH = ROOT / "scripts" / "phone_mgmt_readiness.py"
spec = importlib.util.spec_from_file_location("phone_mgmt_readiness", PATH)
mod = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = mod
spec.loader.exec_module(mod)


def snap(**changes):
    value = {"tailnet_reachable": True, "ssh_tcp_reachable": True, "ssh_authenticated": True, "ubuntu_context_reachable": True, "persistent_context_proven": True}
    value.update(changes)
    return value


class ReadinessTests(unittest.TestCase):
    def test_device_offline(self):
        d = mod.evaluate(snap(tailnet_reachable=False, ssh_tcp_reachable=False, ssh_authenticated=False, ubuntu_context_reachable=False, persistent_context_proven=False))
        self.assertEqual(d, mod.Decision("DEVICE_OFFLINE", False, "tailnet-unreachable"))

    def test_tailnet_only_tcp_not_ready(self):
        d = mod.evaluate(snap(ssh_tcp_reachable=False, ssh_authenticated=False, ubuntu_context_reachable=False, persistent_context_proven=False))
        self.assertEqual(d.state, "TAILNET_ONLY")
        self.assertFalse(d.claim_allowed)

    def test_tailnet_only_auth_not_ready(self):
        d = mod.evaluate(snap(ssh_authenticated=False, ubuntu_context_reachable=False, persistent_context_proven=False))
        self.assertEqual(d.reason, "ssh-auth-failed")

    def test_ssh_ready_ubuntu_not_ready(self):
        d = mod.evaluate(snap(ubuntu_context_reachable=False, persistent_context_proven=False))
        self.assertEqual(d.state, "SSH_READY")
        self.assertFalse(d.claim_allowed)

    def test_ssh_ready_persistence_not_ready(self):
        d = mod.evaluate(snap(persistent_context_proven=False))
        self.assertEqual(d, mod.Decision("SSH_READY", False, "persistent-context-failed"))

    def test_fully_ready_is_only_claim_allowed_state(self):
        d = mod.evaluate(snap())
        self.assertEqual(d, mod.Decision("UBUNTU_PERSISTENT_READY", True, "authorized"))

    def test_unknown_fields_fail_closed(self):
        for field in mod.FIELDS:
            with self.subTest(field=field):
                value = snap()
                value[field] = None
                d = mod.evaluate(value)
                self.assertFalse(d.claim_allowed)
                self.assertNotEqual(d.state, "UBUNTU_PERSISTENT_READY")

    def test_missing_fields_fail_closed(self):
        for field in mod.FIELDS:
            with self.subTest(field=field):
                value = snap()
                del value[field]
                d = mod.evaluate(value)
                self.assertFalse(d.claim_allowed)
                self.assertIn("missing", d.reason)

    def test_invalid_field_type_fails_closed(self):
        d = mod.evaluate(snap(ssh_authenticated="yes"))
        self.assertFalse(d.claim_allowed)
        self.assertEqual(d.reason, "ssh_authenticated-invalid")

    def test_contradictory_tailnet_and_downstream_fails_closed(self):
        d = mod.evaluate(snap(tailnet_reachable=False))
        self.assertEqual(d.state, "DEVICE_OFFLINE")
        self.assertEqual(d.reason, "contradictory-evidence")
        self.assertFalse(d.claim_allowed)

    def test_contradictory_auth_and_ubuntu_fails_closed(self):
        d = mod.evaluate(snap(ssh_authenticated=False))
        self.assertEqual(d.state, "TAILNET_ONLY")
        self.assertEqual(d.reason, "contradictory-evidence")

    def test_controlmaster_metadata_is_ignored(self):
        base = mod.evaluate(snap())
        for value in (True, False, None, "stale"):
            with self.subTest(value=value):
                enriched = snap(controlmaster_present=value, controlmaster_live=value)
                self.assertEqual(mod.evaluate(enriched), base)

    def test_input_is_not_mutated(self):
        value = snap(controlmaster_present=False)
        before = copy.deepcopy(value)
        mod.evaluate(value)
        self.assertEqual(value, before)

    def test_cli_output_is_bounded(self):
        cp = subprocess.run([sys.executable, str(PATH)], input=json.dumps(snap()) + "\n", text=True, capture_output=True, check=False)
        self.assertEqual(cp.returncode, 0)
        row = json.loads(cp.stdout)
        self.assertEqual(set(row), {"state", "claim_allowed", "reason"})
        self.assertTrue(row["claim_allowed"])
        self.assertLessEqual(len(cp.stdout), 180)
        self.assertEqual(cp.stderr, "")

    def test_source_has_no_network_or_mutating_imports(self):
        tree = ast.parse(PATH.read_text())
        banned = {"socket", "subprocess", "urllib", "http", "requests", "aiohttp", "dns", "paramiko", "fabric", "os", "pathlib", "shutil"}
        imported = set()
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                imported.update(alias.name.split(".")[0] for alias in node.names)
            elif isinstance(node, ast.ImportFrom) and node.module:
                imported.add(node.module.split(".")[0])
        self.assertFalse(imported & banned, imported & banned)


if __name__ == "__main__":
    unittest.main()
