# Handoff Profile — env:wsl

用于 WSL 中的 Codex Worker，主要承担需要交互式 Linux debug 的 Task。

Coordinator 在发布后输出：

```text
$task-worker Execute Issue #<issue> using `docs/tasks/<issue>-<slug>/prompt.md`.
```

Skill 不可见时 fallback：

```text
在 WSL 的 liqiangcc/jellyfin-web-media-gateway 工作区同步最新仓库，读取 `AGENTS.md`、Issue #<issue>、`docs/tasks/<issue>-<slug>/prompt.md` 和其引用的 task.md，确认 status:ready + env:wsl + no active owner 后执行当前 Task，并把结果按 Issue lifecycle 回报。
```

执行前确认 Required Capabilities 与 WSL 环境真实匹配；WSL 交互式诊断不能自动替代 `task.md` 要求的最终 Actions/Target Evidence。

最小对外元数据：

```text
Task: <real title>
Issue: #<issue>
Worker: wsl
Environment: env:wsl
Prompt: docs/tasks/<issue>-<slug>/prompt.md
Skill: $task-worker
```
