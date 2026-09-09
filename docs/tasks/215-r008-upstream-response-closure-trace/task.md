# Task — [R008-NAV-DIAGNOSTIC] Trace upstream response closure before navigation completion

## Metadata

```text
GitHub Issue: #215
Parent Goal / Research Item: #68 / R008
Task / Research ID: R008-UPSTREAM-RESPONSE-CLOSURE-TRACE
Task kind: implementation
Base commit: 603e3997bfd7e0b9ff861b7b1c34d709ee3a37ff
Candidate commit: produced by this Task
Session bootstrap prompt: docs/tasks/215-r008-upstream-response-closure-trace/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test
Hard publication dependencies: #188 Attempt 8 Final Acceptance
```

GitHub-hosted Actions are the build/test/package execution backend and do not claim this Issue. The Issue owns live status, owner, Attempt, branch, PR, verification status and result summary; this file is the stable execution contract.

## Goal

Add the smallest safe broker lifecycle evidence needed to determine whether the accepted navigation rejection follows an upstream response start/end, a socket close before body completion, a timeout/abort, or an allowlisted upstream error class. Keep the result finite and sanitized, and preserve the current rejection behavior when the evidence cannot establish a more specific cause.

## Why / Context

Parent #188 Attempt 8 is Final Accepted as a target diagnostic result. Its accepted artifact ran once on tx-node using a fresh headless `gateway-verify` runtime. Browser launch and broker/TLS activity reached the navigation path, then the navigation promise was rejected with `phase=chromium_navigation`, `reason=navigation_promise_rejected`, and `transport_stage=downstream_close`; the session observed 14 requests and 15,224 response bytes with no media request, and cleanup completed without touching `source-runtime`. That evidence identifies the boundary but does not reveal response/socket closure ordering. This Task narrows the next repository experiment to that missing evidence.

## Accepted parent evidence

Issue #188 Attempt 8 provides the exact target evidence and artifact identity:

- implementation Candidate `8246c9f79db64b36b00c0b9a938569b28c55bc29`, PR #213;
- hosted Actions run `34305340293`;
- artifact `10086463816`, 4,215,888 bytes, digest `sha256:3bed203c49a1d5edf815633e390147236cb814083ead242bd049077c87af1bde`;
- target `tx-node` / `gateway-verify` UID/GID 1001, fresh headless runtime;
- browser launch `2xx/success`, `navigation_start`, and broker/TLS success markers;
- `navigation_promise_result=rejected`, `phase=chromium_navigation`, `reason=navigation_promise_rejected`, `transport_stage=downstream_close`, `transport_outcome=failure`, `lifecycle_outcome=rejected`;
- 14 requests, 15,224 response bytes, 0 metadata bytes, no media request, click, play, preload or consumer activity;
- browser/broker/profile/candidate/DNS cleanup complete, staging absent, and existing `source-runtime` untouched.

The target result is diagnostic evidence only. It does not prove playback, source portability or a browser/network root cause.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: The bounded marker implementation, deterministic regressions and manifest/static checks share one experimental probe surface and are expressible in the existing hosted J1/J2/J3a/J3/J4/JI1 matrix. A later #188 target rerun has a separate target Evidence Authority and remains Coordinator-controlled.
```

## Worker Routing Decision

```text
Worker: cloud-codex
Environment: env:cloud
Requested model: gpt-5.6-luna, reasoning high
Fast: enabled by current user routing; may be selected
Execution backend: GitHub-hosted Actions for all build/test/package verification
```

## Work Role

### Implementation

Produce one reviewable Candidate that adds only finite upstream response/socket lifecycle markers and bounded error classes to the experimental Bilibili browser probe/broker. Preserve explicit `unknown` when ordering or cause is not safely established. Add deterministic regressions for the observed boundary and the redaction/finalizer/cleanup invariants. The Candidate must be independently packaged and consumed by hosted Actions; it must not claim that the later #188 target result has changed.

### Verification

Claims are verified against the exact Candidate SHA by hosted Actions. No target or site session is part of this Task.

## Task vs Job Boundary

```text
Task
→ narrow broker lifecycle implementation
→ finite claims
→ GitHub-hosted J1/J2/J3a/J3/J4/JI1 jobs
→ Candidate/artifact/regression evidence
```

The jobs do not claim Issue #215 and do not access tx-node, a browser, Bilibili or production.

## Routing Rationale

The implementation and all required verification are repository-local and deterministic. Codex Cloud is the Worker; GitHub-hosted x64 Actions are the execution backend for build, tests, packaging and static boundary checks. A later Coordinator-controlled #188 Attempt may use the accepted artifact for target evidence, but this Task does not perform or authorize that rerun.

## Preconditions

- Parent evidence: #188 Attempt 8 Final Acceptance and its exact Candidate/artifact/target report.
- Relevant accepted implementation: #211 Final Acceptance and its transport/failure-precedence correction, already included in the parent artifact.
- Read before claim: `AGENTS.md`, `docs/README.md`, `docs/requirements.md`, `docs/architecture.md`, `docs/implementation-contracts.md`, `docs/technical-feasibility-validation.md`, `docs/mvp-plan.md`, `docs/security.md`, `docs/development-environments.md`, `docs/runner-execution-architecture.md`, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, the current Issue #215 and all relevant comments, plus the browser-probe/runbook documents.
- Publication state: this package starts as `status:draft`; the Coordinator must complete the Publication Gate before any Worker claim.
- Required repository verification: GitHub-hosted J1/J2/J3a/J3/J4/JI1 against one exact Candidate.
- Target execution is outside this Task. Do not access tx-node, browser, Bilibili, VNC/CDP, phone/TV or production.

## In Scope

- Experimental Bilibili browser probe/broker response and socket lifecycle instrumentation.
- A finite allowlist of upstream lifecycle markers covering response start, response end, body completion, socket close ordering, timeout/abort and bounded upstream error classes where the implementation can establish them.
- Bounded request/response counters and ordering fields required to interpret the markers.
- Explicit `unknown` fallback for incomplete, conflicting or opaque lifecycle evidence.
- Deterministic tests for upstream response closure before navigation completion, normal response completion, timeout/abort, bounded error classification, late callback/finalizer precedence, cleanup and redaction.
- Existing artifact manifest/static checks needed to bind the new Candidate and hosted artifact.
- GitHub-hosted Actions verification and a reviewable Candidate/PR.

## Out of Scope

- Any tx-node, browser, Bilibili, Gateway production or target execution. A later #188 rerun is Coordinator-controlled.
- Click, play, full preload, independent consumer, media extraction, playback or source portability claims.
- Guessed Chromium/network bypasses, unrestricted retries, proxy rotation, egress relaxation, SSRF exceptions, TLS verification changes or authority changes.
- Raw URLs, authorities, error text, headers, cookies, tokens, certificates, profile data, body/media data, credentials or secrets in output, logs or artifacts.
- Stopping, attaching to, inspecting or reconfiguring `source-runtime`.
- Phone/TV deployment, VNC/CDP observation, production mutation or new slot infrastructure.
- Processing, claiming, editing or waiting on #191 or #195.
- Local build, test, package, dependency installation or compilation.

## Architecture and Security Invariants

- Gateway remains the `PlaybackSession` authority; this diagnostic adds no playback state.
- Site-specific authority remains in the Bilibili plugin; callers cannot supply arbitrary URL, host, proxy, profile, header or credential input.
- Broker egress remains fail-closed under the existing SSRF/public-host/TLS/redirect policy. No open proxy or new private-network exception is introduced.
- Diagnostic output remains finite, bounded and sanitized. Upstream lifecycle values and error classes use closed allowlists with safe `unknown` fallback.
- Finalizer publication remains at-most-once; late callbacks cannot overwrite a sealed result or a successful transport outcome except under an existing explicit budget rule.
- Browser/probe runtime remains non-root and disposable for any future target rerun; this Task does not control target resources or production services.

## Files Expected to Change

- The experimental probe/broker lifecycle modules and their deterministic tests, only where required by the narrow scope.
- Artifact manifest/static verification surfaces only if the implementation files require their existing inventory checks.

## Implementation Requirements

1. Trace every marker or error class to a deterministic event or an explicitly bounded runtime callback. Do not infer an upstream cause from `downstream_close` alone.
2. Use a closed, versioned vocabulary for response/socket lifecycle events and error classes. Keep counters bounded and preserve `unknown` for missing or ambiguous ordering.
3. Record response start/end/body completion and socket closure ordering without exporting raw URL, authority, error, headers, body, media, cookie, token, profile or credential data.
4. Preserve transport/failure precedence from #211: a late opaque callback cannot replace a finalized transport result; only explicitly permitted lifecycle or budget paths may supersede it.
5. Keep cleanup re-entrant and Attempt-owned. Do not broaden cleanup to host-wide processes or services.
6. Add meaningful deterministic regressions for the specific closure boundary and no-raw-data output. Avoid tests that merely assert implementation constants.
7. Keep the fixed plugin-owned selector and all SSRF/egress/secret/no-playback boundaries unchanged.
8. If evidence would require changing a canonical architecture or security invariant, stop and return the evidence to the Coordinator instead of implementing that change.

## Claims

- C1: Upstream response/socket lifecycle evidence is finite, allowlisted, bounded and directly tied to deterministic events; ambiguous causes remain explicitly `unknown`.
- C2: Response closure, navigation lifecycle, transport precedence, finalizer and cleanup remain safe under normal, early-close, timeout/abort, error and late-callback orderings.
- C3: Exact-Candidate GitHub-hosted J1/J2/J3a/J3/J4/JI1 verification passes and J3a/J3 independently produce/consume an artifact bound to that Candidate.
- C4: SSRF/egress, no-secret, no-playback, no-target and source-runtime boundaries remain intact.

## Verification Plan

### Claims

```text
C1: finite upstream response/socket lifecycle markers and bounded error classes are deterministic and sanitized.
C2: closure ordering, precedence, finalizer and cleanup are safe under reproduced lifecycle orderings.
C3: exact-Candidate hosted verification and artifact provenance are complete.
C4: security, no-playback and no-target boundaries remain enforced.
```

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1,C2 | github-actions | github-hosted-x64 | runner-self | yes | probe contract, lifecycle, diagnostic and deterministic response-closure tests | run/job logs and test result |
| J2 | C2,C4 | github-actions | github-hosted-x64 | runner-self | yes | broker transport, upstream closure, redaction, finalizer and cleanup regressions | run/job logs and sanitized output |
| J3a | C3 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate artifact build/manifest/digest gate | artifact ID, manifest and digest |
| J3 | C3,C4 | github-actions | github-hosted-x64 | runner-self | yes | independent exact-Candidate artifact consumer and static contract checks | consumer job and artifact read-back |
| J4 | C1,C4 | github-actions | github-hosted-x64 | runner-self | yes | static allowlist, plugin-owned authority, SSRF/secret/no-playback/no-target checks | static job logs |
| JI1 | C2,C3,C4 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate integration and regression suite | integration job logs |

All required build/test/package commands run only on GitHub-hosted Actions. No job performs target or site execution.

## Freshness / Integration Contract

Freshness policy: dependency-aware.

Semantic authorities:

- #188 Attempt 8 Final Acceptance and its frozen navigation-only target boundary;
- #211 Final Acceptance and transport/failure-precedence behavior;
- `docs/security.md`, `docs/architecture.md` and the experimental broker/plugin authority.

Semantic freshness domains:

- finite diagnostic schema and lifecycle marker vocabulary;
- broker response/socket/transport precedence and finalizer behavior;
- SSRF, secret containment, selector authority and no-playback boundaries.

Integration surfaces:

- `experiments/bilibili-browser-probe/` and plugin-owned Bilibili probe modules;
- deterministic probe/broker tests;
- artifact manifest/static-check workflow surfaces.

Task-owned surfaces:

- upstream response/socket lifecycle marker implementation and its focused tests;
- any exact-Candidate artifact inventory changes caused by those files.

Authority/domain → Claim mapping:

- #188/#211 diagnostic boundary → C1,C2;
- broker and plugin security authority → C1,C2,C4;
- hosted artifact/workflow provenance → C3;
- no-playback/no-target rules → C4.

Integration verification jobs: JI1 plus the exact-Candidate J3a/J3 artifact checks.

If a semantic authority changes, stop and reconcile this contract before implementation. If `main` advances without semantic conflict, record the contract base and actual implementation base; do not invalidate dependency-aware Evidence solely because of a descendant merge. Any later target rerun remains a new Coordinator-controlled #188 Attempt.

## Success Criteria

1. The Candidate adds only evidence-supported, finite upstream response/socket lifecycle markers and bounded error classes, with explicit unknown fallback and no raw-data leakage.
2. Deterministic regressions cover response completion, early closure, timeout/abort, bounded upstream errors, finalizer/late-callback precedence, cleanup and redaction.
3. Exact-Candidate hosted J1/J2/J3a/J3/J4/JI1 jobs pass, and J3a/J3 independently verify an artifact bound to that Candidate.
4. Reports distinguish implementation and hosted verification evidence from any future #188 target result.
5. No tx-node/browser/Bilibili/playback/VNC/CDP/phone/TV/production/#191/#195 activity and no local build/test/package/install occurs in this Task.

## Evidence Contract

Record the real Task/Claim/Attempt, base and Candidate SHAs, PR, workflow/run/job/artifact identities, runner and execution plane, test selectors, implementation result, hosted verification result and limitations. Keep all output sanitized and bounded. Do not record secrets, credentials, raw URLs, response bodies, media payloads, profile paths or raw upstream errors.

## Completion

The Worker posts `[EXECUTION REPORT]` or a genuine `[BLOCKER REPORT]` on Issue #215, transitions it to `status:review` or `status:blocked`, releases ownership and stops. The Worker must not set `status:done` or close the Issue. A later #188 target rerun is outside this Task and requires a new Coordinator handoff.
