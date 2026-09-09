# Task — [R008-NAV-DIAGNOSTIC] Investigate post-upstream navigation rejection before media request

## Metadata

```text
GitHub Issue: #223
Parent Goal / Research Item: #68 / R008
Task / Research ID: R008-POST-UPSTREAM-NAVIGATION-REJECTION
Task kind: combined
Base commit: cd6dfe80e312f45e37c18ca530e0ea9f0cf7ec95
Candidate commit: produced by this Task
Session bootstrap prompt: docs/tasks/223-r008-post-upstream-navigation-rejection/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, cloud-interactive
Hard publication dependencies: #188 Attempt 10 Final Acceptance; #219 Final Acceptance
```

GitHub-hosted Actions are the implementation build/test/package backend. The later bounded tx-node session is target evidence for this Task and does not claim the parent playback goal. The Issue owns live status, owner, Attempt, branch, PR, verification status and result summary; this file is the stable execution contract.

## Goal

Determine, with direct finite evidence, why the accepted browser probe reaches `upstream_response_start`, `upstream_response_end` / `body_complete`, and `upstream_socket_close` / `closed_after_body`, then rejects the Chromium navigation promise with `downstream_close` before any media request. Add only the smallest evidence-supported browser/transport compatibility correction or diagnostic seam needed to distinguish the remaining lifecycle classes. Preserve an explicit `unknown` result when the observed ordering cannot establish a cause.

A successful Task result explains the observed boundary or records a reproducible, bounded unknown class that is actionable for the next Coordinator decision. It does not claim Bilibili source portability or playback success.

## Why / Context

#188 Attempt 10 is Final Accepted. A fresh disposable tx-node session ran the accepted browser probe as `gateway-verify` using Candidate `4ffcb63ef156d042571c39d0fc15a5b4e30a8b54` and produced a finite sanitized result: `chromium_navigation / navigation_promise_rejected / downstream_close`, `transport_outcome=failure`, `lifecycle_outcome=rejected`. The retained marker stream contained `navigation_start`, `upstream_response_start` (`response_started`), `upstream_response_end` (`body_complete`), `upstream_socket_close` (`closed_after_body`), `navigation_promise_result`, `navigation_status`, `navigation_end`, and `finalizer_entry`; it did not contain `browser_disconnect` or `process_termination`. Final counters were 13 requests, 15,224 response bytes and 0 metadata bytes, with no media extraction or playback claim.

#219 corrected marker retention so upstream lifecycle markers survive broker noise. The next narrow question is whether the rejection is caused by a page/navigation lifecycle transition, browser/process termination, downstream tunnel closure, or another allowlisted compatibility class after the upstream body has completed. No cause may be inferred solely from the `downstream_close` label.

## Accepted parent evidence

- Parent Issue: #188 Attempt 10 Final Acceptance.
- Parent accepted implementation Candidate: `4ffcb63ef156d042571c39d0fc15a5b4e30a8b54` (#219 / PR #221).
- Parent hosted Actions run: `34309034915`; required J1/J2/J3a/J3/J4/JI1 passed.
- Parent exact artifact: `10087720720`, 4,218,043 bytes.
- Parent artifact digest: `sha256:a5b9f07eb5ab7d81289539a90574d50a306353d80e4f37f7131e592ab555bcef`.
- Parent target: tx-node, `gateway-verify` UID/GID 1001/1001, one fresh navigation-only session through authenticated Tailscale SSH.
- Parent selector: `bilibili:BV14V411W7r5:part-2`.
- Parent target result: one bounded navigation rejection after upstream body completion/socket close and before media activity; Attempt-owned cleanup passed and existing `source-runtime` was untouched.

## Task Decomposition Decision

```text
Verification mode: inline combined implementation + hosted verification + bounded target evidence
Linked implementation task: n/a; this Task owns the narrow diagnostic seam/correction
Linked verification task: n/a; the target session is bounded and tied to this Candidate
Decision reason: The implementation and deterministic regressions are on the existing probe surface, while the target run is a Coordinator-gated consumer of the exact Candidate artifact. The target evidence has a distinct execution plane but does not require a duplicate business Issue for one or two bounded sessions.
```

## Worker Routing Decision

```text
Worker: cloud-codex
Environment: env:cloud
Requested model: gpt-5.6-luna, reasoning high
Fast: enabled per current user routing; may be selected
Implementation/verification backend: GitHub-hosted Actions
Target control plane: authenticated Tailscale SSH to tx-node, only after the exact-Candidate gate
```

## Preconditions

- The Worker must read `AGENTS.md`, the live Issue #223 and relevant #188/#219 history, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, the canonical documents required by `AGENTS.md`, and the existing browser-probe/diagnostic documentation before claim.
- The Coordinator must complete the Publication Gate before the Worker claims this Issue. The package starts as `status:draft`, `env:cloud`, owner-free.
- #188 Attempt 10 and #219 Final Acceptance are evidence dependencies. Their exact identities above are immutable parent context; a new implementation Candidate and exact hosted artifact are required for this Task.
- All implementation build, test, package and artifact verification runs only on GitHub-hosted Actions. Local compile/test/package/install is forbidden.
- The target run is allowed only after the exact Candidate head, required hosted J1/J2/J3a/J3/J4/JI1 jobs and artifact manifest/digest are read back. Artifact delivery must be integrity-checked before execution.
- Target execution uses only the authenticated Tailscale SSH control plane, a fresh Attempt-owned temporary profile/runtime and `gateway-verify` UID/GID 1001 when practical. Existing `source-runtime` may remain but must not be stopped, attached, inspected for contents, reconfigured, reused or have its profile/display/proxy changed.
- The fixed plugin-owned selector remains `bilibili:BV14V411W7r5:part-2`. No caller-provided URL, authority, proxy, cookie, Authorization header, profile, CDP endpoint or media URL is permitted.

## In Scope

- Minimal evidence-supported changes in `experiments/bilibili-browser-probe/` and its focused deterministic tests that correlate the upstream response/socket boundary with the browser navigation promise, page lifecycle, browser disconnect, process termination and finalizer ordering.
- A finite closed vocabulary for any new diagnostic state needed to distinguish response-complete/downstream-close, page close/crash, browser disconnect, process error/signal, timeout/abort, and unknown. Existing vocabulary should be reused where sufficient.
- Deterministic synthetic regressions for the observed order and the relevant alternative orderings, including upstream completion before downstream close, downstream closure before completion, navigation rejection, page lifecycle events, browser disconnect/process termination, finalizer sealing and late callback precedence.
- Exact-Candidate GitHub-hosted Actions verification and artifact manifest/digest evidence.
- After the hosted gate, at most two fresh tx-node navigation-only sessions to test the exact Candidate against the fixed selector and capture bounded sanitized marker evidence. One session is required first; a second is allowed only if it distinguishes a transient result from a repeatable class or resolves a narrowly defined ambiguity.
- Bounded cleanup/read-back for only Attempt-owned staging, profile, display, browser and probe resources.

## Out of Scope

- Implementing or claiming Gateway playback, source portability, media extraction, independent consumer behavior, click/play/full preload or audible playback.
- Changing #188, #191, #195, #166, #182 or #185; do not process, claim, edit or wait on #191 or #195.
- Any guessed Chromium/network bypass, proxy rotation, TLS verification relaxation, redirect change, SSRF/egress-policy relaxation, open proxy or private-address exception.
- Site secrets, cookies, Authorization, raw URLs/authorities, headers, response bodies, media payloads, certificates, profile contents, raw error strings, tokens, credentials or sensitive paths in code output, logs, artifacts or reports.
- Stopping, attaching to, observing or reconfiguring `source-runtime`; broad host-wide process cleanup; production service mutation.
- VNC/CDP observation, phone or TV deployment, phone/TV playback, local build/test/package/install or dependency installation.
- More than two target sessions, unbounded retries, target execution before the exact hosted artifact gate, or a second Task for the same narrow Claim.

## Architecture and Security Invariants

- Gateway remains the `PlaybackSession` authority; this experiment adds no playback state and no site-specific behavior to Core.
- Bilibili authority remains plugin-owned and the selector remains an opaque fixed input. The caller cannot inject a URL, host, proxy, credential or profile.
- Broker egress remains fail-closed under existing DNS/public-address/TLS/redirect and SSRF policy. This Task may observe bounded lifecycle state but cannot weaken transport authority.
- Diagnostic output is a versioned finite DTO with closed allowlists, bounded counters and `unknown` fallback. No raw callback data escapes.
- Transport/failure precedence and finalizer publication remain at-most-once. Late callbacks cannot overwrite a finalized transport/failure result or append after sealing.
- Browser and probe execution remains disposable, non-root and Attempt-owned. Existing `source-runtime` is outside this Task's authority.

## Files Expected to Change

- Existing experimental probe/broker/lifecycle modules under `experiments/bilibili-browser-probe/`, only where needed for the narrow post-upstream correlation.
- Focused deterministic tests for those modules.
- Artifact manifest/static verification surfaces only if the changed files require existing inventory updates.
- No canonical architecture/security file should change. If one would be required, stop and return the evidence to the Coordinator for design review.

## Implementation Requirements

1. Trace each reported marker and classification to a deterministic event or explicitly bounded callback. Do not label a cause from `downstream_close` alone.
2. Correlate the upstream response/body/socket state with navigation promise settlement and page/browser/process lifecycle ordering using finite allowlisted fields. Preserve `unknown` for missing, contradictory or racing events.
3. Preserve the #211 transport/failure precedence and #219 marker retention policy. A successful transport observation cannot be replaced by a late opaque callback, and a sealed finalizer cannot be reopened.
4. Preserve bounded request, response and metadata counters and current redaction. No raw URL, authority, error text, header, body, profile, cookie, token or credential may enter output.
5. Add meaningful deterministic tests for the observed response-complete/socket-close-then-rejection sequence and at least the relevant page-close/crash, browser-disconnect, process-error/signal, timeout/abort and late-callback orderings. Assert behavior and sanitized output, not only constants.
6. Keep the fixed plugin-owned selector, broker allowlist, SSRF/egress checks, TLS verification and no-playback boundary unchanged.
7. Keep target invocation argv/structured and bounded. Use fresh disposable profile/display/CDP endpoints only when needed, run as `gateway-verify` when practical, and clean only Attempt-owned resources.
8. If the evidence remains insufficient after the correction, publish the finite `unknown` class and the exact missing boundary; do not invent a Chromium cause or add a bypass.
9. If a proposed correction changes a canonical architecture/security invariant or requires source-runtime mutation, stop and return a blocker/design-change request to the Coordinator.

## Claims

- C1: The Candidate deterministically correlates upstream response/body/socket closure with navigation/page/browser/process/finalizer ordering using a finite allowlisted schema and explicit unknown fallback.
- C2: Deterministic regressions preserve transport/failure precedence, marker retention, finalizer sealing, redaction, bounds and Attempt-owned cleanup under the observed and alternative orderings.
- C3: Exact-Candidate GitHub-hosted J1/J2/J3a/J3/J4/JI1 verification passes and the artifact manifest/digest binds the runnable bundle to the Candidate.
- C4: At least one fresh bounded tx-node session produces sanitized evidence for the post-upstream boundary; a second session, if used, either reproduces the class or resolves a stated ambiguity. A concrete failure or explicit unknown is valid evidence; unsupported causation is not.
- C5: SSRF/egress, plugin authority, no-secret, no-playback, no-target-production and source-runtime boundaries remain intact.

## Verification Plan

### Claims

```text
C1: finite post-upstream lifecycle correlation and safe unknown classification
C2: deterministic ordering, precedence, retention, redaction and cleanup safety
C3: exact-Candidate hosted verification and artifact provenance
C4: fresh bounded target navigation evidence
C5: security and no-playback/source-runtime boundaries
```

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1,C2 | github-actions | github-hosted-x64 | runner-self | yes | probe contract and deterministic post-upstream lifecycle tests | run/job logs and sanitized test result |
| J2 | C2,C5 | github-actions | github-hosted-x64 | runner-self | yes | broker closure, transport precedence, redaction, finalizer and cleanup regressions | run/job logs and boundary assertions |
| J3a | C3 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate artifact build/manifest/digest gate | artifact ID, manifest, size and digest |
| J3 | C3,C5 | github-actions | github-hosted-x64 | runner-self | yes | independent exact-Candidate artifact consumer and static contract checks | consumer logs and artifact read-back |
| J4 | C1,C5 | github-actions | github-hosted-x64 | runner-self | yes | static allowlist, selector authority, SSRF/secret/no-playback/no-target checks | static job logs |
| JI1 | C2,C3,C5 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate integration and regression suite | integration job logs |

All required build/test/package commands run only on GitHub-hosted Actions. No Actions job accesses tx-node, Bilibili, production, phone, TV, VNC or CDP.

### Target Plan

After J1/J2/J3a/J3/J4/JI1 and artifact manifest/digest read-back pass:

1. Use one authenticated Tailscale SSH control-plane connection to tx-node with an explicit finite timeout. This is control-plane transport, never a media proxy.
2. Recheck only permitted admission facts needed for this Task: target OS/architecture, Chrome/Node availability, `gateway-verify` identity, proxy-unset execution environment and Attempt-owned staging. Do not inspect `source-runtime` contents or mutate its services/profile/display/proxy.
3. Stage and verify only the exact Candidate artifact in a fresh mode-700 Attempt-owned directory. Create a fresh temporary browser profile and separate display/debug endpoint only when needed; never reuse a profile, display, CDP endpoint or proxy.
4. Run one navigation-only probe invocation with fixed selector `bilibili:BV14V411W7r5:part-2` as `gateway-verify` when practical. Capture only finite marker names/classes, bounded counters, termination outcome and cleanup facts. Do not click, play, preload, run a consumer or extract media.
5. Run a second session only if the first result is transient-looking or leaves a specific ordering ambiguity that the second bounded session can resolve. Do not retry an unclassified failure without a stated evidence purpose.
6. Remove only Attempt-owned staging/profile/display/process resources and read back that they are absent. Leave `source-runtime` and production resources untouched.

Target execution plane is `external-codex` over authenticated Tailscale SSH; target is tx-node VM-0-11-ubuntu, low-privilege `gateway-verify` UID/GID 1001/1001. Target evidence is separate from hosted verification evidence and must be reported as such.

## Freshness / Integration Contract

Freshness policy: dependency-aware.

Semantic authorities:

- #188 Attempt 10 Final Acceptance and its frozen navigation-only target boundary;
- #219 Final Acceptance and finite marker-retention behavior;
- #211 transport/failure precedence;
- `docs/architecture.md`, `docs/security.md` and the existing plugin-owned broker/probe contract.

Semantic freshness domains:

- post-upstream marker correlation and closed diagnostic vocabulary;
- transport/failure precedence, retention, finalizer and cleanup behavior;
- SSRF/egress, selector authority, secret containment and no-playback boundaries.

Integration surfaces:

- `experiments/bilibili-browser-probe/` lifecycle, broker and marker modules;
- deterministic probe tests;
- artifact manifest/static-check workflow surfaces.

Task-owned surfaces are limited to the narrow diagnostic correlation and its focused tests. Unrelated main changes do not invalidate this Task's evidence. Changes touching the listed semantic authorities require Coordinator freshness classification before implementation. Record both the contract base `cd6dfe80e312f45e37c18ca530e0ea9f0cf7ec95` and the actual implementation base; do not silently rebase away accepted parent evidence.

Any target rerun is a new evidence event for #223, not a modification of #188. If a semantic conflict or canonical design change appears, stop and return to the Coordinator.

## Success Criteria

1. The Candidate adds an evidence-supported finite correlation of upstream completion/socket closure and post-navigation lifecycle, with explicit unknown fallback and no raw-data leakage.
2. Deterministic tests cover the observed sequence and relevant closure, lifecycle, timeout/abort, error, late-callback, finalizer and cleanup orderings within existing bounds.
3. Exact-Candidate hosted J1/J2/J3a/J3/J4/JI1 jobs pass, and J3a/J3 independently verify an artifact bound to that Candidate.
4. One fresh bounded tx-node navigation session produces a sanitized, reproducible or explicitly unknown result tied to the exact Candidate; a second is used only for a stated evidence purpose.
5. No playback, media extraction, consumer, source-runtime mutation, proxy/SSRF/secret boundary change, VNC/CDP observation, phone/TV/production action or local build/test/package/install occurs.
6. The report clearly separates implementation result, hosted verification result, target evidence and Coordinator decision; no unsupported Chromium/network cause is claimed.

## Evidence Contract

Record the real Task/Claim/Attempt, contract and implementation base, Candidate SHA, PR, hosted workflow/run/job/artifact identities, manifest size/digest, test selectors, execution plane, runner/target facts, target session count, bounded marker/result summary, cleanup read-back, limitations and result status. Keep all output finite and sanitized. Do not record secrets, credentials, cookies, Authorization, raw URLs/authorities, response bodies, headers, profile paths, raw error text, tokens, certificates or media data.

Separate:

```text
Implementation Result
Verification Result (hosted exact-Candidate jobs)
Target Evidence Result (fresh tx-node session)
Coordinator Review Decision
Parent Goal / Research Gate decision
```

## Failure / Blocked Handling

- A deterministic regression, static boundary check or exact-artifact mismatch is a Task failure requiring a focused correction; do not weaken bounds or security rules.
- Missing Actions, missing artifact, failed artifact delivery, semantic conflict or unavailable required repository capability is `BLOCKED`; do not fall back to local build/test/package/install.
- Target admission/runtime unavailability is `BLOCKED` only when the exact artifact or disposable navigation runtime genuinely cannot execute. A concrete navigation rejection, repeatable downstream-close result or explicit unknown is valid target evidence and must not be mislabeled `BLOCKED`.
- If a second session cannot distinguish the first result, stop after the permitted sessions and report the remaining unknown without guessing.
- If target cleanup cannot be verified, report the cleanup claim as `BLOCKED` and preserve the boundary; do not kill unrelated processes or alter `source-runtime`.

## Deliverables

- Focused implementation/diagnostic changes and deterministic tests under the existing experimental probe surface.
- Exact Candidate commit and PR.
- GitHub-hosted J1/J2/J3a/J3/J4/JI1 run/job evidence and exact artifact manifest/size/digest.
- One bounded tx-node navigation evidence set, with second session only if justified.
- Sanitized `[EXECUTION REPORT]` or genuine `[BLOCKER REPORT]` on Issue #223.

## Issue Feedback / Iteration Protocol

Follow `docs/tasks/issue-lifecycle-protocol.md`:

```text
status:draft
→ Publication Gate
→ status:ready
→ Worker claim / Attempt N
→ status:in-progress
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ status:review or status:blocked
→ release owner and stop
```

Worker must not set `status:done` or close the Issue. A later #188 rerun, playback implementation or parent Goal decision requires Coordinator review and a separate handoff.

## Completion

The Worker posts `[EXECUTION REPORT]` after implementation/hosted/target work or a genuine `[BLOCKER REPORT]` if blocked, transitions #223 to `status:review` or `status:blocked`, releases ownership and stops. Only the Coordinator can review, accept, revise, split, mark done or close.
