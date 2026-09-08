# Session Bootstrap — Issue #165

本文件只负责导航；执行契约是 `docs/tasks/165-bilibili-browser-probe-prep/task.md`。

```text
GitHub Issue: #165
Repository: liqiangcc/jellyfin-web-media-gateway
Expected worker: cloud-codex
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

读取 AGENTS.md 及其启动文档、GitHub Issue #165 全部相关历史、本目录 task.md、docs/tasks/issue-lifecycle-protocol.md、docs/tasks/execution-anchor-recovery-protocol.md、docs/tasks/freshness-integration-protocol.md 和 task.md 引用文档。

从 GitHub 确认 ready + env eligibility + no active owner + Required Capabilities 才 claim；否则停止。候选/PR/状态从 Issue 读取，不从聊天猜测。按 task.md Freshness Contract 执行，不因无关 main 更新机械全量重跑。若有可恢复 Candidate/PR 优先续用。

claim 后记录 Attempt 并 in-progress；first coherent in-scope commit 尽早 push durable branch，适合时建立 draft PR/单次 EXECUTION CHECKPOINT。完成评论 EXECUTION REPORT → review；阻塞评论 BLOCKER REPORT → blocked；释放 owner 后 STOP。不自行 done/close 或启动下一项。模型/构建/实站权限约束以 task.md 为准。
