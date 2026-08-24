---
name: task-worker
description: Claim and execute one published jellyfin-web-media-gateway Task Attempt, then report the result back to its GitHub Issue. Use only when explicitly asked to execute a ready Task; do not publish, review, accept, or close tasks.
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
6. every canonical/topic document explicitly required by `task.md`

If this skill conflicts with those sources, follow the higher-authority repository source.

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

## Execute only the Task Contract

Follow `task.md` exactly:

- Scope / Out of Scope;
- Architecture Invariants;
- Implementation Requirements;
- Claims / Verification Job Matrix;
- Success Criteria;
- Evidence Contract;
- Failure / Blocked Rules.

Do not silently change Success Criteria, add unrelated cleanup, start a different Task, or reinterpret a Runner/Target as a new business Task.

Use the execution plane and target required by the Claim. Generic ARM64 evidence is not phone-target evidence; interactive diagnosis is not automatically Verification PASS.

## Operator privilege versus final runtime privilege

For infrastructure bootstrap Tasks, distinguish the bootstrap operator from the service being created.

An operator may legitimately require privileged setup steps when the Task explicitly permits them. That does not allow the final service/Runner to violate its low-privilege security contract. Verify final runtime identity and access boundaries separately.

## Normal Attempt completion

Before leaving the Attempt:

1. commit/push or otherwise persist in-scope candidate changes when required;
2. collect the Evidence required by `task.md`;
3. comment the current Issue using the exact `[EXECUTION REPORT]` structure from `docs/tasks/issue-lifecycle-protocol.md`;
4. include the real Attempt number, base/candidate SHA, PR when applicable, Claim results, Jobs/commands, execution host, Runner/Target, Evidence, problems, and unverified scope;
5. transition the Issue to `status:review`;
6. release active execution ownership;
7. re-read the Issue to verify report + status are durable;
8. stop.

Worker execution outcome is not Coordinator acceptance.

Do not set `status:done`, close the Issue, or immediately start Attempt N+1.

## Blocked Attempt

If a required permission, GitHub capability, device, Runner, Secret-at-runtime, network condition, dependency, or target capability is unavailable:

1. preserve a safe state;
2. clean temporary resources when required;
3. comment the Issue using `[BLOCKER REPORT]` from `docs/tasks/issue-lifecycle-protocol.md`;
4. state exactly what was completed, where execution stopped, Evidence, minimal resume condition, and cleanup/safe state;
5. transition to `status:blocked`;
6. release active execution ownership unless the repository explicitly requires ownership for blocker recovery;
7. re-read the Issue to verify the report/status;
8. stop.

Never bypass a security boundary or lower Success Criteria to avoid BLOCKED.

## After a REVISE

When a Coordinator has posted `Decision: REVISE` and returned the same Task to `status:ready`:

- read the previous Attempt report and Coordinator Review;
- confirm whether the Contract is unchanged;
- begin a new Attempt only after a fresh claim;
- fix the accepted missing/failed items;
- re-run all verification required by the current Task Contract, not only the single command that failed, when the Contract requires broader regression evidence.

## Completion output to the user

After the durable Issue update, summarize only:

```text
Issue: #<issue>
Attempt: <N>
Execution outcome: COMPLETED | PARTIAL | FAILED | BLOCKED
Issue state: review | blocked
Candidate: <sha or n/a>
Report: posted
Next authority: Web Coordinator
```

The Issue is the recoverable handoff. Chat text is not the state authority.