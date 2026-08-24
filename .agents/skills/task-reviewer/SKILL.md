---
name: task-reviewer
description: Review one jellyfin-web-media-gateway Task Attempt from its GitHub Issue and Evidence, then decide ACCEPT, REVISE, BLOCK, SPLIT, or NOT_PLANNED. Use only when explicitly asked to review, iterate, unblock, or close a task; do not execute the Worker implementation itself.
---

# Task Reviewer

Perform the Coordinator-side Review/Iteration/Closure workflow for one Task.

## Authority

Before deciding anything, read:

1. `AGENTS.md`
2. the target GitHub Issue and relevant comment history
3. the Task Package `task.md`
4. the Task Package `prompt.md`
5. `docs/tasks/issue-lifecycle-protocol.md`
6. relevant canonical docs
7. the actual candidate commit / PR
8. all required Actions runs, artifacts, target Evidence, or linked child Task Evidence

Do not accept a result from chat summary alone.

## GitHub capability

Use an authenticated GitHub read/write path available in the current Codex environment, such as a connected GitHub tool or authenticated `gh` CLI.

If the required Issue history or Evidence cannot be read, the review is BLOCKED. Do not infer a PASS.

## Identify the review unit

Review the latest completed/blocked Attempt that has a durable Issue report.

Confirm:

```text
Attempt N exists
Worker report is durable
Candidate SHA / PR is identified when required
required Claim Evidence is resolvable
Task Contract revision being reviewed is known
```

Keep these distinct:

```text
Worker execution outcome
!= Verification claim result
!= Coordinator Task decision
!= Parent Goal / Research Gate decision
```

## Evaluate against the frozen Contract

For every Task Success Criterion and required Claim, classify the current Evidence.

Never lower a Success Criterion after seeing the result merely to create PASS.

Treat missing required runtime/target Evidence as missing, not as theoretical success.

## Coordinator decision

Post a `[COORDINATOR REVIEW]` comment using the exact structure in `docs/tasks/issue-lifecycle-protocol.md`.

Choose exactly one:

```text
ACCEPT | REVISE | BLOCK | SPLIT | NOT_PLANNED
```

### REVISE

Use when the Task Contract is still correct but the implementation/candidate/Evidence is incomplete or failed.

Examples:

- implementation bug;
- missed existing requirement;
- failed test;
- insufficient Evidence;
- same Claim needs re-verification.

Then:

```text
task.md unchanged
prompt.md unchanged
→ comment [COORDINATOR REVIEW] Decision: REVISE
→ replace status with status:ready
→ release active owner
→ verify the target Worker queue can see the Task
→ output a fresh downstream entry for Attempt N+1
```

Prefer the skill-based handoff:

```text
$task-worker Execute Issue #<issue> using `docs/tasks/<issue>-<slug>/prompt.md`.
```

Do not create a new Issue for an ordinary retry.

### BLOCK

Use when a required external condition/capability is missing and retrying now cannot make progress.

Then:

```text
comment [COORDINATOR REVIEW] Decision: BLOCK
→ status:blocked
→ record the minimal unblock condition
```

When the blocker is resolved, post `[COORDINATOR UNBLOCK]`. If the Contract is unchanged, return to `status:ready`, verify queue visibility, and output the next `$task-worker` handoff.

If resolving the blocker changes the Task Contract, follow Contract revision instead.

### SPLIT

Use only when new work has an independent Scope, lifecycle/owner, Success Criteria, Evidence Authority, or deliverable.

Do not split merely because different Runner/Target environments are involved.

Post `[SPLIT]` on the parent Issue and identify whether the parent is blocked by the child.

Create/publish each child by following the `task-publisher` workflow. Prefer explicit invocation when available:

```text
$task-publisher Publish the child Task required by Issue #<parent>.
```

Evidence from required child Tasks must return to the parent before final acceptance.

### Contract revision

If Review shows that Scope, Claims, Success Criteria, Task decomposition, Evidence Authority, or an architecture/security premise must change, do not encode the new contract only in an Issue comment.

Use:

```text
status:draft
→ update canonical docs when required
→ update task.md
→ update prompt.md only if bootstrap changed
→ perform the task-publisher read-back/publication gate
→ status:ready
→ queue verify
→ output a new downstream entry
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
2. post `[FINAL ACCEPTANCE]` using the canonical template;
3. replace task status with `status:done`;
4. close the Issue with completed reason;
5. re-read the Issue to confirm the final comment, state, and closure are durable.

A closed Task does not automatically make its Parent Goal / Research Gate PASS.

### NOT_PLANNED

Use only when the Coordinator intentionally terminates the Task without claiming success.

Post a `[COORDINATOR REVIEW]` with rationale and identify the Parent Goal impact. Close using a not-planned reason when the available GitHub capability supports it. Do not post `[FINAL ACCEPTANCE]` and do not represent the Task as `status:done`/successful.

## Reopen

If new Evidence directly contradicts previously accepted Success Criteria:

- reopen the Issue;
- post `[COORDINATOR REOPEN]`;
- identify which accepted Evidence is invalidated;
- choose `status:ready`, `status:blocked`, or `status:draft` depending on whether Contract revision is required;
- resume through the same lifecycle instead of creating an unrelated duplicate Task.

## Completion output

Return the durable decision and next action:

```text
Issue: #<issue>
Reviewed Attempt: <N>
Decision: ACCEPT | REVISE | BLOCK | SPLIT | NOT_PLANNED
Issue state: <state>
Contract changed: yes | no
Next action: <close | downstream worker entry | unblock | child task>
```

If the Task returns to `status:ready`, always include the directly copyable `$task-worker` handoff.