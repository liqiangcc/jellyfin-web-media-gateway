# Session Bootstrap — Issue #179

本文件只负责导航；执行契约是 `docs/tasks/179-r008-tx-node-egress-decision/task.md`。

```text
GitHub Issue: #179
Repository: liqiangcc/jellyfin-web-media-gateway
Expected worker: cloud-codex
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

开始前读取 `AGENTS.md`、Issue #179 全部 comments、`docs/tasks/179-r008-tx-node-egress-decision/task.md`、`docs/tasks/issue-lifecycle-protocol.md`、`docs/tasks/execution-anchor-recovery-protocol.md`、`docs/tasks/freshness-integration-protocol.md`、`docs/tasks/handoffs/cloud.md`，以及 task.md 引用的 #166/#176/canonical/security 文档。

只有 Issue #179 已 read back 为 `status:ready + env:cloud` 且无 active owner 时才 claim。按 task.md 的 C0–C3 执行，只做静态研究、文档和 tx-node 只读拓扑检查；不要发起任何 live Bilibili 请求、媒体消费、代理设置、profile/VNC/CDP 操作、手机/TV 部署或本地/tx-node 编译。

完成时按协议在 Issue #179 评论 `[EXECUTION REPORT]` 或 `[BLOCKER REPORT]`，再更新状态并释放 ownership；不要自行关闭 Issue、修改 #166 状态或开始子 Task 实现。

