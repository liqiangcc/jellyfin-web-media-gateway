# Session Bootstrap — Issue #103

Use the repository `task-worker` skill and execute Issue #103 only.

Read completely before acting:

1. `AGENTS.md`
2. `docs/README.md`
3. `docs/requirements.md`
4. `docs/architecture.md`
5. `docs/implementation-contracts.md`
6. `docs/technical-feasibility-validation.md`
7. `docs/mvp-plan.md`
8. `docs/security.md`
9. `docs/development-environments.md`
10. `docs/runner-execution-architecture.md`
11. `docs/tasks/issue-lifecycle-protocol.md`
12. `docs/tasks/execution-anchor-recovery-protocol.md`
13. `docs/tasks/103-generic-ytdlp-resolved-media-compat-prep/task.md`
14. the complete live Issue #103 history
15. the complete live Issue #67 Attempt 9 history and final bounded report
16. accepted #79/#83/#85/#95/#97/#99/#101 authority evidence

Execution rules:

- claim only if Issue #103 is `status:ready + env:cloud + no owner`;
- use the Task Contract as the sole stable Scope authority;
- use only deterministic offline fixtures; do not execute `scripts/generic-ytdlp-real-smoke.sh` or any public/real-site request;
- do not inspect or retain raw stderr, URLs, headers, tokens, cookies, bodies, signed material or media payloads;
- preserve R008, #79/#83/#85/#95/#97/#99/#101, the accepted broker/fd/sandbox boundary and production `DisabledRunner`;
- implement only a repository-owned generic-ytdlp/ResolvedMedia compatibility correction demonstrable from bounded code and fixtures; if that cannot be established safely, report BLOCKED rather than guessing;
- run J1–J4 on hosted x86_64 and native hosted ARM64, report bounded Candidate/PR/Actions evidence, update lifecycle, release ownership and STOP;
- do not execute or modify #67 or execute #68.

The cloud handoff profile is `docs/tasks/handoffs/cloud.md`.
