#!/usr/bin/env python3
"""Pure stale-authority guard for task-worker terminal mutations.

The caller supplies a fresh live Issue snapshot immediately before the first
terminal mutation. This helper performs no GitHub writes; rejected snapshots
therefore have a deterministic zero-mutation result.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass


@dataclass(frozen=True)
class Decision:
    allowed: bool
    reason: str


def evaluate(snapshot: dict, *, expected_attempt: int, expected_owner: str) -> Decision:
    state = str(snapshot.get("state", "")).lower()
    labels = {str(x) for x in snapshot.get("labels", [])}
    comments = [str(x) for x in snapshot.get("comments", [])]
    owner = snapshot.get("owner")
    attempt = snapshot.get("attempt")

    if state != "open":
        return Decision(False, "issue-not-open")
    if "status:done" in labels:
        return Decision(False, "status-done")
    if any("[FINAL ACCEPTANCE]" in c for c in comments):
        return Decision(False, "final-acceptance-present")
    if "status:in-progress" not in labels:
        return Decision(False, "not-in-progress")
    if attempt != expected_attempt:
        return Decision(False, "attempt-superseded")
    if owner != expected_owner:
        return Decision(False, "owner-mismatch")
    return Decision(True, "authorized")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--attempt", type=int, required=True)
    parser.add_argument("--owner", required=True)
    args = parser.parse_args()
    snapshot = json.load(sys.stdin)
    decision = evaluate(snapshot, expected_attempt=args.attempt, expected_owner=args.owner)
    print(json.dumps({"allowed": decision.allowed, "reason": decision.reason}, separators=(",", ":")))
    return 0 if decision.allowed else 3


if __name__ == "__main__":
    raise SystemExit(main())
