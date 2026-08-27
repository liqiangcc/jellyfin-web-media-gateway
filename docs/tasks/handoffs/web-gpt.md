# Handoff Profile — env:web-gpt

用于新的 Web ChatGPT Worker 会话。此环境不依赖 repo-scoped Codex Skill；必须实际使用 GitHub connector 读取/修改仓库和 Issue。

Coordinator 在发布后输出以下独立复制块，并替换所有 placeholder：

```text
@GitHub

执行 liqiangcc/jellyfin-web-media-gateway 的 Issue #<issue>。

必须实际使用 GitHub 读取当前状态，不要根据聊天背景猜测，也不要使用 Web 搜索替代 GitHub。

先读取：
- Issue #<issue> 及 relevant comments
- AGENTS.md
- docs/tasks/<issue>-<slug>/prompt.md
- prompt.md 指向的 task.md 和 canonical docs
- docs/tasks/issue-lifecycle-protocol.md
- docs/tasks/execution-anchor-recovery-protocol.md

确认 Issue 仍为 status:ready + env:web-gpt 且无 active owner 后，按 Worker 协议 claim，开始新的 Attempt N，并严格执行当前 Task Contract。

如果本 Attempt 会修改仓库：
- first coherent in-scope commit 后尽早 push 可恢复 worker branch
- 适合时创建/复用 draft PR
- durable anchor 建立后、如果 Attempt 仍继续，最多留一次标准 [EXECUTION CHECKPOINT]
- 不创建空 commit/空 PR，不发周期性 heartbeat

正常完成：
- 把标准 [EXECUTION REPORT] 评论到 Issue
- 转为 status:review
- 释放 active execution ownership
- 停止，不自动执行下一 Task/Attempt

如果阻塞：
- 把标准 [BLOCKER REPORT] 评论到 Issue
- 转为 status:blocked
- 释放 active execution ownership
- 停止

Worker 不得自行 status:done 或关闭 Issue。
```

最小对外元数据同时给出：

```text
Task: <real title>
Issue: #<issue>
Worker: web
Environment: env:web-gpt
Prompt: docs/tasks/<issue>-<slug>/prompt.md
```
