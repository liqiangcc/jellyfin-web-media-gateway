# Session Bootstrap — R005 Bilibili Site Plugin acquisition

GitHub Issue: #237
Task Contract: `docs/tasks/237-bilibili-site-plugin-acquisition/task.md`
Expected worker: cloud Codex (`env:cloud`), `gpt-5.6-luna`, reasoning high

Before claiming, read `AGENTS.md`, Issue #237 and its comments, this Task Contract, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, and the canonical/plugin documents named by the contract. Confirm `status:ready`, matching `env:cloud`, no active owner, and the current main base.

This is a contract-first production plugin implementation. Keep Bilibili semantics in `plugins/bilibili`; keep Core and Browser Worker generic. Do not perform real Bilibili/login/tx-node/phone/TV/VNC/CDP actions, do not touch #191/#195, and do not run local build/test/install. Required verification is GitHub Actions on the exact Candidate SHA. Follow the worker lifecycle: claim one Attempt, implement, report, move to `status:review` or `status:blocked`, release ownership, then stop.
