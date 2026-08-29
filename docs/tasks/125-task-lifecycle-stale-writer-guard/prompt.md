# Session Bootstrap — Issue #125

Execute Issue #125 `TASK-LIFECYCLE-STALE-WRITER-GUARD` as a cloud implementation Worker.

Read live #125, this Task Contract, `docs/tasks/issue-lifecycle-protocol.md`, `.agents/skills/task-worker/SKILL.md`, and `docs/tasks/task-worker-terminal-write-guard.md` before claim. Claim only from OPEN + ready + env:cloud + no owner. Implement/test only stale terminal-write protection. Before your own terminal report, use the new fresh-authority rule. No product/runtime/security changes and no #67/#117 execution.