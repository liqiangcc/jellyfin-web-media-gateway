# Session Bootstrap — PLUGIN-CONFORMANCE-PREP

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 Issue #39。

本文件只负责 bootstrap / navigation；唯一执行契约是：

```text
docs/tasks/39-plugin-conformance-prep/task.md
```

## Execution Context

```text
GitHub Issue: #39
Task Contract: docs/tasks/39-plugin-conformance-prep/task.md
Expected worker: cloud-codex
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

## Start Protocol

开始前必须实际从 GitHub 读取：

1. `AGENTS.md`
2. Issue #39 及 relevant comments
3. `docs/tasks/39-plugin-conformance-prep/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/handoffs/cloud.md`
6. `docs/site-plugin-architecture.md`
7. `docs/implementation-contracts.md`
8. `docs/security.md`
9. 当前 `site-adapter-api/src/lib.rs` 与 `plugins/generic-direct`
10. Issue #23 / PR #37 只用于确认 blocked/non-authority 边界，不得把其未接受 API 当 canonical

Claim 前确认：

```text
open
status:ready
env:cloud
no active owner
```

然后按 Worker 协议 claim，开始新的 `Attempt N`。

Task-specific reminder：**只围绕 live accepted generic SiteAdapter boundary 建 conformance/architecture guard；不要从 blocked #23 Candidate 反向冻结 navigation/DASH/expiry 等 API。**

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

Worker 不得自行 `status:done`、关闭 #39、合并自己的 PR、自动开始下一 Task 或自行改变 Task Contract。