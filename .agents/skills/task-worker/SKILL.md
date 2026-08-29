---
name: task-worker
description: Claim and execute one published jellyfin-web-media-gateway Task Attempt, then report the result back to its GitHub Issue. Supports normal implementation/verification Attempts and Coordinator-directed integration-only Attempts. Use only when explicitly asked to execute a ready Task; do not publish, review, accept, or close tasks.
---

# Task Worker

Execute exactly one published Task Attempt and return all durable feedback to the GitHub Issue.

## Authority

This skill does not define Task scope.

Before execution, read:

1. `AGENTS.md`
2. the target GitHub Issue and all relevant comments
3. the Task Package `prompt.md`
4. the Task Package `task.md`
5. `docs/tasks/issue-lifecycle-protocol.md`
6. `docs/tasks/execution-anchor-recovery-protocol.md`
7. `docs/tasks/freshness-integration-protocol.md`
8. every canonical/topic document explicitly required by `task.md`

If this skill conflicts with those sources, follow the higher-authority repository source.

For a Task published before the freshness protocol was adopted, do not use the new default to lower a stricter frozen Task Contract. Explicit strict-main/current-main requirements remain binding until formal Contract Revision.

## Inputs

Prefer an explicit Issue number and `prompt.md` path.

If no Issue is given, a Worker may query its exact eligible queue only when the current environment is known, for example:

```text
status:ready + env:ubuntu-arm64
```

Do not infer a Worker environment only from CPU architecture when multiple repository roles are possible. If several Tasks are claimable and repository priority does not identify one unique Task, stop rather than choosing arbitrarily.

## GitHub capability

Use an authenticated GitHub read/write path available in the current Codex environment, such as a connected GitHub tool or authenticated `gh` CLI.

If Issue comments/status/ownership cannot be updated, do not start write-side Task work because the Attempt cannot be closed-loop reported.

## Pre-claim checks

Re-read the Issue immediately before claim and confirm:

```text
Issue is open
status = ready
current environment is eligible
Required Capabilities are available
no active execution owner exists
linked task.md resolves
linked prompt.md resolves
Task Contract is executable as published
Freshness / Integration Contract is understood when present
```

If any condition fails, stop without making implementation changes.

## Claim and Attempt

Each successful transition from `status:ready` to `status:in-progress` begins a new Attempt.

Determine `Attempt N` from Issue history. Do not reuse a previous Attempt number.

Claim using the repository's live Issue state mechanism:

- preserve unrelated labels;
- replace only the task status with `status:in-progress`;
- record/assign active ownership using the supported repository mechanism;
- preserve `env:*` eligibility;
- re-read the Issue after mutation to confirm the claim is visible.

If the claim cannot be confirmed, stop. Do not execute concurrently on an unconfirmed claim.

## Durable execution anchor

After claim, follow `docs/tasks/execution-anchor-recovery-protocol.md`.

For a repository-mutating Attempt:

1. work on the Task-specific/existing worker branch rather than an unrelated branch;
2. once the first **coherent in-scope** change exists, commit and push it;
3. if the Task normally delivers through a PR and work will continue, create or reuse a draft PR as soon as it is useful for recovery/review;
4. if the Attempt remains in progress after the first durable anchor exists, post exactly one `[EXECUTION CHECKPOINT]` with branch / durable commit / draft PR / workflow anchor;
5. continue implementation and verification normally.

Do **not** create empty/no-op commits or empty PRs merely to satisfy the anchor rule. Do not post periodic heartbeat comments.

A checkpoint commit is not the final Task Candidate and does not prove any Claim. Final exact-Candidate Evidence is still governed by `task.md`.

Before long-running Actions/target execution, ensure the candidate/harness identity being exercised is already durable and identifiable by SHA/ref.

Attempts that were already `status:in-progress` before the execution-anchor protocol was introduced are not retroactively invalidated for lacking a checkpoint.

## Execute only the Task Contract

Follow `task.md` exactly:

- Scope / Out of Scope;
- Architecture Invariants;
- Implementation Requirements;
- Claims / Verification Job Matrix;
- Success Criteria;
- Freshness / Integration Contract when present;
- Evidence Contract;
- Failure / Blocked Rules.

Do not silently change Success Criteria, add unrelated cleanup, start a different Task, or reinterpret a Runner/Target as a new business Task.

Use the execution plane and target required by the Claim. Generic ARM64 evidence is not phone-target evidence; interactive diagnosis is not automatically Verification PASS.

## Dependency-aware freshness behavior

For `Freshness policy: dependency-aware`:

```text
main advanced
!= automatically stale
```

A normal Worker Attempt should prove the Task's Claims on an exact Task Candidate. Do not continuously chase `main` or repeatedly merge/rebase solely because unrelated commits land while the Task is running.

Before the final `[EXECUTION REPORT]`, record the actual identities available to the Coordinator:

```text
Task Candidate SHA
Task-specific Evidence run/job IDs
Evidence Base / accepted main snapshot actually included when known
Observed current main SHA at report time
```

If `main` advanced, Worker may note an initial suspected overlap, but **final freshness classification belongs to Coordinator Review**.

Do not self-trigger a full rerun simply because Current Main != Evidence Base unless:

- the frozen Task uses `Freshness policy: strict-main`; or
- the Task Contract explicitly requires a semantic authority integration before reporting; or
- a Coordinator Review already directed a `SEMANTIC_FRESHNESS` or `INTEGRATION_ONLY` Attempt.

## Integration-only Attempt

If the latest Coordinator Review / `[INTEGRATION GATE]` says `Revision class: INTEGRATION_ONLY`, reuse the same Issue/branch/PR, preserve Task Candidate ancestry, integrate the frozen base narrowly, run only declared integration jobs, and report Task Candidate / Integration Base / Integration Candidate separately. If conflict resolution changes Task-owned semantics, stop and let Coordinator reclassify.

## Semantic-freshness Attempt

If Coordinator says `Revision class: SEMANTIC_FRESHNESS`, integrate/reconcile only the specified accepted authority and rerun only the Claims/Jobs required by the Coordinator/Task Freshness Contract. If Scope/Claims/Success Criteria would change, stop rather than silently rewriting the Task Contract.

## Operator privilege versus final runtime privilege

For infrastructure bootstrap Tasks, distinguish the bootstrap operator from the service being created. Privileged setup explicitly allowed by the Task does not permit the final service/Runner to violate its low-privilege security contract.

## Fresh terminal-write authority guard

A startup/claim read is not authority to write terminal state later. Immediately before the first irreversible Worker terminal mutation, perform a fresh live Issue read-back. This guard applies to normal and blocked completion and covers the report comment, status transition, and owner release.

Require the fresh snapshot to prove:

```text
Issue is OPEN
status:in-progress is still present
current Attempt is this Worker Attempt
active owner/claim still matches this Worker
status:done is absent
no durable [FINAL ACCEPTANCE] is present
no newer Coordinator gate / Attempt supersedes this Worker
```

If any check fails or authority is ambiguous, fail closed: post no terminal report, change no status, release/reassign no owner, do not reopen the Issue, and STOP with `STALE_AUTHORITY`. A stale Worker must never clean up ownership that may belong to a newer Attempt.

`scripts/task-worker-terminal-guard.py` is the repository-owned pure decision helper for a normalized freshly fetched snapshot. It performs no GitHub mutation and does not replace the authoritative live GitHub read.

Evaluate the guard at the last safe point before the first terminal write. GitHub multi-operation atomicity is not claimed. If authority is known to become stale after an earlier mutation, do not continue later status/owner writes; stop and defer reconciliation to Coordinator.

Coordinator Final Acceptance/close is not a Worker terminal write and remains governed by the Final Acceptance Gate.

## Normal Attempt completion

Before leaving the Attempt:

1. commit/push or otherwise persist in-scope candidate changes when required;
2. confirm final Candidate / PR / Evidence identities;
3. collect the Evidence required by `task.md` and the latest Coordinator Review;
4. prepare the complete `[EXECUTION REPORT]` payload without posting it;
5. perform the fresh terminal-write authority guard above; if rejected, STOP with zero Issue mutations;
6. only while authorized, post the report and include the real Attempt, SHA/PR, Claims, commands, host/target, Evidence, problems, freshness identities and unverified scope;
7. transition to `status:review` only while the same authority remains current;
8. release active execution ownership only while the same authority remains current;
9. re-read the Issue to verify report + status are durable;
10. stop.

Worker execution outcome is not Coordinator acceptance.

Do not set `status:done`, close the Issue, or immediately start Attempt N+1.

## Blocked Attempt

If a required permission, GitHub capability, device, Runner, Secret-at-runtime, network condition, dependency, or target capability is unavailable:

1. preserve a safe state and any durable branch/commit/PR/Evidence anchor;
2. clean temporary resources when required;
3. prepare the complete `[BLOCKER REPORT]` payload without posting it;
4. perform the fresh terminal-write authority guard above; if rejected, STOP with zero Issue mutations;
5. only while authorized, post the blocker with exact completed work, stop point, Evidence, minimal resume condition, cleanup/safe state and reusable anchor;
6. transition to `status:blocked` only while the same authority remains current;
7. release active execution ownership only while the same authority remains current and repository policy permits;
8. re-read the Issue to verify the report/status;
9. stop.

Never bypass a security boundary or lower Success Criteria to avoid BLOCKED.

## After a REVISE

Read the previous Attempt report and Coordinator Review/revision class, reuse a valid durable branch/PR, begin a new Attempt only after a fresh claim, and execute only the required revision class.

## Completion output to the user

After an authorized durable Issue update, summarize Issue, Attempt, outcome, Issue state, Candidate identities, report posted, and `Next authority: Web Coordinator`. If the terminal guard rejects the Worker, report `STALE_AUTHORITY` and `Issue mutations: none`.
