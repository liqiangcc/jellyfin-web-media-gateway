# Session Bootstrap — GENERIC-YTDLP-PREP

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 Issue #46。

本文件只负责 bootstrap / navigation；唯一执行契约是：

```text
docs/tasks/46-generic-ytdlp-prep/task.md
```

## Execution Context

```text
GitHub Issue: #46
Task Contract: docs/tasks/46-generic-ytdlp-prep/task.md
Expected worker: cloud-codex
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

## Start Protocol

开始前必须实际从 GitHub 读取：

1. `AGENTS.md`
2. Issue #46 及 relevant comments
3. `docs/tasks/46-generic-ytdlp-prep/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/handoffs/cloud.md`
6. `docs/site-plugin-architecture.md`
7. 当前 `site-adapter-api/src/lib.rs`, `conformance.rs`, `security.rs`
8. 当前 `plugins/generic-direct` 与 accepted #39 Final Acceptance
9. `docs/security.md` / accepted R008 network+Secret authority
10. Issue #23 / PR #37 只用于确认 blocked/non-authority 边界

Claim 前确认：

```text
open
status:ready
env:cloud
no active owner
```

然后按 Worker 协议 claim，开始新的 `Attempt N`。

Task-specific reminder：**本 Task 只做 deterministic generic-ytdlp PREP。Required Evidence 禁止真实外网请求；不要把 real yt-dlp subprocess network 接入生产 Registry，也不要通过进程绕过 R008。**

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

Worker 不得自行 `status:done`、关闭 #46、执行 #23/#36、启用真实网络 yt-dlp runtime、合并自己的 PR、自动开始下一 Task 或自行改变 Task Contract。