#!/usr/bin/env python3
"""Pure stale-authority guard for task-worker terminal mutations.

The caller supplies a normalized *fresh live* Issue snapshot immediately before
each terminal mutation. This helper performs no GitHub writes.
"""
from __future__ import annotations

import argparse
import json
import sys
from typing import NamedTuple


class Decision(NamedTuple):
    allowed: bool
    reason: str


def evaluate(snapshot: dict, *, expected_attempt: int, expected_owner: str) -> Decision:
    state = str(snapshot.get("state", "")).lower()
    labels_raw = snapshot.get("labels")
    labels = {str(x) for x in labels_raw} if isinstance(labels_raw, list) else set()

    if state != "open":
        return Decision(False, "issue-not-open")
    if "status:done" in labels:
        return Decision(False, "status-done")
    if "status:in-progress" not in labels:
        return Decision(False, "not-in-progress")
    if snapshot.get("attempt") != expected_attempt:
        return Decision(False, "attempt-superseded")
    if snapshot.get("owner") != expected_owner:
        return Decision(False, "owner-mismatch")

    final_after = snapshot.get("final_acceptance_after_claim")
    superseded_after = snapshot.get("superseded_after_claim")
    if not isinstance(final_after, bool) or not isinstance(superseded_after, bool):
        return Decision(False, "authority-ambiguous")
    if final_after:
        return Decision(False, "final-acceptance-after-claim")
    if superseded_after:
        return Decision(False, "newer-authority-after-claim")
    return Decision(True, "authorized")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--attempt", type=int, required=True)
    parser.add_argument("--owner", required=True)
    args = parser.parse_args()
    snapshot = json.load(sys.stdin)
    decision = evaluate(snapshot, expected_attempt=args.attempt, expected_owner=args.owner)
    result = "AUTHORIZED" if decision.allowed else "STALE_AUTHORITY"
    print(json.dumps({"result": result, "allowed": decision.allowed, "reason": decision.reason}, separators=(",", ":")))
    return 0 if decision.allowed else 3


if __name__ == "__main__":
    raise SystemExit(main())
