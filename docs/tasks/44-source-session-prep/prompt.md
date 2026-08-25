# Session Bootstrap — SOURCE-SESSION-PREP

You are executing Issue #44 in `liqiangcc/jellyfin-web-media-gateway`.

## Execution Context

```text
GitHub Issue: #44
Task Contract: docs/tasks/44-source-session-prep/task.md
Expected worker: cloud
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
```

## Start Protocol

Before claim, actually read GitHub/live repository state, then read:

1. `AGENTS.md`
2. Issue #44 and all relevant comments
3. `docs/tasks/44-source-session-prep/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/execution-anchor-recovery-protocol.md`
6. `docs/tasks/freshness-integration-protocol.md`
7. accepted #45 Web Display implementation/API on current main
8. canonical docs referenced by the Task Contract

Claim only when Issue #44 is still open, `status:ready + env:cloud`, has no active owner, and current environment has the required capabilities.

A successful claim starts the next Attempt and must be read back before write-side implementation begins. Reuse any durable branch/PR from prior Attempts if present.

Execute only the frozen Task Contract. In particular, do not invent new Display lease/generation authority, do not reintroduce a public raw-media/session seed API, and do not add concrete site/yt-dlp branches to Core.

For freshness, follow the Task's `dependency-aware` contract. Do not chase unrelated `main` changes or self-trigger a full rerun solely because the main SHA advanced; report Task Candidate, Evidence Base/observed main and exact Evidence so Coordinator can classify freshness.

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

Do not merge the PR, set `status:done`, close the Issue, execute #47/#48, or start another Attempt automatically.
