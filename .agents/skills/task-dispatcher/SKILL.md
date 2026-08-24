---
name: task-dispatcher
description: Dispatch a complete GitHub Task prompt to an issue-linked local Codex tmux session after safely fast-forwarding the repository. Use only when explicitly asked to dispatch or resume a Worker session.
---

# Task Dispatcher

Use this skill to hand a complete Web GPT Worker prompt to a local Codex CLI session. This is an orchestration/bootstrap procedure; `$task-worker` remains responsible for claiming the Issue, executing `task.md`, posting the execution report, and changing GitHub task state.

## Required input

The user must provide the complete Worker prompt. Extract exactly one Issue number from it, normally in the form `Issue #123`, and use the prompt's referenced `prompt.md` unchanged.

Stop before launching if:

- no Issue number can be identified;
- multiple conflicting Issue numbers are present;
- the prompt path is missing or does not resolve in the repository;
- the required local tools or GitHub authentication are unavailable.

Do not rewrite the Worker prompt or duplicate the Task Contract into the launch command.

## Repository preflight

Resolve the repository root from the current working directory and require:

1. a clean worktree (`git status --porcelain` is empty);
2. the `main` branch;
3. `git fetch origin` followed by `git pull --ff-only origin main`.

If the worktree is dirty, the branch is not `main`, or fast-forward sync fails, stop and report the exact condition. Never use `reset --hard`, `clean`, `stash`, or checkout to hide local changes.

The sync must happen before creating or reusing a Worker session.

## Issue-linked tmux session

Derive the tmux name deterministically:

```text
Issue #123 → codex-issue-123
```

If that tmux session already exists, do not overwrite it or start a second Codex for the same Issue. Inspect/report its state and let the user choose whether to resume it.

Start tmux with a shell wrapper, then start Codex inside the shell. This allows Codex to exit while the issue-linked tmux session remains available for later tracking or resumption.

When the user explicitly requests full permissions, start Codex with:

```text
--dangerously-bypass-approvals-and-sandbox
```

This flag is opt-in only. Do not infer it from the existence of this Skill. Preserve the repository working directory with `-C`/`--cd`.

Transport the prompt as literal terminal input after Codex is ready; do not interpolate arbitrary prompt text into a shell command. If Codex asks whether to trust the repository directory, confirm only the exact repository being dispatched.

## Session mapping and resume

Always report:

```text
Issue: #<N>
tmux: codex-issue-<N>
repository: <absolute path>
base commit: <SHA after sync>
Codex session ID: <UUID when available>
```

The tmux name is the stable Issue mapping. When a Codex Session ID is available, use it for precise recovery:

```bash
codex resume <SESSION_ID>
```

Use `codex resume --last` only when there is no ambiguity about the most recent session.

## Completion and resource policy

- While Codex reports `Working`, leave the tmux session running.
- After the Worker posts its durable `[EXECUTION REPORT]` and the Issue is `status:review`, Codex may exit while tmux remains open.
- Do not close the tmux session automatically; it is retained for inspection and later resume unless the user asks to remove it.
- Before exiting Codex, ensure no required background command is still running.
- Never mark the Issue done, merge a PR, close the Issue, or start another Attempt from this Skill.

## Safety boundary

This Skill may mutate the local checkout, start processes, and send the user-provided prompt to Codex. It does not grant permission for unrelated external writes. The Worker must still obey `AGENTS.md`, the target `task.md`, the target `prompt.md`, and the repository Issue lifecycle protocol.
