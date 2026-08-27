# Session Bootstrap — CONTROL-SHELL-PREP

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 Issue #38。

本文件只是 bootstrap / navigation 入口；唯一执行契约是：

```text
docs/tasks/38-control-shell-prep/task.md
```

## Execution Context

```text
GitHub Issue: #38
Task Contract: docs/tasks/38-control-shell-prep/task.md
Expected worker: cloud-codex
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

## Start Protocol

开始前必须实际从 GitHub 读取当前状态，不得按聊天背景猜测：

1. `AGENTS.md`
2. Issue #38 及 relevant comments
3. `docs/tasks/38-control-shell-prep/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/handoffs/cloud.md`
6. task.md 引用的 canonical docs，尤其：
   - `docs/control-experience-architecture.md`
   - `docs/implementation-contracts.md`
   - `docs/security.md`
   - `docs/mvp-plan.md`
7. 当前 `gateway-core/src/playback.rs` 与 accepted R007 tests

Claim 前确认 live Issue 仍满足：

```text
open
status:ready
env:cloud
no active owner
```

然后按 Worker 协议 claim，开始新的 `Attempt N`。

本 Task 的关键启动提醒只有一条：**包装并消费 accepted R007 Playback authority，不得为了 HTTP/Control 便利重新定义 session revision、request-id、item/media freshness、display generation 或 handoff 语义。**

正常完成：

```text
[EXECUTION REPORT]
→ status:review
→ release active ownership
→ STOP
```

阻塞：

```text
[BLOCKER REPORT]
→ status:blocked
→ release active ownership
→ STOP
```

Worker 不得自行 `status:done`、关闭 #38、自动开始下一 Task 或自行开始下一 Attempt。

如果本 prompt 与 `task.md`、`AGENTS.md` 或 canonical docs 冲突，以更高 authority 为准。