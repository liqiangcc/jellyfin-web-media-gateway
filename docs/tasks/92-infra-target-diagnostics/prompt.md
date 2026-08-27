# Session Bootstrap — Issue #92

Use this file only as the Worker entrypoint. The stable execution contract is `task.md`.

## Start

Repository:

```text
liqiangcc/jellyfin-web-media-gateway
```

Issue:

```text
#92 INFRA-004-TARGET-DIAGNOSTICS
```

Task Contract:

```text
docs/tasks/92-infra-target-diagnostics/task.md
```

Handoff profile:

```text
docs/tasks/handoffs/cloud.md
```

Expected Worker / environment:

```text
cloud-codex
env:cloud
```

Target Evidence must come from the trusted `ubuntu-arm64-target-phone` GitHub Actions Runner. Cloud is the repository Worker/orchestrator, not the target.

## Mandatory read order

Before claim, actually read from GitHub:

1. Issue #92 and all comments;
2. `AGENTS.md`;
3. `docs/tasks/92-infra-target-diagnostics/task.md`;
4. `docs/tasks/issue-lifecycle-protocol.md`;
5. `docs/tasks/freshness-integration-protocol.md`;
6. `docs/tasks/handoffs/cloud.md`;
7. `.github/workflows/target-runner-smoke.yml` on current main;
8. `docs/runner-execution-architecture.md`;
9. relevant `docs/security.md` target/Secret/TLS sections;
10. #90 Attempt 1 checkpoint and PR #91 only to understand the downstream consumer/bootstrap problem.

Do not infer live state from old chat.

## Claim gate

Claim only if GitHub live state shows:

```text
status:ready
env:cloud
no active owner
```

Then start the next Attempt according to `issue-lifecycle-protocol.md`.

## Critical startup reminder

Do **not** create another brand-new task-specific `workflow_dispatch` workflow as the primary implementation. The Task exists specifically to reuse and extend the already-default-branch trusted workflow identity:

```text
.github/workflows/target-runner-smoke.yml
```

Preserve existing smoke behavior while adding only the fixed first profiles:

```text
baseline
github-transport
```

The diagnostics API must be capability-oriented, not command-oriented. No arbitrary command/script/URL/proxy/path input, no env dump, no Secret output, no TLS weakening, no product/site work.

The key verification requirement is **pre-merge real Target Evidence against the exact worker branch/ref** using the existing workflow identity. If that branch/ref execution model is unavailable, report a blocker instead of asking to merge unverified target workflow code.

For `github-transport`, use the preserved #85 Candidate when an exact object-transfer probe is required:

```text
4af64b124af4d1599a87bd211395ee832e9d7e4b
```

A failed object fetch is acceptable diagnostic output for #92 if faithfully captured; #92 proves the diagnostics mechanism, not #90 transport recovery.

## Finish

Normal completion:

```text
post [EXECUTION REPORT] to Issue #92
→ status:review
→ release active owner
→ STOP
```

Blocked:

```text
post [BLOCKER REPORT] to Issue #92
→ status:blocked
→ release active owner
→ STOP
```

Worker must not set `status:done`, close #92, unblock #90, merge PR #91, resume #85, or execute #67.