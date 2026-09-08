# Session Bootstrap — Issue #169

本文件只负责导航；执行契约是 `docs/tasks/169-bilibili-browser-live-probe/task.md`。

```text
GitHub Issue: #169
Repository: liqiangcc/jellyfin-web-media-gateway
Expected worker: cloud-codex
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

读取 `AGENTS.md`、GitHub Issue #169 全部 relevant history、本目录 `task.md`、`docs/tasks/issue-lifecycle-protocol.md`、`docs/tasks/execution-anchor-recovery-protocol.md`、`docs/tasks/freshness-integration-protocol.md`、`docs/tasks/handoffs/README.md` 与 `cloud.md`，以及 task.md 引用的 canonical/research docs。

从 GitHub 确认 `status:ready`、`env:cloud` eligibility、无 active owner 和 Required Capabilities 后才 claim。按 task.md 的 C1–C4、Freshness Contract 和全远程编译约束执行；保持 synthetic mode，禁止在此 Task 发起 live Bilibili 请求、手机/TV 部署或本地/tx-node 编译。完成时先在 Issue 评论 `[EXECUTION REPORT]`/`[BLOCKER REPORT]`，再切换状态并释放 ownership；不要自行关闭 Issue 或启动 #166。
