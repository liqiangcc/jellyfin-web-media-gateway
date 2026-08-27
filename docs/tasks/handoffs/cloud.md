# Handoff Profile — env:cloud

用于 **Codex Cloud Worker**。

当前仓库的默认实现路由是 **Codex-first**：当 Task 是普通仓库代码实现、修复、重构、测试/CI authoring 或 GitHub Actions 编排，并且不依赖 Windows/ADB、Ubuntu ARM64 现场交互或真实 TV 人工观察时，优先使用 `env:cloud` + Codex。

`env:cloud` 表示 Worker / orchestration 环境，不是 Runner。真正的 build/test/portable verification 仍优先通过 GitHub Actions 的 GitHub-hosted Runner；phone-specific proof 仍通过受信 Ubuntu ARM64 Target Runner；实体电视仍走 Manual TV。

Coordinator 在发布后输出：

```text
$task-worker Execute Issue #<issue> using `docs/tasks/<issue>-<slug>/prompt.md`.
```

`$task-worker` 会读取：

- `docs/tasks/issue-lifecycle-protocol.md`
- `docs/tasks/execution-anchor-recovery-protocol.md`

因此新的 repository-mutation Attempt 在 first coherent in-scope commit 后应尽早 push durable branch，并在适合时建立 draft PR / 单次 `[EXECUTION CHECKPOINT]`；不使用周期性 heartbeat。

Skill 不可见时 fallback：

```text
在 Codex Cloud 的 liqiangcc/jellyfin-web-media-gateway 工作区同步最新仓库，读取 `AGENTS.md`、Issue #<issue>、`docs/tasks/<issue>-<slug>/prompt.md` 和其引用的 task.md，以及 `docs/tasks/issue-lifecycle-protocol.md`、`docs/tasks/execution-anchor-recovery-protocol.md`。确认 status:ready + env:cloud + no active owner 后执行当前 Task；如果会修改仓库，在 first coherent in-scope commit 后尽早 push 可恢复 branch，适合时建立 draft PR，并按协议最多留一次 `[EXECUTION CHECKPOINT]`。Attempt 结束后按 Issue lifecycle 回报结果。
```

路由例外：

- `env:wsl`：需要本地 Linux 交互诊断；
- `env:windows`：需要 Windows / ADB / Android host 能力；
- `env:ubuntu-arm64`：需要目标手机上的交互式安装、恢复或现场 debug；
- `env:manual-tv`：真实 TV / 遥控器 / 可听播放人工 Evidence；
- `env:web-gpt`：GitHub-only 轻量执行、无法/不值得启动 Codex 的 Task，或 Coordinator 明确指定 Web Worker。

Cloud 不作为 self-hosted Runner，也不能冒充 Ubuntu ARM64 phone、家庭 LAN 或 TV Evidence。最终 Verification Authority 以 `task.md` 为准。

最小对外元数据：

```text
Task: <real title>
Issue: #<issue>
Worker: cloud-codex
Environment: env:cloud
Prompt: docs/tasks/<issue>-<slug>/prompt.md
Skill: $task-worker
```
