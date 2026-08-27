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
4. `docs/tasks/freshness-contract.template.md`
5. `docs/tasks/prompt.template.md`
6. `docs/tasks/issue-lifecycle-protocol.md`
7. `docs/tasks/freshness-integration-protocol.md`
8. `docs/tasks/handoffs/README.md`
9. the handoff profile(s) for the intended eligible environment(s)
10. the canonical docs relevant to the requested Goal / Research Item

Do not copy those documents into this skill or silently redefine Scope, Claims, Success Criteria, architecture, security, freshness policy, or Evidence requirements.

## Inputs

Resolve from the explicit request and repository state:

- Goal / Parent Goal / Research Item;
- Task kind: `implementation | verification | combined | research`;
- expected Worker;
- eligible `env:*` label(s);
- Required Capabilities;
- Task decomposition decision;
- Success Criteria / Claims / Evidence Contract;
- planning/base commit from the actual target branch;
- **hard dependencies**, if any;
- **semantic authorities** used by the Task;
- **semantic freshness domains**;
- **integration surfaces**;
- **task-owned surfaces**;
- authority/domain → Claim mapping;
- declared `JI*` integration verification jobs;
- freshness policy: `dependency-aware` by default, `strict-main` only with a frozen correctness reason.

If a required value cannot be resolved, stop before publication rather than inventing it.

## Default Worker / client routing

Current project policy is **Codex-first for repository execution**.

Select Worker/client before Runner:

```text
ordinary repository implementation / bug fix / refactor / test or CI authoring
→ Codex Cloud Worker
→ env:cloud

GitHub-only lightweight execution where a coding workspace is unnecessary,
or Coordinator explicitly chooses Web Worker
→ Web Worker
→ env:web-gpt

local Linux interactive diagnosis
→ Codex / WSL
→ env:wsl

Windows / ADB / Android-host work
→ Codex / Windows
→ env:windows

interactive target-phone installation / recovery / diagnosis
→ Codex on Ubuntu ARM64 target
→ env:ubuntu-arm64

physical TV / remote / audible UX Evidence
→ manual verifier
→ env:manual-tv
```

Important separation:

```text
Worker / client
!= GitHub Actions
!= Runner
!= Target
```

Even when the Worker is Codex Cloud, portable build/test defaults to GitHub-hosted Actions; phone-specific proof uses the trusted Ubuntu ARM64 Target Runner; real TV proof remains Manual.

Do not use `env:cloud` merely because a Task contains long-running verification. The Worker may be Codex Cloud while the actual verification runs on the Runner/Target required by the Claims.

If the explicit Task capability requires a different environment, capability wins over the default. Do not route Windows/ADB, target-phone interactive work, or physical TV work through generic Cloud just to satisfy Codex-first.

## Dependency rule

Do not confuse sequencing preference, business dependency, semantic freshness, and integration overlap.

Classify relationships before publication:

```text
Hard dependency
= without the upstream result, this Task cannot be correctly implemented or verified

Semantic authority dependency
= Task can be published now, but later changes to an accepted contract/API/invariant may invalidate mapped Claims

Soft ordering
= preferred sequence / Research order / review order, but this Task can execute independently

Integration overlap
= shared files, Cargo/workspace metadata, shared build/router/schema surface, eventual merge/composition risk
```

Publication policy:

```text
hard dependency unresolved
→ keep status:draft / status:blocked as appropriate

semantic authority already accepted
→ may publish; record it in Freshness Contract

soft ordering only
→ do NOT block publication

integration overlap only
→ do NOT block publication; record integration surface + JI jobs

no hard dependency
→ publish as soon as Publication Gate is otherwise PASS
```

Rules:

- Research ID order or arrows in planning docs do not automatically create a hard dependency unless the canonical text explicitly makes one necessary for correctness;
- shared files or probable merge conflicts are integration work, not a business Task dependency;
- “another Task may change an interface later” is not enough to block publication unless the current Task actually needs that undecided result to execute correctly;
- when Tasks run in parallel, freeze authority boundaries in `task.md` so one Task cannot silently redefine the other's domain;
- if later Evidence truly invalidates a parallel Task assumption, use the freshness protocol / Review lifecycle then—do not preemptively serialize all work.

Default principle:

> **No hard dependency → publish early and run in parallel. Freshness is handled at Review/Integration Gate, not by globally serializing publication.**

## Freshness Contract rule

Every new materialized/republished Task must include a `## Freshness / Integration Contract` section based on:

```text
docs/tasks/freshness-contract.template.md
```

Default:

```text
Freshness policy: dependency-aware
Unrelated-main policy: existing exact-Candidate semantic Evidence remains valid
Integration overlap: preserve semantic Evidence + run declared JI jobs
Semantic authority change: rerun mapped affected Claims
```

Do **not** write generic requirements such as:

```text
Worker must integrate any newer main before acceptance
Final Candidate must always include latest main
```

unless the Task is explicitly:

```text
Freshness policy: strict-main
Strict-main reason: <specific correctness reason frozen before execution>
```

`Base commit` is the planning/execution baseline, not automatically a permanent freshness lock.

## GitHub capability

Use an authenticated GitHub write path available in the current environment.

If Issues, labels, repository files, or required Issue comments/state cannot be read/written, report the missing capability and stop. Do not simulate publication locally.

## Procedure

### 1. Preflight and deduplicate

- Read the current intended branch and HEAD.
- Search open and closed Issues for the same Task / Goal.
- Reuse/update an existing Task when appropriate; a failed/stalled Attempt is not a reason to create a duplicate Issue.
- Confirm Task-vs-Job decomposition before materializing.
- Identify and classify each relationship as hard dependency / semantic authority / soft ordering / integration overlap.
- Resolve the Freshness Contract, including `JI*` jobs, before publication.
- Resolve the Worker/client using the Codex-first routing rule unless an explicit capability requires another environment.
- If no unresolved hard dependency exists, do not leave the Task draft merely because an earlier Research/Task is unfinished.
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

Even if `task.template.md` has not yet been updated in an older branch, the published `task.md` must append the current `Freshness / Integration Contract` template before Publication Gate.

The Issue must link:

- real `task.md`;
- real `prompt.md`;
- actual planning/base commit;
- Parent Goal / Research Item when applicable;
- preferred Worker/client;
- eligible environment(s) / Required Capabilities;
- unresolved hard dependency or explicit `none`;
- when parallelism/integration matters, a short summary of semantic authorities and integration-overlap risk.

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
preferred worker / eligible env / Required Capabilities are correct
hard dependencies are either satisfied or explicitly none
Success Criteria / Evidence Contract are present
Freshness / Integration Contract is present
freshness policy is dependency-aware or strict-main with explicit reason
semantic authorities/domains and integration surfaces are bounded
JI jobs are explicit or deliberately n/a
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

Only after read-back passes and no unresolved hard dependency remains:

- preserve unrelated labels;
- set all intended eligible `env:*` labels;
- replace Task status with `status:ready`;
- keep no active execution owner.

If required labels cannot safely be created/applied, keep the Task draft and report the blocker.

### 5. Post-publish queue verification

For every eligible environment, use a query equivalent to that Worker's real queue, for example:

```text
status:ready + env:cloud
status:ready + env:web-gpt
status:ready + env:ubuntu-arm64
```

Verify the expected Issue is claimable, the intended environment is present, no active owner exists, and linked `task.md` / `prompt.md` still resolve.

If any required eligible environment cannot see the Task, publication is incomplete. Fix it or return the Task to draft.

### 6. Environment-specific downstream handoff

Task Contract is shared; launch syntax is not.

Select handoff profile by real environment:

```text
env:cloud         → docs/tasks/handoffs/cloud.md
env:web-gpt       → docs/tasks/handoffs/web-gpt.md
env:ubuntu-arm64  → docs/tasks/handoffs/ubuntu-arm64.md
env:wsl           → docs/tasks/handoffs/wsl.md
env:windows       → docs/tasks/handoffs/windows.md
env:manual-tv     → docs/tasks/handoffs/manual-tv.md
```

Rules:

- one eligible environment → output exactly one standalone copy block;
- multiple eligible environments → output one standalone copy block per environment;
- do not combine different clients into one prompt;
- replace all Issue/path/environment placeholders from **post-publish GitHub read-back**;
- never paste the full `task.md` into handoff;
- generic Codex-first repository implementation normally emits the `env:cloud` `$task-worker` entry first;
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

## Republish after bootstrap/Contract/routing/freshness change

If Scope, Claims, Success Criteria, decomposition, Evidence Authority, architecture/security premise, dependency classification, eligible Worker/environment routing, **Freshness / Integration Contract**, or task-specific bootstrap changes materially:

```text
status:draft when required to make the package non-claimable
→ update canonical/process docs when required
→ update task.md for Contract/dependency/routing/freshness changes
→ update prompt.md for bootstrap changes
→ read-back verify
→ status:ready + eligible env
→ queue verify for every eligible env
→ emit fresh environment-specific handoff(s)
```

A normal implementation bug, failed test, insufficient Evidence, or current-main advance classified by the already-frozen freshness policy does not itself require Contract republication; that is Issue iteration handled by `task-reviewer` + the matching Worker environment.

## Completion rule

Do not say “published”, “ready”, or “Worker can execute” unless all are true:

```text
Issue read-back PASS
+ task.md read-back PASS
+ prompt.md read-back PASS
+ no unresolved hard dependency
+ Freshness Contract read-back PASS
+ ready/env state read-back PASS
+ every required target worker queue search PASS
+ environment-specific downstream handoff emitted for every eligible env
```

Plan is not execution. Write success is not publication.