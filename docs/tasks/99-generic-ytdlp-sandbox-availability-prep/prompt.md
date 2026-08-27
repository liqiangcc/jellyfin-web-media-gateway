# Session Bootstrap — Issue #99

Use the repository `task-worker` skill and execute Issue #99 only.

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
12. `docs/tasks/99-generic-ytdlp-sandbox-availability-prep/task.md`
13. the complete live Issue #99 history
14. #67 R7 / Attempt 7 blocker report
15. accepted #83 sandbox and #79/#85/#95/#97 authority evidence

Execution rules:

- claim only if Issue #99 is `status:ready + env:cloud + no owner`;
- reuse the published Task Contract without broadening Scope;
- implement the smallest clean-build/sandbox-binding repair;
- do not make the sandbox path caller-selectable;
- do not add an unsandboxed fallback;
- do not execute a real Bilibili/site request;
- preserve #79/#83/#85/#95/#97/R008 and production DisabledRunner;
- produce exact-Candidate J1–J4 Evidence;
- post `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, update lifecycle state,
  release ownership and STOP;
- do not execute or republish #67.

Critical expected result:

```text
isolated clean target directory
→ exact-Candidate real-smoke and ytdlp-sandbox both built
→ exact sibling sandbox bound
→ ARM64 deterministic broker-capable runtime reached
→ process_error != SANDBOX_UNAVAILABLE
→ no real site request
```
