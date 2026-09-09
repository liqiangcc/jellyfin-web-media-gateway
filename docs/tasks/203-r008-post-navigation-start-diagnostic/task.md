# Task — [R008-NAV-DIAGNOSTIC] Explain post-navigation-start termination on tx-node

## Metadata

```text
GitHub Issue: #203
Parent Goal / Research Item: #68 / R008
Task / Research ID: R008-POST-NAVIGATION-START-DIAGNOSTIC
Task kind: implementation
Base commit: current main at Task claim time
Candidate commit: produced by this Task
Session bootstrap prompt: docs/tasks/203-r008-post-navigation-start-diagnostic/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test
Hard publication dependencies: #188 Attempt 5 accepted evidence; #199 accepted stage-marker implementation
```

> GitHub Actions / Runner 是 execution backend，不是会 claim Issue 的 Worker，因此不使用 `env:actions` / `env:runner`。
>
> 实时状态、owner、Attempt、branch、PR、verification status 和 result summary 只保存在 GitHub Issue；本文件是稳定执行契约。

## Goal

Add the smallest safe diagnostic extension that explains a failure occurring after `navigation_start` by distinguishing a bounded navigation-promise rejection or timeout, a page/browser lifecycle termination, and process-level error or signal termination. The result must remain finite, sanitized, cleanup-aware, and compatible with the accepted stage-marker output.

## Why / Context

Issue #188 Attempt 5 ran the accepted stage-marker artifact on tx-node and supplied the first useful post-navigation boundary, but the terminal classifier remained `failure / error / unclassified_failure`. The exact parent evidence was:

- Candidate `c95beda94c87f166a1dd5056efb1834bdb79cc10`;
- hosted Actions run `34297453241`;
- artifact `10083700408`, accepted digest `sha256:e0f289cff7852aad60300863d0a7c86f8840d6291a5511e9e70b85095894fbb2`;
- target `VM-0-11-ubuntu` / `tx-node`, probe as `gateway-verify` UID/GID 1001;
- browser launch `2xx / success`, `navigation_start` observed, repeated broker/TLS `2xx` markers;
- no `navigation_status` or `navigation_end` before `finalizer_entry`;
- terminal result `failure / error / unclassified_failure`, 14 requests, 15,224 response bytes, no media request;
- fresh runtime and Attempt-owned cleanup complete, with existing `source-runtime` untouched.

Issue #199 accepted the finite stage-marker and finalizer implementation that produced this boundary. This Task adds only the next diagnostic distinction needed before a Coordinator-controlled #188 rerun; it does not claim browser portability, media extraction, playback, or production readiness.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: One focused experimental diagnostic implementation and its portable hosted verification share the same Candidate. A later tx-node rerun is Coordinator-controlled evidence outside this Task lifecycle.
```

## Worker Routing Decision

```text
Worker: cloud-codex
Environment: env:cloud
Requested model: gpt-5.6-luna, reasoning high
Fast: disabled by user; do not enable/select Fast
Execution backend: GitHub-hosted Actions for all build/test/package verification
```

## Work Role

### Implementation

The Candidate must extend the experimental browser diagnostic and its tests so that post-`navigation_start` termination is observable through finite stage markers and a finite diagnostic reason/status vocabulary. The implementation must be limited to the probe, finalizer, stage-marker, artifact-manifest and required workflow/test surfaces; it must not alter the Gateway playback authority or site-plugin contract.

### Verification

Claims to verify:

- C1: The extension is finite, allowlisted, bounded and backward-compatible with the accepted sanitized result shape.
- C2: Deterministic tests distinguish navigation-promise rejection/timeout, page/browser lifecycle termination, and process-level termination, and late callbacks cannot replace a finalized result.
- C3: Exact-Candidate hosted Actions verify the implementation, artifact manifest and independent artifact consumption.
- C4: Redaction, egress/SSRF, cleanup and diagnostic-only boundaries remain intact.

## Task vs Job Boundary

```text
Task
→ bounded post-navigation diagnostic implementation
→ Claims C1–C4
→ GitHub-hosted verification Jobs
→ Candidate/artifact evidence
→ Coordinator-controlled later #188 target rerun
```

Jobs do not claim Issue #203 and do not perform tx-node or Bilibili execution.

## Routing Rationale

This is ordinary repository implementation and regression work. Cloud Codex authors the Candidate, while GitHub-hosted Actions provide the build/test/package and artifact evidence. No target-specific capability is required for this Task.

## Preconditions

- Parent evidence: Issue #188 Attempt 5 report and Coordinator acceptance, including the exact stage-marker boundary above.
- Accepted implementation dependency: Issue #199 Final Acceptance, Candidate `c95beda94c87f166a1dd5056efb1834bdb79cc10`, hosted run `34297453241`, artifact `10083700408`, digest `sha256:e0f289cff7852aad60300863d0a7c86f8840d6291a5511e9e70b85095894fbb2`.
- Current main and the live Issue #203 must be read before claim; the worker must record the actual base SHA and new Candidate SHA.
- Required verification workflow: the repository's `bilibili-browser-probe` hosted workflow, with all required J1/J2/J3a/J3/J4/JI1 jobs bound to the exact Candidate.
- No target access is required or authorized. A later Coordinator-controlled #188 rerun may consume the accepted artifact only after review.
- Fast is disabled by the user; do not select or enable Fast.

## In Scope

- Extend the finite stage-marker vocabulary only as needed around post-`navigation_start` lifecycle boundaries.
- Record bounded, sanitized markers for navigation-promise settled/rejected/timeout/abort, page or browser lifecycle termination, and process-level error/signal termination.
- Preserve the existing allowlisted events: `browser_launch_start`, `browser_launch_result`, `navigation_start`, `navigation_status`, `navigation_end`, `broker_request_start`, `broker_request_result`, `transport_outcome`, and `finalizer_entry`.
- Keep a hard marker-count bound, monotonic sequence values, finalizer sealing, and first-terminal-result behavior so stale or late callbacks cannot overwrite a finalized result.
- Keep all reason/status values finite and allowlisted; omit raw error text and all URL, authority, header, cookie, token, certificate, profile, DOM, body, media and credential data.
- Add deterministic unit/contract coverage for each new termination class, normal completion, bounded timeout/abort, process signal/error publication and late callback ordering.
- Update the manifest-addressed artifact inventory and hosted workflow checks when implementation files change.
- Run all required verification through GitHub-hosted Actions on the exact Candidate.

## Out of Scope

- Any tx-node, browser, Bilibili, Gateway, VNC, CDP, phone or TV execution.
- Any click, play, full preload, independent consumer, media extraction, source portability or playback claim.
- Reusing, stopping, attaching to, inspecting or reconfiguring `source-runtime`.
- Proxy rotation, egress authorization, SSRF policy changes, credentials, cookies, Authorization, secret handling changes or production mutations.
- Changes to #188, #191 or #195 status/history, except that a Coordinator may later consume this Task's accepted artifact for #188.
- General browser-slot provisioning, UID/GID/cgroup/network namespace work, or a new target runner.
- Local build, test, package, dependency installation or compilation.
- Guessing a failure class from an unobserved raw error; unknown remains an allowed sanitized outcome.

## Architecture Invariants

- Gateway remains the `PlaybackSession` authority; this experimental diagnostic does not add playback state.
- Site-specific authority remains in the Bilibili plugin; Core and the diagnostic do not accept caller-controlled URLs or proxy authorities.
- The broker remains fail-closed and plugin-owned; no open proxy, SSRF relaxation or secret forwarding is introduced.
- Diagnostic output remains sanitized and bounded, and no Display Adapter or target runtime gains Vault access.
- Existing source-runtime and production services remain outside this Task's ownership.

## Files Expected to Change

- `experiments/bilibili-browser-probe/stage-markers.mjs`
- `experiments/bilibili-browser-probe/diagnostic.mjs`
- `experiments/bilibili-browser-probe/finalizer.mjs`
- `experiments/bilibili-browser-probe/live.mjs`
- `experiments/bilibili-browser-probe/probe.mjs`
- `experiments/bilibili-browser-probe/*test.mjs` for deterministic coverage
- `experiments/bilibili-browser-probe/artifact.mjs`
- `.github/workflows/bilibili-browser-probe.yml`

Only files needed to implement and verify this diagnostic may be changed; unrelated Issues and production surfaces remain untouched.

## Implementation Requirements

1. Keep the marker schema finite and bounded. Every marker uses a fixed event name, schema version, monotonic sequence and bounded counters. Unknown event/status input is rejected or omitted rather than serialized.
2. Add an explicit finite distinction for the post-navigation path, using an allowlisted vocabulary that covers navigation promise rejection, navigation timeout/abort, page or browser disconnect/termination, and process-level error/signal termination. Do not serialize exception names, messages, stack traces, URLs or request details.
3. Emit lifecycle markers at navigation promise settlement and page/browser termination boundaries, including the finalizer entry path. A normal completion must remain distinguishable from an interrupted or rejected navigation.
4. Make finalizer publication idempotent and sealed. Late promise callbacks, broker callbacks, page events and process events must not replace the first finalized result or append unbounded data after sealing.
5. Preserve the current result schema and activity/cleanup semantics. When process-level termination prevents proof of cleanup, use the existing finite `unknown` cleanup states rather than claiming cleanup.
6. Add deterministic tests that inject each finite class and verify ordering, cap behavior, redaction, finalizer precedence, and compatibility with the accepted #199 marker output.
7. Update artifact inventory and hosted checks so a fresh artifact is manifest-addressed to the exact Candidate. Do not reuse #199 runtime evidence as verification for changed code.

## Verification Plan

### Claims

```text
C1: The stage-marker and diagnostic output is finite, allowlisted, bounded, sanitized and compatible with the accepted result shape.
C2: Tests prove deterministic post-navigation classification and finalizer/late-callback safety.
C3: Exact-Candidate hosted Actions and artifact manifest/consumption gates pass.
C4: Static and hosted checks preserve no-playback, no-secret, no-proxy, SSRF/egress and cleanup boundaries.
```

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1,C2 | github-actions | github-hosted-x64 | runner-self | yes | probe contract, stage-marker, diagnostic and finalizer tests | exact run/job logs |
| J2 | C2,C4 | github-actions | github-hosted-x64 | runner-self | yes | Chromium broker containment, lifecycle and cleanup regressions | exact run/job logs |
| J3a | C3 | github-actions | github-hosted-x64 | runner-self | yes | build manifest-addressed artifact for exact Candidate | artifact manifest/digest |
| J3 | C3,C4 | github-actions | github-hosted-x64 | runner-self | yes | download and independently verify exact artifact; no target/site execution | artifact consumer logs |
| J4 | C1,C4 | github-actions | github-hosted-x64 | runner-self | yes | static boundary and live-admission checks; no target launch | static-check logs |
| JI1 | C2,C3,C4 | github-actions | github-hosted-x64 | runner-self | yes | remote workspace integration and regression suite | integration logs |

## Execution Plane

```text
Execution plane: github-actions
Target proof required: no
Later target rerun: Coordinator-controlled under Issue #188, after this Candidate is accepted
```

All required implementation verification, artifact creation and artifact consumption are GitHub-hosted. Cloud Codex does not substitute for a Runner.

## Runner Selection

```text
portable x64 build/test/lint
→ github-hosted-x64
```

No phone, TV or self-hosted target runner is part of this Task.

## Long-running / repeated verification

No long-running or target repetition is required. Deterministic lifecycle tests must cover repeated and late callback ordering within bounded test processes. A later #188 rerun is outside this Task and remains Coordinator-controlled.

## Interactive debugging

```text
WSL / Windows / Ubuntu ARM64 external Codex required: no
Reason: the required evidence is repository and GitHub-hosted diagnostic verification; no interactive target observation is authorized.
```

## Target verification

```text
Target proof required: no
Target: n/a
Why target evidence is required: n/a for this implementation Task; #188 owns any later target rerun.
```

## Runner Security Constraints

```text
Trusted candidate only: yes
Dedicated low-privilege runner user: GitHub-hosted runner isolation
Vault/profile access: forbidden
Production service mutation: forbidden
Cleanup / timeout requirements: bounded hosted jobs with artifact cleanup; no target residue is created
```

## Success Criteria

### Task success

1. A new exact Candidate implements finite post-`navigation_start` lifecycle markers and sanitized classifications for navigation rejection/timeout/abort, page/browser termination, and process error/signal paths.
2. Deterministic tests prove bounded output, redaction, finalizer idempotence and protection against stale/late callback overwrite.
3. The exact Candidate passes required hosted J1/J2/J3a/J3/J4/JI1 checks and produces a manifest-addressed artifact whose Candidate identity is verified.
4. The implementation changes no target, production, source-runtime, phone/TV, #188, #191 or #195 state and performs no local build/test/install.
5. The accepted evidence gives Coordinator a concrete, bounded artifact for a later #188 rerun; it does not claim playback success.

### Verification claim success

```text
C1 PASS when every emitted marker/reason/status is finite, allowlisted, bounded, sanitized and compatible with the accepted result shape.
C2 PASS when deterministic tests cover each required lifecycle class and prove finalizer sealing/first-result precedence against late callbacks.
C3 PASS when exact-Candidate hosted J1/J2/J3a/J3/J4/JI1 all pass and artifact manifest/Candidate identity are independently read back.
C4 PASS when hosted/static evidence confirms no raw sensitive output, no open proxy/SSRF/secret relaxation, no playback/media/target operation, and bounded cleanup.
```

## Evidence Contract

The worker report must separate implementation and verification results and record:

```text
Role: implementation + verification
Task / Claim: #203 / C1–C4
Attempt: N
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high
Fast: disabled by user
Execution plane: github-actions
Runner class: github-hosted-x64
Execution host: GitHub-hosted runner
Target host/device: n/a
OS / architecture: Actions runner facts from exact job
Base / Candidate commit: exact SHAs
Workflow / run / job: exact bilibili-browser-probe run and J1/J2/J3a/J3/J4/JI1 jobs
Commands / steps: bounded hosted test/build/manifest steps
Duration / repetitions / shards: exact hosted values
Metrics / artifact / raw evidence location: sanitized logs and artifact ID/digest
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

Do not submit secrets, cookies, tokens, account data, raw errors, complete sensitive URLs, DOM/HAR/body data or unnecessary large files.

## Failure / Blocked Handling

- `FAIL` means the implementation emits an unbounded/unknown raw value, cannot distinguish the required finite lifecycle class in deterministic tests, allows a late callback to overwrite a finalized result, or violates the broker/secret/playback boundary.
- `BLOCKED` means required GitHub-hosted Actions, artifact publication, or repository access is unavailable after bounded recovery. A target or browser is never needed to unblock this Task.
- A still-unknown runtime classification in a future #188 target result is valid evidence; this Task must not guess or broaden the vocabulary without evidence.
- If hosted Actions fail, fix the Candidate or report the exact hosted blocker. Do not run local build/test/package/install and do not use tx-node as a substitute.
- If a change would alter architecture/security contracts, stop and return the design change to the Coordinator before implementation.

## Deliverables

- Implementation: bounded post-navigation lifecycle markers, finite sanitized classifications, finalizer safety and deterministic tests.
- Candidate commit / PR: produced by the Worker and verified on exact Candidate.
- Session bootstrap prompt: `docs/tasks/203-r008-post-navigation-start-diagnostic/prompt.md`.
- Linked verification task: n/a; hosted jobs are inline.
- Verification jobs / runs: exact J1/J2/J3a/J3/J4/JI1 run and artifact recorded in the Issue report.
- Target evidence: none in this Task; later #188 rerun is Coordinator-controlled.
- Research evidence doc: n/a.

## Issue Feedback / Iteration Protocol

Follow:

```text
docs/tasks/issue-lifecycle-protocol.md
```

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ Execution Report / Blocker Report
→ status:review / status:blocked
→ Coordinator ACCEPT / REVISE / BLOCK / FINAL ACCEPTANCE
```

A code defect, hosted verification failure or evidence gap stays on Issue #203 and receives another Attempt. Do not create or execute a target Task from this package.

## Completion Protocol

Worker must submit the Candidate and hosted evidence, post `[EXECUTION REPORT]` or `[BLOCKER REPORT]` on #203, set `status:review` or `status:blocked`, release ownership and stop. The Worker must not set `status:done` or close the Issue. Later #188 target execution requires Coordinator control and a separate live Issue state.
