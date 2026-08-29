# Task Worker Terminal-Write Authority Guard

This document is normative lifecycle guidance used with `issue-lifecycle-protocol.md` and `.agents/skills/task-worker/SKILL.md`.

## Rule

Claim-time authority expires as live Issue state changes. Immediately before the first irreversible Worker terminal mutation, freshly read the live Issue.

A terminal sequence includes `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, transition to `status:review` or `status:blocked`, and release/reassignment of active execution ownership.

Proceed only when the fresh snapshot proves all of:

- Issue is open;
- `status:in-progress` is current;
- current Attempt matches this Worker;
- active owner/claim matches this Worker;
- `status:done` is absent;
- no durable `[FINAL ACCEPTANCE]` exists;
- no newer Coordinator gate/Attempt supersedes this Worker.

If any condition fails or authority is ambiguous, fail closed with `STALE_AUTHORITY`: post no terminal report, change no status, release/reassign no owner, do not reopen the Issue, and STOP. In particular, a stale Worker must not "clean up" ownership that may belong to a newer Attempt.

This is a last-safe-point guard, not a distributed lock. GitHub multi-operation atomicity is not claimed. If authority becomes known stale after an earlier mutation, do not continue later status/owner writes. Coordinator Final Acceptance/close remains governed by the existing Final Acceptance Gate.

`scripts/task-worker-terminal-guard.py` is a pure local decision helper for normalized freshly-read snapshots. It performs no GitHub mutation and never replaces the live GitHub read.