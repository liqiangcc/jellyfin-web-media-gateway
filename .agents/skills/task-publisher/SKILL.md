---
name: task-publisher
description: Publish or republish an independent Worker Task for jellyfin-web-media-gateway. Use only when explicitly asked to create, materialize, publish, or republish a Task Package; do not use to execute or review an existing task.
---

# Task Publisher

Publish one independent Worker Task through the repository's Task Publication Gate and produce environment-specific downstream handoff entries.

## Authority

This skill is an execution procedure, not a source of product scope.

Before writing anything, read:

1. `AGENTS.md`
2. `docs/tasks/README.md`
3. `docs/tasks/task.template.md`
4. `docs/tasks/prompt.template.md`
5. `docs/tasks/issue-lifecycle-protocol.md`
6. `docs/tasks/handoffs/README.md`
7. the handoff profile(s) for the intended eligible environment(s)
8. the canonical docs relevant to the requested Goal / Research Item

Do not copy those documents into this skill or silently redefine Scope, Claims, Success Criteria, architecture, security, or Evidence requirements.

## Inputs

Resolve from the explicit request and repository state:

- Goal / Parent Goal / Research Item;
- Task kind: `implementation | verification | combined | research`;
- expected Worker;
- eligible `env:*` label(s);
- Required Capabilities;
- Task decomposition decision;
- Success Criteria / Claims / Evidence Contract;
- base commit from the actual target branch.

If a required value cannot be resolved, stop before publication rather than inventing it.

## GitHub capability

Use an authenticated GitHub write path available in the current environment.

If Issues, labels, repository files, or required Issue comments/state cannot be read/written, report the missing capability and stop. Do not simulate publication locally.

## Procedure

### 1. Preflight and deduplicate

- Read the current intended branch and HEAD.
- Search open and closed Issues for the same Task / Goal.
- Reuse/update an existing Task when appropriate; a failed Attempt is not a reason to create a duplicate Issue.
- Confirm Task-vs-Job decomposition before materializing.
- Confirm every eligible `env:*` has a dedicated profile under `docs/tasks/handoffs/`.

If an environment has no stable handoff profile, keep the Task draft and add the profile first rather than inventing an ad-hoc launch command.

### 2. Materialize as draft

Create the real GitHub Issue first as non-claimable `status:draft`.

Obtain the real Issue number, then create:

```text
docs/tasks/<issue>-<slug>/task.md
docs/tasks/<issue>-<slug>/prompt.md
```

Generate them from repository templates and replace all task-specific placeholders.

The Issue must link:

- real `task.md`;
- real `prompt.md`;
- actual base commit;
- Parent Goal / Research Item when applicable;
- eligible environment(s) / Required Capabilities.

Do not persist registration tokens, Cookies, PATs, SSH private keys, or other long-lived secrets.

### 3. Read-back verification

A successful create/update call is not publication.

Re-read from GitHub and independently verify:

```text
Issue exists/open and real number matches package path
Issue is unclaimed
Issue links task.md + prompt.md + base commit

task.md exists on intended branch
prompt.md exists on intended branch
prompt.md points to same Issue/task.md
prompt/task do not store stale live status

no task-specific placeholders remain
eligible env / Required Capabilities are correct
Success Criteria / Evidence Contract are present
no secret/token was persisted
```

If any check fails:

```text
keep status:draft
→ fix
→ read back again
```

Do not announce publication.

### 4. Publish last

Only after read-back passes:

- preserve unrelated labels;
- set all intended eligible `env:*` labels;
- replace Task status with `status:ready`;
- keep no active execution owner.

If required labels cannot safely be created/applied, keep the Task draft and report the blocker.

### 5. Post-publish queue verification

For every eligible environment, use a query equivalent to that Worker's real queue, for example:

```text
status:ready + env:web-gpt
status:ready + env:ubuntu-arm64
```

Verify the expected Issue is claimable, the intended environment is present, no active owner exists, and linked `task.md` / `prompt.md` still resolve.

If any required eligible environment cannot see the Task, publication is incomplete. Fix it or return the Task to draft.

### 6. Environment-specific downstream handoff

Task Contract is shared; launch syntax is not.

Select handoff profile by real environment:

```text
env:web-gpt       → docs/tasks/handoffs/web-gpt.md
env:ubuntu-arm64  → docs/tasks/handoffs/ubuntu-arm64.md
env:wsl           → docs/tasks/handoffs/wsl.md
env:windows       → docs/tasks/handoffs/windows.md
env:cloud         → docs/tasks/handoffs/cloud.md
env:manual-tv     → docs/tasks/handoffs/manual-tv.md
```

Rules:

- one eligible environment → output exactly one standalone copy block;
- multiple eligible environments → output one standalone copy block per environment;
- do not combine different clients into one prompt;
- replace all Issue/path/environment placeholders from **post-publish GitHub read-back**;
- never paste the full `task.md` into handoff;
- `env:web-gpt` must use the Web ChatGPT + GitHub connector profile and must **not** require `$task-worker`;
- Codex environments use their own profile and normally invoke `$task-worker`;
- manual TV uses the manual verification profile, not a fake Codex/Actions command.

Always include real handoff metadata:

```text
Task: <real title>
Issue: #<real issue>
Worker: <real worker>
Environment: env:<real environment>
Prompt: docs/tasks/<real-issue>-<real-slug>/prompt.md
```

## Republish after bootstrap/Contract change

If Scope, Claims, Success Criteria, decomposition, Evidence Authority, architecture/security premise, or the task-specific bootstrap changes materially:

```text
status:draft when required to make the package non-claimable
→ update canonical docs when required
→ update task.md for Contract changes
→ update prompt.md for bootstrap changes
→ read-back verify
→ status:ready + eligible env
→ queue verify for every eligible env
→ emit fresh environment-specific handoff(s)
```

A normal implementation bug, failed test, or insufficient Evidence does not require Contract republication; that is Issue iteration handled by `task-reviewer` + the matching Worker environment.

## Completion rule

Do not say “published”, “ready”, or “Worker can execute” unless all are true:

```text
Issue read-back PASS
+ task.md read-back PASS
+ prompt.md read-back PASS
+ ready/env state read-back PASS
+ every required target worker queue search PASS
+ environment-specific downstream handoff emitted for every eligible env
```

Plan is not execution. Write success is not publication.