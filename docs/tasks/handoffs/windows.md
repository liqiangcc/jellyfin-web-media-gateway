# Handoff Profile — env:windows

用于 Windows Codex Worker，主要承担 Windows-native、ADB/Android host 或 Windows-only 交互能力。

Coordinator 在发布后输出：

```text
$task-worker Execute Issue #<issue> using `docs/tasks/<issue>-<slug>/prompt.md`.
```

Skill 不可见时 fallback：

```text
在 Windows 的 liqiangcc/jellyfin-web-media-gateway 工作区同步最新仓库，读取 `AGENTS.md`、Issue #<issue>、`docs/tasks/<issue>-<slug>/prompt.md` 和其引用的 task.md，确认 status:ready + env:windows + no active owner 后执行当前 Task，并按 Issue lifecycle 回报结果。
```

不要把 Windows/ADB 交互式操作本身当成 Linux/Target runtime PASS；Evidence Authority 仍以 `task.md` 为准。

最小对外元数据：

```text
Task: <real title>
Issue: #<issue>
Worker: windows
Environment: env:windows
Prompt: docs/tasks/<issue>-<slug>/prompt.md
Skill: $task-worker
```
