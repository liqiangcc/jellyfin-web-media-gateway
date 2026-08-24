# Repository Codex Skills

This directory contains repo-scoped Codex Skills for the Task lifecycle.

Codex discovers repository skills from `.agents/skills/<skill>/SKILL.md` when launched inside the repository.

## Skills

```text
$task-publisher
→ Coordinator: materialize + verify + publish a Task Package

$task-worker
→ Worker: claim + execute exactly one Attempt + report to Issue

$task-reviewer
→ Coordinator: review Evidence + REVISE/BLOCK/SPLIT/ACCEPT + close when valid
```

Lifecycle:

```text
$task-publisher
      ↓
status:ready
      ↓
$task-worker
      ↓
status:review / status:blocked
      ↓
$task-reviewer
      │
      ├── REVISE → status:ready → $task-worker
      ├── BLOCK → unblock → status:ready → $task-worker
      ├── SPLIT → $task-publisher for child Task(s)
      └── ACCEPT → [FINAL ACCEPTANCE] → status:done → close
```

## Authority

Skills are procedures, not Task Contracts.

```text
canonical docs
→ AGENTS.md
→ task.md
→ prompt.md

Issue fields / labels
→ live state

Issue comments
→ append-only Attempt / Review history

Skills
→ how to execute the lifecycle correctly
```

The detailed workflow remains in:

- `AGENTS.md`
- `docs/tasks/README.md`
- `docs/tasks/issue-lifecycle-protocol.md`
- `docs/tasks/task.template.md`
- `docs/tasks/prompt.template.md`

Do not duplicate those documents into Skills.

## Invocation policy

All three lifecycle skills set `allow_implicit_invocation: false` because they can mutate GitHub Task state. Invoke them explicitly with `$task-publisher`, `$task-worker`, or `$task-reviewer`.

Examples:

```text
$task-publisher Publish the Ubuntu ARM64 Target Runner bootstrap Task.

$task-worker Execute Issue #123 using `docs/tasks/123-runner-bootstrap/prompt.md`.

$task-reviewer Review Issue #123 and continue the Task lifecycle.
```

## Scripts

The first version is intentionally instruction-only. Add scripts only for checks that prove repetitive and benefit from deterministic validation after real Task Attempts expose stable automation requirements.
