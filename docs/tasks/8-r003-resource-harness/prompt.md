# Session Bootstrap — R003-PREP Resource Harness

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 R003-PREP Task。

本文件只是 bootstrap/navigation，不是 Task Contract，也不保存实时 Task 状态。

## Execution Context

```text
GitHub Issue: #8
Task Contract: docs/tasks/8-r003-resource-harness/task.md
Expected worker: cloud-codex
Expected environment label after publication: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Research Item: R003
Linked target verification: Issue #9
Accepted R001 Candidate: 42c92db2a380895ec3909cdc9afa847478150eb0
R001 merged main commit: 2b0a1a0ea95753ff416e41759b7c33823be1b9e0
```

## Preferred Codex Entry

```text
$task-worker Execute Issue #8 using `docs/tasks/8-r003-resource-harness/prompt.md`.
```

Skill 不可见时按 `docs/tasks/handoffs/cloud.md` fallback 执行。

## Live Gate

Codex Worker 必须先实际读取 Issue #8：

```text
status:draft
→ STOP，不 claim、不实现、不自行发布

status:ready + env:cloud + no active owner
→ 可以 claim，开始 Attempt N

其他状态
→ 按 docs/tasks/issue-lifecycle-protocol.md 停止或交回 Coordinator
```

R001 publication dependency 已由 Coordinator 通过 Issue #3 Final Acceptance 解除。Worker 仍必须从 GitHub 读取 Issue #3 的 accepted Candidate/merge state，不得根据本 prompt 猜测动态状态。

## Start Protocol

1. 必须使用 GitHub 读取 Issue #8、relevant comments、`AGENTS.md`、本 prompt、`task.md`、`docs/tasks/issue-lifecycle-protocol.md`。
2. 读取 `docs/technical-feasibility-validation.md` 的 R003、`docs/runner-execution-architecture.md`、`docs/mvp-plan.md`，以及 Issue #3 Final Acceptance / accepted R001 Evidence。
3. 确认 live state 为 `status:ready + env:cloud` 且无 owner；确认当前 main 仍包含 accepted R001/R007 authority。
4. claim → `status:in-progress` → Attempt N。
5. 只实现 metrics/scenario harness + 受信 target workflow；不得宣称真实手机 R003 PASS/FAIL。
6. target workflow 必须 manual/trusted-candidate only；不得让 untrusted PR 自动命中 self-hosted phone Runner。
7. required hosted Evidence 通过 GitHub Actions 产生并绑定 exact Candidate SHA；Codex 本地测试仅用于开发/诊断。
8. hosted CI 只证明 harness/workflow mechanics；不要用 hosted ARM64 冒充 phone metrics。
9. 正常结束 `[EXECUTION REPORT]` → `status:review` → release owner → STOP；阻塞则 `[BLOCKER REPORT]` → `status:blocked` → release owner → STOP。

## Trust Reminder

必须分别记录：

```text
Harness / workflow SHA
Measured candidate SHA
```

Issue #9 后续只能使用 Coordinator 接受/合并后的受信 workflow/harness；不要从被测 candidate 静默替换测量逻辑。

## Stop Boundary

完成当前 R003-PREP Attempt 后立即停止。Worker 不得自行执行 Issue #9，不得自行 `status:done` 或关闭 Issue #8。