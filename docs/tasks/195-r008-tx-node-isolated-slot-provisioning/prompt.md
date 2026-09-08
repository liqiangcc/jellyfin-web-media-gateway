# Session Bootstrap — R008 tx-node isolated slot provisioning

You are starting a future Worker session for `liqiangcc/jellyfin-web-media-gateway` Issue #195.

```text
GitHub Issue: #195
Task Contract: docs/tasks/195-r008-tx-node-isolated-slot-provisioning/task.md
Expected Worker: cloud-codex
Expected environment: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Requested model: gpt-5.6-luna, reasoning high
Fast availability: record actual availability; originating runtime unavailable
```

Before any action, read `AGENTS.md`, live Issue #195 and all comments, the Task Contract above, `docs/tasks/README.md`, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, all canonical documents required by `AGENTS.md`, and the complete relevant histories of #193, #188 and #191 plus #157/#166/#182/#185.

Confirm that #195 is `status:ready`, unclaimed and eligible for `env:cloud`. While it is `status:draft`, do not claim or execute it. Before provisioning, require written owner/admin coexistence authorization, an approved low-privilege access route and a sanitized slot descriptor. If any is absent, report BLOCKED without touching tx-node.

The Task Contract is the only source for Goal, Scope, Claims, Success Criteria, Evidence and boundaries. The existing `source-runtime` session is never to be inspected, attached to, stopped, reconfigured, reused or cleaned. Do not use its VNC/Chrome/profile/proxy/display/CDP/ports or access profile/Vault contents. Root SSH, if explicitly authorized, is control-plane provisioning only; final runtime and checks must be non-root and owner-scoped.

Any repository changes must use an exact Candidate and GitHub-hosted Actions for build/test/package verification. Do not build, test, package or install locally, on tx-node, phone or TV. Do not launch Chrome/Node/VNC/CDP/X/Wayland, access Bilibili or any site, send DNS/TLS/CONNECT/media traffic, provision before authorization, or alter #188/#191/#193/#157/#166/#182/#185.

After a valid claim, start Attempt N, follow the Issue lifecycle, post `[EXECUTION REPORT]` or `[BLOCKER REPORT]` with separate implementation/verification/Coordinator boundaries, set `status:review` or `status:blocked`, release ownership and stop. Do not set done, close the Issue, change #191, or begin #188.
