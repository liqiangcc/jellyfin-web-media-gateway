# Session Bootstrap — R008 post-navigation-start diagnostic

You are executing `liqiangcc/jellyfin-web-media-gateway` Issue #203.

## Execution Context

```text
GitHub Issue: #203
Task Contract: docs/tasks/203-r008-post-navigation-start-diagnostic/task.md
Expected worker: cloud
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Requested model: gpt-5.6-luna, reasoning high
Fast: disabled by user; do not enable/select Fast
```

## Start Protocol

Read the live Issue #203 and all relevant comments, then read:

- `AGENTS.md`;
- `docs/tasks/203-r008-post-navigation-start-diagnostic/task.md`;
- `docs/tasks/issue-lifecycle-protocol.md`;
- `docs/tasks/execution-anchor-recovery-protocol.md`;
- `docs/tasks/freshness-integration-protocol.md`;
- the canonical documents and probe/runbook documents referenced by the Task Contract;
- Issue #188 Attempt 5 evidence and Issue #199 accepted stage-marker history.

Confirm #203 is `status:ready`, `env:cloud`, and owner-free before claiming a new Attempt. Use only gpt-5.6-luna high; Fast is disabled by the user.

## Task-specific Entry Note

Implement and verify the bounded post-navigation-start diagnostic described by `task.md` using GitHub-hosted Actions only. Preserve the finite sanitized marker and finalizer boundaries. Do not access tx-node, a browser, Bilibili, playback, VNC/CDP, phone/TV, or local build/test/package/install. Do not process, claim, edit, or wait on #191/#195. A later #188 target rerun is Coordinator-controlled.

## Completion

Follow the repository lifecycle: publish `[EXECUTION REPORT]` or `[BLOCKER REPORT]` on #203, transition to `status:review` or `status:blocked`, release ownership and stop. Do not set `status:done`, close #203, or begin another Task.
