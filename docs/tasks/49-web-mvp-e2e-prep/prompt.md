# Session Bootstrap — WEB-MVP-E2E-PREP Hosted Web-only MVP product integration gate

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 中已经发布的独立 Task #49。

本文件只是 bootstrap / navigation 入口，不是 Task Contract。

## Execution Context

```text
GitHub Issue: #49
Task Contract: docs/tasks/49-web-mvp-e2e-prep/task.md
Expected worker: cloud
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

## Start Protocol

开始前必须实际读取：

- `AGENTS.md`
- GitHub Issue #49 及 relevant comments
- `docs/tasks/49-web-mvp-e2e-prep/task.md`
- `docs/tasks/issue-lifecycle-protocol.md`
- `docs/tasks/execution-anchor-recovery-protocol.md`
- `docs/tasks/freshness-integration-protocol.md`
- task.md 引用的 canonical / accepted dependency facts

确认 live Issue 仍为：

```text
open
status:ready
env:cloud
no active owner
```

然后按协议 claim，开始新的 Attempt N，再执行当前 Task Contract。

## Task-specific Entry Note

- Planning / Evidence Base 以 task.md 为准；不要根据旧聊天或 Issue 旧草案猜接口。
- 先读 current main 的真实 `/`、`/display`、`/control`、session、display context/rendering routes，再决定 G1/G2/G3 哪些最小 glue 仍需要实现。
- 必须复用 accepted #44/#45/#47/#48 authority；不要为了 E2E 重新实现或弱化它们。
- required browser/runtime Evidence 通过 GitHub Actions / GitHub-hosted runner 产生；Cloud 是 Worker，不是 Runner。

正常结束：

```text
[EXECUTION REPORT]
→ status:review
→ release active owner
→ STOP
```

阻塞：

```text
[BLOCKER REPORT]
→ status:blocked
→ release active owner
→ STOP
```

Worker 不得自行 `status:done`、关闭 Issue、合并自己的 PR，或自动执行 #7/#9/#22/#23/#36/#50。
