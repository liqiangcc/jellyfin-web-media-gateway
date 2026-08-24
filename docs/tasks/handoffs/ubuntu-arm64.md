# Handoff Profile — env:ubuntu-arm64

用于目标 Ubuntu ARM64 设备上的 Codex Worker。

Coordinator 在发布后输出以下独立复制块，并替换 placeholder：

```text
$task-worker Execute Issue #<issue> using `docs/tasks/<issue>-<slug>/prompt.md`.
```

如果 repo Skill 暂不可见：

```text
先同步 liqiangcc/jellyfin-web-media-gateway 最新 main，然后读取 `AGENTS.md`、Issue #<issue> 和 `docs/tasks/<issue>-<slug>/prompt.md`，按其中 Worker 协议执行当前 Task。
```

执行前必须重新确认：

```text
status:ready
env:ubuntu-arm64
no active owner
Required Capabilities available
```

不要因为当前 bootstrap operator shell 是 root 就自动判定 BLOCKED；是否允许 operator 特权以及最终 runtime 权限边界以当前 `task.md` 为准。

最小对外元数据：

```text
Task: <real title>
Issue: #<issue>
Worker: ubuntu-arm64
Environment: env:ubuntu-arm64
Prompt: docs/tasks/<issue>-<slug>/prompt.md
Skill: $task-worker
```
