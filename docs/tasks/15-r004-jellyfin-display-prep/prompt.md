# Session Bootstrap — R004-PREP Jellyfin DisplayAdapter PoC

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 R004-PREP Task。

本文件只是 Task bootstrap/navigation，不是 Task Contract，也不保存实时状态。

## Execution Context

```text
GitHub Issue: #15
Task Contract: docs/tasks/15-r004-jellyfin-display-prep/task.md
Expected worker: cloud-codex
Expected environment label after publication: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Research Item: R004
Linked physical verification: Issue #16
```

## Preferred Codex Entry

```text
$task-worker Execute Issue #15 using `docs/tasks/15-r004-jellyfin-display-prep/prompt.md`.
```

Skill 不可见时按 `docs/tasks/handoffs/cloud.md` fallback。

## Live Gate

Worker 必须先实际读取 Issue #15：

```text
status:draft
→ STOP，不 claim、不实现、不自行发布

status:ready + env:cloud + no active owner
→ 可以 claim，开始新的 Attempt N

其他状态
→ 按 docs/tasks/issue-lifecycle-protocol.md 停止或交回 Coordinator
```

## Start Protocol

1. 使用 GitHub 读取 Issue #15、relevant comments、`AGENTS.md`、本 prompt、`task.md`、`docs/tasks/issue-lifecycle-protocol.md`。
2. 读取 `task.md` 列出的 R004/canonical docs，以及 Issue #2/R007、Issue #3/R001 的 accepted Evidence。
3. 确认 live state 为 `status:ready + env:cloud`、无 owner、当前环境具备 Required Capabilities。
4. claim → `status:in-progress` → 新的 Attempt N。
5. 只实现 Jellyfin DisplayAdapter/client PoC + hosted verification mechanics；不得把 Jellyfin 变成 Playback authority，不得重定义 R007 handoff/revision，不得绕过 R001 media/Secret boundary。
6. hosted/mock Jellyfin 结果只证明 adapter mechanics；不得宣称真实 Android TV R004 PASS/FAIL。
7. required J1/J2 Evidence 必须由 GitHub Actions 绑定 exact Candidate SHA。
8. 正常结束 `[EXECUTION REPORT]` → `status:review` → release owner → STOP；阻塞则 `[BLOCKER REPORT]` → `status:blocked` → release owner → STOP。

## Stop Boundary

完成当前 R004-PREP Attempt 后停止。Worker 不得执行 Issue #16，不得自行 merge、`status:done` 或关闭 Issue #15。