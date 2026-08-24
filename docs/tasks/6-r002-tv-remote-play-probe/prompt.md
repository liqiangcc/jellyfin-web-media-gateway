# Session Bootstrap — R002-PREP TV Remote Playback Probe

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 R002-PREP Task。

本文件只是 Task bootstrap/navigation，不是 Task Contract，也不保存实时状态。

## Execution Context

```text
GitHub Issue: #6
Task Contract: docs/tasks/6-r002-tv-remote-play-probe/task.md
Expected worker: web
Expected environment label after publication: env:web-gpt
Downstream handoff profile: docs/tasks/handoffs/web-gpt.md
Research Item: R002
Linked physical verification: Issue #7
```

## Live Gate

新 Web Worker 必须先实际读取 Issue #6：

```text
status:draft
→ 停止，不 claim、不实现、不自行发布

status:ready + env:web-gpt + no active owner
→ 可以 claim，开始新的 Attempt N
```

Publication 前 Coordinator 必须确认 Issue #3 已提供一个明确、稳定、可复用的 R001 Web Display/media candidate，并在 Issue #6 记录具体 candidate/base；仅存在一个仍在变化的 open PR 不足以解除 hard dependency。

## Start Protocol

1. 使用 GitHub 读取 Issue #6、comments、`AGENTS.md`、本 prompt、`task.md`、`docs/tasks/issue-lifecycle-protocol.md`。
2. 读取 `task.md` 引用的 R002 canonical docs，并读取 Coordinator 指定的 R001 candidate/Issue #3 evidence。
3. 确认 live state 为 `status:ready + env:web-gpt`、无 owner、R001 dependency 已被 Coordinator 明确解除。
4. claim → `status:in-progress` → Attempt N。
5. 只准备 R002 probe；不得宣称物理 TV PASS，也不得重新实现 R001 media path 或 R007 Playback concurrency authority。
6. hosted browser/CI 只证明 probe mechanics，不得用桌面 Chromium 替代 Issue #7 physical TV Evidence。
7. 正常结束 `[EXECUTION REPORT]` → `status:review` → release owner → STOP；阻塞则 `[BLOCKER REPORT]` → `status:blocked` → release owner → STOP。

## Stop Boundary

完成当前 R002-PREP Attempt 后停止。Worker 不得执行 Issue #7，不得自行关闭 Issue #6。