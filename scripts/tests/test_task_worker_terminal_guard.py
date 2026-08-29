import importlib.util
import pathlib
import sys
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
PATH = ROOT / "scripts" / "task-worker-terminal-guard.py"
spec = importlib.util.spec_from_file_location("task_worker_terminal_guard", PATH)
mod = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = mod
spec.loader.exec_module(mod)


def snap(*, state="open", labels=None, comments=None, owner="alice", attempt=2):
    return {
        "state": state,
        "labels": labels or ["env:cloud", "status:in-progress"],
        "comments": comments or [],
        "owner": owner,
        "attempt": attempt,
    }


class TerminalGuardTests(unittest.TestCase):
    def decision(self, snapshot):
        return mod.evaluate(snapshot, expected_attempt=2, expected_owner="alice")

    def test_authorized_execution_report_path(self):
        self.assertEqual(self.decision(snap()), mod.Decision(True, "authorized"))

    def test_authorized_blocker_report_path_uses_same_guard(self):
        self.assertTrue(self.decision(snap()).allowed)

    def test_closed_issue_rejected_without_mutation_authority(self):
        self.assertEqual(self.decision(snap(state="closed")).reason, "issue-not-open")

    def test_status_done_rejected(self):
        self.assertEqual(self.decision(snap(labels=["env:cloud", "status:done"])).reason, "status-done")

    def test_final_acceptance_rejected(self):
        self.assertEqual(self.decision(snap(comments=["[FINAL ACCEPTANCE]\nDecision: ACCEPT"])).reason, "final-acceptance-present")

    def test_owner_mismatch_rejected(self):
        self.assertEqual(self.decision(snap(owner="bob")).reason, "owner-mismatch")

    def test_superseded_attempt_rejected(self):
        self.assertEqual(self.decision(snap(attempt=3)).reason, "attempt-superseded")

    def test_not_in_progress_rejected(self):
        self.assertEqual(self.decision(snap(labels=["env:cloud", "status:review"])).reason, "not-in-progress")

    def test_rejected_guard_is_pure_and_cannot_mutate_snapshot(self):
        original = snap(comments=["[FINAL ACCEPTANCE]"])
        before = repr(original)
        self.assertFalse(self.decision(original).allowed)
        self.assertEqual(repr(original), before)


if __name__ == "__main__":
    unittest.main()
