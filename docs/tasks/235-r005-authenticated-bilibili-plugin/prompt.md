# Session Bootstrap — Authenticated Bilibili Site Plugin feasibility and contract

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 中的 GitHub Issue #235。

```text
GitHub Issue: #235
Task Contract: docs/tasks/235-r005-authenticated-bilibili-plugin/task.md
Expected worker: cloud-codex
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Model: gpt-5.6-luna, reasoning high
Fast: enabled per current user routing; record actual availability
```

开始前从 GitHub 读取 live Issue #235、全部 relevant comments、`AGENTS.md`、本 Task Contract、`docs/tasks/issue-lifecycle-protocol.md`、`docs/tasks/execution-anchor-recovery-protocol.md`、`docs/tasks/freshness-integration-protocol.md` 以及 Contract 引用的 canonical/auth/security 文档；同时核对 #28、#33、#232 与 #26 的当前历史和精确 Evidence。

只有 Issue 已由 Coordinator 发布为 `status:ready + env:cloud` 且 owner-free 时才 claim Attempt。严格按 `task.md` 的 Scope、外部授权门槛、PASS/CONDITIONAL PASS/FAIL/BLOCKED 和报告/状态/释放 owner 协议执行。

本 Task 研究的是经过授权的 authenticated Bilibili Site Plugin 可行性与契约。它不自动获得账号、密码、验证码、Cookie、profile、VNC/CDP、手机/TV、tx-node 或生产权限。没有书面授权、专用测试账号和批准的交互式登录通道时，目标证据必须为 `BLOCKED`；不得用个人账号/profile 或手工 Cookie 注入替代。

所有需要编译、构建、安装或测试的实现验证只能使用 GitHub-hosted Actions；不要在本地、手机或 tx-node 构建/安装。不要修改 #26、#28、#33、#191、#195、#166、#182、#188、#223、#226、#229 或 #232，不要播放、点击、full preload、媒体提取或部署设备。完成后按协议发布 `[EXECUTION REPORT]` 或 `[BLOCKER REPORT]`，转 `status:review`/`status:blocked`，释放 owner 并停止；不要自行 done/close 或启动后续 Task。
