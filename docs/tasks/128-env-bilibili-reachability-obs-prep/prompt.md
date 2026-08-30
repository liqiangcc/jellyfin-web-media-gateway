# Session Bootstrap — Issue #128

Execute Issue #128 `ENV-BILIBILI-REACHABILITY-OBS-PREP` as a cloud implementation Worker.

1. Read live Issue #128, this Task Contract, `AGENTS.md`, `.agents/skills/task-worker/SKILL.md`, `docs/tasks/issue-lifecycle-protocol.md`, and #113's current frozen reachability Task Contract before claim.
2. Proceed only if #128 is OPEN + `status:ready` + `env:cloud` + no active owner.
3. Claim durably, read back the claim, and use a Task-specific branch/PR.
4. Implement only the smallest pure/offline privacy-safe endpoint-correlation helper plus deterministic tests/docs required by `task.md`.
5. Do not run Bilibili or any other real-site probe. Do not execute #67/#68/#113 and do not change the ordinary direct/no-proxy HTTP request shape.
6. Raw remote IP/DNS/address data must never enter durable output, logs, argv, fixtures, Issue comments, or artifacts. Do not use a plain/unsalted hash as the endpoint alias.
7. Preserve #113's existing two-consecutive-`2xx` PASS rule and all anti-bypass restrictions.
8. Run the required offline tests, scoped regressions, diff-scope review, and targeted secret/leak checks.
9. Before each terminal Issue mutation, follow the fresh terminal-write authority guard from the current task-worker protocol.
10. Post a bounded `[EXECUTION REPORT]`, transition to `status:review`, release ownership only while authority remains current, and STOP.

Output must explicitly state `Live Bilibili probe: NOT RUN`.
