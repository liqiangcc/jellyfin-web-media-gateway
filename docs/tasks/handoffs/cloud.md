# Handoff Profile — env:cloud

用于 Cloud Codex Worker，只承担 Task Contract 明确要求的 Cloud-specific reproduction、长期交互 state 或授权的 remote orchestration。

Coordinator 在发布后输出：

```text
$task-worker Execute Issue #<issue> using `docs/tasks/<issue>-<slug>/prompt.md`.
```

Skill 不可见时 fallback：

```text
在 Cloud 的 liqiangcc/jellyfin-web-media-gateway 工作区同步最新仓库，读取 `AGENTS.md`、Issue #<issue>、`docs/tasks/<issue>-<slug>/prompt.md` 和其引用的 task.md，确认 status:ready + env:cloud + no active owner 后执行当前 Task，并按 Issue lifecycle 回报结果。
```

Cloud 不作为默认 Runner，也不能冒充 Ubuntu ARM64 phone / 家庭 LAN / TV Evidence。最终 Verification Authority 以 `task.md` 为准。

最小对外元数据：

```text
Task: <real title>
Issue: #<issue>
Worker: cloud
Environment: env:cloud
Prompt: docs/tasks/<issue>-<slug>/prompt.md
Skill: $task-worker
```
