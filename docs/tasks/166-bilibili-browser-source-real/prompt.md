# Session Bootstrap — Issue #166

本文件只负责导航；执行契约是 `docs/tasks/166-bilibili-browser-source-real/task.md`。

```text
GitHub Issue: #166
Repository: liqiangcc/jellyfin-web-media-gateway
Expected worker: cloud-codex
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

读取 `AGENTS.md`、GitHub Issue #166 全部 relevant history、本目录 `task.md`、`docs/tasks/issue-lifecycle-protocol.md`、`docs/tasks/execution-anchor-recovery-protocol.md`、`docs/tasks/freshness-integration-protocol.md`、`docs/tasks/handoffs/README.md` 与 `cloud.md`，以及 task.md 引用的 canonical/research docs。

Publication Gate 已冻结 #169 live probe、#172 target runtime、tx-node admission、固定 selector 和 runbook；先从 GitHub read-back 验证 `status:ready + env:cloud + no active owner`，再 claim Attempt。只使用已下载的 target-runnable bundle：`playwright-core@1.55.0` 已随产物携带，tx-node 不得 npm install。所有编译/测试证据来自 GitHub Actions；不要本地编译、不要访问手机/TV/VNC/生产服务。

按 task.md 运行最多两次 clean anonymous tx-node browser sessions，执行 `bilibili:BV14V411W7r5:part-2`，记录脱敏 C0/C1/C2 Evidence。完成时按协议先评论 `[EXECUTION REPORT]`/`[BLOCKER REPORT]`，再切换 review/blocked、释放 ownership 并停止；不要自行关闭 Issue 或启动后续实现 Task。
