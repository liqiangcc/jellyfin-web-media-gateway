# Task Worker Terminal-Write Authority Guard

This document is normative lifecycle guidance used with `issue-lifecycle-protocol.md` and `.agents/skills/task-worker/SKILL.md`.

## Rule

Claim-time authority expires as live Issue state changes. Immediately before **each** irreversible Worker terminal mutation, freshly read the live Issue.

A terminal sequence can include `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, transition to `status:review` or `status:blocked`, and release/reassignment of active execution ownership. Each mutation gets its own last-safe-point read; one read before the whole sequence is not enough.

Proceed with the pending mutation only when the fresh normalized snapshot proves all of:

- Issue is open;
- the status expected for the pending mutation is current:
  - `status:in-progress` before report or status mutation;
  - `status:review` before normal owner release;
  - `status:blocked` before blocker owner release;
- current Attempt matches this Worker;
- active owner/claim matches this Worker;
- `status:done` is absent;
- no Final Acceptance newer than this Attempt's claim/checkpoint authority exists;
- no newer Coordinator gate/Attempt supersedes this Worker.

A historical Final Acceptance that predates an explicit Coordinator Reopen and this new claim does not by itself reject the new Attempt. The live reader must compare append-only history to the current durable claim/checkpoint; ambiguous chronology fails closed.

If any condition fails or authority is ambiguous, return `STALE_AUTHORITY`: do not perform the pending terminal mutation, do not reopen the Issue, and STOP. A stale Worker must never release/reassign an owner that may belong to a newer Attempt.

If authority becomes stale after an earlier terminal mutation, preserve that earlier append-only history but perform no later status/owner mutations. This guard does not claim GitHub multi-operation atomicity and does not introduce a distributed lock. Coordinator Final Acceptance/close remains governed by the existing Final Acceptance Gate.

## Pure normalized helper

`scripts/task-worker-terminal-guard.py` is an optional pure decision helper. It performs no GitHub mutation and never replaces the authoritative live GitHub read. The caller normalizes the fresh live state into:

```json
{
  "state": "open",
  "labels": ["status:in-progress", "env:cloud"],
  "owner": "worker-login",
  "attempt": 3,
  "status": "status:review",
  "final_acceptance_after_claim": false,
  "superseded_after_claim": false
}
```

The caller also supplies the expected status for the pending mutation. Both chronology booleans are required. Missing/non-boolean values are `authority-ambiguous` and fail closed.
