# Session Bootstrap — Authorized Bilibili target playback verification

```text
GitHub Issue: #246
Task Contract: docs/tasks/246-authorized-bilibili-target/task.md
Expected worker: cloud
Expected environment: env:cloud
Handoff profile: docs/tasks/handoffs/cloud.md
```

Read `AGENTS.md`, Issue #246 and all comments, this Task Contract, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, and accepted #235/#237/#240/#243/#248/#251/#255/#257/#259 evidence and verify current main `289f4357558b211ebb9be68b7f6fb8b731006324`.

Before claiming, verify the Issue is `status:ready`, `env:cloud`, owner-free, and contains all six authorization/target prerequisites in `task.md`. If any is absent, do not perform login or target activity; post `[BLOCKER REPORT]`, keep/revert the Issue to `status:blocked`, release ownership, and stop. No phone/TV/VNC/CDP, no copied profile, no Cookie/token smuggling, no #191/#195, no anonymous rerun, and no local build/test/install.

If unblocked, claim one Attempt, use only the approved isolated tx-node channel, capture sanitized bounded evidence, classify each claim, post `[EXECUTION REPORT]`, set `status:review`, release ownership, and stop. Never close the Issue or declare parent #68 accepted.
