---
name: task-reviewer
description: Review one jellyfin-web-media-gateway Task Attempt from its GitHub Issue and Evidence, recover interrupted in-progress ownership when needed, then decide ACCEPT, REVISE, BLOCK, SPLIT, or NOT_PLANNED. Use only when explicitly asked to review, iterate, recover, unblock, or close a task; do not execute the Worker implementation itself.
---

# Task Reviewer

Perform the Coordinator-side Review / Recovery / Iteration / Closure workflow for one Task.

## Authority

Before deciding anything, read:

1. `AGENTS.md`
2. target GitHub Issue and relevant comment history
3. Task Package `task.md`
4. Task Package `prompt.md`
5. `docs/tasks/issue-lifecycle-protocol.md`
6. `docs/tasks/execution-anchor-recovery-protocol.md`
7. `docs/tasks/handoffs/README.md`
8. the handoff profile(s) for the Task's current eligible environment(s)
9. relevant canonical docs
10. actual candidate commit / PR
11. all required Actions runs, artifacts, target Evidence, or linked child Task Evidence

Do not accept a result from chat summary alone.

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
Candidate SHA / PR is identified when required
required Claim Evidence is resolvable
Task Contract revision being reviewed is known
```

Keep distinct:

```text
Worker execution outcome
!= Verification claim result
!= Coordinator Task decision
!= Parent Goal / Research Gate decision
```

A draft PR or early checkpoint is only a recovery anchor; it is not automatically the final Candidate or accepted Evidence.

## Evaluate against frozen Contract

For every Task Success Criterion and required Claim, classify the current Evidence.

Never lower Success Criteria after seeing results merely to manufacture PASS. Missing required runtime/target Evidence remains missing.

## Coordinator decision

Post `[COORDINATOR REVIEW]` using `docs/tasks/issue-lifecycle-protocol.md` and choose exactly one:

```text
ACCEPT | REVISE | BLOCK | SPLIT | NOT_PLANNED
```

### REVISE

Use when Contract is still correct but implementation/candidate/Evidence is incomplete or failed.

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

Do not split merely because different Runner/Target environments are involved.

Post `[SPLIT]` on parent Issue and publish child Task(s) through `task-publisher`.

### Contract / bootstrap revision

If Scope, Claims, Success Criteria, decomposition, Evidence Authority, architecture/security premise, or task-specific bootstrap must change, do not encode the change only in Issue comments.

Use:

```text
status:draft when needed to make package non-claimable
→ update canonical docs when required
→ update task.md for Contract changes
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
+ all required Claims accepted
+ required Verification Evidence reviewed
+ required Candidate / PR accepted
+ no unresolved blocker
+ no required linked child Task still open
```

Then:

1. post `[COORDINATOR REVIEW]` with `Decision: ACCEPT`;
2. post `[FINAL ACCEPTANCE]`;
3. set `status:done`;
4. close Issue as completed;
5. re-read Issue to confirm final comment/state/closure.

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
Issue state: <state>
Contract changed: yes | no
Next action: <close | downstream handoff(s) | unblock | child task>
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
