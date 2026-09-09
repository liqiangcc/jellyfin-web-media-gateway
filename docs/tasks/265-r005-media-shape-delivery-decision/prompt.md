# Session Bootstrap — Generic separated A/V media delivery decision

```text
GitHub Issue: #265
Task Contract: docs/tasks/265-r005-media-shape-delivery-decision/task.md
Expected worker: cloud
Expected environment: env:cloud
Handoff profile: docs/tasks/handoffs/cloud.md
```

Read `AGENTS.md`, Issue #265 and all comments, this Task Contract, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, current `main@91cf910e2e280dc48ac5359f74c6bf629f9d590e`, and accepted #237/#240/#243/#248/#251/#255/#257/#259 evidence.

Before claiming, verify the Issue is `status:ready`, `env:cloud`, owner-free, and the package files read back from GitHub. Do not perform live Bilibili/login/tx-node/phone/TV/VNC/CDP activity, do not process #191/#195, and do not compile/test/package/install locally. All required verification must run through GitHub-hosted Actions.

Use synthetic, secret-free fixtures only. Preserve muxed HTTP-file/HLS behavior, Core site-agnostic boundaries, Vault/EgressPolicy rules, and Playback/Display stale-generation semantics. Compare delivery routes and produce a concrete selected route or explicit `CONDITIONAL PASS`/`BLOCKED` decision plus a follow-up implementation Task outline. Post `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, set `status:review` or `status:blocked`, release ownership and stop. Never claim real Bilibili playback or close the Issue.
