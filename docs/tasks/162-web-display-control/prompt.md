# Session Bootstrap — WEB-DISPLAY-CONTROL Apply Gateway playback commands on TV Display

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 GitHub Issue #162。

## Execution Context

```text
GitHub Issue: #162
Task Contract: docs/tasks/162-web-display-control/task.md
Expected worker: cloud
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

## Start

使用仓库的 `$task-worker` workflow（若可用）：

```text
$task-worker Execute Issue #162 using `docs/tasks/162-web-display-control/prompt.md`.
```

先读取 `AGENTS.md`、Issue #162 及 relevant comments、Task Contract、`docs/tasks/issue-lifecycle-protocol.md`、`docs/tasks/execution-anchor-recovery-protocol.md`、`docs/tasks/freshness-integration-protocol.md` 和 Task Contract 引用的 accepted dependency contracts。确认 Issue 已通过 Publication Gate，状态为 `status:ready`、环境为 `env:cloud` 且没有 active owner。

严格按 Task Contract 执行。核心验证是 Gateway Control 命令实际改变 TV Web Display 的媒体元素状态；保留显式 TV 激活和 Chromium 自动播放边界。所有 required build/test/browser evidence 必须来自 GitHub-hosted Actions；不要在本地或 tx-node 编译。Worker 完成后发布 `[EXECUTION REPORT]`，切换 `status:review`，释放 ownership 并停止；不要自行关闭 Issue。
