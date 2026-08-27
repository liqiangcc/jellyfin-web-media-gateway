# Task — INFRA-003 Target Runner Post-#85 Recovery Verification

## Metadata

```text
GitHub Issue: #87
Task ID: INFRA-003-TARGET-RUNNER-POST-85-RECOVERY
Task kind: coordinator verification / incident follow-up
Incident: #85 J4 run 32972479047 / job 98189229745
Runner: id 2 / ubuntu-arm64-target-phone
Incident shape: assigned + in_progress + steps=[] until cancellation
Planning Base: 913c8af37623917a72c0936f4e6148195f15d54c
Downstream Candidate: #85 98b9ca30636c3c4edbfe71841ebf883d685d138f
Historical recovery authority: #21 remains Final Accepted for the separate 2026-08-24 incident
```

> This Task records the distinct 2026-08-26 runner incident. It does not reopen or rewrite the historical success of #21.

## Governance / imported recovery Evidence

The Coordinator initially misrouted the 2026-08-26 recovery action into a reopened #21. That governance decision was corrected and #21 restored to its original completed state.

The operational observations collected during that misrouted execution remain valid evidence for this incident:

- existing runner id/configuration preserved;
- `gateway-runnerctl restart` used;
- old supervisor/helper/listener PIDs removed;
- no stale `Runner.Worker` remained;
- new Listener returned as uid 999 `gateway-runner` in `/home/gateway-runner/actions-runner`;
- no sudo/admin expansion, re-registration or credential rotation;
- GitHub API reported runner id 2 online/idle on repeated reads.

These observations prove a bounded recovery action occurred. They do **not** by themselves prove current post-recovery schedulability.

## Goal

Use a fresh trusted target-runner smoke to prove the recovered existing runner can again execute a normal GitHub Actions job with accepted security/workspace boundaries.

Required path:

```text
recovered runner id 2
→ fresh trusted target-runner smoke
→ normal step/log visibility
→ runner ubuntu-arm64-target-phone / ARM64
→ uid 999 gateway-runner
→ isolated _work workspace
→ temporary workspace cleanup PASS
→ no sudo/admin expansion
→ runner returns online/idle
```

## Verification Authority

Use the existing trusted `target-runner-smoke` workflow/job authority that previously established accepted target schedulability. A fresh rerun is sufficient if it uses the same trusted workflow identity and reports current runner execution.

The Coordinator owns this immediate smoke; no new product/runtime implementation is permitted here.

## Claims

```text
C1 — The 2026-08-26 #85 J4 incident has its own durable identity and is not folded into #21.
C2 — Existing runner identity/configuration remains preserved after recovery.
C3 — Fresh smoke is actually executed by ubuntu-arm64-target-phone and exposes normal steps/logs.
C4 — Runtime remains uid 999 gateway-runner / non-root / accepted isolated workspace.
C5 — Smoke cleanup succeeds and no stale task process/workspace remains.
C6 — Runner returns online/idle/claimable after the smoke.
C7 — No #85 code, #67 execution, product Secret, credential rotation, or security weakening occurs.
```

## Success Criteria

1. A fresh trusted target-runner smoke reaches a terminal result.
2. The job is assigned to `ubuntu-arm64-target-phone` and exposes normal completed steps; `steps=[]` after assignment is not accepted.
3. ARM64/aarch64 and uid 999 `gateway-runner` identity are confirmed.
4. The accepted `_work` isolation and temporary create/cleanup proof pass.
5. sudo/admin expansion remains absent; no credential/re-registration change occurs.
6. Runner is online/idle after completion when observable.
7. If the smoke again hangs after assignment with no steps, classify BLOCKED and stop rather than loop retries.
8. PASS only restores downstream permission to retry #85 J4 on the same exact Candidate; it does not accept #85 itself.

## Frozen Boundaries

- no product or #85 code changes;
- no Bilibili/site request;
- no #67 Attempt 5;
- no R003/performance workload;
- no root/sudo capability granted to `gateway-runner`;
- no Secret file inspection or credential rotation;
- no new runner registration unless future evidence separately proves accepted identity corruption.

## Result Routing

### PASS

```text
fresh smoke PASS
→ Coordinator Final Accept #87
→ retry #85 J4 only
→ same Candidate 98b9ca30636c3c4edbfe71841ebf883d685d138f
```

### BLOCKED

If the fresh smoke again becomes assigned/in_progress with no steps/logs, or the runner cannot execute normally:

```text
#87 BLOCKED
→ preserve incident evidence
→ do not retry #85 J4 repeatedly
→ plan a new bounded runner-lifecycle repair based on the new evidence
```

## Completion

This is a Coordinator verification record rather than a product implementation Task. The Coordinator records the smoke Evidence directly in #87 and then ACCEPTs/closes or BLOCKs it. No Worker should modify #85 or start #67 from this Task.
