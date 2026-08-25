# Session Bootstrap — WEB-DISPLAY-SESSION-PREP

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 Issue #45。

本文件只负责 bootstrap / navigation；唯一执行契约是：

```text
docs/tasks/45-web-display-session-prep/task.md
```

## Execution Context

```text
GitHub Issue: #45
Task Contract: docs/tasks/45-web-display-session-prep/task.md
Expected worker: cloud-codex
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

## Start Protocol

开始前必须实际从 GitHub 读取：

1. `AGENTS.md`
2. Issue #45 及 relevant comments
3. `docs/tasks/45-web-display-session-prep/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/handoffs/cloud.md`
6. `docs/control-experience-architecture.md`
7. accepted #38 / #40 Final Acceptance 与当前 `gateway-core/src/control.rs`、`control_view.rs`
8. 当前 `gateway-core/src/playback.rs` 与 `display-adapter-api/src/lib.rs`
9. `docs/security.md` 与 accepted R008 Host/Origin/Egress/Secret boundary

Claim 前确认：

```text
open
status:ready
env:cloud
no active owner
```

然后按 Worker 协议 claim，开始新的 `Attempt N`。

Task-specific reminder：**page lease/attachment epoch 只管理浏览器实例 liveness；绝不能成为第二套 `display_generation`。所有 Playback/display generation/handoff authority 继续属于 R007。**

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

Worker 不得自行 `status:done`、关闭 #45、执行 #7/#44/#48、合并自己的 PR、自动开始下一 Task 或自行改变 Task Contract。