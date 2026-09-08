# Session Bootstrap — R008 browser navigation diagnostic

You are starting the independent `liqiangcc/jellyfin-web-media-gateway` Task for GitHub Issue #188.

## Execution Context

```text
GitHub Issue: #188
Task Contract: docs/tasks/188-r008-browser-navigation-diagnostic/task.md
Expected worker: cloud-codex (gpt-5.6-luna, reasoning high; record actual Fast availability)
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

## Start Protocol

Before claiming the Issue, read `AGENTS.md`, Issue #188 and all relevant comments, this Task Contract, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, the required canonical documents, the #166/#182/#185 Final Acceptance history, the accepted #187 refresh, and the current probe/diagnostic/runbook files.

Confirm from GitHub that #188 is `status:ready`, `env:cloud`, owner-free, and that the current Candidate/artifact dependencies are read back before any target action. Claim the Issue and set `status:in-progress` before writing code. Use the Task Contract as the only Scope/Claims/Success Criteria authority.

All build, test, package and artifact work belongs in GitHub-hosted Actions. Do not run local build/test/package/install. The tx-node route, if admitted by the Contract after exact-Candidate hosted checks, is only the existing authenticated Tailscale SSH control plane. Do not broaden it to a proxy, relay, phone, TV, VNC, CDP, production service or #166 mutation.

On completion, post the required `[EXECUTION REPORT]` (or `[BLOCKER REPORT]`), transition to `status:review` or `status:blocked`, release ownership, and stop. Do not set `status:done`, close the Issue, start another Attempt, or run a later playback task.