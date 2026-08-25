# Session Bootstrap — CONTROL-UI-PREP

You are executing Issue #47 in `liqiangcc/jellyfin-web-media-gateway`.

```text
GitHub Issue: #47
Task Contract: docs/tasks/47-control-ui-prep/task.md
Expected worker: cloud
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

Before claim, actually read live GitHub/repository state, then read `AGENTS.md`, Issue #47 and relevant comments, the Task Contract, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, accepted #40/#45 APIs and the canonical docs required by the Task.

Claim only if Issue #47 is still open, `status:ready + env:cloud`, has no active owner and current environment has the required capabilities. Read back the claim before implementation.

Execute only the frozen Contract. Do not absorb #44 source/session creation, expose a production session seed, create client-side Playback/Display authority, or redefine #38/R007/#40/#45 semantics.

Use the Task's dependency-aware freshness policy. Do not chase unrelated main changes; report Task Candidate, Evidence Base/observed main and exact J1/J2/J3 Evidence for Coordinator freshness classification.

Normal completion:

```text
[EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Blocked completion:

```text
[BLOCKER REPORT]
→ status:blocked
→ release owner
→ STOP
```

Do not merge, set `status:done`, close #47, execute #44/#48, or start another Attempt automatically.
