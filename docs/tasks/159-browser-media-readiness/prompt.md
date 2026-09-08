# Session Bootstrap — BROWSER-E2E-MEDIA-READINESS Fail closed on media readiness and playback progression

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 GitHub Issue #159。

## Execution Context

```text
GitHub Issue: #159
Task Contract: docs/tasks/159-browser-media-readiness/task.md
Expected worker: cloud
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

## Start

Use the repository's `$task-worker` workflow when available:

```text
$task-worker Execute Issue #159 using `docs/tasks/159-browser-media-readiness/prompt.md`.
```

Before claiming the Task, read `AGENTS.md`, Issue #159 and all relevant comments, the Task Contract, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, and the accepted dependency contracts named by `task.md`. Confirm the Issue is `status:ready`, `env:cloud`, unowned, and based on the current main.

This Task closes a browser E2E false-positive: require a successful media response, usable media metadata, explicit Display activation, and currentTime progression; replace the unresolvable happy-path placeholder source. Keep Gateway authority, EgressPolicy, secret boundaries, and all phone/physical-TV exclusions intact. All binary-producing builds and required tests must run through GitHub-hosted Actions; do not compile locally or on tx-node.

Follow the Worker lifecycle: claim a new Attempt, implement only the Task Contract, post `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, transition to `status:review` or `status:blocked`, release ownership, and stop. Do not set `status:done`, close the Issue, or start another Task.
