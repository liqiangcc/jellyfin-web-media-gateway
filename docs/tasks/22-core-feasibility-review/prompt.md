# Session Bootstrap — Web-only Core Feasibility Review

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 Issue #22 / `CORE-FEASIBILITY-REVIEW`。

本文件只是 bootstrap/navigation，不是 Task Contract，也不保存动态 Gate 结果。

## Execution Context

```text
GitHub Issue: #22
Task Contract: docs/tasks/22-core-feasibility-review/task.md
Expected worker: web
Expected environment after publication: env:web-gpt
Downstream handoff profile: docs/tasks/handoffs/web-gpt.md
Task kind: research / verification synthesis
```

## Live Gate

先实际读取 Issue #22：

```text
status:draft
→ STOP，不 claim、不自行发布

status:ready + env:web-gpt + no active owner
→ claim，开始新的 Attempt N
```

## Start Protocol

1. 使用 GitHub 读取 Issue #22、全部 relevant comments、`AGENTS.md`、本 prompt、`task.md` 和 `docs/tasks/issue-lifecycle-protocol.md`。
2. 读取 `task.md` 指定的 canonical docs，以及 Issue #2/#3/#7/#9/#14 的当前 Final Acceptance 和 accepted Evidence。
3. 动态 GitHub 状态优先；若任一 required child 缺 Final Acceptance、被 reopen 或出现未解决的新矛盾，按 Task Contract 停止/报告 blocker，不使用历史聊天补全。
4. 确认 live state 为 `status:ready + env:web-gpt`、无 owner、Required Capabilities 可用后，claim → `status:in-progress` → Attempt N。
5. 只做 accepted Evidence synthesis / canonical cross-check / Gate decision；不要重新跑 TV、phone、browser 或 target实验，不要修 child implementation。
6. 严格保留 child `PASS / CONDITIONAL PASS / FAIL` 语义；不得为了得到 GO 事后降低标准。
7. 需要 canonical doc 变更时，按仓库 authority 顺序做最小一致性修订，并 read back。
8. 正常结束：标准 `[EXECUTION REPORT]` → `status:review` → release owner → STOP。
9. 阻塞：标准 `[BLOCKER REPORT]` → `status:blocked` → release owner → STOP。

## Required Reminder

这个 Task 的完整 Goal、Claims、Gate Decision Rules、Success Criteria、Evidence Contract 都只在：

```text
docs/tasks/22-core-feasibility-review/task.md
```

本 prompt 不重定义它们。

## Stop Boundary

Worker 不得：

- 自行发布 draft Task；
- 自行 `status:done` / close #22；
- 把 R004/R005/R006 当作 Web-only Core Gate 的必需输入；
- 用可选 Jellyfin 路线掩盖 R002/R003/R008 的 Core failure；
- 在缺失 required child Evidence 时猜测 Gate 结果；
- 自动开始 Phase 1 或下一 Task。