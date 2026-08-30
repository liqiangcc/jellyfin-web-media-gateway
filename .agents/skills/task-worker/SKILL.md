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

If the latest Coordinator Review / `[INTEGRATION GATE]` says:

```text
Revision class: INTEGRATION_ONLY
```

then this Attempt is intentionally narrow.

Required behavior:

1. reuse the same Issue / branch / PR;
2. identify the previously accepted `Task Candidate`;
3. identify the Coordinator-frozen `Integration Base SHA`;
4. **prefer merging that exact Integration Base into the worker branch** rather than rebasing/re-writing the Task Candidate, so Task Candidate ancestry remains auditable;
5. do not make unrelated product/implementation changes;
6. if the merge is clean and does not alter Task-owned semantic implementation, run only the declared `JI*` integration jobs;
7. produce an exact `Integration Candidate SHA`;
8. report Task Candidate + Integration Base + Integration Candidate + exact JI Evidence separately.

If merge/conflict resolution touches Task-owned semantic code, changes an accepted authority assumption, or makes the previous Task Candidate Evidence no longer safely reusable:

- do not guess that it is still integration-only;
- preserve the branch/PR state;
- report the semantic conflict/problem;
- let Coordinator reclassify to `SEMANTIC_FRESHNESS` / Contract Revision as appropriate.

Do not claim that a JI-only run re-proved all C1-Cn. Explicitly state which semantic Evidence is reused and which new integration Evidence was produced.

## Semantic-freshness Attempt

If Coordinator says:

```text
Revision class: SEMANTIC_FRESHNESS
```

then integrate/reconcile the specified accepted authority and rerun only the Claims/Jobs listed by Coordinator/Task Freshness Contract, unless the Contract requires broader verification.

If the required authority change invalidates Scope/Claims/Success Criteria, stop and report rather than silently rewriting the Task Contract.

## Operator privilege versus final runtime privilege

For infrastructure bootstrap Tasks, distinguish the bootstrap operator from the service being created.

An operator may legitimately require privileged setup steps when the Task explicitly permits them. That does not allow the final service/Runner to violate its low-privilege security contract. Verify final runtime identity and access boundaries separately.

## Fresh terminal-write authority guard

Claim-time authority is not authority to write terminal state later. Immediately before each irreversible Worker terminal mutation, freshly read the live Issue and apply `docs/tasks/task-worker-terminal-write-guard.md` (optionally using `scripts/task-worker-terminal-guard.py` for the pure normalized decision).

Proceed only if the fresh snapshot proves all of:

- the Issue is open;
- the status expected for the pending mutation is current (`status:in-progress` before report/status; `status:review` before normal owner release; `status:blocked` before blocker owner release);
- the current Attempt still matches this Worker;
- active execution ownership still matches this Worker;
- `status:done` is absent;
- no `[FINAL ACCEPTANCE]` newer than the current claim/checkpoint authority exists;
- no newer Coordinator gate / Attempt has superseded this Worker.

A historical Final Acceptance that predates an explicit Coordinator REOPEN and the current fresh claim does not by itself reject the new Attempt. Chronology or authority ambiguity fails closed.

If any condition fails or authority is ambiguous, fail closed with `STALE_AUTHORITY`: do not perform the pending terminal mutation, do not reopen the Issue, and STOP. In particular, a stale Worker must not release or reassign ownership that may belong to a newer Attempt.

This is a repeated last-safe-point guard, not a distributed lock. GitHub multi-operation atomicity is not claimed. If authority becomes stale after an earlier terminal mutation, that earlier write remains append-only history but all later status/owner writes must stop. Coordinator Final Acceptance/close remains governed by the existing Final Acceptance Gate.

## Normal Attempt completion

Before leaving the Attempt:

1. commit/push or otherwise persist in-scope candidate changes when required;
2. confirm the final Task Candidate / Integration Candidate / PR identities supersede any early checkpoint identity as appropriate;
3. collect the Evidence required by `task.md` and the latest Coordinator Review;
4. prepare the complete `[EXECUTION REPORT]` payload, including the real Attempt number, base/task-candidate/integration-candidate SHA as applicable, PR, Claim results, Jobs/commands, execution host, Runner/Target, Evidence, problems, freshness identities, and unverified scope, but do not post it yet;
5. freshly re-read the Issue and apply the terminal-write authority guard; if it rejects, STOP with zero terminal Issue mutations;
6. only after guard PASS, post the `[EXECUTION REPORT]`;
7. freshly re-read and reapply the guard expecting `status:in-progress` before changing status; only after PASS transition the Issue to `status:review`;
8. freshly re-read and reapply the guard expecting `status:review` before ownership mutation; only after PASS release active execution ownership;
9. re-read the Issue to verify report + status are durable;
10. stop.

Worker execution outcome is not Coordinator acceptance.

Do not set `status:done`, close the Issue, or immediately start Attempt N+1.

## Blocked Attempt

If a required permission, GitHub capability, device, Runner, Secret-at-runtime, network condition, dependency, or target capability is unavailable:

1. preserve a safe state;
2. preserve/reuse any existing durable branch / commit / draft PR / Evidence anchor;
3. clean temporary resources when required;
4. prepare the complete `[BLOCKER REPORT]` payload, including exactly what was completed, where execution stopped, Evidence, minimal resume condition, cleanup/safe state, and reusable durable anchor, but do not post it yet;
5. freshly re-read the Issue and apply the terminal-write authority guard; if it rejects, STOP with zero terminal Issue mutations;
6. only after guard PASS, post the `[BLOCKER REPORT]`;
7. freshly re-read and reapply the guard expecting `status:in-progress` before changing status; only after PASS transition to `status:blocked`;
8. if ownership should be released, freshly re-read and reapply the guard expecting `status:blocked` before ownership mutation; only after PASS release active execution ownership;
9. re-read the Issue to verify the report/status;
10. stop.

Never bypass a security boundary or lower Success Criteria to avoid BLOCKED.

## After a REVISE

When a Coordinator has posted `Decision: REVISE` and returned the same Task to `status:ready`:

- read the previous Attempt report and Coordinator Review;
- read the revision class (`IMPLEMENTATION | EVIDENCE | SEMANTIC_FRESHNESS | INTEGRATION_ONLY`) when present;
- confirm whether the Contract is unchanged;
- inspect and reuse the previous durable branch/PR when it remains valid;
- begin a new Attempt only after a fresh claim;
- execute only the required revision class;
- re-run the verification required by the current Task Contract/Coordinator Review, not mechanically every previous job when integration protocol explicitly preserves semantic Evidence.

## Completion output to the user

After the durable Issue update, summarize only:

```text
Issue: #<issue>
Attempt: <N>
Execution outcome: COMPLETED | PARTIAL | FAILED | BLOCKED
Issue state: review | blocked
Task Candidate: <sha or n/a>
Integration Candidate: <sha or n/a>
Report: posted
Next authority: Web Coordinator
```

The Issue plus durable branch/PR/Evidence anchor is the recoverable handoff. Chat text is not the state authority.
