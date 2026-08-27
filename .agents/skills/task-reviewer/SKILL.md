---
name: task-reviewer
description: Review one jellyfin-web-media-gateway Task Attempt from its GitHub Issue and Evidence, recover interrupted in-progress ownership when needed, classify dependency-aware freshness/integration state, then decide ACCEPT, REVISE, BLOCK, SPLIT, or NOT_PLANNED. Use only when explicitly asked to review, iterate, recover, unblock, or close a task; do not execute the Worker implementation itself.
---

# Task Reviewer

Perform the Coordinator-side Review / Recovery / Freshness / Integration / Iteration / Closure workflow for one Task.

## Authority

Before deciding anything, read:

1. `AGENTS.md`
2. target GitHub Issue and relevant comment history
3. Task Package `task.md`
4. Task Package `prompt.md`
5. `docs/tasks/issue-lifecycle-protocol.md`
6. `docs/tasks/execution-anchor-recovery-protocol.md`
7. `docs/tasks/freshness-integration-protocol.md`
8. `docs/tasks/handoffs/README.md`
9. the handoff profile(s) for the Task's current eligible environment(s)
10. relevant canonical docs
11. actual candidate commit / PR
12. all required Actions runs, artifacts, target Evidence, or linked child Task Evidence
13. live current `main` and compare/patch evidence needed for freshness classification

Do not accept a result from chat summary alone.

For a Task published before the freshness protocol was adopted, do not retroactively lower a stricter frozen Task Contract. Existing explicit strict-main/current-main requirements remain authoritative until formal Contract Revision.

## In-progress recovery is not Review

A Task that is still `status:in-progress` without a completed Worker report is not a normal Review unit.

If the active Worker/session is known to have terminated or ownership is otherwise concretely stale, follow `docs/tasks/execution-anchor-recovery-protocol.md` instead of fabricating an `[EXECUTION REPORT]` or Coordinator Review.

Before recovery inspect:

```text
Issue + comments
worker branch / commits
draft/open PR
Actions runs/jobs/artifacts
current main
current Task Contract
```

Elapsed wall-clock time alone is not sufficient proof that ownership is stale. Missing PR/checkpoint alone is also not proof.

When recovery is justified:

1. post `[COORDINATOR RECOVERY]` with the interrupted Attempt N and durable anchor found;
2. preserve existing branch/PR/Evidence when reusable;
3. release stale active ownership;
4. choose `status:ready`, `status:draft`, or `status:blocked` according to the unchanged/changed Contract and external conditions;
5. if returning to ready, the next claim starts **Attempt N+1**;
6. instruct the replacement Worker to reuse the durable branch/PR rather than starting over;
7. verify the target Worker queue and emit the correct environment-specific handoff.

Do not assign a replacement Worker into the same interrupted Attempt number, and do not create a duplicate business Issue merely because the Worker changed.

Attempts that were already in progress before the execution-anchor protocol was introduced are not noncompliant merely because they lack `[EXECUTION CHECKPOINT]`.

## Review unit

Review the latest completed/blocked Attempt that has a durable Issue report.

Confirm:

```text
Attempt N exists
Worker report is durable
Task Candidate SHA / PR is identified when required
required Claim Evidence is resolvable
Task Contract revision being reviewed is known
Freshness policy is known
```

Keep distinct:

```text
Worker execution outcome
!= Verification claim result
!= Freshness classification
!= Integration Gate result
!= Coordinator Task decision
!= Parent Goal / Research Gate decision
```

A draft PR or early checkpoint is only a recovery anchor; it is not automatically the final Candidate or accepted Evidence.

## Evaluate against frozen Contract

For every Task Success Criterion and required Claim, classify the current Evidence.

Never lower Success Criteria after seeing results merely to manufacture PASS. Missing required runtime/target Evidence remains missing.

First evaluate Task semantics on the exact Task Candidate. Then evaluate current-main freshness separately. Do not mix “the implementation is wrong” with “the implementation is semantically accepted but needs composition proof”.

## Freshness Review

For Tasks using `Freshness policy: dependency-aware`, before the final decision:

1. identify Task Candidate SHA;
2. identify Evidence Base / accepted main snapshot actually included when required semantic Evidence ran;
3. live-read Current Main SHA;
4. compare Evidence Base → Current Main;
5. inspect changed files/patches when path-only classification is insufficient;
6. classify using `docs/tasks/freshness-integration-protocol.md`:

```text
NONE
UNRELATED
INTEGRATION_OVERLAP
SEMANTIC_AUTHORITY
CONTRACT_INVALIDATING
```

Record in `[COORDINATOR REVIEW]`:

```text
Task Candidate:
Evidence Base:
Current Main:
Freshness policy:
Freshness classification:
Changed main surface reviewed:
Semantic Evidence reuse: yes | no | partial
Affected Claims requiring reverify:
Integration Gate required: yes | no
Integration Base / Candidate: <sha or n/a>
Integration Evidence: <jobs or n/a>
```

### NONE / UNRELATED

Do not require rebase/full rerun solely because `main` advanced.

Existing exact-Candidate semantic Evidence remains valid when the delta is truly unrelated to the declared semantic/integration surfaces.

A clean, mergeable PR may proceed toward final merge with expected-head protection.

### INTEGRATION_OVERLAP

If Task-specific semantic review otherwise passes:

- preserve accepted semantic Evidence;
- do not demand full J1/J2/... rerun;
- open `[INTEGRATION GATE]` using the protocol;
- freeze `Integration Base = live current main`;
- specify only the declared `JI*` jobs;
- return Task to `status:ready` as **Revision class: INTEGRATION_ONLY**;
- require reuse of the existing Issue/branch/PR.

This is a small new Attempt for auditable ownership/recovery, not a re-opening of accepted semantic Claims.

While the Integration Slot is open, do not merge another Task that touches the protected integration surfaces. Unrelated merges may proceed and do not invalidate the slot.

### SEMANTIC_AUTHORITY

Require integration/reconciliation with the new accepted authority and rerun only mapped affected Claims/Jobs when the Task Freshness Contract supports a safe bounded mapping.

If impact cannot be bounded safely, expand verification conservatively and explain why.

If semantic changes alter Scope/Claims/Success Criteria/Evidence Authority, escalate to Contract Revision instead of ordinary retry.

### CONTRACT_INVALIDATING

Use:

```text
status:draft
→ Contract Revision / Publication Gate
```

Do not disguise it as freshness-only REVISE.

### strict-main

If the frozen Task Contract explicitly uses `Freshness policy: strict-main`, enforce it exactly. The dependency-aware default cannot be used after the fact to lower that Task's published requirement.

## Reviewing an Integration-only Attempt

When the previous Coordinator decision established `[INTEGRATION GATE]` / `Revision class: INTEGRATION_ONLY`, verify:

```text
Original Task Candidate is preserved/identified
Integration Base matches the frozen gate SHA
Integration Candidate includes Task Candidate ancestry (merge preferred)
no semantic/task-owned conflict changed the accepted Task implementation
required JI jobs ran on exact Integration Candidate
JI results PASS
```

If clean integration preserves Task semantics, reuse the earlier semantic Evidence and do not rerun unrelated Claims.

If conflict resolution or integration changes Task-owned semantic implementation, reclassify as `SEMANTIC_AUTHORITY` and require affected Claim re-verification.

Immediately before merge, re-read Current Main:

- if it only advanced with `UNRELATED` changes, the Integration Slot remains valid;
- if an overlapping/semantic change was merged, reclassify against the new main before accepting.

## Coordinator decision

Post `[COORDINATOR REVIEW]` using `docs/tasks/issue-lifecycle-protocol.md` and choose exactly one:

```text
ACCEPT | REVISE | BLOCK | SPLIT | NOT_PLANNED
```

### REVISE

Use when Contract is still correct but implementation/candidate/Evidence/freshness composition is incomplete.

Set an explicit revision class when useful:

```text
IMPLEMENTATION
EVIDENCE
SEMANTIC_FRESHNESS
INTEGRATION_ONLY
```

Then:

```text
task.md unchanged
prompt.md unchanged unless bootstrap itself is wrong
→ post Decision: REVISE
→ status:ready + existing eligible env(s)
→ release active owner
→ verify each eligible Worker queue
→ emit fresh environment-specific handoff(s) for Attempt N+1
```

Do not create a new Issue for an ordinary retry.

When an existing branch/PR remains valid, explicitly tell Attempt N+1 to reuse it. Do not restart from scratch solely because the Worker/session changed.

For `INTEGRATION_ONLY`, explicitly preserve the previously accepted semantic Claims and list only the JI work required.

Select next-entry profile from `docs/tasks/handoffs/`:

```text
env:web-gpt       → web-gpt.md
env:ubuntu-arm64  → ubuntu-arm64.md
env:wsl           → wsl.md
env:windows       → windows.md
env:cloud         → cloud.md
env:manual-tv     → manual-tv.md
```

A Web Task must receive the Web `@GitHub` copy block, not `$task-worker`. Codex environments use their own Codex handoff profile.

If multiple environments remain eligible, output one independent copy block per environment.

### BLOCK

Use when a required external condition/capability is missing and retrying now cannot make progress.

```text
post Decision: BLOCK
→ status:blocked
→ record minimal unblock condition
```

When resolved, post `[COORDINATOR UNBLOCK]`.

If Contract/bootstrap is unchanged:

```text
status:ready + eligible env(s)
→ verify each queue
→ emit environment-specific handoff(s)
```

If resolving blocker changes Contract/bootstrap, republish through `task-publisher` rules first.

### SPLIT

Use only when new work has an independent Scope, lifecycle/owner, Success Criteria, Evidence Authority, or deliverable.

Do not split merely because different Runner/Target environments are involved or because integration verification is needed.

Post `[SPLIT]` on parent Issue and publish child Task(s) through `task-publisher`.

### Contract / bootstrap revision

If Scope, Claims, Success Criteria, decomposition, Evidence Authority, architecture/security premise, Freshness Contract, or task-specific bootstrap must change, do not encode the change only in Issue comments.

Use:

```text
status:draft when needed to make package non-claimable
→ update canonical/process docs when required
→ update task.md for Contract/freshness changes
→ update prompt.md for bootstrap changes
→ task-publisher read-back/publication gate
→ status:ready + eligible env(s)
→ per-environment queue verify
→ fresh environment-specific handoff(s)
```

### ACCEPT

Use only when Final Acceptance Gate is satisfied:

```text
Task Success Criteria accepted
+ all required semantic Claims accepted
+ required Verification Evidence reviewed
+ freshness classification resolved
+ required Integration Gate PASS when applicable
+ required Candidate / PR accepted
+ no unresolved blocker
+ no required linked child Task still open
```

Before merge, verify exact expected PR head.

Final Acceptance should record both identities when they differ:

```text
Accepted Task Candidate: <sha>
Accepted Integration Candidate: <sha or same/n/a>
Freshness classification at merge: <...>
Integration Base: <sha or n/a>
```

Then follow the repository's normal exact-head merge/read-back/Final Acceptance sequence.

A closed Task does not automatically make its Parent Goal / Research Gate PASS.

### NOT_PLANNED

Post Coordinator Review with rationale and Parent Goal impact. Close as not planned when supported. Do not post `[FINAL ACCEPTANCE]` and do not represent it as successful `status:done`.

## Reopen

If new Evidence contradicts previously accepted Success Criteria:

- reopen Issue;
- post `[COORDINATOR REOPEN]`;
- identify invalidated Evidence;
- choose `status:ready`, `status:blocked`, or `status:draft` according to whether Contract/bootstrap revision is needed;
- resume through the same lifecycle.

If reopening returns the Task to ready, verify eligible queues and emit environment-specific handoff(s).

## Completion output

For normal Review, return durable decision and next action:

```text
Issue: #<issue>
Reviewed Attempt: <N>
Decision: ACCEPT | REVISE | BLOCK | SPLIT | NOT_PLANNED
Freshness: NONE | UNRELATED | INTEGRATION_OVERLAP | SEMANTIC_AUTHORITY | CONTRACT_INVALIDATING | strict-main
Issue state: <state>
Contract changed: yes | no
Next action: <close | integration-only handoff | downstream handoff(s) | unblock | child task>
```

For interrupted execution recovery, return:

```text
Issue: #<issue>
Interrupted Attempt: <N>
Recovery: recorded
Durable anchor: <branch/commit/PR/evidence or none>
Issue state: ready | draft | blocked
Next Attempt: <N+1 or n/a>
Next action: <handoff | contract revision | unblock>
```

Whenever the Task returns to `status:ready`, always include one directly copyable handoff block for every currently eligible environment.
