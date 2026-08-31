# Session Bootstrap — Issue #136

Execute Issue #136 `INFRA-RUNNER-REREG-PREP` as a cloud PREP Worker.

1. Read live #136 including the `[CONTRACT REVISION — OFFICIAL TOKEN INTERFACE]`, this Task Contract, #131 Attempt 5 + latest Coordinator recovery authority, `AGENTS.md`, `.agents/skills/task-worker/SKILL.md`, and lifecycle protocols before claim.
2. Proceed only if #136 is OPEN + `status:ready` + `env:cloud` + no active owner.
3. Claim durably and use a Task-specific worker branch/PR.
4. Implement only the smallest pure precondition/identity guard, token-safe fake-tested adapter, deterministic offline tests, and recovery runbook required by `task.md`.
5. Tests MUST use a fake `config.sh`; do not create real registration/remove tokens and do not connect to or mutate the phone/GitHub Runner.
6. Preserve the official-interface token constraint: the secret may exist only in the short-lived fake/official `config.sh --token` child argv; wrapper input is stdin/ephemeral, tracing is disabled, and no token reflection/persistence is allowed.
7. Freeze identity preservation as repository + runner name `ubuntu-arm64-target-phone` + live-frozen labels + accepted work directory + final uid 999 boundary. Do not claim numeric GitHub runner id preservation across re-registration.
8. Do not use `--replace`, retries, re-registration loops, runner lifecycle mutation, workflow rerun/cancel/dispatch, Bilibili/#67/#68/#113 work, or product changes.
9. Run deterministic offline tests, syntax/lint/compile checks, diff-scope review, and targeted secret/leak checks.
10. Before each terminal Issue mutation, follow the current fresh terminal-write authority guard.
11. Report explicitly: `Live Runner mutation: NOT RUN` and `Live phone connection: NOT RUN`, transition to `status:review`, release owner, and STOP.
