# TASK-LIFECYCLE-STALE-WRITER-GUARD

## Identity

- Issue: #125
- Kind: workflow / lifecycle protocol hardening
- Environment: `env:cloud`
- Scope: task-worker lifecycle protocol, coordinator/worker helper logic, deterministic tests and documentation only
- Product/runtime/security semantics: unchanged

## Problem

A Worker may remain alive after Coordinator Final Acceptance. Without a terminal-write guard, that stale Worker can later post `[EXECUTION REPORT]` / `[BLOCKER REPORT]`, change `status:*`, alter ownership, or reopen/overwrite the durable terminal state. #117 exposed this race after Final Acceptance.

## Goal

Make terminal Worker writes fail closed when live Issue authority no longer belongs to that Attempt.

## Required behavior

Immediately before every terminal Worker mutation sequence, the Worker must perform a fresh live-state read-back of the Issue. Terminal mutations include:

- posting `[EXECUTION REPORT]`;
- posting `[BLOCKER REPORT]`;
- changing `status:in-progress` to `status:review` or `status:blocked`;
- releasing/changing active execution ownership as part of Attempt completion.

The guard must reject the mutation sequence and STOP without writing when any of these is true:

1. Issue is closed;
2. `status:done` is present;
3. durable `[FINAL ACCEPTANCE]` exists;
4. live state no longer identifies the current Attempt as authorized/in-progress;
5. active owner/claim no longer matches the Worker where ownership is required.

Coordinator-only Final Acceptance / close operations are not Worker terminal writes and remain governed by the existing Final Acceptance Gate.

## Race requirement

A single read at Worker startup is insufficient. The terminal guard must be evaluated immediately before terminal mutation. If the implementation performs multiple GitHub mutations, it must revalidate authority at the last safe point before the first irreversible terminal write and avoid later state writes when that authority has been invalidated.

This Task does not claim GitHub multi-operation atomicity. It must make stale writes fail closed and minimize the mutation window without inventing a distributed lock.

## Claims

- C1: A normally authorized in-progress Worker can still report and transition to review/blocked.
- C2: A Worker observing `closed` is rejected before terminal write.
- C3: A Worker observing `status:done` is rejected before terminal write.
- C4: A Worker observing durable Final Acceptance is rejected before terminal write.
- C5: A Worker whose claim/owner/Attempt authority was superseded is rejected before terminal write.
- C6: Rejection performs no report/status/ownership mutation and returns a bounded stale-authority result.
- C7: Coordinator Final Acceptance lifecycle remains unchanged.
- C8: Existing lifecycle protocol examples explicitly require fresh terminal read-back.

## Required evidence

Deterministic tests must cover at least:

- authorized execution-report path;
- authorized blocker-report path;
- closed Issue negative;
- `status:done` negative;
- Final Acceptance negative;
- owner/claim mismatch negative;
- stale Attempt/superseded authority negative;
- no-mutation assertion for every rejected case.

If the repository has no executable task-worker helper, implement the smallest repository-owned guard/helper plus tests that can be consumed by Worker tooling; do not create unrelated orchestration infrastructure.

## Boundaries

- no Gateway product/runtime behavior change;
- no media/site/browser/network behavior change;
- no security-policy weakening;
- no rewrite/deletion of append-only Issue history;
- no attempt to repair #117 evidence itself;
- no new distributed lock/service;
- no GitHub token/credential material in tests or docs.

## Success criteria

PASS requires C1-C8 with deterministic tests, lifecycle documentation updated coherently, clean secret/leak review, and a narrowly scoped Candidate/PR. Existing task lifecycle behavior must remain compatible for non-stale Workers.
