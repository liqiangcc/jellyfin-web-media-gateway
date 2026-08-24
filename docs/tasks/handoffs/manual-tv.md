# Handoff Profile — env:manual-tv

用于真实电视/遥控器/可听播放等物理 UX 验证。此环境是人工 Evidence Authority，不伪装成 Codex Skill 或 GitHub Actions Runner。

Coordinator 在发布后输出以下独立复制块：

```text
执行 liqiangcc/jellyfin-web-media-gateway Issue #<issue> 的 Manual TV Verification。

先读取：
- Issue #<issue> 及 relevant comments
- docs/tasks/<issue>-<slug>/prompt.md
- prompt.md 指向的 task.md
- docs/tasks/issue-lifecycle-protocol.md

开始前确认 Issue 为 status:ready + env:manual-tv 且无 active owner。

严格按 task.md 的真实 TV / remote / audible / timing 观察步骤执行，不用桌面浏览器、模拟器或理论判断替代物理 Evidence。

完成后把结果以标准 [EXECUTION REPORT] 写回 Issue 并转 status:review；无法继续则写 [BLOCKER REPORT] 并转 status:blocked。不要自行关闭 Issue。
```

最小对外元数据：

```text
Task: <real title>
Issue: #<issue>
Worker: manual-tv
Environment: env:manual-tv
Prompt: docs/tasks/<issue>-<slug>/prompt.md
```
