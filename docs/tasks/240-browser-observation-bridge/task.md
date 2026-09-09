# Task Contract — R005 Browser Observation Bridge

- Issue: #240
- Parent: #68 public Web playback
- Prerequisite: #237 Final Acceptance, main `749a3233f1d776843971a08e558e17b8589c92aa`
- Task kind: `implementation`
- Preferred worker: Codex Cloud (`env:cloud`), `gpt-5.6-luna`, reasoning high, Fast enabled

## Goal

Connect the existing generic Chromium Browser Worker to the observation contract accepted in #237. Produce a bounded, redacted `BrowserObservation` and server-owned media handoff that the Bilibili SiteAdapter can consume, while keeping Chromium and Gateway Core unaware of Bilibili URL/DOM/private API semantics.

This Task establishes the real runtime bridge only. It does not claim live Bilibili reachability or playback.

## Scope

1. Extend the generic Browser Worker contract and Chromium implementation to capture bounded network/resource facts needed to form `BrowserObservation` candidates (protocol, media kind, finite status/range/expiry hints, egress decision and opaque server-owned reference). Worker events must contain no site identifiers, selectors, cookies, Authorization values, signed URLs or raw response bodies.
2. Add an explicit server-owned handoff boundary from the worker/runtime to the plugin-facing API. Preserve R008 EgressPolicy for every request, including redirects/service workers/WebSocket or equivalent paths covered by the implementation; do not create an open proxy or arbitrary host authority.
3. Add orchestration that binds an observation to the requested opaque `SourceLocator` without Core parsing Bilibili identifiers. The Bilibili plugin remains the only owner of page URL/part interpretation and candidate selection.
4. Ensure browser/profile/session lifecycle handles cancellation, timeout, crash, expiry and cleanup without leaking profile paths, credentials, signed URLs or unbounded resource data. Native Site Panel failures must not interrupt an already accepted PlaybackSession.
5. Add deterministic fake-worker and Chromium-boundary tests for event schema/version, budgets, stale observations, candidate/handoff binding, secret redaction, egress denial and cleanup. Keep existing Browser Worker, SourceSession, Playback and security regressions passing.
6. Add a GitHub Actions workflow/job that asserts exact Candidate SHA and runs all required x64 format/clippy/unit/integration/security/architecture checks. Do not run local compile/test/package/install.

## Out of scope

- Real Bilibili/login/tx-node requests, authenticated accounts, media extraction from a live site, playback click/consumer, phone/TV/VNC/CDP observation or deployment.
- Site-specific code in `gateway-core/src/browser*.rs`, open CONNECT/proxy behavior, SSRF bypass, Cookie/profile/token smuggling, DRM/CAPTCHA/access-control bypass, #191/#195.
- Replacing PlaybackSession authority, changing Display contracts, or introducing a second Secret owner.

## Claims / success criteria

- C1 PASS: generic Browser Worker emits versioned bounded observations with no site/secret data; fake and Chromium contract tests pass.
- C2 PASS: server-owned candidate handoff is bound to operation/session/locator context, finite and stale-safe; no raw URL/credential reaches browser events, Control, Display or logs.
- C3 PASS: all Browser Worker egress remains governed by R008, including denial and redirect/error paths covered by tests.
- C4 PASS: cancellation/timeout/crash/profile cleanup and Native Panel failure isolation are deterministic; prior Playback authority remains intact.
- C5 PASS: #237 Bilibili adapter consumes only the generic observation/handoff and existing SourceSession/Playback/security suites remain green.
- C6 BLOCKED/NOT RUN: live Bilibili reachability and actual Web Display playback require a later separately published target verification Task.

## Verification matrix

All required build/test/format/clippy jobs run on GitHub-hosted x64 Actions using the exact Candidate SHA. Include affected `site-adapter-api`, `gateway-core`, `plugins/bilibili` tests, Browser Worker tests, architecture/secret guards and existing concurrency/security regressions. No target job is included. Evidence must record exact run/job IDs and separate implementation, verification and coordinator decisions.

## Freshness / lifecycle

Use dependency-aware freshness and preserve the same Issue/PR for revisions. Worker claims only after `status:ready`, posts one `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, sets `status:review`/`blocked`, releases ownership and stops. Worker cannot close the Issue.
