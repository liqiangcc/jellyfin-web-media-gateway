# Session Bootstrap — Issue #139

Execute Issue #139 `INFRA-PHONE-MGMT-RELIABILITY-PREP` as a cloud PREP Worker.

1. Read live #139, this Task Contract, `AGENTS.md`, `.agents/skills/task-worker/SKILL.md`, `docs/tasks/issue-lifecycle-protocol.md`, `docs/security.md` section 16.5, and current #113/#131 authority before claim.
2. Proceed only if #139 is OPEN + `status:ready` + `env:cloud` + no active owner.
3. Claim durably and use a Task-specific branch/PR.
4. Implement only a pure non-secret readiness classifier, deterministic offline tests, and management recovery runbook required by `task.md`.
5. Do not run Tailscale ping, SSH, phone connection, phone mutation, Runner action, workflow action, Bilibili or product work in this Task.
6. Freeze canonical readiness states: `DEVICE_OFFLINE -> TAILNET_ONLY -> SSH_READY -> UBUNTU_PERSISTENT_READY`; only the final state may set `claim_allowed=true`.
7. Treat SSH ControlMaster as optional optimization only, never authority.
8. Run deterministic offline tests, syntax/compile checks, diff-scope review and targeted secret/leak checks.
9. Before each terminal Issue mutation, follow the fresh terminal-write authority guard.
10. Report explicitly: `Live phone probe: NOT RUN` and `Live phone mutation: NOT RUN`; transition to `status:review`, release owner and STOP.
