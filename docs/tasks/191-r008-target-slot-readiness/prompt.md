# Session Bootstrap — R008 tx-node clean slot readiness for #188

You are starting a future Worker session for `liqiangcc/jellyfin-web-media-gateway` Issue #191.

```text
GitHub Issue: #191
Task Contract: docs/tasks/191-r008-target-slot-readiness/task.md
Expected worker: cloud-codex
Expected environment: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

Before any action, read `AGENTS.md`, the live Issue #191 and all relevant comments, the Task Contract above, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, and Issue #188 with its complete history. Confirm that #191 is `status:ready`, unclaimed and eligible for `env:cloud`; while it is `status:draft`, do not claim or execute it.

The Task Contract is the only source for Goal, Scope, Claims, Success Criteria, Evidence and boundaries. Use the authenticated Tailscale SSH path only for the one bounded read-only tx-node admission defined there. Record the requested `gpt-5.6-luna` high-reasoning runtime and actual Fast availability. If any existing source-runtime/session/profile/remote-debugging/proxy or production Gateway state remains, leave it untouched and report `BLOCKED`.

After a valid claim, follow the Issue lifecycle: start a new Attempt, post `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, set the corresponding review/blocked state, release ownership and stop. Do not start a #188 Attempt, download or execute an artifact, launch/connect/terminate Chrome or Node, access Bilibili, use VNC/CDP/phone/TV, mutate production, or modify #188/#166/#182/#185.
