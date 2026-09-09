# Task — [R008-NAV-DIAGNOSTIC] Classify the remaining navigation promise rejection

## Metadata

```text
GitHub Issue: #207
Parent Goal / Research Item: #68 / R008
Task / Research ID: R008-NAVIGATION-REJECTION-CLASSIFICATION
Task kind: implementation
Base commit: c3dc7d7 (current main at package materialization)
Candidate commit: produced by this Task
Session bootstrap prompt: docs/tasks/207-r008-navigation-rejection-classification/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test
Hard publication dependencies: #188 Attempt 6 Final Acceptance; #203 Final Acceptance
```

GitHub Actions / Runner 是 execution backend，不是会 claim Issue 的 Worker。实时状态、owner、Attempt、branch、PR、verification status 和 result summary 只保存在 GitHub Issue；本文件是稳定执行契约。

## Goal

Extend the experimental browser diagnostic by the smallest safe allowlisted classifier needed to explain the remaining navigation promise rejection observed by #188 Attempt 6. Preserve the finite marker, redaction, finalizer, egress and cleanup boundaries while distinguishing a known navigation timeout/abort/target-closure class from an unknown rejection. Do not infer a lower-level cause that the evidence does not establish.

## Why / Context

Issue #188 Attempt 6 is Final Accepted and supplied the following exact target evidence:

- Candidate `e277220b625a2c4846ad3469a841398c685ad13b` from PR #205;
- hosted Actions run `34300151962`;
- artifact `10084651588`, 4,215,293 bytes, digest `sha256:f58e4ece689595b4e73808e8a8843dca25becef738209644658f77cf22713e50`;
- target `VM-0-11-ubuntu` / `tx-node`, probe as `gateway-verify` UID/GID 1001;
- browser launch `2xx / success`, `navigation_start`, broker `proxy_response` and TLS `2xx / success` markers;
- `navigation_promise_result=rejected`, followed by bounded `navigation_status`, `navigation_end` and `finalizer_entry`;
- terminal safe result `failure / error / unclassified_failure`, lifecycle outcome `unknown`, 13 requests, 15,224 response bytes, no media request;
- one fresh headless runtime and complete Attempt-owned cleanup, with existing `source-runtime` untouched.

Issue #203 Final Acceptance accepted the finite post-navigation diagnostic on PR #205 and merged it to `main` as `4b2590bf24c445ee1366a4c1e2101a7830efa357`. Its evidence shows that the promise settled as rejected but does not expose the underlying exception text or a more specific target cause. This Task narrows the next implementation step to deterministic classification and preservation of an explicit unknown fallback. It does not claim browser portability, media extraction, playback or completion of parent Goal #68.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: One focused diagnostic classifier and its portable hosted verification share one Candidate. A later tx-node rerun is Coordinator-controlled under #188 and is outside this Task lifecycle.
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

The Candidate must add only the finite classification and deterministic coverage required to explain the observed navigation-promise rejection. It may extend the existing diagnostic, stage-marker or finalizer modules and their artifact/workflow surfaces when necessary. It must not alter Gateway playback authority, Site Plugin authority, egress policy or production runtime behavior.

### Verification

Claims to verify:

- C1: The rejection classifier is finite, deterministic, bounded and compatible with the accepted sanitized result shape.
- C2: Tests cover known timeout/abort/target-closure and unknown rejection paths, plus finalizer sealing and late-callback precedence, without exporting raw error data.
- C3: Exact-Candidate hosted Actions verify implementation, artifact manifest and independent artifact consumption.
- C4: Static and hosted evidence preserve redaction, broker/SSRF/egress, no-secret, cleanup and diagnostic-only boundaries.

## Task vs Job Boundary

```text
Task
→ narrow navigation rejection classification
→ Claims C1–C4
→ GitHub-hosted verification Jobs
→ Candidate/artifact evidence
→ Coordinator-controlled later #188 target rerun
```

Jobs do not claim Issue #207 and do not perform tx-node, browser, Bilibili or production execution.

## Preconditions

- Issue #188 Attempt 6 Final Acceptance and its exact evidence above have been read back.
- Issue #203 Final Acceptance and Candidate/artifact evidence have been read back.
- Current main and live Issue #207 must be read before claim; the worker records the actual base and Candidate SHA.
- The repository `bilibili-browser-probe` hosted workflow is the required implementation verification authority.
- No target access is required or authorized. A later Coordinator-controlled #188 rerun may consume an accepted artifact after review.
- Fast is disabled by the user; do not select or enable Fast.

## In Scope

- Extend the existing experimental diagnostic classifier only as needed to map the observed navigation-promise rejection into a finite allowlisted class.
- Preserve or minimally extend the existing finite lifecycle vocabulary for navigation promise settlement, timeout/abort, page/browser termination, process error/signal termination and unknown fallback.
- Use only coarse structured error fields such as an allowlisted error code/name or explicit injected lifecycle hint; never serialize raw exception text.
- Ensure a classifier result cannot overwrite an already finalized result, and late page, broker, promise or process callbacks remain bounded after finalizer sealing.
- Add deterministic tests for known timeout, abort and target/page-closure inputs, unknown rejection fallback, redaction, marker bounds and first-final-result precedence. Tests must prove that a plausible but unobserved lower-level cause remains `unknown`.
- Update artifact inventory and hosted static checks when implementation files change.
- Run all required implementation verification through GitHub-hosted Actions on the exact Candidate.

## Out of Scope

- Any tx-node, browser, Bilibili, Gateway, VNC, CDP, phone or TV execution.
- Any click, play, full preload, independent consumer, media extraction, playback or source-portability claim.
- Any Coordinator-controlled #188 rerun; this Task only supplies a reviewable artifact for that later action.
- Reusing, stopping, attaching to, inspecting or reconfiguring `source-runtime`.
- Proxy rotation, egress authorization, SSRF relaxation, credentials, cookies, Authorization, secret handling changes or production mutations.
- Changes to #188 status/history or any processing of #191/#195.
- General browser-slot provisioning, UID/GID/cgroup/network namespace work, new target runner or architecture redesign.
- Local build, test, package, dependency installation or compilation.
- Guessing a specific Chromium/network cause from the accepted `unknown` evidence.

## Architecture Invariants

- Gateway remains the `PlaybackSession` authority; this experiment adds no playback state.
- Site-specific authority remains in the Bilibili plugin; the diagnostic does not accept caller-controlled URL or proxy authority.
- The broker remains fail-closed and plugin-owned; no open proxy, SSRF relaxation or secret forwarding is introduced.
- Diagnostic output remains finite, sanitized and bounded; no URL, authority, header, cookie, token, certificate, profile, DOM, body, media or credential data leaves the process.
- Existing `source-runtime` and production services remain outside this Task's ownership.

## Files Expected to Change

- `experiments/bilibili-browser-probe/diagnostic.mjs`
- `experiments/bilibili-browser-probe/stage-markers.mjs`
- `experiments/bilibili-browser-probe/finalizer.mjs`
- `experiments/bilibili-browser-probe/live.mjs`
- `experiments/bilibili-browser-probe/probe.mjs`
- `experiments/bilibili-browser-probe/*test.mjs` for deterministic coverage
- `experiments/bilibili-browser-probe/artifact.mjs`
- `.github/workflows/bilibili-browser-probe.yml`

Only files required for the narrow diagnostic and its verification may change. Do not modify #188, #191, #195 or production playback surfaces.

## Implementation Requirements

1. Keep every diagnostic reason, phase, transport stage, lifecycle outcome and stage event in a closed allowlist with hard bounds and safe unknown fallback.
2. Classify only evidence supplied by an explicit allowlisted code/name or deterministic lifecycle hint. The classifier must not inspect or export an Error message, stack, URL, host, address, certificate, header, body, profile path or browser content.
3. Preserve the distinction between a promise rejection and a page/browser/process termination marker. A successful broker/TLS marker must not be relabeled as a navigation success, and an unobserved cause must remain `unknown`.
4. Keep finalizer publication idempotent and sealed. A late callback must neither replace the first finalized result nor append unbounded marker data.
5. Preserve the accepted result schema and cleanup semantics. If process termination prevents cleanup proof, use the existing finite `unknown` cleanup state.
6. Add deterministic tests covering each new allowlisted class, unknown fallback, malformed/sensitive inputs, marker-cap behavior and late-callback/finalizer ordering.
7. Update manifest-addressed artifact inventory and hosted checks so a fresh artifact is bound to the exact Candidate. Do not reuse #203 runtime evidence as verification for changed code.

## Freshness / Integration Contract

Freshness policy: dependency-aware

Semantic authorities:
- #188 Attempt 6 Final Acceptance and its frozen navigation-only target contract
- #203 Final Acceptance and the finite diagnostic/stage-marker/finalizer contract
- `docs/security.md` and the existing Bilibili experimental broker boundary

Semantic freshness domains:
- `experiments/bilibili-browser-probe/diagnostic.mjs`
- `experiments/bilibili-browser-probe/stage-markers.mjs`
- `experiments/bilibili-browser-probe/finalizer.mjs`
- `experiments/bilibili-browser-probe/live.mjs` and `probe.mjs` lifecycle publication
- accepted sanitized diagnostic schema and redaction/egress invariants

Integration surfaces:
- `experiments/bilibili-browser-probe/artifact.mjs`
- `.github/workflows/bilibili-browser-probe.yml`
- hosted Node/Chromium probe test harness

Task-owned surfaces:
- narrow navigation rejection classification and deterministic tests under `experiments/bilibili-browser-probe/`
- manifest/static verification surfaces required for those files

Authority/domain → Claim mapping:
- #188/#203 finite diagnostic and marker contract: C1,C2
- artifact manifest and hosted workflow: C3
- security, redaction, broker and cleanup boundaries: C1,C4

Integration verification:
- JI1: exact-Candidate remote workspace integration and regression suite in `bilibili-browser-probe`

Unrelated-main policy:
- existing exact-Candidate semantic Evidence remains valid; no rebase/full rerun solely because main advances

Integration-overlap policy:
- preserve accepted semantic Evidence; compose the Task Candidate with the Coordinator-frozen Integration Base and run declared JI1 unless a conflict changes Task semantics

Semantic-authority-change policy:
- reconcile any live change to the #188/#203 diagnostic or security authority and rerun mapped Claims; broaden verification only when impact cannot be safely bounded

Strict-main reason:
- n/a

## Verification Plan

### Claims

```text
C1: Finite allowlisted rejection classification and sanitized compatibility.
C2: Deterministic known/unknown rejection, redaction and finalizer-ordering coverage.
C3: Exact-Candidate hosted Actions and artifact manifest/independent-consumer gate.
C4: Preserved no-secret, egress/SSRF, cleanup, no-playback and diagnostic-only boundaries.
```

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1,C2 | github-actions | github-hosted-x64 | runner-self | yes | probe contract, diagnostic, stage-marker and finalizer tests | exact run/job logs |
| J2 | C2,C4 | github-actions | github-hosted-x64 | runner-self | yes | Chromium broker containment, lifecycle, rejection and cleanup regressions | exact run/job logs |
| J3a | C3 | github-actions | github-hosted-x64 | runner-self | yes | manifest-addressed artifact for exact Candidate | artifact manifest/digest |
| J3 | C3,C4 | github-actions | github-hosted-x64 | runner-self | yes | download and independently consume exact artifact; no target/site execution | consumer logs |
| J4 | C1,C4 | github-actions | github-hosted-x64 | runner-self | yes | static boundary and live-admission checks; no target launch | static-check logs |
| JI1 | C2,C3,C4 | github-actions | github-hosted-x64 | runner-self | yes | remote workspace integration and regression suite | integration logs |

## Execution Plane

```text
Execution plane: github-actions
Target proof required: no
Later target rerun: Coordinator-controlled under Issue #188 after this Candidate is accepted
```

All implementation verification, artifact creation and artifact consumption are GitHub-hosted. Cloud Codex does not substitute for a Runner.

## Runner Selection

```text
portable x64 build/test/lint
→ github-hosted-x64
```

No phone, TV or self-hosted target runner is part of this Task.

## Target Verification

```text
Target proof required: no
Target: n/a
Reason: this Task produces an implementation artifact; #188 owns any later tx-node rerun.
```

## Runner Security Constraints

```text
Trusted candidate only: yes
Dedicated low-privilege runner user: GitHub-hosted runner isolation
Vault/profile access: forbidden
Production service mutation: forbidden
Cleanup / timeout requirements: bounded hosted jobs; no target residue is created
```

## Success Criteria

### Task success

1. A new exact Candidate classifies the remaining navigation promise rejection only through finite allowlisted evidence and preserves an explicit unknown fallback.
2. Deterministic tests prove timeout/abort/target-closure and unknown rejection behavior, redaction, marker bounds and finalizer/late-callback safety.
3. Exact-Candidate hosted J1/J2/J3a/J3/J4/JI1 checks pass and produce a manifest-addressed artifact whose Candidate identity is independently verified.
4. No target, production, source-runtime, phone/TV, #188, #191 or #195 action is performed, and no local build/test/install occurs.
5. The accepted artifact gives the Coordinator a bounded next step for a later #188 rerun without claiming playback success.

### Verification claim success

```text
C1 PASS when all emitted class/event/status values are finite, allowlisted, bounded, sanitized and compatible with the accepted schema.
C2 PASS when deterministic tests cover known timeout/abort/target-closure and unknown rejection paths, malformed/sensitive input and finalizer precedence.
C3 PASS when exact-Candidate hosted J1/J2/J3a/J3/J4/JI1 all pass and artifact manifest/Candidate identity are independently read back.
C4 PASS when hosted/static evidence confirms no raw sensitive output, no open proxy/SSRF/secret relaxation, no playback/target operation and bounded cleanup.
```

## Evidence Contract

The worker report must separate implementation and verification results and record:

```text
Role: implementation + verification
Task / Claim: #207 / C1–C4
Attempt: N
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high
Fast: disabled by user
Execution plane: github-actions
Runner class: github-hosted-x64
Execution host: GitHub-hosted runner
Target host/device: n/a
Base / Candidate commit: exact SHAs
Workflow / run / job: exact bilibili-browser-probe run and J1/J2/J3a/J3/J4/JI1 jobs
Commands / steps: bounded hosted test/build/manifest steps
Duration / repetitions / shards: exact hosted values
Metrics / artifact / raw evidence location: sanitized logs and artifact ID/digest
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

Do not submit secrets, cookies, tokens, account data, raw errors, complete sensitive URLs, DOM/HAR/body data or unnecessary large files.

## Failure / Blocked Handling

- `FAIL` means the implementation emits an unbounded or raw value, misclassifies unobserved evidence as a known cause, cannot prove finalizer precedence, or violates broker/secret/playback boundaries.
- `BLOCKED` means required GitHub-hosted Actions, artifact publication, or repository access is unavailable after bounded recovery. Target access is never needed to unblock this Task.
- If the target evidence cannot justify a more specific class, `unknown` is a valid result and must remain visible; do not guess.
- If hosted Actions fail, fix the Candidate or report the exact hosted blocker. Do not run local build/test/package/install and do not use tx-node as a substitute.
- If a change would alter architecture/security contracts, stop and return the design change to the Coordinator before implementation.

## Deliverables

- Implementation: finite navigation rejection classifier and deterministic sanitized/finalizer regression coverage.
- Candidate commit / PR: produced by the Worker and verified on the exact Candidate.
- Later target use: only through a new Coordinator-controlled #188 Attempt after review.
