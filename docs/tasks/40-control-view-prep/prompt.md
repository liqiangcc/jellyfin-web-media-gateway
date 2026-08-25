# Session Bootstrap — CONTROL-VIEW-PREP

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 Issue #40。

本文件只负责 bootstrap / navigation；唯一执行契约是：

```text
docs/tasks/40-control-view-prep/task.md
```

## Execution Context

```text
GitHub Issue: #40
Task Contract: docs/tasks/40-control-view-prep/task.md
Expected worker: cloud-codex
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

## Start Protocol

开始前必须实际从 GitHub 读取：

1. `AGENTS.md`
2. Issue #40 及 relevant comments
3. `docs/tasks/40-control-view-prep/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/handoffs/cloud.md`
6. `docs/control-experience-architecture.md`
7. 当前 accepted `gateway-core/src/control.rs`, `auth.rs`, `browser.rs`
8. 当前 `display-adapter-api/src/lib.rs`
9. Issue #38 Final Acceptance / accepted Control snapshot/reconnect semantics
10. accepted #28/#33 boundaries

Claim 前确认：

```text
open
status:ready
env:cloud
no active owner
```

然后按 Worker 协议 claim，开始新的 `Attempt N`。

Task-specific reminder：**ControlView 只能投影 authoritative domain snapshots/status；不得新增第二套 Playback/Site/Display/Browser 真状态，也不得发明 global control revision。**

正常完成：

```text
[EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

阻塞：

```text
[BLOCKER REPORT]
→ status:blocked
→ release owner
→ STOP
```

Worker 不得自行 `status:done`、关闭 #40、合并自己的 PR、自动开始下一 Task 或自行改变 Task Contract。