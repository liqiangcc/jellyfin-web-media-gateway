# Session Bootstrap — GENERIC-YTDLP-BOUNDED-UNSUPPORTED-STAGE-PREP

Execute Issue #107 using the repository Worker protocol.

## Claim Gate

Claim only if live #107 is exactly:

```text
status:ready
env:cloud
no active owner
```

If it is draft, blocked, review, done/closed, or already owned, STOP.

## Read Before Claim

Read:

- `AGENTS.md`;
- live Issue #107 and comments;
- `docs/tasks/107-generic-ytdlp-bounded-unsupported-stage-prep/task.md`;
- `docs/tasks/issue-lifecycle-protocol.md`;
- `docs/tasks/freshness-integration-protocol.md`;
- `docs/tasks/execution-anchor-recovery-protocol.md`;
- #67 Attempt 11 bounded report / Coordinator Review;
- accepted #101 and #105 authorities plus the security/runtime authorities referenced by `task.md`.

Planning Base:

```text
6034eb1cd1837988161d955ef0d1f67d60ce0257
```

## Goal

Implement only the bounded secondary-stage classification defined by `task.md`.

The top-level worker result remains:

```text
UNSUPPORTED_FORMAT
```

The only new diagnostic authority is one small closed repository-owned stage enum covering the frozen semantic phases in `task.md`. No arbitrary string may cross the worker/runtime Evidence boundary.

## Critical Boundaries

- no public Bilibili or other real-site request;
- do not execute or modify #67 or #68;
- do not implement DASH, remux, FFmpeg, separate-A/V support, or any media-format repair;
- do not inspect/persist/publish raw stderr, exception/message text, source/media URL, headers, bodies, signed query data, credentials, Cookie/Auth/token/profile state, or media payloads;
- do not broaden #105 admission, add a caller-selectable fallback action, or create a generic retry path;
- do not weaken R008/#95/#97/#99/#83/#85/#101, sandbox/fd isolation, broker-only egress, or `DisabledRunner`;
- unknown/forged/malformed/unmapped secondary stages fail closed;
- do not infer a real media-format cause from #67's `UNSUPPORTED_FORMAT` or broker request count.

## Execution

Use deterministic offline fixtures only.

Implement the smallest in-scope Candidate, then run the exact-Candidate verification matrix from `task.md`:

```text
J1 hosted x86_64 stage taxonomy
J2 native hosted ARM64 equivalence
J3 security/static/sentinel guards
J4 full affected regressions
```

Prove C1–C8 explicitly.

For repository mutation, after the first coherent in-scope commit:

- push a durable worker branch;
- create/update one focused draft PR when appropriate;
- at most one `[EXECUTION CHECKPOINT]` after a durable recovery anchor exists;
- no heartbeat comments.

## Freshness

Record Task Candidate, Evidence Base, observed Current Main, changed-main classification, semantic Evidence reuse, affected Claims, and any JI evidence required by the dependency-aware Freshness Contract.

Do not silently rebase onto moving main merely because unrelated commits exist.

## Completion

Normal:

```text
[EXECUTION REPORT]
→ status:review
→ release active owner
→ STOP
```

Blocked:

```text
[BLOCKER REPORT]
→ status:blocked
→ release active owner
→ STOP
```

Worker must not merge, set `status:done`, close #107, create a downstream media-format Task, re-freeze/run #67, or start #68. Coordinator owns all downstream decisions.