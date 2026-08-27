# Coordinator Bootstrap — INFRA-003 Target Runner Post-#85 Recovery Verification

This Issue is a Coordinator verification record for the distinct 2026-08-26 Target Runner incident that blocked #85 J4.

Read first:

1. Issue #87 and comments
2. `docs/tasks/87-infra-target-runner-post-85-recovery/task.md`
3. #85 Attempt 1 Blocker Report
4. #21 `[CORRECTION]` plus the misrouted 2026-08-26 recovery Evidence
5. `docs/tasks/issue-lifecycle-protocol.md`
6. `docs/runner-execution-architecture.md`
7. `docs/security.md`

Frozen incident:

```text
#85 workflow run: 32972479047
J4 job: 98189229745
runner: ubuntu-arm64-target-phone / id 2
exact #85 Candidate: 98b9ca30636c3c4edbfe71841ebf883d685d138f
symptom: assigned + in_progress + steps=[] until cancellation
```

Goal:

```text
fresh trusted target-runner smoke
→ normal steps/logs
→ ARM64 + uid 999 gateway-runner
→ isolated workspace + cleanup
→ runner online/idle
```

Do not modify #85, rerun Bilibili, start #67, run R003, inspect Secrets, rotate credentials or re-register the runner.

If fresh smoke PASSes, Coordinator may Final Accept #87 and then retry only #85 J4 on the same Candidate. If smoke repeats the no-step hang, BLOCK #87 and stop repeated retries.
