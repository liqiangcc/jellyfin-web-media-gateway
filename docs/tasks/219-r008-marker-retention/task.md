# Task — [R008-NAV-DIAGNOSTIC] Reserve upstream markers under bounded retention

## Metadata

```text
GitHub Issue: #219
Parent Goal / Research Item: #68 / R008
Task / Research ID: R008-MARKER-RETENTION
Task kind: implementation
Base commit: 00e0794823548049ba557b3825d221455dfb938e
Candidate commit: produced by this Task
Session bootstrap prompt: docs/tasks/219-r008-marker-retention/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test
Hard publication dependencies: #188 Attempt 9 Final Acceptance; #215 Final Acceptance
```

GitHub-hosted Actions are the build/test/package execution backend. They do not claim this Issue. The Issue owns live status, owner, Attempt, branch, PR, verification status and result summary; this file is the stable execution contract.

## Goal

Correct the experimental probe's finite marker-retention policy so the required upstream response/socket boundary markers remain observable together with the navigation and finalizer boundary under the request noise reproduced by #188 Attempt 9. Keep output bounded, allowlisted and sanitized, and retain an explicit `unknown` fallback whenever the lifecycle cannot be established safely.

## Why / Context

#188 Attempt 9 is Final Accepted. Two fresh `tx-node` / `gateway-verify` navigation-only sessions using the accepted #215 artifact reproduced `navigation_promise_result=rejected` with `transport_stage=downstream_close`, 14 requests and 15,224 response bytes, with no media request. Each session emitted 29 markers and ended at `finalizer_entry`; zero `upstream_*` markers survived because broker noise consumed the finite stream while post-navigation and finalizer markers were reserved. The result proves a deterministic retention gap, not an upstream socket cause. This Task addresses that gap before any further Coordinator-controlled #188 target rerun.

## Accepted parent evidence

- Parent: #188 Attempt 9 Final Acceptance.
- Accepted implementation Candidate: `47c000bd25abf22b4c759de36b50c2cf259a07a8`, PR #217.
- Hosted Actions run: `34307216913`; artifact `10087091648`, 4,217,514 bytes.
- Artifact digest: `sha256:3d30d09c74d071c51747088b90cfad88b37da3588ce07c7a6b87bdde28cee0f9`.
- Target: `tx-node` / `gateway-verify` UID/GID 1001, two fresh navigation-only sessions.
- Both sessions: `chromium_navigation / navigation_promise_rejected / downstream_close`, 14 requests, 15,224 response bytes, no media activity, cleanup complete, existing `source-runtime` untouched.
- Both sessions emitted 29 finite markers and zero retained `upstream_*` markers. No upstream response/socket cause is inferred.

#215 Final Acceptance established the finite upstream response/body/socket lifecycle vocabulary and bounded error classes that this Task must preserve.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: The retention policy, deterministic regressions, static boundary checks and exact artifact provenance share the existing experimental probe surface. A later #188 target rerun has a separate target Evidence Authority and remains Coordinator-controlled.
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

Produce one reviewable Candidate that changes only the finite marker retention/reservation behavior and focused deterministic tests needed to retain upstream markers under observed broker noise. Preserve the current marker schema, allowlists, counters, redaction, transport/failure precedence, finalizer sealing and cleanup behavior unless a narrowly justified compatibility-preserving adjustment is required. The Candidate must be independently packaged and consumed by hosted Actions; it must not claim a changed target or playback result.

### Verification

Claims are verified against the exact Candidate SHA by GitHub-hosted Actions. This Task performs no tx-node, browser, Bilibili or production execution.

## Task vs Job Boundary

```text
Task
→ finite marker-retention correction
→ deterministic retention/redaction/finalizer regressions
→ GitHub-hosted J1/J2/J3a/J3/J4/JI1 jobs
→ Candidate/artifact evidence
```

Jobs do not claim this Issue and do not access tx-node, a browser, Bilibili or production.

## Routing Rationale

This is repository implementation with deterministic tests and artifact checks. Codex Cloud is the Worker; GitHub-hosted x64 Actions are the execution backend for all required build, test, packaging and static verification. A later Coordinator-controlled #188 target rerun may consume the accepted artifact, but this Task neither performs nor authorizes that rerun.

## Preconditions

- Read before claim: `AGENTS.md`, `docs/README.md`, `docs/requirements.md`, `docs/architecture.md`, `docs/implementation-contracts.md`, `docs/technical-feasibility-validation.md`, `docs/mvp-plan.md`, `docs/security.md`, `docs/development-environments.md`, `docs/runner-execution-architecture.md`, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, the live Issue #219 and relevant #188/#215 history, and the browser-probe documentation.
- Parent evidence: #188 Attempt 9 Final Acceptance and exact artifact identity above.
- Existing implementation authority: #215 Final Acceptance and its finite upstream marker vocabulary.
- Publication state: this package starts as `status:draft`; the Coordinator must complete the Publication Gate before a Worker claim.
- Required repository verification: GitHub-hosted J1/J2/J3a/J3/J4/JI1 against one exact Candidate.
- Target execution is outside this Task. Do not access tx-node, browser, Bilibili, VNC/CDP, phone/TV or production.

## In Scope

- `experiments/bilibili-browser-probe/stage-markers.mjs` retention/reservation policy and focused tests.
- Broker/probe integration tests needed to reproduce noisy pre-navigation activity and prove upstream marker retention.
- Deterministic assertions for bounded marker count, event ordering, finalizer sealing, redaction and explicit unknown fallback.
- Existing artifact manifest/static verification surfaces only if the implementation files require their inventory checks.
- GitHub-hosted Actions verification and a reviewable Candidate/PR.

## Out of Scope

- Any tx-node, browser, Bilibili, Gateway production or target execution. A later #188 rerun is Coordinator-controlled.
- Click, play, full preload, independent consumer, media extraction, playback or source portability claims.
- Guessed Chromium/network bypasses, proxy rotation, egress relaxation, SSRF exceptions, TLS verification changes or authority changes.
- Raw URLs, authorities, error text, headers, cookies, tokens, certificates, profile data, response bodies, media data, credentials or secrets in output, logs or artifacts.
- Stopping, attaching to, inspecting or reconfiguring `source-runtime`.
- Phone/TV deployment, VNC/CDP observation, production mutation or new slot infrastructure.
- Processing, claiming, editing or waiting on #191 or #195.
- Local build, test, package, dependency installation or compilation.

## Architecture and Security Invariants

- Gateway remains the `PlaybackSession` authority; this diagnostic adds no playback state.
- Marker output is a finite, versioned, allowlisted DTO. Unknown or ambiguous lifecycle evidence is represented as `unknown`; raw callback data is never exported.
- Upstream marker retention must not weaken SSRF, `EgressPolicy`, TLS, redirect or plugin-owned authority boundaries.
- Finalizer publication remains at-most-once; late callbacks cannot append after sealing or replace a finalized transport/failure result.
- No site Secret, Cookie, Authorization, URL, response body, profile path or media payload enters diagnostics.
- This Task changes no production service and no target runtime.

## Files Expected to Change

- `experiments/bilibili-browser-probe/stage-markers.mjs`
- `experiments/bilibili-browser-probe/stage-markers.test.mjs`
- `experiments/bilibili-browser-probe/live-broker.test.mjs` or another existing focused test surface only if required by the retention regression.
- Artifact/static verification files only if required to inventory the changed implementation files.

## Implementation Requirements

1. Define a deterministic retention/reservation policy within the existing finite marker budget. It must retain the required upstream response/socket boundary markers under the observed pre-navigation broker noise while preserving navigation and finalizer observability.
2. Keep the event vocabulary, bounded counters and sanitizer closed and versioned. Invalid or caller-controlled event/field values must be rejected or normalized to `unknown`.
3. Make retention behavior deterministic for normal completion, early closure, timeout/abort, upstream error, navigation rejection and finalizer entry. Do not infer a cause merely from `downstream_close`.
4. Preserve the #211 transport/failure precedence and at-most-once finalizer semantics. Late callbacks must not overwrite a finalized result or append after sealing.
5. Add meaningful deterministic regressions that reproduce at least the 29-marker/no-upstream observation and prove the required upstream markers are retained, bounded and ordered. Cover unknown fallback, redaction and cleanup/finalizer boundaries where the affected code path requires it.
6. Keep the fixed plugin-owned selector, SSRF/egress policy, secret boundary and no-playback/no-target boundaries unchanged.
7. Do not add a second marker stream, unbounded queue, raw-error escape hatch or caller-controlled reservation authority.
8. If a safe correction requires changing a canonical architecture or security invariant, stop and return the evidence to the Coordinator instead of implementing that change.

## Claims

- C1: The retention policy deterministically retains required upstream response/socket markers under reproduced broker noise within the finite output bound, with explicit `unknown` fallback for incomplete or ambiguous evidence.
- C2: Marker ordering, redaction, transport precedence, finalizer sealing and cleanup remain safe across normal, early-close, timeout/abort, error and late-callback orderings.
- C3: Exact-Candidate GitHub-hosted J1/J2/J3a/J3/J4/JI1 verification passes, and J3a/J3 independently produce/consume an artifact bound to that Candidate.
- C4: SSRF/egress, no-secret, no-playback, no-target and source-runtime boundaries remain intact.

## Verification Plan

### Claims

```text
C1: finite marker retention preserves required upstream lifecycle evidence under observed noise.
C2: lifecycle ordering, precedence, finalizer, redaction and cleanup remain safe.
C3: exact-Candidate hosted verification and artifact provenance are complete.
C4: security, no-playback, no-target and source-runtime boundaries remain enforced.
```

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1,C2 | github-actions | github-hosted-x64 | runner-self | yes | stage-marker retention and deterministic lifecycle tests | run/job logs and test result |
| J2 | C2,C4 | github-actions | github-hosted-x64 | runner-self | yes | broker closure, redaction, precedence, finalizer and cleanup regressions | run/job logs and sanitized output |
| J3a | C3 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate artifact build/manifest/digest gate | artifact ID, manifest and digest |
| J3 | C3,C4 | github-actions | github-hosted-x64 | runner-self | yes | independent exact-Candidate artifact consumer and static contract checks | consumer job and artifact read-back |
| J4 | C1,C4 | github-actions | github-hosted-x64 | runner-self | yes | static allowlist, plugin-owned authority, SSRF/secret/no-playback/no-target checks | static job logs |
| JI1 | C2,C3,C4 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate integration and regression suite | integration job logs |

All required build/test/package commands run only on GitHub-hosted Actions. No job performs target or site execution.

## Freshness / Integration Contract

Freshness policy: dependency-aware.

Semantic authorities:

- #188 Attempt 9 Final Acceptance and its frozen navigation-only target boundary;
- #215 Final Acceptance and finite upstream lifecycle marker vocabulary;
- `docs/security.md`, `docs/architecture.md` and the experimental broker/plugin authority.

Semantic freshness domains:

- finite marker schema, retention/reservation policy and lifecycle ordering;
- broker transport/failure precedence, finalizer and cleanup behavior;
- SSRF, secret containment, selector authority and no-playback boundaries.

Integration surfaces:

- `experiments/bilibili-browser-probe/` marker, broker and test modules;
- artifact manifest/static-check workflow surfaces.

Task-owned surfaces:

- marker retention/reservation implementation and focused deterministic tests;
- any exact-Candidate artifact inventory changes caused by those files.

Authority/domain → Claim mapping:

- #188/#215 diagnostic boundary → C1,C2;
- broker/plugin security authority → C1,C2,C4;
- hosted artifact/workflow provenance → C3;
- no-playback/no-target rules → C4.

Integration verification jobs: JI1 plus exact-Candidate J3a/J3 artifact checks.

Unrelated-main policy: unrelated changes do not invalidate Task-specific Evidence. Main changes touching the listed semantic authorities or integration surfaces require Coordinator freshness classification; do not silently rerun or declare stale.

If a semantic authority changes, stop and reconcile this contract before implementation. If main advances without semantic conflict, record both the planning/contract base and actual implementation base; do not invalidate dependency-aware Evidence solely because of a descendant merge. Any later target rerun remains a new Coordinator-controlled #188 Attempt.

## Success Criteria

1. The Candidate provides an evidence-supported finite retention/reservation correction that retains required upstream markers under the reproduced noise while preserving navigation/finalizer visibility and explicit unknown fallback.
2. Deterministic regressions cover noisy retention, normal/early closure, timeout/abort or error ordering as applicable, finalizer/late-callback precedence, bounded output and redaction.
3. Exact-Candidate hosted J1/J2/J3a/J3/J4/JI1 jobs pass, and J3a/J3 independently verify an artifact bound to that Candidate.
4. Reports distinguish implementation and hosted verification evidence from any future #188 target result.
5. No tx-node/browser/Bilibili/playback/VNC/CDP/phone/TV/production/#191/#195 activity and no local build/test/package/install occurs in this Task.

## Evidence Contract

Record the real Task/Claim/Attempt, base and Candidate SHAs, PR, workflow/run/job/artifact identities, runner and execution plane, test selectors, implementation result, hosted verification result and limitations. Keep output sanitized and bounded. Do not record secrets, credentials, raw URLs, response bodies, profile paths or raw upstream errors.

## Failure / Blocked Handling

A failed deterministic regression, missing artifact provenance, or boundary/static-check failure is a Task failure requiring correction; it is not a reason to weaken the retention bound or security rules. Missing GitHub-hosted Actions capability, unavailable artifact delivery or a semantic authority conflict is `BLOCKED` until the Coordinator restores the minimum condition. Target/browser/site unavailability is outside this Task because no target execution is authorized. Do not fall back to local build/test/package/install.

## Completion

The Worker posts `[EXECUTION REPORT]` or a genuine `[BLOCKER REPORT]` on Issue #219, transitions it to `status:review` or `status:blocked`, releases ownership and stops. The Worker must not set `status:done` or close the Issue. A later #188 target rerun requires a new Coordinator handoff.
