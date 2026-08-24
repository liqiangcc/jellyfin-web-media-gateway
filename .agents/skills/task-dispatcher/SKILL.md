---
name: task-dispatcher
description: Dispatch or track an issue-linked local Codex Worker session through tmux with an isolated git worktree. Use only when explicitly asked to dispatch, inspect, track, or resume a Worker session.
---

# Task Dispatcher

Use this skill from a long-lived dispatcher/coordinator Codex to launch and track issue-linked **local Codex CLI** Worker sessions.

This is orchestration/bootstrap only. `$task-worker` remains responsible for claiming the Issue, executing `task.md`, posting the execution/blocker report, and changing Worker lifecycle state. `$task-reviewer` / Web Coordinator remains responsible for ACCEPT / REVISE / BLOCK / SPLIT, merging, Final Acceptance, and closing.

## Required input

For a new dispatch, the user/upstream Coordinator must provide the complete Worker handoff. It should normally contain exactly one Issue and one repository `prompt.md`, for example:

```text
$task-worker Execute Issue #123 using `docs/tasks/123-example/prompt.md`.
```

Extract exactly one Issue number and the referenced `prompt.md` path. Preserve the Worker handoff unchanged when sending it to the child Codex.

For a tracking/resume request, an Issue number is sufficient if the existing issue-linked session/worktree can be resolved unambiguously.

Stop before launching if:

- no Issue number can be identified;
- multiple conflicting Issue numbers are present;
- the prompt path is missing or does not resolve in the repository;
- the required local tools or GitHub authentication are unavailable.

Do not rewrite the Worker prompt or duplicate the Task Contract into the launch command.

## GitHub and routing preflight

Before launching any new child Worker, read the live GitHub Issue first.

Require:

```text
Issue is open
status = ready
no active execution owner
prompt.md / task.md resolve
actual child Worker environment is explicitly known and eligible for the Issue
```

Do not launch a duplicate Worker for an Issue already `status:in-progress`, `status:review`, `status:blocked`, or `status:done`. In those states, switch to tracking/recovery reporting instead.

Worker environment is not inferred from CPU architecture or from the fact that a machine is remote. In particular:

```text
env:cloud
= Codex Cloud Worker
!= arbitrary local/server Codex CLI inside tmux
```

If the Issue is published only for an environment that does not match the actual local child Worker, stop and report the routing mismatch. Do not silently treat a local tmux session as Codex Cloud, Windows, target-phone, or Manual-TV Evidence authority.

## Dispatcher repository preflight

Resolve the dispatcher repository root from the current working directory and require:

1. a clean dispatcher checkout (`git status --porcelain` is empty);
2. the dispatcher checkout is on `main`;
3. `git fetch origin` followed by `git pull --ff-only origin main` succeeds.

If the dispatcher checkout is dirty, not on `main`, or cannot fast-forward, stop and report the exact condition. Never use `reset --hard`, `clean`, `stash`, or checkout to hide local changes.

Record the exact synced `main` SHA before creating a new Worker worktree.

## One Issue = one isolated worktree

Never launch child Workers from the dispatcher checkout itself. Multiple issue-linked Codex sessions sharing one worktree can change branches/files underneath each other and corrupt parallel execution.

Use a deterministic sibling worktree root, conceptually:

```text
<repo-root>.worktrees/issue-123
```

For a new Issue session:

1. ensure no tmux session for the Issue already exists;
2. ensure the target worktree path does not already contain unresolved/stale work;
3. create a detached worktree from the exact synced `origin/main` / recorded main SHA, for example with `git worktree add --detach`;
4. launch the child Codex with that Issue worktree as its repository directory.

The child `$task-worker` may then create/switch to its own task branch as required by the Task Contract.

Do not automatically delete, reset, reuse, or repoint an existing issue worktree. If a worktree exists without a healthy tmux session, inspect GitHub state + worktree status and report it as a recovery case.

## Issue-linked tmux session

Derive the tmux name deterministically:

```text
Issue #123 → codex-issue-123
```

If that tmux session already exists, do not overwrite it or start a second Codex for the same Issue. Inspect/report its state instead.

Start tmux in the **issue worktree**, with a shell wrapper, then start Codex inside that shell. This allows Codex to exit while the issue-linked tmux session remains available for later tracking or recovery.

When the user explicitly requests full permissions, start Codex with:

```text
--dangerously-bypass-approvals-and-sandbox
```

This flag is opt-in only. Do not infer it from the existence of this Skill. Preserve the issue worktree directory with `-C` / `--cd`.

Transport the Worker handoff as literal terminal input after Codex is ready; do not interpolate arbitrary prompt text into a shell command. If Codex asks whether to trust the repository directory, confirm only the exact issue worktree being dispatched.

## Progress tracking

GitHub durable state is the authority; tmux is only process/liveness evidence.

When asked to track progress:

1. read the Issue and latest relevant comments first;
2. inspect the issue-linked tmux session if it exists;
3. inspect the issue worktree branch/HEAD/dirty state without mutating it;
4. summarize both durable Task state and local Worker liveness.

Use this interpretation:

```text
status:ready + no tmux
→ published, not currently running locally

status:in-progress + live Codex/tmux
→ active Worker Attempt

status:in-progress + missing/dead Codex/tmux
→ possible stale Worker; report for Coordinator recovery, do not auto-start a replacement

status:review
→ Worker finished durably; Coordinator review is next

status:blocked
→ durable blocker exists; do not auto-resume until lifecycle authority resolves it

status:done / closed
→ Task complete; retained tmux/worktree is only historical/debug state
```

Do not treat terminal text such as `Working`, a quiet pane, or Codex process exit as Task completion. Completion requires the durable Issue report/state defined by `docs/tasks/issue-lifecycle-protocol.md`.

A tracking summary should include, when available:

```text
Issue: #<N>
GitHub status / owner:
tmux: codex-issue-<N> (alive/dead/missing)
worktree: <absolute path>
worktree branch / HEAD:
Codex session ID: <UUID when available>
latest durable report/comment:
next authority/action:
```

## Session mapping and resume

Always report after a new dispatch:

```text
Issue: #<N>
tmux: codex-issue-<N>
dispatcher repository: <absolute main checkout>
worker worktree: <absolute issue worktree>
base commit: <exact synced main SHA used to create worktree>
Codex session ID: <UUID when available>
```

The tmux name + worktree path are the stable Issue mapping. When a Codex Session ID is available, use it for precise recovery:

```bash
codex resume <SESSION_ID>
```

Use `codex resume --last` only when there is no ambiguity about the most recent session **inside that issue worktree/session context**.

Never resume merely because a process died. First reconcile the Issue lifecycle. A dead local process with `status:in-progress` is a Coordinator recovery condition, not permission to create a new Attempt automatically.

## Completion and resource policy

- While the Worker is genuinely active, leave its tmux session and issue worktree intact.
- After the Worker posts a durable `[EXECUTION REPORT]` and the Issue is `status:review`, Codex may exit while tmux/worktree remain available for inspection.
- Do not close tmux or remove the issue worktree automatically.
- Remove a retained tmux/worktree only when the user explicitly requests cleanup, no Worker is active, and the worktree has no uncommitted/unpushed work that would be lost.
- Before exiting Codex, ensure no required background command is still running.
- Never mark the Issue done, merge a PR, close the Issue, publish another Task, or start another Attempt from this Skill.

## Safety boundary

This Skill may fast-forward the dispatcher checkout, create isolated git worktrees, start local processes, inspect tmux state, and send the user-provided Worker handoff to Codex. It does not grant permission for unrelated external writes or for changing Task routing/lifecycle authority.

The child Worker must still obey `AGENTS.md`, the target Issue, `task.md`, `prompt.md`, `docs/tasks/issue-lifecycle-protocol.md`, and the actual environment/Runner/Target authority required by the Task.
