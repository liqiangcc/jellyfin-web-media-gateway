# Session Bootstrap — Issue #90

Use this file only as the Worker entrypoint. The stable execution contract is `task.md`.

## Start

Repository:

```text
liqiangcc/jellyfin-web-media-gateway
```

Issue:

```text
#90 ENV-ARM64-GITHUB-SMARTHTTP-PREP
```

Task Contract:

```text
docs/tasks/90-env-arm64-github-smarthttp-prep/task.md
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

Target Evidence is collected through the trusted `ubuntu-arm64-target-phone` GitHub Actions Runner; do not treat Cloud itself as the target.

## Mandatory read order

Before claim, actually read from GitHub:

1. Issue #90 and all comments;
2. `AGENTS.md`;
3. `docs/tasks/90-env-arm64-github-smarthttp-prep/task.md`;
4. `docs/tasks/issue-lifecycle-protocol.md`;
5. `docs/tasks/freshness-integration-protocol.md`;
6. `docs/tasks/handoffs/cloud.md`;
7. Issue #89, especially Attempt 1 `[BLOCKER REPORT]`;
8. Issue #85 current comments and PR #86 identity;
9. `.github/workflows/broker-fd-isolation.yml` at preserved Candidate `4af64b124af4d1599a87bd211395ee832e9d7e4b`;
10. `docs/runner-execution-architecture.md` and relevant `docs/security.md` sections.

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

#89 has already proved that simple curl and `git ls-remote` can succeed while exact repository-object fetch fails. Do not repeat its full generic Direct/Proxy survey as if nothing is known.

The new Task must use a trusted **pre-checkout** target workflow to examine the real Actions Git transport context without leaking workflow credentials. If smart-HTTP remains unreliable, the same Task may prove the exact hosted source-bundle artifact fallback defined by `task.md`; it must not silently convert that fallback into a claim that `actions/checkout` works.

Preserve the exact #85 Candidate unless Coordinator revises the Contract:

```text
4af64b124af4d1599a87bd211395ee832e9d7e4b
```

Do not modify #85 fd-isolation semantics, do not run #85 J4 fd-isolation proof, and do not start #67.

## Finish

Normal completion:

```text
post [EXECUTION REPORT] to Issue #90
→ status:review
→ release active owner
→ STOP
```

Blocked:

```text
post [BLOCKER REPORT] to Issue #90
→ status:blocked
→ release active owner
→ STOP
```

Worker must not set `status:done`, close Issue #90, unblock/close #89, resume #85, merge PR #86, or execute the next Task.