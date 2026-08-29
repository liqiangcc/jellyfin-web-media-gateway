# TASK-LIFECYCLE-STALE-WRITER-GUARD

## Identity

- Issue: #125
- Kind: workflow / lifecycle protocol hardening
- Environment: `env:cloud`
- Scope: task-worker lifecycle protocol, helper logic, deterministic tests and documentation only
- Product/runtime/security semantics: unchanged

## Problem

A Worker may remain alive after Coordinator Final Acceptance. Without a terminal-write guard, that stale Worker can later post terminal reports, change status/ownership, or overwrite durable terminal state. #117 exposed this race.

## Goal

Make terminal Worker writes fail closed when live Issue authority no longer belongs to that Attempt.

## Required behavior

Immediately before every terminal Worker mutation sequence, freshly read the Issue. Reject and STOP without writing when the Issue is closed, `status:done`, durable `[FINAL ACCEPTANCE]` exists, the Attempt is superseded, or active owner/claim no longer matches. Coordinator Final Acceptance remains unchanged.

## Claims

- C1 authorized in-progress Worker can report normally.
- C2 closed Issue rejected.
- C3 `status:done` rejected.
- C4 Final Acceptance rejected.
- C5 owner/Attempt supersession rejected.
- C6 rejection has zero Issue mutation authority and bounded result.
- C7 Coordinator Final Acceptance unchanged.
- C8 lifecycle documentation requires fresh terminal read-back.

## Required evidence

Deterministic tests cover authorized execution/blocker paths, closed/done/final-acceptance/owner-mismatch/attempt-superseded negatives and no-mutation purity.

## Boundaries

No product/media/browser/site/security change; no history rewrite; no distributed lock; no credential material.

## Success criteria

PASS requires C1-C8, deterministic tests, coherent lifecycle documentation, clean secret/leak review, and narrow Candidate/PR.