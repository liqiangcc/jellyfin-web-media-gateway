# Session Bootstrap — R003-PREP Resource Harness

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 R003-PREP Task。

本文件只是 bootstrap/navigation，不是 Task Contract，也不保存实时 Task 状态。

## Execution Context

```text
GitHub Issue: #8
Task Contract: docs/tasks/8-r003-resource-harness/task.md
Expected worker: web
Expected environment label after publication: env:web-gpt
Downstream handoff profile: docs/tasks/handoffs/web-gpt.md
Research Item: R003
Linked target verification: Issue #9
```

## Live Gate

新 Web Worker 必须先实际读取 Issue #8：

```text
status:draft
→ STOP，不 claim、不实现、不自行发布

status:ready + env:web-gpt + no active owner
→ 可以 claim，开始 Attempt N
```

Publication 前 Coordinator 必须在 Issue #8 明确记录可供 R003 harness 集成的 R001 candidate/interface SHA。仅存在仍在变化的 open PR 不足以解除 hard dependency。

## Start Protocol

1. 必须使用 GitHub 读取 Issue #8、relevant comments、`AGENTS.md`、本 prompt、`task.md`、`docs/tasks/issue-lifecycle-protocol.md`。
2. 读取 `docs/technical-feasibility-validation.md` 的 R003、`docs/runner-execution-architecture.md`、`docs/mvp-plan.md`，以及 Coordinator 指定的 R001 candidate/Issue #3 Evidence。
3. 确认 live state 为 `status:ready + env:web-gpt` 且无 owner；确认 R001 dependency 已被 Coordinator 明确解除。
4. claim → `status:in-progress` → Attempt N。
5. 只实现 metrics/scenario harness + 受信 target workflow；不得宣称真实手机 R003 PASS/FAIL。
6. target workflow 必须 manual/trusted-candidate only；不得让 untrusted PR 自动命中 self-hosted phone Runner。
7. hosted CI 只证明 harness/workflow mechanics；不要用 hosted ARM64 冒充 phone metrics。
8. 正常结束 `[EXECUTION REPORT]` → `status:review` → release owner → STOP；阻塞则 `[BLOCKER REPORT]` → `status:blocked` → release owner → STOP。

## Trust Reminder

必须分别记录：

```text
Harness / workflow SHA
Measured candidate SHA
```

Issue #9 后续只能使用 Coordinator 接受/合并后的受信 workflow/harness；不要从被测 candidate 静默替换测量逻辑。

## Stop Boundary

完成当前 R003-PREP Attempt 后立即停止。Worker 不得自行执行 Issue #9，不得自行 `status:done` 或关闭 Issue #8。