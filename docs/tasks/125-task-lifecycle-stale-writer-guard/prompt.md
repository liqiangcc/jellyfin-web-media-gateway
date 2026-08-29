# Session Bootstrap — Issue #125

Execute Issue #125 `TASK-LIFECYCLE-STALE-WRITER-GUARD` as a cloud implementation Worker.

1. Read live Issue #125, this Task Contract, and `docs/tasks/issue-lifecycle-protocol.md` before claim.
2. Claim only if live state is OPEN + `status:ready` + `env:cloud` + no active owner.
3. Implement the smallest repository-owned terminal-write guard and deterministic tests required by `task.md`.
4. Update lifecycle documentation so every Worker terminal report/status/ownership sequence performs a fresh live-state authority read-back immediately before mutation.
5. Stale authority must fail closed: no report, status, owner, reopen, or other terminal mutation.
6. Preserve Coordinator Final Acceptance semantics and all product/runtime/security behavior.
7. Run scoped tests plus relevant workflow/protocol regressions and secret/leak checks.
8. Commit/push a Candidate, open a PR if repository protocol requires it, post a bounded `[EXECUTION REPORT]`, transition to `status:review`, release ownership, and STOP.

Do not modify product/media/browser/site/security semantics. Do not execute #67/#68/#113/#117. Do not rewrite existing Issue history.
