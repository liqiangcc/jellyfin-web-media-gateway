# Task — Add safe stage markers for repeatable tx-node navigation failure

## Metadata

```text
GitHub Issue: #199
Parent Goal / Research Item: #68 / R008 / #188 browser-navigation fast path
Task / Research ID: R008-NAV-STAGE-MARKERS
Task kind: implementation
Base commit: afe734ab322ba9b82f5f6ae37e02364b44142cdc
Candidate commit: n/a until Worker implementation
Session bootstrap prompt: docs/tasks/199-r008-navigation-stage-markers/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, code-authoring, automated-test, repository-static-analysis
Hard publication dependencies: Task Publication Gate for #199; no target or device dependency
```

GitHub Actions / Runner 是 verification backend，不是会 claim Issue 的 Worker。实时 status、owner、Attempt、branch、Candidate、PR、run、artifact 和结果只保存在 Issue comments/labels。

## Session Bootstrap

独立 Worker 从 `docs/tasks/199-r008-navigation-stage-markers/prompt.md` 开始。Prompt 只导航到 Issue、契约和必要入口；本文件拥有 Goal、Scope、Claims、Success Criteria、Architecture Invariants、Verification Job Matrix 和 Evidence 判断标准。

## Goal

Extend the existing experimental Bilibili browser diagnostic so every bounded browser-navigation attempt exposes a finite, allowlisted sequence of safe stage markers covering browser launch, navigation start/end/status, broker request/transport outcome, and finalizer entry. The markers must identify the lifecycle boundary of the repeatable failure without exposing raw errors, URLs, headers, cookies, credentials, profiles, media, or caller-controlled authority.

## Why / Context

#188 Attempts 3 and 4 used the accepted Candidate and independently reached the tx-node navigation probe, but both produced the same sanitized result: `failure / error / unclassified_failure`, `phase=unknown`, `transport_stage=unknown`, 15 requests, 15,224 response bytes, and no media request. The result is valid evidence that the disposable navigation path runs, but it does not identify whether the failure occurs during browser launch, navigation, broker transport, or finalization. This focused follow-up adds only the missing lifecycle evidence. It does not play Bilibili, rerun #188, or claim source portability.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: #188, Coordinator-controlled follow-up only
Decision reason: The stage-marker implementation and its deterministic hosted tests share one small diagnostic surface. The later tx-node navigation rerun has a separate target Evidence Authority and remains outside this Task.
```

## Worker Routing Decision

```text
Worker: cloud-codex
Eligible environment: env:cloud
Requested model: gpt-5.6-luna, reasoning high
Fast mode: disabled by user; do not enable or select Fast
```

Use GitHub-hosted Actions for all build, package, and test verification. The Cloud Worker is not a runtime or target runner.

## Work Role

### Implementation

The Worker must implement the smallest focused change in the existing experimental probe and tests. The Candidate must:

- add a bounded `stage_markers` representation while preserving the existing sanitized result contract;
- use a finite allowlist for marker event names and coarse status/transport values;
- emit markers around browser launch, navigation start/end/status, broker request/result and transport outcome, and finalizer entry on both normal and failure paths where the boundary is reached;
- preserve idempotent finalization and ensure late callbacks cannot overwrite a finalized diagnostic or its marker sequence;
- add deterministic tests for marker allowlisting, ordering/sequence bounds, launch/navigation/broker/finalizer failure paths, redaction, and late-callback behavior;
- update artifact/static-boundary or runbook material only when needed to keep the packaged Candidate and hosted checks authoritative.

Do not change production Gateway/Playback/Display/Control semantics. If implementation requires a canonical architecture, security, egress, or authority change, stop and return the Task to the Coordinator for contract review.

## Task vs Job Boundary

```text
Task
→ safe stage-marker implementation and its diagnostic Claims
→ GitHub-hosted verification Jobs
→ Candidate/Evidence for Coordinator Review

Job
→ runs one declared check on the exact Candidate
→ has no Issue owner and does not perform target navigation
```

The later #188 target rerun is not a Job of this Task and must be scheduled only by the Coordinator after Candidate acceptance.

## Routing Rationale

Repository implementation, static analysis, deterministic tests, artifact packaging, and integration checks are all expressible on GitHub-hosted runners. No phone, TV, tx-node, VNC, CDP, site session, or interactive Linux capability is required for this Task. A later #188 target rerun is a separate Coordinator-controlled evidence step.

## Preconditions

- #199 has passed the Publication Gate and is `status:ready`, `env:cloud`, and owner-free before claim.
- Current main/base identity is `afe734ab322ba9b82f5f6ae37e02364b44142cdc` unless the Coordinator republishes the package with a new real base.
- Read #188 Attempts 3 and 4 and its latest fast-path contract for context; do not alter #188 from this implementation Task.
- The accepted #188 Candidate/artifact may be used as diagnostic context only; this Task must produce a new Candidate and exact hosted Evidence after any code change.
- Fast is disabled by the user and must remain disabled.
- No tx-node access, browser/site access, credentials, VNC/CDP, phone/TV, production Gateway, or local build/test/package/install is needed or authorized.
- Do not process, claim, edit, or wait on #191 or #195.

## In Scope

- Experimental browser-probe diagnostic/finalizer lifecycle instrumentation.
- Finite, bounded, sanitized stage-marker schema and its validation.
- Deterministic unit/contract/static tests and required packaged-artifact updates.
- GitHub-hosted exact-Candidate J1–JI1 verification and artifact identity evidence.

## Out of Scope

- Any tx-node or other target execution, navigation, Bilibili request, playback, click, play, preload, media extraction, or independent consumer.
- Any VNC, CDP observation, phone, TV, production Gateway, proxy configuration, secret/cookie/auth handling, or profile inspection.
- General browser-slot/isolation platform work, dedicated UID/GID/cgroup/network namespace provisioning, or changes to #191/#195.
- Source portability or successful Bilibili playback claims.
- Local compilation, local package installation, local test execution, or target-side build/test/package/install.

## Architecture Invariants

- Core/Gateway remains the PlaybackSession authority; the experimental probe is not production playback logic.
- Site-specific semantics remain plugin-owned and the probe may only use the existing plugin-owned opaque selector boundary.
- No Site Plugin or diagnostic path reads Vault, exports cookies/Authorization, bypasses EgressPolicy, or becomes an open proxy.
- Chromium, broker, TLS, and media candidate details remain bounded and sanitized; no raw URL, body, header, certificate, address, secret, or credential enters durable Evidence.
- Cleanup remains bounded and owner-scoped; stale or late asynchronous results cannot replace a finalized observation.

## Files Expected to Change

- `experiments/bilibili-browser-probe/` diagnostic, live, finalizer, probe, and focused test files as required by the implementation.
- `plugins/bilibili/` only if the existing experimental observation contract requires a minimal compatible schema adjustment.
- `.github/workflows/bilibili-browser-probe.yml` or `docs/research/bilibili-browser-probe-runbook.md` only when the exact-Candidate checks/package boundary must be updated.

No production Gateway/Playback/Display/Control files are expected to change.

## Implementation Requirements

1. Define one finite marker event vocabulary. It must include, at minimum, `browser_launch_start`, `browser_launch_result`, `navigation_start`, `navigation_end`, `navigation_status`, `broker_request_start`, `broker_request_result`, `transport_outcome`, and `finalizer_entry` (or an equivalent reviewed naming set with the same explicit coverage).
2. Bound the marker count and every numeric field. Marker records may contain only allowlisted event names, coarse `status_class`, existing finite transport stage/outcome values, bounded sequence/counter fields, and no free-form error or authority fields. Do not include timestamps, URLs, host/address, request paths, headers, bodies, certificates, profile paths, cookies, tokens, or media candidate URLs.
3. Publish markers through the same bounded result path as the existing diagnostic. Normal completion, caught navigation/broker failures, process-level failure, signal termination, and finalizer entry must remain finite and idempotent. Unknown conditions must stay explicitly `unknown`/`unclassified_failure`.
4. Preserve the existing diagnostic schema and cleanup evidence unless a compatibility-preserving optional field is required. Add deterministic tests proving both allowlist rejection and sensitive-input non-export.
5. Prove sequence monotonicity and stale-callback safety. Once finalizer/result publication is committed, a late broker/browser callback must not mutate the finalized diagnostic, marker list, or cleanup outcome.
6. Keep all implementation and verification inside the existing experimental boundary. Do not add a new runtime dependency, caller-controlled transport/browser authority, production coupling, or target execution path.

## Freshness / Integration Contract

```text
Freshness policy: dependency-aware
Planning Base: afe734ab322ba9b82f5f6ae37e02364b44142cdc
Semantic authorities: AGENTS.md; docs/security.md; existing experimental probe/finalizer schema; plugin-owned selector and egress boundary
Semantic freshness domains: experiments/bilibili-browser-probe/**; plugins/bilibili/** when the observation DTO changes; the bilibili-browser-probe workflow's static/security assertions
Integration surfaces: package-lock/runtime inventory, artifact manifest builder, bilibili-browser-probe workflow, workspace-wide fmt/clippy/test integration
Task-owned surfaces: finite marker DTO/validation, lifecycle emission, finalizer idempotence, focused tests, and required package/static assertions
Authority/domain → Claim mapping: probe/finalizer schema and lifecycle → C1/C2; redaction/egress/secret boundary → C3; workflow/artifact/integration surfaces → C4; target/no-side-effect boundary → C5
Integration verification: JI1 exact-Candidate remote workspace integration and regressions
Unrelated-main policy: unrelated documentation or independent plugin changes do not invalidate exact-Candidate semantic Evidence; Coordinator classifies freshness at Review
Strict-main reason: none
```

If main changes a semantic authority or a protected integration surface before Review, the Coordinator decides whether to reverify affected Claims. The Worker must not silently broaden Scope or invalidate all Evidence solely because main advanced.

## Verification Plan

### Claims

```text
C1: The marker schema is finite, bounded, allowlisted, and compatible with the existing sanitized diagnostic contract.
C2: Browser launch/navigation/broker transport/finalizer lifecycle markers are emitted deterministically on reachable paths, and stale/late callbacks cannot overwrite a finalized result.
C3: No raw error, URL, header, cookie, token, profile, certificate, body, media, or caller-controlled authority can enter markers or durable diagnostic output; cleanup remains bounded.
C4: The exact Candidate passes the required GitHub-hosted probe contract, Chromium containment, artifact build/consumer, static boundary, and integration jobs, with an independently verified manifest/artifact identity.
C5: The implementation remains diagnostic-only and introduces no target, playback, production Gateway, phone/TV, VNC/CDP, proxy, or local-build path; a later #188 target rerun remains Coordinator-controlled.
```

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1,C2,C3 | github-actions | github-hosted x64 | runner-self | yes | `bilibili-browser-probe` J1 contract/diagnostic/transport/artifact tests plus new marker/finalizer tests | exact Candidate run/job and sanitized test output |
| J2 | C2,C3,C5 | github-actions | github-hosted x64 | synthetic Chromium fixture | yes | existing J2 containment/cleanup probe and live broker harness; assert markers remain bounded and sanitized | exact Candidate run/job and fixture evidence |
| J3a | C1,C4 | github-actions | github-hosted x64 | runner-self | yes | build and verify manifest-addressed artifact from exact Candidate | artifact manifest, size, digest and run/job |
| J3 | C4,C5 | github-actions | github-hosted x64 | downloaded artifact fixture | yes | verify J3a artifact and run the declared independent hosted consumer check | exact Candidate run/job and artifact-backed evidence |
| J4 | C3,C5 | github-actions | github-hosted x64 | runner-self | yes | existing live-entry/static boundary checks, including no caller-controlled browser/transport authority and no production coupling | exact Candidate run/job/log |
| JI1 | C1–C5 | github-actions | github-hosted x64 | workspace | yes | exact-Candidate `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --all-targets` as declared by the workflow | integration run/job |

All required jobs must checkout and assert the same Candidate SHA. Build/test/package commands run only on GitHub-hosted Actions; no equivalent command is run in the Codex workspace or on tx-node.

### Execution Plane

```text
Execution plane: github-actions
Target proof required: no
```

The later #188 navigation rerun is a Coordinator-controlled separate target Evidence step and is not required to accept this implementation Task.

### Runner Security Constraints

```text
Trusted candidate only: yes
Dedicated low-privilege runner user: required by hosted runner controls
Vault/profile access: forbidden
Production service mutation: forbidden
Cleanup / timeout requirements: use the existing bilibili-browser-probe workflow timeouts and finally/fixture cleanup; do not leave browser, broker, profile, or artifact residue
```

## Success Criteria

### Task success

1. The implementation adds finite, allowlisted, bounded stage markers that cover the required browser, navigation, broker/transport, and finalizer boundaries.
2. Deterministic tests demonstrate lifecycle emission, sequence bounds, redaction, idempotent finalization, and late-callback safety.
3. Existing no-raw-error/no-authority/no-media/cleanup invariants remain enforced, with no production coupling or target execution.
4. A reviewable Candidate/PR exists and all required exact-Candidate hosted jobs J1, J2, J3a, J3, J4, and JI1 pass; the artifact manifest and digest are independently read back.

### Verification claim success

```text
C1 PASS when the exact Candidate's tests reject unknown marker/event fields, enforce finite values and bounds, and preserve the sanitized diagnostic schema.
C2 PASS when deterministic tests and hosted evidence show required lifecycle markers on reachable paths and prove finalized output is monotonic/idempotent against late callbacks.
C3 PASS when focused leakage/static tests show no raw error, URL, header, cookie, token, profile, certificate, body, media, or caller-controlled authority in marker/result output and cleanup remains bounded.
C4 PASS when all required exact-Candidate hosted jobs and the manifest-addressed artifact verification pass.
C5 PASS when the diff remains inside the experimental diagnostic boundary, uses no target or production action, and explicitly leaves #188 target execution to the Coordinator.
```

## Evidence Contract

The Worker report must identify the real Task/Attempt, Candidate SHA, PR, Actions run/jobs, artifact name/size/digest, and the actual execution layers:

```text
Role: implementation + verification
Task / Claim: #199 / C1–C5
Attempt: <Issue Attempt>
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high
Fast availability: disabled by user; do not enable/select Fast
Execution plane: github-actions
Runner class / image: github-hosted x64 / actual Actions image
Execution host: GitHub Actions run
Target host/device: none for this Task
OS / architecture: actual hosted runner facts
Base / Candidate commit: real SHAs
Workflow / run / job: bilibili-browser-probe J1/J2/J3a/J3/J4/JI1
Commands / selectors: actual workflow steps and focused test selectors
Duration / repetitions / shards: actual values
Metrics / artifact / logs: sanitized marker/test evidence and manifest digest
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

Do not persist secrets, cookies, tokens, account data, raw URLs, raw error strings, response bodies, profile paths, or unnecessary large files. Distinguish implementation result, hosted verification result, and Coordinator decision.

## Failure / Blocked Handling

- `FAIL` if markers are unbounded, contain a non-allowlisted/free-form value, leak sensitive input, fail to cover a reachable lifecycle boundary, permit stale callbacks to mutate finalized output, or break required hosted tests.
- `BLOCKED` only if the Worker cannot perform the repository implementation or required GitHub-hosted verification because the required capability/workflow/artifact path is unavailable. Absence of target evidence is not a blocker for this Task because target execution is out of scope.
- If a failure identifies a required architecture/security/egress contract change, stop implementation, preserve sanitized evidence, and return the Task to the Coordinator for contract revision.
- If the Actions artifact or run is missing/incompatible, report the exact missing identity and do not fall back to local build/test/package or an unverified artifact.
- On normal completion, post `[EXECUTION REPORT]`, set #199 to `status:review`, release ownership, and stop. On a genuine blocker, post `[BLOCKER REPORT]`, set `status:blocked`, release ownership, and stop. The Worker must not mark #199 done or close it.

## Deliverables

- Focused diagnostic/finalizer implementation and deterministic tests.
- Any narrowly required workflow/runbook/static-boundary update.
- Exact Candidate commit and PR.
- Exact hosted J1/J2/J3a/J3/J4/JI1 evidence and artifact manifest/digest.
- `[EXECUTION REPORT]` or `[BLOCKER REPORT]` on #199 with status transition and owner release.
- No tx-node/browser/Bilibili evidence in this Task; any later #188 target result is Coordinator-owned.

## Issue Feedback / Iteration Protocol

Follow `docs/tasks/issue-lifecycle-protocol.md` and `docs/tasks/execution-anchor-recovery-protocol.md`:

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ implementation + exact hosted Evidence
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ status:review / status:blocked
→ release owner
→ stop
```

Coordinator Review must read Issue history, this contract, Candidate/PR, exact run/jobs/artifact and freshness classification before deciding `ACCEPT`, `REVISE`, `BLOCK`, `SPLIT`, or `NOT_PLANNED`. Only a later Coordinator `[FINAL ACCEPTANCE]` may move the Task to `status:done` and close it.

## Completion Protocol

The Worker must not process, claim, edit, or wait on #191 or #195. The Worker must not start #188 target execution from this Task. Finish the implementation/report lifecycle for #199, release ownership, and stop.
