# Session Bootstrap — R008 P0 Security Baseline

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 R008 P0 Security Baseline Task。

本文件只是 Task bootstrap/navigation，不是 Task Contract，也不保存实时状态。

## Execution Context

```text
GitHub Issue: #14
Task Contract: docs/tasks/14-r008-p0-security-baseline/task.md
Expected worker: cloud-codex
Expected environment label after publication: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Research Item: R008
```

## Preferred Codex Entry

```text
$task-worker Execute Issue #14 using `docs/tasks/14-r008-p0-security-baseline/prompt.md`.
```

Skill 不可见时按 `docs/tasks/handoffs/cloud.md` fallback。

## Live Gate

Worker 必须先实际读取 Issue #14：

```text
status:draft
→ STOP，不 claim、不实现、不自行发布

status:ready + env:cloud + no active owner
→ 可以 claim，开始新的 Attempt N

其他状态
→ 按 docs/tasks/issue-lifecycle-protocol.md 停止或交回 Coordinator
```

## Start Protocol

1. 实际使用 GitHub 读取 Issue #14 与 relevant comments；不得根据聊天背景猜测 live state。
2. 读取 `AGENTS.md`、本 prompt、`task.md`、`docs/tasks/issue-lifecycle-protocol.md`。
3. 读取 `task.md` 明确列出的 canonical security/architecture/runner 文档，以及 Issue #1/#2/#3 的 accepted Evidence。
4. 确认 live state 为 `status:ready + env:cloud`、无 active owner、当前环境具备 Required Capabilities。
5. claim → `status:in-progress` → 新的 Attempt N。
6. 严格执行 `task.md`；R008 不得通过关闭 SSRF、泄露 Secret、放宽 Target Runner trust gate 或伪造未实现 Browser Worker runtime PASS 来获得成功。
7. required Evidence 必须由 GitHub Actions 绑定 exact Candidate SHA；Codex shell 仅用于开发/诊断。
8. 正常结束：`[EXECUTION REPORT]` → `status:review` → release owner → STOP。
9. 阻塞：`[BLOCKER REPORT]` → `status:blocked` → release owner → STOP。

## Stop Boundary

完成当前 R008 Attempt 后立即停止。Worker 不得执行 #6/#7/#8/#9，不得自行 merge、`status:done` 或关闭 Issue #14。