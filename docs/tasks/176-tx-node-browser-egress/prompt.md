# Session Bootstrap — Issue #176

本文件只负责导航；执行契约是 `docs/tasks/176-tx-node-browser-egress/task.md`。

```text
GitHub Issue: #176
Repository: liqiangcc/jellyfin-web-media-gateway
Expected worker: cloud-codex
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

开始前读取 `AGENTS.md`、Issue #176 全部 comments、`docs/tasks/176-tx-node-browser-egress/task.md`、`docs/tasks/issue-lifecycle-protocol.md`、`docs/tasks/execution-anchor-recovery-protocol.md`、`docs/tasks/freshness-integration-protocol.md`、`docs/tasks/handoffs/cloud.md`，以及 task.md 引用的 canonical/security/browser research 文档。

只有 GitHub Issue #176 已 read back 为 `status:ready + env:cloud` 且无 active owner 时才 claim。按 task.md 的 C0–C3、全远程构建约束和 target diagnostic boundary 执行；不要修改 #166 的状态，不要部署手机/TV、连接 VNC/CDP、复用现有 Chrome/profile 或运行本地/tx-node 编译。

完成时按协议在 Issue #176 评论 `[EXECUTION REPORT]` 或 `[BLOCKER REPORT]`，再更新状态并释放 ownership；不要自行关闭 Issue、启动 #166 或开始媒体/插件实现。
