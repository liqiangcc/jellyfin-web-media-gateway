import copy
import importlib.util
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import time
import unittest
import uuid

ROOT = pathlib.Path(__file__).resolve().parents[2]
GUARD = ROOT / "scripts" / "runner_rereg_guard.py"
WRAPPER = ROOT / "scripts" / "runner_rereg_token_wrapper.sh"

spec = importlib.util.spec_from_file_location("runner_rereg_guard", GUARD)
guard = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = guard
spec.loader.exec_module(guard)


def good_snapshot():
    return {
        "tailnet_reachable": True,
        "ssh_reachable": True,
        "persistent_context": True,
        "authority_current": True,
        "identity_frozen": True,
        "rollback_ready": True,
        "runner_busy": False,
        "active_job": False,
        "listener_running": False,
        "concurrent_recovery": False,
        "scope": "repository",
        "runner_name": "ubuntu-arm64-target-phone",
        "labels": ["self-hosted", "Linux", "ARM64", "target-phone"],
        "labels_fingerprint": "bounded-non-secret-fingerprint",
        "work_dir": "_work",
        "final_uid": 999,
        "final_user": "gateway-runner",
    }


FAKE_CONFIG = r'''#!/usr/bin/env bash
set -eu
expected="${FAKE_EXPECTED_TOKEN:?}"
capture="${FAKE_CAPTURE:?}"
sleep_secs="${FAKE_SLEEP:-0}"
exit_code="${FAKE_EXIT_CODE:-0}"
safe=()
seen=0
while [ "$#" -gt 0 ]; do
  arg="$1"
  shift
  if [ "$arg" = "--token" ]; then
    [ "$#" -gt 0 ] || exit 90
    value="$1"
    shift
    [ "$value" = "$expected" ] || exit 91
    seen=$((seen + 1))
    safe+=("--token" "<redacted>")
  else
    safe+=("$arg")
  fi
done
[ "$seen" -eq 1 ] || exit 92
printf '%s\n' "${safe[*]}" > "$capture"
if [ "$sleep_secs" != "0" ]; then sleep "$sleep_secs"; fi
exit "$exit_code"
'''


class GuardTests(unittest.TestCase):
    def test_authorized_snapshot(self):
        self.assertEqual(guard.evaluate(good_snapshot()), guard.Decision(True, "authorized"))

    def test_guard_is_pure(self):
        snapshot = good_snapshot()
        before = copy.deepcopy(snapshot)
        guard.evaluate(snapshot)
        self.assertEqual(snapshot, before)

    def test_boolean_precondition_negatives(self):
        cases = {
            "tailnet_reachable": "tailnet-unreachable",
            "ssh_reachable": "ssh-unreachable",
            "persistent_context": "persistent-context-missing",
            "authority_current": "authority-stale",
            "identity_frozen": "identity-not-frozen",
            "rollback_ready": "rollback-not-ready",
        }
        for field, reason in cases.items():
            with self.subTest(field=field):
                value = good_snapshot()
                value[field] = False
                self.assertEqual(guard.evaluate(value).reason, reason)

    def test_busy_and_concurrency_negatives(self):
        cases = {
            "runner_busy": "runner-busy",
            "active_job": "active-job-present",
            "listener_running": "old-listener-running",
            "concurrent_recovery": "concurrent-recovery",
        }
        for field, reason in cases.items():
            with self.subTest(field=field):
                value = good_snapshot()
                value[field] = True
                self.assertEqual(guard.evaluate(value).reason, reason)

    def test_identity_negatives(self):
        cases = [
            ("scope", "organization", "scope-mismatch"),
            ("runner_name", "other", "runner-name-mismatch"),
            ("work_dir", "other", "work-dir-mismatch"),
            ("final_uid", 0, "final-uid-mismatch"),
            ("final_user", "root", "final-user-mismatch"),
            ("labels", [], "labels-snapshot-invalid"),
        ]
        for field, changed, reason in cases:
            with self.subTest(field=field):
                value = good_snapshot()
                value[field] = changed
                self.assertEqual(guard.evaluate(value).reason, reason)


class WrapperTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = pathlib.Path(self.tmp.name)
        self.config = self.dir / "config.sh"
        self.config.write_text(FAKE_CONFIG)
        self.config.chmod(0o755)
        self.capture = self.dir / "capture.txt"
        self.token = "SENTINEL_" + uuid.uuid4().hex
        self.env = os.environ.copy()
        self.env.update(
            {
                "FAKE_EXPECTED_TOKEN": self.token,
                "FAKE_CAPTURE": str(self.capture),
            }
        )

    def tearDown(self):
        self.tmp.cleanup()

    def run_wrapper(self, args, token=None, env=None):
        cp = subprocess.run(
            ["bash", str(WRAPPER), *args],
            input=(self.token if token is None else token) + "\n",
            text=True,
            capture_output=True,
            env=self.env if env is None else env,
            check=False,
        )
        self.assertNotIn(self.token, cp.stdout + cp.stderr)
        if self.capture.exists():
            self.assertNotIn(self.token, self.capture.read_text())
        return cp

    def test_remove_token_stdin_non_reflective(self):
        cp = self.run_wrapper(["remove", str(self.config)])
        self.assertEqual(cp.returncode, 0)
        self.assertEqual(json.loads(cp.stdout)["result"], "SUCCESS")
        self.assertEqual(self.capture.read_text().strip(), "remove --token <redacted>")

    def test_register_identity_and_token_non_reflective(self):
        cp = self.run_wrapper(
            [
                "register",
                str(self.config),
                "https://github.com/liqiangcc/jellyfin-web-media-gateway",
                "ubuntu-arm64-target-phone",
                "target-phone",
                "_work",
            ]
        )
        self.assertEqual(cp.returncode, 0)
        safe = self.capture.read_text()
        self.assertIn("--unattended", safe)
        self.assertIn("--name ubuntu-arm64-target-phone", safe)
        self.assertIn("--work _work", safe)
        self.assertIn("--labels target-phone", safe)
        self.assertIn("--token <redacted>", safe)
        self.assertNotIn("--replace", safe)

    def test_register_can_omit_custom_labels_with_sentinel(self):
        cp = self.run_wrapper(
            [
                "register",
                str(self.config),
                "https://github.com/liqiangcc/jellyfin-web-media-gateway",
                "ubuntu-arm64-target-phone",
                "-",
                "_work",
            ]
        )
        self.assertEqual(cp.returncode, 0)
        safe = self.capture.read_text()
        self.assertNotIn("--labels", safe)
        self.assertIn("--work _work", safe)

    def test_child_failure_is_bounded_and_non_reflective(self):
        env = self.env.copy()
        env["FAKE_EXIT_CODE"] = "42"
        cp = self.run_wrapper(["remove", str(self.config)], env=env)
        self.assertEqual(cp.returncode, 4)
        result = json.loads(cp.stdout)
        self.assertEqual(result, {"result": "CHILD_FAILED", "mode": "remove", "exit_code": 42})

    def test_missing_token_rejected(self):
        cp = subprocess.run(
            ["bash", str(WRAPPER), "remove", str(self.config)],
            input="",
            text=True,
            capture_output=True,
            env=self.env,
            check=False,
        )
        self.assertEqual(cp.returncode, 2)
        self.assertEqual(json.loads(cp.stdout)["reason"], "token-missing")

    def test_unknown_mode_and_bad_path_rejected_without_child(self):
        cp = self.run_wrapper(["other", str(self.config)])
        self.assertEqual(cp.returncode, 2)
        self.assertFalse(self.capture.exists())
        other = self.dir / "not-config"
        other.write_text(FAKE_CONFIG)
        other.chmod(0o755)
        cp = self.run_wrapper(["remove", str(other)])
        self.assertEqual(cp.returncode, 2)
        self.assertFalse(self.capture.exists())

    def test_register_identity_mutations_and_replace_rejected(self):
        base = [
            "register",
            str(self.config),
            "https://github.com/liqiangcc/jellyfin-web-media-gateway",
            "ubuntu-arm64-target-phone",
            "target-phone",
            "_work",
        ]
        for index, value in [
            (3, "other-runner"),
            (5, "other-work"),
            (4, "self-hosted,--replace"),
        ]:
            with self.subTest(index=index):
                args = list(base)
                args[index] = value
                cp = self.run_wrapper(args)
                self.assertEqual(cp.returncode, 2)
                self.assertFalse(self.capture.exists())

    def test_wrapper_process_argv_never_contains_token(self):
        env = self.env.copy()
        env["FAKE_SLEEP"] = "0.6"
        proc = subprocess.Popen(
            ["bash", str(WRAPPER), "remove", str(self.config)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            env=env,
        )
        assert proc.stdin is not None
        proc.stdin.write(self.token + "\n")
        proc.stdin.close()
        cmdline = b""
        deadline = time.time() + 0.3
        while time.time() < deadline and proc.poll() is None:
            path = pathlib.Path(f"/proc/{proc.pid}/cmdline")
            if path.exists():
                cmdline = path.read_bytes()
                if cmdline:
                    break
            time.sleep(0.01)
        assert proc.stdin is not None
        proc.stdin.close()
        proc.stdin = None
        out, err = proc.communicate()
        rc = proc.returncode
        self.assertEqual(rc, 0)
        self.assertNotIn(self.token.encode(), cmdline)
        self.assertNotIn(self.token, out + err)

    def test_one_invocation_only(self):
        cp = self.run_wrapper(["remove", str(self.config)])
        self.assertEqual(cp.returncode, 0)
        lines = self.capture.read_text().splitlines()
        self.assertEqual(len(lines), 1)

    def test_static_wrapper_anti_leak_contract(self):
        source = WRAPPER.read_text()
        self.assertNotIn("set -x", source)
        self.assertIn("set +x", source)
        self.assertNotIn("mktemp", source)
        self.assertNotIn("history ", source)
        self.assertNotIn('echo "$token"', source)
        self.assertIn('*--replace*) fail "replace-rejected"', source)
        self.assertNotIn("\n    --replace ", source)


if __name__ == "__main__":
    unittest.main()
