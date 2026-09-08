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

Publication Gate 现在还必须冻结已接受的 #182/#185 artifact delivery：唯一 live Attempt artifact 是 Candidate `7de63b231fc4582fc29ceb5a359050f4ffe22fcb`、Actions artifact `10060036115`、digest `sha256:53c57ce5c22219753ff8ad7f3fbea6c112958d88f84daa799bc03aa8e3f02259`。保留并核对 #169 live probe 与 #172 target runtime 的完整 provenance；#182 的 `proxy_response / success / 2xx` 只是无页面 transport admission evidence，不是播放证明。先从 GitHub read-back 验证 Coordinator 已完成 contract refresh、`status:ready + env:cloud + no active owner`，再 claim **Attempt 2**。只使用该精确 artifact，重新做 manifest/digest 校验并在 tx-node fresh staging；`playwright-core@1.55.0` 已随产物携带，tx-node 不得 npm install。所有编译/测试证据来自 GitHub Actions；不要本地编译、不要访问手机/TV/VNC/生产服务。

按 task.md 运行最多两次 clean anonymous tx-node browser sessions，且仅执行 `bilibili:BV14V411W7r5:part-2`。Attempt 2 必须同时记录脱敏 transport admission、page/source candidate、browser-exit 后 independent consumer 和 cleanup Evidence；transport success 单独不能使 C1/C2 PASS。完成时按协议先评论 `[EXECUTION REPORT]`/`[BLOCKER REPORT]`，再切换 `status:review`/`status:blocked`、释放 ownership 并停止；不要自行关闭 Issue、跳过状态生命周期或启动后续实现 Task。
