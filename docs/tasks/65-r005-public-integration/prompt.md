# Session Bootstrap — R005-PUBLIC-INTEGRATION

You are executing the integration-only child Task for the preserved Bilibili public adapter.

## Execution Context

```text
Repository: liqiangcc/jellyfin-web-media-gateway
GitHub Issue: #65
Task Contract: docs/tasks/65-r005-public-integration/task.md
Expected worker: cloud-codex
Expected environment: env:cloud
Reuse branch: worker/issue-23-r005-public
Reuse PR: #37
Preserved Task Candidate: eb03c199481191a88897ba6b45252bbaa957a63e
Frozen Integration Base: aae02b505bde65b39c6eab1e5ee441decbe8186a
Parent: #23 (must remain blocked waiting #36)
```

## Start

Actually read and obey:

1. `AGENTS.md`
2. Issue #65 and relevant comments
3. `docs/tasks/65-r005-public-integration/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/freshness-integration-protocol.md`
6. Issue #23 latest blocker/decomposition/preservation comments
7. Issue #36 current draft
8. PR #37 current state
9. current accepted R008/SiteAdapter/SourceSession authority referenced by task.md

Claim only if live #65 is still:

```text
status:ready
env:cloud
no active owner
```

## Critical boundaries

- integration-only: preserve `eb03c199...` ancestry and reuse PR #37;
- prefer merge commit of frozen Integration Base; no rebase/force-push rewriting;
- if conflict touches Bilibili semantics, SiteAdapter meaning, R007, R008/Secret authority or needs redesign, BLOCK rather than guessing;
- run deterministic/current integration Evidence only;
- **do not make a real Bilibili request** in #65;
- do not login/import Cookie/profile/use proxy/fingerprint/CAPTCHA/access-control bypass;
- do not merge PR #37;
- do not unblock/finalize #23 or publish/execute #36;
- PR #37 remains draft;
- normal finish: `[EXECUTION REPORT] -> status:review -> release owner -> STOP`;
- blocker: `[BLOCKER REPORT] -> status:blocked -> release owner -> STOP`.

The exact Integration Candidate produced here is intended to be frozen later by the Coordinator for #36 real-site J3.