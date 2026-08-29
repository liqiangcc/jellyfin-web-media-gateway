#!/usr/bin/env python3
from __future__ import annotations
import argparse, json, sys
from typing import NamedTuple

class Decision(NamedTuple):
    allowed: bool
    reason: str

def evaluate(snapshot: dict, *, expected_attempt: int, expected_owner: str) -> Decision:
    state = str(snapshot.get('state', '')).lower()
    labels = {str(x) for x in snapshot.get('labels', [])}
    comments = [str(x) for x in snapshot.get('comments', [])]
    if state != 'open': return Decision(False, 'issue-not-open')
    if 'status:done' in labels: return Decision(False, 'status-done')
    if any('[FINAL ACCEPTANCE]' in c for c in comments): return Decision(False, 'final-acceptance-present')
    if 'status:in-progress' not in labels: return Decision(False, 'not-in-progress')
    if snapshot.get('attempt') != expected_attempt: return Decision(False, 'attempt-superseded')
    if snapshot.get('owner') != expected_owner: return Decision(False, 'owner-mismatch')
    return Decision(True, 'authorized')

def main() -> int:
    p=argparse.ArgumentParser(); p.add_argument('--attempt',type=int,required=True); p.add_argument('--owner',required=True); a=p.parse_args()
    d=evaluate(json.load(sys.stdin),expected_attempt=a.attempt,expected_owner=a.owner)
    print(json.dumps({'allowed':d.allowed,'reason':d.reason},separators=(',',':')))
    return 0 if d.allowed else 3
if __name__=='__main__': raise SystemExit(main())
