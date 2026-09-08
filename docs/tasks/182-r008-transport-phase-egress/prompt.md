# Session Bootstrap — R008 broker transport phase and tx-node egress preflight

You are executing GitHub Issue #182 in `liqiangcc/jellyfin-web-media-gateway`.

```text
Task Contract: docs/tasks/182-r008-transport-phase-egress/task.md
Expected Worker: cloud-codex (gpt-5.6-luna, reasoning high; record Fast availability)
Expected environment: env:cloud
Handoff profile: docs/tasks/handoffs/cloud.md
```

Before claiming the Issue, read `AGENTS.md`, the live Issue #182 and all relevant comments, the Task Contract, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, and the canonical/R008 documents named by the Contract. Confirm `status:ready`, `env:cloud`, no active owner, exact base/dependency provenance, and the no-page/no-media/no-proxy/no-phone boundary.

Claim the Issue and start a new Attempt before writing code. Use GitHub Actions for all required build/test/package work; do not compile, install packages, or run a browser page on the local workspace or tx-node. Follow the Contract’s order: hosted exact-Candidate checks and artifact first, then at most one sanitized no-page transport preflight on tx-node as `gateway-verify`. The target preflight must not use a Bilibili selector or media request.

When the Attempt ends, post `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, transition to `status:review` or `status:blocked`, release ownership, and stop. Do not modify or unlock Issue #166, set `done`, close an Issue, start another Task, or retry outside the frozen limits.
