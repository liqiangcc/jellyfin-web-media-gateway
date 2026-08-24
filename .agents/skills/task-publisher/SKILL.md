---
name: task-publisher
description: Publish or republish an independent Worker Task for jellyfin-web-media-gateway. Use only when explicitly asked to create, materialize, publish, or republish a Task Package; do not use to execute or review an existing task.
---

# Task Publisher

Publish one independent Worker Task through the repository's Task Publication Gate.

## Authority

This skill is an execution procedure, not a source of product scope.

Before writing anything, read:

1. `AGENTS.md`
2. `docs/tasks/README.md`
3. `docs/tasks/task.template.md`
4. `docs/tasks/prompt.template.md`
5. `docs/tasks/issue-lifecycle-protocol.md`
6. the canonical docs relevant to the requested Goal / Research Item

Use those files as authority. Do not copy their full contents into this skill or silently redefine Scope, Claims, Success Criteria, architecture, security, or Evidence requirements.

## Inputs

Resolve these from the explicit request and repository state:

- Goal / Parent Goal / Research Item;
- Task kind: `implementation | verification | combined | research`;
- expected Worker;
- eligible `env:*` label(s);
- Required Capabilities;
- Task decomposition decision;
- Success Criteria / Claims / Evidence Contract;
- base commit from the actual target branch.

If a required value cannot be resolved from the request or canonical repository state, stop before publication rather than inventing it.

## GitHub capability

Use an authenticated GitHub write path available in the current Codex environment, such as a connected GitHub tool or an authenticated `gh` CLI.

If the environment cannot create/update Issues, labels, and repository files, report the missing capability and stop. Do not simulate publication in local files only.

## Procedure

### 1. Preflight and deduplicate

- Read the current default branch / intended publication branch and current HEAD.
- Search existing open and closed Issues for the same Task / Goal.
- If the requested Task already exists, update or republish that Task when appropriate; do not create a duplicate merely because a previous Attempt failed.
- Confirm that splitting this work into a new Task follows the repository's Task-vs-Job rules.

### 2. Materialize as draft

Create the real GitHub Issue first and keep it non-claimable as `status:draft`.

Obtain the real Issue number, then create:

```text
docs/tasks/<issue>-<slug>/task.md
docs/tasks/<issue>-<slug>/prompt.md
```

Generate them from the repository templates. Replace all placeholders with real values.

The Issue must link:

- the real `task.md` path;
- the real `prompt.md` path;
- the actual base commit;
- Parent Goal / Research Item when applicable;
- eligible environment / Required Capabilities.

Do not put secrets, registration tokens, Cookies, PATs, SSH private keys, or long-lived credentials in Issue/task/prompt content.

### 3. Read-back verification

A successful create/update call is not publication.

Re-read from GitHub and verify independently:

```text
Issue exists and is open
Issue number matches the Task Package path
Issue is unclaimed
Issue links task.md + prompt.md + base commit

task.md exists on the intended branch
prompt.md exists on the intended branch
prompt.md points to the same Issue and task.md

no template placeholders remain in task-specific fields
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
- set the intended `env:*` eligibility;
- replace the task status with `status:ready`;
- keep the Issue without an active execution owner.

If required labels do not exist, create them when the available GitHub capability safely supports that operation. Otherwise keep the Task draft and report the publication blocker.

### 5. Queue verification

Use a query equivalent to the target Worker's real queue, for example:

```text
status:ready + env:ubuntu-arm64
```

Verify that the expected Issue appears as claimable, with the correct environment and no active owner. Resolve the linked `task.md` and `prompt.md` again.

If the target queue cannot see the Task, publication failed. Fix it or return it to draft.

### 6. Downstream handoff

Only after queue verification passes, return the real handoff:

```text
Task: <real title>
Issue: #<real issue>
Worker: <real worker>
Environment: env:<real environment>
Prompt: docs/tasks/<real-issue>-<real-slug>/prompt.md
```

Then provide a directly copyable Codex entry that explicitly invokes the Worker skill:

```text
$task-worker Execute Issue #<real-issue> using `docs/tasks/<real-issue>-<real-slug>/prompt.md`.
```

Do not paste the entire `task.md` into the handoff.

## Republish after Contract revision

If an existing Task's Scope, Claims, Success Criteria, decomposition, Evidence Authority, or architecture/security premise changes:

```text
status:draft
→ update canonical docs when required
→ update task.md
→ update prompt.md only if bootstrap changed
→ read-back verify
→ status:ready
→ queue verify
→ output a new downstream handoff
```

A normal implementation bug, failed test, or insufficient Evidence does not require Contract republication; that is an Issue iteration handled by `task-reviewer` + `task-worker`.

## Completion rule

Do not say “published”, “ready”, or “Worker can execute” unless all of these are true:

```text
Issue read-back PASS
+ task.md read-back PASS
+ prompt.md read-back PASS
+ ready/env state read-back PASS
+ target worker queue search PASS
+ downstream handoff emitted
```

Plan is not execution. Write success is not publication.