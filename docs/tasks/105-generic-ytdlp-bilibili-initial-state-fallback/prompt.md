# Session Bootstrap — Issue #105

Use the repository `task-worker` skill and execute Issue #105 only.

Read before acting:

1. `AGENTS.md`
2. the live GitHub Issue #105 and relevant #67/#103 history
3. `docs/tasks/105-generic-ytdlp-bilibili-initial-state-fallback/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/execution-anchor-recovery-protocol.md`
6. `docs/tasks/freshness-integration-protocol.md`
7. the canonical generic-ytdlp, security, runner and `ResolvedMedia` documents named by the Task

Expected Worker: `cloud-codex`
Expected environment: `env:cloud`
Handoff profile: `docs/tasks/handoffs/cloud.md`

Claim only when #105 is live `status:ready + env:cloud + no active owner`. Execute exactly the published Task Contract with deterministic offline fixtures and the verified yt-dlp `2026.08.19` wheel provenance. Prove the missing-initial-state/detail-data continuation and muxed `http-file` `ResolvedMedia` only through repository-owned fixtures.

Do not make public or real-site requests, call `scripts/generic-ytdlp-real-smoke.sh`, execute or modify #67, execute #68, inspect raw diagnostics, or weaken R008, broker/fd, sandbox, Secret, cleanup, `DisabledRunner` or Core/site boundaries. If only separate A/V is reachable, report the bounded result and recommend a later DASH/remux split; do not implement it here.

Run J1–J4 as defined by `task.md`, then post the standard `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, transition lifecycle state, release ownership and STOP. Do not merge, close, set `status:done`, or create another task.
