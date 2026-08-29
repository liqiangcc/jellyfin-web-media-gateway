---
name: task-worker
description: Claim and execute one published jellyfin-web-media-gateway Task Attempt, then report the result back to its GitHub Issue. Supports normal implementation/verification Attempts and Coordinator-directed integration-only Attempts. Use only when explicitly asked to execute a ready Task; do not publish, review, accept, or close tasks.
---

# Task Worker

Execute exactly one published Task Attempt and return all durable feedback to the GitHub Issue.

## Authority

This skill does not define Task scope. Before execution, read `AGENTS.md`, the live Issue/relevant comments, Task `prompt.md`/`task.md`, and the repository lifecycle/freshness/recovery protocols plus canonical documents required by `task.md`. Higher-authority repository sources win.

## Pre-claim and claim

Immediately before claim, live-read the Issue and require: open, `status:ready`, eligible environment/capabilities, no active owner, resolvable executable Task Package. A successful `ready -> in-progress` claim starts a new Attempt. Preserve unrelated labels, set only the task status, record active ownership, then read back the claim. Never execute concurrently on an unconfirmed claim.

## Execute only the Task Contract

Follow Scope, invariants, Claims, Success Criteria, Evidence, Freshness/Integration and blocked rules exactly. Do not lower criteria, widen scope, or reinterpret an execution target. Persist coherent repository work using the execution-anchor protocol; checkpoint comments are recovery anchors, not acceptance.

For dependency-aware freshness, report Task Candidate, Evidence identity/base, and observed current main; Coordinator owns final freshness classification. Integration-only and semantic-freshness Attempts must obey the latest Coordinator Review and preserve/reuse prior accepted evidence only as the repository protocols permit.

## Fresh terminal-write authority guard

A startup/claim read is **not** authority to write terminal state later. Immediately before the first irreversible Worker terminal mutation, perform a new live Issue read-back. This applies equally to normal completion and blocked completion and covers the whole terminal sequence: report comment, `status:review`/`status:blocked`, and owner release.

The fresh snapshot must prove all of the following:

```text
Issue is OPEN
status:in-progress is still present
current Attempt is the Worker Attempt
active owner/claim still matches this Worker
status:done is absent
no durable [FINAL ACCEPTANCE] is present
no newer Coordinator gate / Attempt has superseded this Worker authority
```

If any check fails or authority is ambiguous, **fail closed**: post no `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, change no status, release/reassign no owner, do not reopen the Issue, and STOP with a bounded `STALE_AUTHORITY` result. A stale Worker must never "clean up" ownership that may now belong to a newer Attempt.

Where a caller wants a deterministic local decision, `scripts/task-worker-terminal-guard.py` is the repository-owned pure guard. It consumes a freshly fetched normalized snapshot and performs no GitHub mutation itself. The GitHub read remains authoritative; the helper does not replace it.

The guard is evaluated at the last safe point before the first terminal write. GitHub multi-operation atomicity is not claimed. If authority is invalidated after the report write, do not continue later status/owner mutations on known-stale authority; stop and defer reconciliation to Coordinator.

Coordinator Final Acceptance/close is not a Worker terminal write and remains governed by the Final Acceptance Gate.

## Normal Attempt completion

1. persist required candidate changes and collect exact Evidence;
2. prepare the complete `[EXECUTION REPORT]` payload without posting it;
3. perform the **fresh terminal-write authority guard** above;
4. only if authorized, post `[EXECUTION REPORT]`;
5. transition to `status:review` only while the same authority is still current;
6. release active execution ownership only while the same authority is still current;
7. re-read the Issue to verify the durable terminal state;
8. stop.

Worker execution outcome is not Coordinator acceptance. Never set `status:done`, close the Issue, or immediately start Attempt N+1.

## Blocked Attempt

1. preserve safe state/durable anchors and clean temporary resources as required;
2. prepare the complete `[BLOCKER REPORT]` payload without posting it;
3. perform the **fresh terminal-write authority guard** above;
4. only if authorized, post `[BLOCKER REPORT]`;
5. transition to `status:blocked` only while the same authority is still current;
6. release active execution ownership only while the same authority is still current and repository policy permits;
7. re-read the Issue to verify the durable terminal state;
8. stop.

Never bypass a security boundary or lower Success Criteria to avoid BLOCKED.

## After REVISE / integration recovery

Read the previous Attempt, Coordinator Review and revision class; reuse the existing durable branch/PR when valid; claim a new Attempt before execution; execute only the requested revision class. For infrastructure bootstrap, distinguish privileged operator setup from final low-privilege runtime requirements.

## Completion output to the user

After a durable authorized Issue update, summarize Issue, Attempt, execution outcome, Issue state, Candidate identities, report posted, and `Next authority: Web Coordinator`. If the terminal guard rejects the Worker, report `STALE_AUTHORITY` and explicitly state `Issue mutations: none`.
