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
Accepted R001 Candidate: 42c92db2a380895ec3909cdc9afa847478150eb0
R001 merged main commit: 2b0a1a0ea95753ff416e41759b7c33823be1b9e0
```

## Live Gate

Web Worker 必须先实际读取 Issue #6：

```text
status:draft
→ 停止，不 claim、不实现、不自行发布

status:ready + env:web-gpt + no active owner
→ 可以 claim，开始新的 Attempt N

其他状态
→ 按 docs/tasks/issue-lifecycle-protocol.md 停止或交回 Coordinator
```

R001 publication dependency 已由 Coordinator 通过 Issue #3 Final Acceptance 解除。Worker 仍必须从 GitHub 读取 Issue #3 的 accepted Candidate/merge state，不得根据本 prompt 猜测动态状态。

## Attempt 2 Entry Note

这是 Attempt 1 Coordinator Review 后的 routing revision。Web Worker 必须先读取 Issue #6 的最新 `[COORDINATOR REVIEW]`，并继续已有 PR #12 / durable candidate work；不要创建第二个独立实现。

当前 Review 要求保持 Task Contract 不变，重点是：

- integrate current `main` into the existing PR/candidate;
- add hosted mechanics Evidence that actually observes `visibilitychange` for C5;
- rerun required exact-SHA R002 hosted Evidence and affected R001/R007 regressions;
- preserve all physical-TV claims for Issue #7.

## Start Protocol

1. 使用 GitHub 读取 Issue #6、全部 relevant comments、`AGENTS.md`、本 prompt、`task.md`、`docs/tasks/issue-lifecycle-protocol.md`。
2. 读取 `task.md` 引用的 R002 canonical docs，并读取 Issue #3 Final Acceptance / accepted R001 candidate evidence。
3. 确认 live state 为 `status:ready + env:web-gpt`、无 owner，并确认当前 main 仍包含 accepted R001/R007 authority。
4. claim → `status:in-progress` → Attempt N。
5. 优先复用/集成 PR #12 和已有 candidate；不得从零重建同一 Task。
6. 只准备/修订 R002 probe；不得宣称物理 TV PASS，也不得重新实现 R001 media path 或 R007 Playback concurrency authority。
7. required hosted browser/runtime Evidence 通过 GitHub Actions 产生并绑定 exact Candidate SHA；Web Worker 负责 GitHub 修改、Actions orchestration 和 Evidence 读取。
8. hosted browser/CI 只证明 probe mechanics，不得用桌面 Chromium 替代 Issue #7 physical TV Evidence。
9. 正常结束 `[EXECUTION REPORT]` → `status:review` → release owner → STOP；阻塞则 `[BLOCKER REPORT]` → `status:blocked` → release owner → STOP。

## Stop Boundary

完成当前 R002-PREP Attempt 后停止。Worker 不得执行 Issue #7，不得自行 `status:done` 或关闭 Issue #6。