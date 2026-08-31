#!/usr/bin/env python3
"""Pure precondition guard for destructive self-hosted Runner re-registration.

Consumes a normalized non-secret JSON snapshot from stdin and emits one bounded
decision. It performs no network, subprocess, filesystem mutation, or token work.
"""
from __future__ import annotations

import argparse
import json
import sys
from typing import NamedTuple

EXPECTED_RUNNER_NAME = "ubuntu-arm64-target-phone"
EXPECTED_WORK_DIR = "_work"
EXPECTED_FINAL_UID = 999
EXPECTED_FINAL_USER = "gateway-runner"
EXPECTED_SCOPE = "repository"


class Decision(NamedTuple):
    allowed: bool
    reason: str


def evaluate(snapshot: object) -> Decision:
    if not isinstance(snapshot, dict):
        return Decision(False, "snapshot-invalid")

    required_true = (
        ("tailnet_reachable", "tailnet-unreachable"),
        ("ssh_reachable", "ssh-unreachable"),
        ("persistent_context", "persistent-context-missing"),
        ("authority_current", "authority-stale"),
        ("identity_frozen", "identity-not-frozen"),
        ("rollback_ready", "rollback-not-ready"),
    )
    for key, reason in required_true:
        if snapshot.get(key) is not True:
            return Decision(False, reason)

    required_false = (
        ("runner_busy", "runner-busy"),
        ("active_job", "active-job-present"),
        ("listener_running", "old-listener-running"),
        ("concurrent_recovery", "concurrent-recovery"),
    )
    for key, reason in required_false:
        if snapshot.get(key) is not False:
            return Decision(False, reason)

    if snapshot.get("scope") != EXPECTED_SCOPE:
        return Decision(False, "scope-mismatch")
    if snapshot.get("runner_name") != EXPECTED_RUNNER_NAME:
        return Decision(False, "runner-name-mismatch")
    if snapshot.get("work_dir") != EXPECTED_WORK_DIR:
        return Decision(False, "work-dir-mismatch")
    if snapshot.get("final_uid") != EXPECTED_FINAL_UID:
        return Decision(False, "final-uid-mismatch")
    if snapshot.get("final_user") != EXPECTED_FINAL_USER:
        return Decision(False, "final-user-mismatch")

    labels = snapshot.get("labels")
    if (
        not isinstance(labels, list)
        or not labels
        or any(not isinstance(label, str) or not label or len(label) > 64 for label in labels)
        or len(labels) > 32
    ):
        return Decision(False, "labels-snapshot-invalid")

    fingerprint = snapshot.get("labels_fingerprint", "")
    if fingerprint and (not isinstance(fingerprint, str) or len(fingerprint) > 128):
        return Decision(False, "labels-fingerprint-invalid")

    return Decision(True, "authorized")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.parse_args()
    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, UnicodeError):
        payload = None
    decision = evaluate(payload)
    result = "AUTHORIZED" if decision.allowed else "BLOCKED"
    print(json.dumps({"result": result, "reason": decision.reason}, separators=(",", ":")))
    return 0 if decision.allowed else 3


if __name__ == "__main__":
    raise SystemExit(main())
