# Session Bootstrap — BROWSER-CONTROL-E2E Browser Control to Web Display loop

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 GitHub Issue #154。

## Execution Context

```text
GitHub Issue: #154
Task Contract: docs/tasks/154-browser-control-e2e/task.md
Expected worker: cloud
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

## Start

Use the repository's `$task-worker` workflow when available:

```text
$task-worker Execute Issue #154 using `docs/tasks/154-browser-control-e2e/prompt.md`.
```

Before claiming the Task, read `AGENTS.md`, Issue #154 and all relevant comments, the Task Contract, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, and the accepted dependency contracts named by `task.md`. Confirm the Issue is `status:ready`, `env:cloud`, unowned, and based on the current main.

The current delivery target is browser Control → Gateway PlaybackSession → Web Display on ordinary Linux. Do not make #67 real-site compatibility, phone/ADB, physical-TV, Jellyfin, or direct Bilibili-page interaction a prerequisite. All binary-producing builds and required tests must run through GitHub-hosted Actions; do not compile locally or on tx-node. Use only the narrow approved browser/SSH topology described in `task.md`.

Follow the Worker lifecycle: claim a new Attempt, implement only the Task Contract, post `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, transition to `status:review` or `status:blocked`, release ownership, and stop. Do not set `status:done`, close the Issue, or start another Task.
