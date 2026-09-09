# Session Bootstrap — Vault-bound authenticated Browser Worker runtime

Execution context:

```text
GitHub Issue: #243
Task Contract: docs/tasks/243-browser-auth-runtime/task.md
Expected worker: cloud
Expected environment label: env:cloud
Handoff profile: docs/tasks/handoffs/cloud.md
```

Read `AGENTS.md`, Issue #243 and all comments, this Task Contract, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, #235/#237/#240 accepted records, and the canonical docs named by the Task.

Before work, confirm `status:ready`, `env:cloud`, no active owner, and the required capabilities. Claim Attempt N and set `status:in-progress`. Reuse any existing Candidate/PR rather than rebuilding it. Implement only the Task Contract. Do not run local build/test/package/install; use the exact Candidate GitHub Actions workflow for verification. Do not perform live Bilibili/login/tx-node/phone/TV/VNC/CDP actions and do not touch #191/#195.

Preserve Vault/R008/PlaybackSession/Display/Site Plugin boundaries and keep secrets out of locators, events, logs, artifacts and DTOs. On success, post `[EXECUTION REPORT]` with Candidate SHA and exact Actions evidence, set `status:review`, release ownership, and stop. On a blocker, post `[BLOCKER REPORT]`, set `status:blocked`, release ownership, and stop. Do not close the Issue or claim authenticated playback.
