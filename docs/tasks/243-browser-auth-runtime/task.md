# Task — Vault-bound authenticated Browser Worker runtime

## Metadata

```text
GitHub Issue: #243
Parent Goal / Research Item: #68 Bilibili Web E2E; R005 authenticated source-site session
Task / Research ID: R005-AUTH-RUNTIME
Task kind: implementation
Base commit: 58e41ed1210071f65d235473a04d40eb89b1aa9e
Candidate commit: n/a until Worker attempt
Session bootstrap prompt: docs/tasks/243-browser-auth-runtime/prompt.md
Preferred worker: cloud
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, cloud-interactive
Hard publication dependencies: accepted #235 feasibility; merged #240 Browser Observation Bridge; Coordinator Publication Gate
```

## Goal

Implement the smallest generic authenticated Browser Worker runtime contract that can materialize a fresh disposable browser profile, accept a Vault-owned opaque attachment reference, expose bounded auth events, and return a server-owned candidate session handoff for the Bilibili Site Plugin. The implementation must preserve Vault, R008, PlaybackSession, Display and Site Plugin boundaries and must not claim real login or playback.

## Why / Context

#235 established that #28 Vault and #33 generic Browser Worker contracts compose, but real profile attach, login-state interpretation and authenticated resolve are missing. #237 added the production Bilibili Site Plugin contract and #240 added generic resource observations/server-owned handoff. Earlier anonymous tx-node runs ended in stable 4xx navigation with no media; anonymous evidence cannot be upgraded to authenticated playback. This Task closes the repository/runtime contract gap so a later authorized target proof has a real boundary to exercise.

## Task Decomposition Decision

```text
Verification mode: separate-task
Linked implementation task: #243 (this Task)
Linked verification task: n/a until implementation is accepted
Decision reason: repository implementation can be verified on hosted x64; legal account use and live target evidence have separate authority and must remain a later Task.
```

## Worker Routing Decision

```text
Worker: Codex Cloud, env:cloud
Verification: GitHub Actions hosted x64; no local build/test/package/install
Target proof: separate future Task only, after explicit authorization gate
```

## Preconditions

- Read `AGENTS.md`, all canonical docs listed there, #243 history, #235 accepted feasibility, #237 accepted plugin contract, #240 accepted Browser Observation Bridge, and `docs/tasks/issue-lifecycle-protocol.md`.
- Preserve the existing `SiteAdapterRegistry`, `BrowserObservationHandoff`, `Session Vault`, `EgressPolicy`, `PlaybackSession`, and `DisplayAdapter` authorities.
- Use a fresh profile materialization abstraction; never attach a personal/copied profile or persist raw account material.
- No #191/#195, phone deployment, TV/VNC/CDP, live Bilibili/login, or local build/test/package/install.

## In Scope

1. Generic Browser Worker auth-mode runtime:
   - fresh disposable profile/session materialization owned by server-side runtime;
   - opaque `ProfileAttachmentRef`/Vault capability handoff with scope, expiry, account/session binding and one-shot use;
   - bounded auth events and state transitions (`required`, `input-needed`, `candidate-ready`, `cancelled`, `expired`, `crashed`) with redacted diagnostics;
   - timeout, cancellation, crash, disconnect, expiry and cleanup paths that never replace a valid session with an unvalidated candidate.
2. Server-side session handoff:
   - plugin-facing candidate session reference and observation/resource handoff only through existing generic APIs;
   - candidate validation and atomic Vault replacement; failed attempts preserve the previous valid session;
   - R008 EgressPolicy applies to every browser request/redirect and auth capability does not widen destination authority.
3. Bilibili plugin integration:
   - plugin interprets generic auth observations to account/session state and opaque authenticated locator semantics;
   - no Bilibili URL/DOM/private API/login-success logic in Core or generic Worker;
   - authenticated `ResolvedMedia` uses existing server-owned handoff and never exposes Cookie/Authorization/profile data.
4. Deterministic fake/runtime/security/architecture tests and a focused hosted x64 Actions workflow. Extend accepted regressions without changing unrelated workflows.

## Out of Scope

- Live account login, CAPTCHA/DRM/paywall/region bypass, token or Cookie injection, personal profile reuse, QR/password/verification-code capture, or raw page/HAR/body retention.
- tx-node, phone, TV, VNC, CDP, Jellyfin deployment, physical UX, or playback success evidence.
- Open proxy, SSRF bypass, arbitrary private-network egress, Core site semantics, or a second Secret owner.
- Changes to #191/#195 or reopening closed diagnostics (#188/#232).
- Local compile/test/install; required verification must run on GitHub Actions.

## Architecture Invariants

- Session Vault is the sole owner of source-site Secret/profile material; workers, plugins, Display and Control receive refs/capabilities only.
- Browser Worker is generic Chromium infrastructure; Bilibili login/session interpretation belongs to the Bilibili Site Plugin.
- Core does not parse Bilibili URLs, DOM, private APIs, cookies, signed URLs or login rules.
- R008 validates every destination and redirect; auth never widens egress.
- Gateway remains PlaybackSession authority; auth retry obeys revision/transition rules and cannot overwrite newer playback state.
- Display Adapter never reads Vault; Native Panel failure cannot stop Web Display.
- Target runtime is low privilege and isolated from production Vault, root/ADB, SSH and Tailscale credentials.

## Files Expected to Change

- `gateway-core/src/browser*.rs` or the generic Browser Worker module;
- `site-adapter-api/src/*` only for generic versioned auth/session handoff types;
- `plugins/bilibili/src/*` only for plugin-owned interpretation/consumer integration;
- focused tests/fixtures and `.github/workflows/issue-243-browser-auth-runtime.yml`;
- no unrelated product or deployment changes.

## Claims

```text
C1: Fresh disposable profile and opaque Vault capability handoff are scoped, expiring, one-shot and secret-safe.
C2: Generic auth lifecycle handles cancellation, timeout, crash/disconnect, expiry and cleanup without corrupting a valid session.
C3: Candidate validation and atomic Vault replacement preserve R007 PlaybackSession authority and R008 egress boundaries.
C4: Bilibili plugin consumes generic auth/session observations while Core/Worker remain site-agnostic; ResolvedMedia and diagnostics contain no secrets.
C5: Hosted x64 deterministic tests and architecture/security guards pass for the exact Candidate SHA.
C6: Live authenticated Bilibili login/playback is BLOCKED/NOT RUN in this implementation Task and requires a later authorized target Task.
```

## Verification Job Matrix

| Job | Claims | Plane | Runner | Required | Evidence |
|---|---|---|---|---|---|
| J1 | C1-C5 | GitHub Actions | hosted x64 | yes | exact Candidate SHA fmt/clippy/unit/contract/security/architecture/workspace logs |
| J2 | C2-C4 | GitHub Actions | hosted x64 | yes | fake/runtime lifecycle, Vault replacement, R008 and plugin consumer tests |
| J3 | C6 | external target | approved future channel | no | separate sanitized target report only after authorization |

Candidate SHA, run/job URLs and artifact digests must be recorded in the Issue execution report. No target job may run from this package.

## Success Criteria

- Candidate PR changes only the generic auth runtime, generic handoff API, Bilibili plugin consumer, focused tests and workflow described above.
- J1/J2 pass on GitHub-hosted x64 for the exact Candidate SHA; no local build/test/install is used as verification.
- Secret/static guards prove no Cookie, Authorization, profile archive, signed URL, raw body or personal account data crosses Core/Display/log/artifact boundaries.
- Worker posts `[EXECUTION REPORT]`, sets Issue `status:review`, and releases ownership; Coordinator must review before merge.
- C6 is explicitly `BLOCKED/NOT RUN`; this Task must never be reported as authenticated playback success.
