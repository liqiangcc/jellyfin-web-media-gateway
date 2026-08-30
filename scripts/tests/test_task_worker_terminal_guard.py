import copy
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


def snap(*, state="open", labels=None, owner="alice", attempt=2, final_after=False, superseded_after=False, **extra):
    value = {
        "state": state,
        "labels": ["env:cloud", "status:in-progress"] if labels is None else labels,
        "owner": owner,
        "attempt": attempt,
        "final_acceptance_after_claim": final_after,
        "superseded_after_claim": superseded_after,
    }
    value.update(extra)
    return value


class TerminalGuardTests(unittest.TestCase):
    def decision(self, snapshot, expected_status="status:in-progress"):
        return mod.evaluate(snapshot, expected_attempt=2, expected_owner="alice", expected_status=expected_status)

    def test_authorized_execution_report_path(self):
        self.assertEqual(self.decision(snap()), mod.Decision(True, "authorized"))

    def test_authorized_blocker_report_path_uses_same_guard(self):
        self.assertTrue(self.decision(snap()).allowed)

    def test_authorized_normal_owner_release_from_review(self):
        self.assertTrue(self.decision(snap(labels=["env:cloud", "status:review"]), expected_status="status:review").allowed)

    def test_authorized_blocker_owner_release_from_blocked(self):
        self.assertTrue(self.decision(snap(labels=["env:cloud", "status:blocked"]), expected_status="status:blocked").allowed)

    def test_historical_final_acceptance_before_reopen_does_not_block_new_claim(self):
        self.assertTrue(self.decision(snap(historical_final_acceptance_present=True)).allowed)

    def test_closed_issue_rejected(self):
        self.assertEqual(self.decision(snap(state="closed")).reason, "issue-not-open")

    def test_status_done_rejected(self):
        self.assertEqual(self.decision(snap(labels=["env:cloud", "status:done"])).reason, "status-done")

    def test_final_acceptance_after_claim_rejected(self):
        self.assertEqual(self.decision(snap(final_after=True)).reason, "final-acceptance-after-claim")

    def test_owner_mismatch_rejected(self):
        self.assertEqual(self.decision(snap(owner="bob")).reason, "owner-mismatch")

    def test_superseded_attempt_rejected(self):
        self.assertEqual(self.decision(snap(attempt=3)).reason, "attempt-superseded")

    def test_newer_coordinator_authority_rejected(self):
        self.assertEqual(self.decision(snap(superseded_after=True)).reason, "newer-authority-after-claim")

    def test_not_in_progress_rejected(self):
        self.assertEqual(self.decision(snap(labels=["env:cloud", "status:review"])).reason, "unexpected-status")

    def test_ambiguous_chronology_fails_closed(self):
        value = snap()
        value.pop("final_acceptance_after_claim")
        self.assertEqual(self.decision(value).reason, "authority-ambiguous")

    def test_every_rejected_case_is_pure(self):
        cases = [
            snap(state="closed"),
            snap(labels=["env:cloud", "status:done"]),
            snap(final_after=True),
            snap(owner="bob"),
            snap(attempt=3),
            snap(superseded_after=True),
            snap(labels=["env:cloud", "status:review"]),
        ]
        ambiguous = snap()
        ambiguous.pop("superseded_after_claim")
        cases.append(ambiguous)
        for value in cases:
            with self.subTest(value=value):
                before = copy.deepcopy(value)
                decision = self.decision(value)
                self.assertFalse(decision.allowed)
                self.assertEqual(value, before)


if __name__ == "__main__":
    unittest.main()
