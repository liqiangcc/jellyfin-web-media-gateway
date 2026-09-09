# Task — [R008-NAV-DIAGNOSTIC] Resolve downstream-close navigation rejection

## Metadata

```text
GitHub Issue: #211
Parent Goal / Research Item: #68 / R008
Task / Research ID: R008-DOWNSTREAM-CLOSE-NAV-CORRECTION
Task kind: implementation
Base commit: 3c50d46 (current main at package materialization)
Candidate commit: produced by this Task
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test
Hard dependencies: #188 Attempt 7 Final Acceptance; #207 Final Acceptance
```

GitHub Actions / Runner is the execution backend and does not claim this Issue. The Issue owns live status, owner, Attempt, branch, PR, verification status and result summary; this file is the stable execution contract.

## Goal

Determine and implement the smallest evidence-supported browser/transport compatibility correction for the accepted downstream-close navigation rejection. Preserve the existing finite diagnostic and security boundaries, and do not claim playback until a later Coordinator-controlled #188 rerun proves a changed target result.

A correction may be made only when repository evidence, hosted regressions or an explicitly reproduced deterministic harness behavior supports it. If the evidence cannot establish a safe correction, preserve the current behavior and add only the narrow diagnostic evidence needed to distinguish the boundary; do not guess a Chromium or network cause.

## Accepted parent evidence

Issue #188 Attempt 7 is Final Accepted and supplies the exact target evidence:

- accepted implementation Candidate `c8ff3f9e1b5f468e2b68d3f274da82eba0b9a76f`, PR #209;
- hosted run `34302593582`;
- artifact `10085488784`, 4,215,814 bytes, digest `sha256:9a074b534a4067e307d1e0108dfa66f6f296e6baeaf52e096ab8b01ee6957cb6`;
- target `tx-node` / `gateway-verify` UID/GID 1001, fresh headless runtime;
- browser launch `2xx/success`, `navigation_start`, broker/TLS success;
- `navigation_promise_result=rejected`, classified as `phase=chromium_navigation`, `reason=navigation_promise_rejected`, `transport_stage=downstream_close`;
- 13 requests, 15,224 response bytes, no media request, no click/play/consumer activity;
- browser, broker, temporary profile, candidate and DNS cleanup complete; existing `source-runtime` untouched.

This evidence proves a bounded rejected navigation promise after successful browser and broker/TLS markers. It does not identify a lower-level Chromium, page, TLS or network cause.

## Accepted implementation anchor

#207 Final Acceptance accepted the finite classifier and hosted artifact above. Reuse the anchor for comparison and do not substitute an older artifact:

```text
Candidate: c8ff3f9e1b5f468e2b68d3f274da82eba0b9a76f
PR: #209
Hosted run: 34302593582
Artifact: 10085488784
Artifact size: 4,215,814 bytes
Artifact digest: sha256:9a074b534a4067e307d1e0108dfa66f6f296e6baeaf52e096ab8b01ee6957cb6
Selector: bilibili:BV14V411W7r5:part-2
```

## Scope

- Inspect the experimental browser probe, plugin-owned navigation descriptor, broker transport lifecycle and deterministic hosted harnesses for an evidence-supported correction.
- Implement the smallest compatibility change that addresses a demonstrated downstream-close failure mode, or add a narrowly bounded diagnostic seam when evidence is insufficient for a correction.
- Preserve and extend finite allowlists only when required by the observed lifecycle; retain explicit unknown fallback.
- Add deterministic regression coverage for the reproduced failure boundary, successful transport precedence, finalizer sealing, cleanup and redaction.
- Update manifest-addressed artifact inventory or hosted static checks only when implementation surfaces require it.
- Run all required build/test/package verification on GitHub-hosted Actions against one exact Candidate.
- Produce a reviewable Candidate and evidence for a later Coordinator-controlled #188 target rerun.

## Out of scope

- Any tx-node, browser, Bilibili, Gateway production or target execution in this Task. A later #188 Attempt is the only target rerun authority.
- Click, play, full preload, independent consumer, media extraction, playback or source portability claims.
- Guessed Chromium/network bypasses, unrestricted retries, proxy rotation, egress relaxation, SSRF exceptions, disabling TLS checks or changing the plugin-owned authority.
- Cookies, Authorization, proxy credentials, Vault/profile access, raw URL/error/header/body/certificate/token output.
- Stopping, attaching to, inspecting or reconfiguring `source-runtime`.
- Phone/TV deployment, VNC/CDP observation, phone/TV/production mutation.
- Processing, claiming, editing or waiting on #191 or #195.
- General browser-slot provisioning, new runner infrastructure, unrelated architecture/security refactors.
- Local build, test, package, dependency installation or compilation.

## Architecture and security invariants

- Gateway remains the `PlaybackSession` authority; this experiment adds no playback state.
- Site-specific authority remains in the Bilibili plugin; Core receives no concrete site branch.
- The broker remains plugin-owned and fail-closed. No open proxy, SSRF relaxation or arbitrary caller-controlled authority is introduced.
- Diagnostic output remains finite, bounded and sanitized. Raw errors, URLs, authorities, headers, cookies, tokens, certificates, profile paths, DOM, body and media data never leave the process.
- Existing source-runtime and production services remain outside this Task's ownership.
- Browser/probe processes remain non-root and use only explicit temporary resources in any future target rerun.

## Worker routing

```text
Worker: cloud-codex
Environment: env:cloud
Requested model: gpt-5.6-luna, reasoning high
Fast: enabled by user; may be selected
Execution backend: GitHub-hosted Actions for all build/test/package verification
```

## Preconditions and evidence boundary

- Re-read live Issue #211 and all relevant comments before claim.
- Re-read #188 Attempt 7 Final Acceptance and #207 Final Acceptance, including exact Candidate, run, artifact, digest and target result.
- Re-read `AGENTS.md`, `docs/README.md`, `docs/requirements.md`, `docs/architecture.md`, `docs/implementation-contracts.md`, `docs/security.md`, `docs/development-environments.md`, `docs/runner-execution-architecture.md`, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, and the relevant browser-probe/runbook documents.
- Confirm Issue #211 is `status:ready`, `env:cloud`, owner-free before claim. This package begins as `status:draft`; the Coordinator owns Publication Gate and readiness.
- Target execution is not required or authorized for this implementation Task. The candidate must be suitable for a later #188 rerun after Coordinator review.

## Implementation requirements

1. Trace every correction to a concrete accepted marker, deterministic harness result or repository invariant. Do not infer an unobserved lower-level cause from `downstream_close` alone.
2. Keep navigation authority plugin-owned and preserve the fixed opaque selector boundary. Do not add caller-supplied URL, host, headers, profile, proxy or credential input.
3. Keep all lifecycle, diagnostic reason, transport stage/outcome and stage event values in closed allowlists with hard bounds and safe unknown fallback.
4. Ensure a late page, broker, promise or process callback cannot replace the first finalized result or overwrite a successful transport observation unless the existing explicit budget/failure rule allows it.
5. Keep cleanup re-entrant and Attempt-owned. Do not broaden cleanup to host-wide process or service termination.
6. Add meaningful deterministic regression coverage for the corrected boundary and no-raw-data behavior. Avoid tests that merely mirror implementation.
7. Keep artifact manifest Candidate identity and hosted static boundaries exact. Do not reuse prior runtime Evidence for changed code.
8. If a focused correction would require changing a canonical architecture/security invariant, stop with Evidence and request Coordinator review instead of implementing the change.

## Claims

- C1: The proposed correction or diagnostic-only fallback is directly supported by accepted evidence or a deterministic reproduction and remains finite/sanitized.
- C2: Navigation, transport precedence, finalizer and cleanup behavior remain safe under the corrected boundary and late callbacks.
- C3: Exact-Candidate GitHub-hosted J1/J2/J3a/J3/J4/JI1 verification passes and produces a manifest-addressed artifact independently consumed from that Candidate.
- C4: Egress/SSRF, no-secret, no-playback, no-target and source-runtime boundaries remain intact.

## Verification job matrix

| Job ID | Claim(s) | Execution plane | Runner / target | Required evidence |
|---|---|---|---|---|
| J1 | C1,C2 | github-actions | github-hosted-x64 / runner-self | probe contract, diagnostic, lifecycle, finalizer and deterministic regression tests |
| J2 | C2,C4 | github-actions | github-hosted-x64 / runner-self | broker containment, transport precedence, cleanup and redaction regressions |
| J3a | C3 | github-actions | github-hosted-x64 / runner-self | exact-Candidate manifest-addressed artifact and digest |
| J3 | C3,C4 | github-actions | github-hosted-x64 / runner-self | download and independent artifact consumer without target/site execution |
| J4 | C1,C4 | github-actions | github-hosted-x64 / runner-self | static authority, egress, secret, no-playback and lifecycle boundary checks |
| JI1 | C2,C3,C4 | github-actions | github-hosted-x64 / runner-self | exact-Candidate integration and regression suite |

Jobs do not claim Issue #211 and do not access tx-node, browser, Bilibili or production.

## Freshness and integration

Freshness policy: dependency-aware.

Semantic authorities:
- #188 Attempt 7 Final Acceptance and its frozen navigation-only contract;
- #207 Final Acceptance and finite rejection-classifier contract;
- `docs/security.md`, `docs/architecture.md` and the experimental broker boundary.

Task-owned semantic surfaces:
- `experiments/bilibili-browser-probe/` diagnostic, lifecycle, broker and artifact surfaces;
- deterministic tests and hosted static checks for those surfaces.

A later target result must be produced by a new Coordinator-controlled #188 Attempt. This Task's hosted artifact and implementation result do not automatically prove target compatibility or playback.

If a live authority changes, stop and reconcile the contract before implementation. If `main` advances without semantic conflict, record both the contract base and actual implementation base; do not invalidate dependency-aware Evidence solely because of the descendant merge.

## Success criteria

1. The Candidate contains a narrowly scoped correction or diagnostic-only fallback supported by concrete evidence, without guessed bypasses or weakened boundaries.
2. Deterministic tests cover the reproduced downstream-close boundary, known/unknown lifecycle behavior, redaction, finalizer ordering and Attempt-owned cleanup.
3. Exact-Candidate hosted J1/J2/J3a/J3/J4/JI1 jobs pass, and J3a/J3 independently verify a manifest-addressed artifact bound to that Candidate.
4. The implementation and verification reports clearly distinguish Candidate behavior from future #188 target proof.
5. No tx-node/browser/Bilibili/production/#188 target action, #191/#195 activity or local build/test/package/install occurs during this Task.

## Completion

The Worker posts `[EXECUTION REPORT]` or a genuine `[BLOCKER REPORT]` on Issue #211, then transitions it to `status:review` or `status:blocked`, releases ownership and stops. The Worker must not set `status:done` or close the Issue.

