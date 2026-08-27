# Session Bootstrap — GENERIC-YTDLP-EGRESS-RESEARCH

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的独立 Research Task。

本文件只负责 bootstrap / navigation，不拥有 Scope、Claims 或 Success Criteria。

## Execution Context

```text
GitHub Issue: #50
Task Contract: docs/tasks/50-generic-ytdlp-egress-research/task.md
Expected worker: web
Expected environment label: env:web-gpt
Downstream handoff profile: docs/tasks/handoffs/web-gpt.md
```

## Start Protocol

开始前必须实际读取：

- Issue #50 及 relevant comments；
- `AGENTS.md`；
- `docs/tasks/50-generic-ytdlp-egress-research/task.md`；
- `docs/tasks/issue-lifecycle-protocol.md`；
- `docs/tasks/execution-anchor-recovery-protocol.md`；
- `docs/tasks/freshness-integration-protocol.md`；
- Task Contract 指向的 #46 / #39 / #14-R008 accepted evidence、actual repository seams 和 canonical docs。

确认 live state 仍为：

```text
status:ready
env:web-gpt
no active owner
```

并确认当前 Web Worker 同时具备 GitHub read/write 与 Web/primary-source research capability 后，才能 claim 并开始新的 Attempt N。

## Task-specific Entry Note

本 Task 是研究/架构决策，不是 production runtime implementation。

启动后先做两类 read-back：

1. 仓库实际 accepted seam：当前 generic-ytdlp runner/parser/registration 与 R008 EgressPolicy；
2. 当前 external primary sources：yt-dlp 官方文档/源码及必要依赖文档。

不要根据聊天背景或旧印象猜测 yt-dlp 当前网络行为；外部事实要记录版本/tag/commit/date。不要为了得到 `SUPPORTED` 结论而降低 R008、TLS、SSRF、Secret 或 open-proxy 标准。

如果研究结果是 `DEFER` 或 `DROP`，只要 Evidence 和决策完整，仍可正常完成 Task。

## Completion

严格按 Task Contract 执行。

正常完成：

```text
post [EXECUTION REPORT]
→ status:review
→ release active owner
→ STOP
```

阻塞：

```text
post [BLOCKER REPORT]
→ status:blocked
→ release active owner
→ STOP
```

Worker 不得自行：

- `status:done`；
- 关闭 #50；
- 启用 production real-network yt-dlp；
- 发布/执行后续 runtime implementation Task；
- 自动开始其它 Task。

如果本文件与 `task.md`、`AGENTS.md` 或 canonical docs 冲突，以更高 authority 为准。