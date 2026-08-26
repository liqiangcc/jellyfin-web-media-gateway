# Session Bootstrap — GENERIC-YTDLP-RUNTIME-PREP

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的独立 combined Task。

本文件只负责 bootstrap / navigation，不拥有 Scope、Claims 或 Success Criteria。

## Execution Context

```text
GitHub Issue: #60
Task Contract: docs/tasks/60-generic-ytdlp-runtime-prep/task.md
Expected worker: cloud-codex
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

## Start Protocol

开始前必须实际读取：

- Issue #60 及 relevant comments；
- `AGENTS.md`；
- `docs/tasks/60-generic-ytdlp-runtime-prep/task.md`；
- `docs/tasks/issue-lifecycle-protocol.md`；
- `docs/tasks/execution-anchor-recovery-protocol.md`；
- `docs/tasks/freshness-integration-protocol.md`；
- Task Contract 指向的 #50 / #46 / #39 / #14-R008 Final Acceptance、accepted repository seams 与 canonical docs。

确认 live state 仍为：

```text
status:ready
env:cloud
no active owner
```

且当前 Cloud Worker 具备 Rust/Python/Linux sandbox/Unix IPC/GitHub Actions 能力后，才能 claim 并开始新的 Attempt N。

## Task-specific Entry Note

本 Task 实现 #50 已冻结的 brokered-worker PREP，但**不启用 production real-network generic-ytdlp**。

必须使用 Task Contract 冻结的 yt-dlp 版本/commit，不得静默升级 master/latest。

重点首先证明：

- dedicated Python API worker；
- structured inherited IPC capability；
- Gateway-owned R008 broker；
- worker/descendants OS-level direct AF_INET/AF_INET6 denial；
- anonymous Secret/config/plugin/runtime escape fail-closed；
- lifecycle cleanup；
- production default 仍为 `DisabledRunner`。

真实站点兼容性、登录和 production enablement 都不是本 Attempt 的成功条件，也不得顺手开启。

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

- merge PR；
- `status:done`；
- 关闭 #60；
- 启用 production generic-ytdlp networking；
- 发布/执行后续 real-network Verification Task；
- 自动开始其它 Task。

如果本文件与 `task.md`、`AGENTS.md` 或 canonical docs 冲突，以更高 authority 为准。