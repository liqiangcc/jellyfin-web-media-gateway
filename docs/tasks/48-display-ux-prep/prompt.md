# Session Bootstrap — DISPLAY-UX-PREP

You are executing Issue #48 in `liqiangcc/jellyfin-web-media-gateway`.

```text
GitHub Issue: #48
Task Contract: docs/tasks/48-display-ux-prep/task.md
Expected worker: cloud
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

Before claim, actually read live GitHub/repository state, then read `AGENTS.md`, Issue #48 and relevant comments, the Task Contract, lifecycle/recovery/freshness protocols, accepted #45 implementation, current `site-adapter-api`/plugins/R001 media path and canonical docs referenced by the Contract.

Claim only if Issue #48 is still open, `status:ready + env:cloud`, unowned, and the environment has the required capabilities. Read back the claim before implementation.

Execute only the frozen Contract. Do not absorb #44 source/session creation, #47 Control UI, physical-TV #7 Evidence, real-site subtitle extraction, or redesign R007/#45 authority.

Use `Freshness policy: dependency-aware`. In particular, changes to `ResolvedMedia` are semantic for tasks that consume it; unrelated main changes are not automatic full-rerun triggers. Report Task Candidate, Evidence Base/observed main and exact Evidence for Coordinator classification.

Normal completion:

```text
[EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Blocker:

```text
[BLOCKER REPORT]
→ status:blocked
→ release owner
→ STOP
```

Do not merge, set `status:done`, close #48, execute #44/#47/#7, or start another Attempt automatically.
