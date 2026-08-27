# Task — INFRA-004-TARGET-DIAGNOSTICS

## Metadata

```text
GitHub Issue: #92
Task ID: INFRA-004-TARGET-DIAGNOSTICS
Task kind: infrastructure implementation + target verification
Planning Base: 634f036617db4360526eccb661e07139cfc9f6f2
Parent consumer: #90 ENV-ARM64-GITHUB-SMARTHTTP-PREP
Existing trusted workflow: .github/workflows/target-runner-smoke.yml
Preferred worker: cloud-codex
Eligible environment: env:cloud
Target: ubuntu-arm64-target-phone
Required capabilities: github-read-write, workflow-authoring, actions-trigger-read, target-runner-routing, security-evidence-authoring
Session Bootstrap: docs/tasks/92-infra-target-diagnostics/prompt.md
Downstream Handoff: docs/tasks/handoffs/cloud.md
Freshness policy: dependency-aware
```

> Issue #92 owns live status, Attempt, owner and result. This file owns the stable reusable target-diagnostics contract.

## Trigger Evidence

#90 Attempt 1 created Draft PR #91 and a task-specific new workflow intended to run before repository checkout. The workflow implementation exists only on the worker branch and no #90 target Evidence run exists yet.

The repository already has a trusted default-branch workflow:

```text
.github/workflows/target-runner-smoke.yml
```

That workflow already proves a useful minimal boundary without repository checkout:

- explicit `workflow_dispatch` input;
- target labels `[self-hosted, linux, ARM64, ubuntu-arm64, target-device]`;
- target identity / architecture / user / workspace recording;
- non-root and no-passwordless-sudo checks;
- production Vault isolation check;
- bounded temporary workspace create/cleanup.

The missing capability is a reusable, structured diagnostics layer that Cloud Workers can invoke when target-specific behavior fails before or around checkout.

## Goal

Extend the existing trusted `target-runner-smoke.yml` into a backwards-compatible, bounded diagnostics entrypoint for Cloud-driven target debugging.

First version supports only:

```text
profile=baseline
profile=github-transport
```

The intended control flow is:

```text
Cloud Worker / Coordinator
→ dispatch existing trusted workflow path
→ fixed allowlisted profile
→ ubuntu-arm64-target-phone
→ no repository checkout required for bootstrap diagnostics
→ bounded human summary + versioned structured diagnostics artifact
→ Cloud Worker / Coordinator reads Evidence
```

This is not a remote shell facility.

## Bootstrap / Pre-merge Verification Requirement

The implementation must modify the existing workflow path rather than add a brand-new workflow file solely for this Task:

```text
.github/workflows/target-runner-smoke.yml
```

Because this workflow already exists on the default branch, the Worker must attempt to dispatch the modified workflow against the exact worker branch/ref before merge and record the actual run/job identity.

Required pre-merge proof:

```text
existing default-branch workflow identity
→ dispatch exact worker branch/ref
→ modified workflow definition executes on target
→ baseline profile PASS
→ github-transport profile bounded Evidence
```

If GitHub does not permit the modified branch/ref workflow to execute through the existing workflow identity, STOP with `[BLOCKER REPORT]`. Do not ask the Coordinator to merge unverified target workflow code merely to activate it.

## Architecture / Security Invariants

1. Target execution remains under the accepted low-privilege `gateway-runner` boundary.
2. No root, sudo, ADB, production Vault, site Secret or long-lived GitHub credential authority is added.
3. The diagnostics workflow must not expose a generic command/script/eval input.
4. The diagnostics workflow must not accept arbitrary URL, proxy endpoint, filesystem path or executable input.
5. Do not print complete process environments or `/proc/*/environ`.
6. Do not print `GITHUB_TOKEN`, Authorization headers, cookies, proxy credentials, Vault data or sensitive endpoint credentials.
7. Do not set `sslVerify=false` or otherwise weaken TLS.
8. Diagnostics must be bounded by explicit timeouts and bounded repetition counts.
9. No Bilibili/site request is part of this Task.
10. No #85 fd-isolation proof and no #67 execution is part of this Task.
11. Product/runtime source under `gateway-core/**`, `gateway-egress/**`, `site-adapter-api/**`, `display-adapter-api/**`, `plugins/**` is out of scope.
12. Existing `target-runner-smoke` semantics remain available; the new profile input must not silently remove the accepted baseline boundary checks.

## Allowed Inputs

Prefer a compact fixed schema such as:

```text
profile:
  baseline | github-transport

candidate_sha:
  optional 40-hex SHA; required only by probes that need an exact repository object

repetitions:
  bounded enum such as 1 | 3 | 5
```

No free-form string input may become shell code, arbitrary command arguments, arbitrary URL/proxy/path selection or unbounded loop control.

## Required Profiles

### P1 — baseline

Must preserve and structure the existing target smoke boundary. Record only safe bounded fields such as:

```text
runner name / OS / arch
uname / kernel
uid/gid / whoami
workspace / runner temp class
bounded disk and memory summary
presence/version of git/curl/python/rust when available
non-root assertion
passwordless-sudo absent assertion
production Vault inaccessible assertion
temporary workspace create/cleanup
```

Absence of an optional tool is a structured observation, not a reason to install packages.

### P2 — github-transport

Must support the transport layer needed by #90 and future Cloud diagnostics, while remaining generic and bounded.

Record safely:

```text
proxy variable presence/classification only, not credential values
GitHub DNS IPv4/IPv6 availability
curl version / TLS backend summary
git version / linked HTTP/TLS backend summary when safely observable
bounded HTTPS reachability to fixed GitHub repository/service targets
git advertisement / ls-remote result
exact candidate fetch/object-transfer result when candidate_sha is supplied
failure stage classification where possible
repetition counts
```

The target hostname/repository used by the Git profile must be repository-owned/fixed by the workflow. Do not expose arbitrary destination inputs.

The workflow may use the ephemeral Actions repository-read credential only through a command-scoped, non-persisted mechanism when a specific probe requires authenticated repository access. Credential values must never be printed or persisted.

## Structured Output Contract

The Task must produce both:

1. `GITHUB_STEP_SUMMARY` or equivalent concise human-readable summary;
2. a versioned structured artifact, preferably `diagnostics.json`.

Minimum conceptual schema:

```json
{
  "schema_version": 1,
  "profile": "baseline|github-transport",
  "candidate_sha": "<redacted-or-40hex-or-null>",
  "target": {
    "arch": "aarch64",
    "kernel": "...",
    "uid": 999
  },
  "checks": {
    "low_privilege": "pass|fail|unavailable",
    "github_https": "pass|fail|not_run",
    "git_advertisement": "pass|fail|not_run",
    "git_object_fetch": "pass|fail|not_run"
  },
  "classification": "...",
  "result": "pass|conditional|blocked|fail"
}
```

The exact schema may differ, but it must be stable/versioned, bounded, sanitized and useful to another Worker without parsing arbitrary shell logs.

## Claims

```text
C1 — Existing smoke boundary is preserved
The accepted non-root/workspace/Vault/cleanup smoke remains valid under the new baseline profile.

C2 — No-checkout diagnostics are reusable
Cloud/Coordinator can invoke a fixed diagnostics profile on the accepted target without requiring repository checkout before bootstrap collection.

C3 — Input surface is capability-oriented, not command-oriented
No arbitrary shell/script/URL/proxy/path execution surface is introduced.

C4 — Structured output is safe and reusable
The workflow emits bounded human summary plus versioned structured diagnostics without Secret leakage.

C5 — GitHub transport diagnostics are sufficient for #90
The github-transport profile can distinguish basic HTTPS/advertisement behavior from exact repository object-transfer behavior on the target and can accept the preserved #85 Candidate as a bounded 40-hex input.

C6 — Worker-branch target proof is real
The modified existing workflow is dispatched against the exact worker branch/ref and produces target run/job Evidence before merge.

C7 — Security and cleanup remain fail-closed
No TLS weakening, privilege expansion, production state access or persistent credential/proxy mutation is needed.
```

Claim vocabulary: `PASS | CONDITIONAL PASS | FAIL | BLOCKED | NOT RUN`.

## Implementation / Verification Plan

### J0 — Live state and authority read-back

Read:

- Issue #92 and all comments;
- this task;
- `AGENTS.md`;
- `docs/tasks/issue-lifecycle-protocol.md`;
- `docs/tasks/freshness-integration-protocol.md`;
- `docs/runner-execution-architecture.md`;
- relevant `docs/security.md` sections;
- current `.github/workflows/target-runner-smoke.yml`;
- #90 Attempt 1 checkpoint / PR #91 only as consumer evidence, not implementation authority.

### J1 — Extend existing trusted workflow path

Modify `target-runner-smoke.yml` only as needed to introduce the fixed profiles and structured output while preserving current smoke behavior.

Prefer no additional repository helper script unless the workflow becomes unmaintainable without one. Any helper remains infrastructure-only and must not require product checkout for the bootstrap portion.

### J2 — Static security verification

Prove at minimum:

- inputs are enum/strict validated;
- candidate SHA is strict 40-hex where used;
- no `eval`, arbitrary command/script input, arbitrary URL/proxy/path input;
- no full env dump;
- no token printing;
- no `sslVerify=false` or TLS weakening;
- target labels remain accepted labels;
- product/site commands absent;
- bounded timeouts/repetitions.

### J3 — Baseline target proof on exact worker branch/ref

Dispatch the existing workflow identity against the exact worker branch/ref.

Record:

```text
worker branch/ref
workflow run id
job id
runner identity
profile=baseline
result
structured artifact identity
```

Require the accepted low-privilege smoke boundary and cleanup to PASS.

### J4 — GitHub transport target proof on exact worker branch/ref

Dispatch:

```text
profile=github-transport
candidate_sha=4af64b124af4d1599a87bd211395ee832e9d7e4b
bounded repetitions
```

This Task does not need to solve #90's transport blocker. It must prove the diagnostics mechanism can safely collect the relevant transport result/classification and structured Evidence.

A `git_object_fetch=fail` result can still satisfy #92 if the diagnostic faithfully and safely captures the failure; #90 owns transport recovery.

### J5 — Artifact/readability proof

Verify the human summary and `diagnostics.json` can be read back from GitHub Actions and contain no sensitive values.

### J6 — Cleanup / regression

Verify:

- no persistent user/system Git/proxy mutation;
- no credentials left on target;
- no production state access;
- temporary directories cleaned;
- target Runner returns to a normal post-job state;
- existing baseline smoke semantics remain supported.

### J7 — Worker report

Post `[EXECUTION REPORT]` with exact Candidate, PR, workflow run/job IDs, target identity, claim results and artifact identity. Then set `status:review`, release owner and STOP.

Do not merge or close #92 and do not resume #90.

## Result Criteria

### PASS

- C1-C7 supported;
- existing workflow path is extended safely;
- baseline and github-transport profiles both execute on the actual target from the exact worker branch/ref before merge;
- structured artifact is produced/readable/sanitized;
- no security boundary weakened.

### CONDITIONAL PASS

One optional diagnostic dimension (for example native IPv6) is unavailable, but the workflow mechanism, low-privilege boundary, structured output and github object-transfer classification are still proven.

### FAIL

The implementation requires arbitrary command execution, Secret leakage/persistence, TLS weakening, privilege expansion, unsafe untrusted trigger behavior or product/site coupling.

### BLOCKED

Examples:

- existing workflow identity cannot execute the modified worker branch/ref;
- Target Runner unavailable;
- required Actions artifact/read-back capability unavailable;
- safe structured output cannot be produced without prohibited authority.

## Success Criteria

1. Existing target smoke security boundary is preserved.
2. `baseline` and `github-transport` profiles exist with strict allowlisted inputs.
3. Bootstrap diagnostics run without repository checkout.
4. No arbitrary remote shell or arbitrary destination capability is introduced.
5. Human summary + versioned structured artifact are produced.
6. Exact worker branch/ref receives real target run/job Evidence before merge.
7. `github-transport` safely records advertisement vs object-transfer behavior for the preserved #85 Candidate.
8. Cleanup and Secret/TLS/privilege boundaries pass.
9. Worker reports exact Candidate/PR/run/job/artifact and STOPs.

## Freshness / Integration Contract

### Policy

```text
Freshness policy: dependency-aware
```

### Semantic authorities

- accepted Target Runner low-privilege/security boundary;
- existing `target-runner-smoke.yml` baseline behavior;
- GitHub Actions target trust model in `docs/runner-execution-architecture.md`;
- `docs/security.md` Secret/TLS/target isolation rules.

### Semantic freshness domains

```text
Target Runner labels / privilege boundary
workflow_dispatch / target workflow trust model
existing target-runner-smoke behavior
Actions artifact/token semantics used by diagnostics
```

### Integration surfaces

```text
.github/workflows/target-runner-smoke.yml
potential diagnostics-only helper/output schema
```

### Unrelated-main policy

Unrelated product/docs movement does not invalidate Task-specific target Evidence. Material changes to target labels/security, workflow trust rules or the existing smoke workflow require freshness review.

## Lifecycle

```text
status:draft
→ Publication Gate
→ status:ready + env:cloud + no owner
→ Worker claim / Attempt N
→ implementation + pre-merge target proof
→ [EXECUTION REPORT] / [BLOCKER REPORT]
→ status:review / status:blocked
→ release owner
→ STOP
→ Coordinator Review / merge / Final Acceptance
```

Worker must not set `status:done`, close #92, unblock #90, merge PR #91, resume #85 or start #67.