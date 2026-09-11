# Session Bootstrap — Authorized Bilibili target playback verification

```text
GitHub Issue: #246
Task Contract: docs/tasks/246-authorized-bilibili-target/task.md
Expected worker: cloud
Expected environment: env:cloud
Handoff profile: docs/tasks/handoffs/cloud.md
```

Read `AGENTS.md`, Issue #246 and all comments, this Task Contract, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, accepted #235/#237/#240/#243/#248/#251/#255/#257/#259/#271 evidence, and verify current accepted integration base `fae389b2320b58331a2cf14d505ccde9ee02eda2`.

Freshness anchors to read before any Publication Gate:

```text
#271 accepted Candidate: b0d0b8c6ac2935038a593b9a1175f6158588bf04
#271 merge: fae389b2320b58331a2cf14d505ccde9ee02eda2
#271 required hosted run: 34479335766
merge-head portable-ci: 34481256400
merge-head r002-deployment-validation: 34481256373
```

The historical integration base `289f4357558b211ebb9be68b7f6fb8b731006324` is stale and must not be used as the target execution identity. The Coordinator must freeze an exact current execution Candidate and J3 hosted freshness evidence at Publication Gate.

Before claiming, verify the Issue is `status:ready`, `env:cloud`, owner-free, this refreshed package has been independently read back, and all six authorization/target prerequisites in `task.md` are explicitly recorded in the Issue. If any prerequisite is absent, do not claim and do not perform login or target activity; keep the Issue `status:blocked`, owner-free, and stop.

No phone/TV/VNC/CDP, no copied profile, no Cookie/token smuggling, no #191/#195, no anonymous rerun, and no local build/test/install. Never request or persist passwords, cookies, raw tokens, profile archives or signed media URLs in Issue comments/artifacts.

If and only if the Publication Gate is complete and the Issue is `status:ready`, claim one Attempt, use only the approved isolated tx-node channel/low-privilege identity, capture sanitized bounded evidence, classify each claim, post `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, transition to `status:review` or `status:blocked`, release ownership, and stop. Never close the Issue or declare parent #68 accepted.
