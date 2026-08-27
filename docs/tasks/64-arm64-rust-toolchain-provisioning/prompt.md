# Session Bootstrap — ENV-ARM64-RUST

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 Ubuntu ARM64 环境 provisioning Task。

本文件只负责会话启动；稳定执行契约在：

`docs/tasks/64-arm64-rust-toolchain-provisioning/task.md`

## Execution Context

```text
GitHub Issue: #64
Expected worker: ubuntu-arm64
Expected environment: env:ubuntu-arm64
Target: Ubuntu ARM64 phone
Runtime user: gateway-runner
Downstream blocked Task: #63 ENV-ARM64-READY
Handoff: docs/tasks/handoffs/ubuntu-arm64.md
```

## Start

实际读取并遵守：

1. `AGENTS.md`
2. Issue #64 及 relevant comments
3. `docs/tasks/64-arm64-rust-toolchain-provisioning/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. Issue #63 latest `[BLOCKER REPORT]`
6. Issue #1 / #21 accepted Runner evidence
7. `docs/runner-execution-architecture.md`
8. `docs/security.md`

只在 live #64 仍满足：

```text
status:ready
env:ubuntu-arm64
no active owner
```

时 claim 并开始新的 Attempt。

## Critical reminders

- 目标是给 `gateway-runner` 安装**用户拥有**的 Rust toolchain；不要暴露 root 的 `.cargo/.rustup`。
- 不给 `gateway-runner` sudo/root/admin/capabilities。
- Rust 至少满足仓库 `rust-version = 1.85`。
- 必须让非交互环境/Runner.Listener 的 PATH 稳定找到 `/home/gateway-runner/.cargo/bin`；不能只改交互式 `.bashrc`。
- 更新 root-owned supervisor 时只允许最小 PATH 集成，并必须保留已有 proxy stripping、低权限启动、capability drop 和 workspace 隔离。
- 不安装 FFmpeg/Chromium/Node，不跑 #63 J2，不跑 #9 性能，不改产品代码。
- 完成后 `[EXECUTION REPORT] -> status:review -> release owner -> STOP`。
- 阻塞时 `[BLOCKER REPORT] -> status:blocked -> release owner -> STOP`。
- Worker 不自行执行 #63 Attempt 2，不设置 done，不关闭 Issue。
