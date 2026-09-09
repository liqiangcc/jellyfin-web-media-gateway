# Task Contract — R005 Generic Browser Auth Experience

- Issue: #259
- Parent Goal / Research Item: #68 Bilibili Web E2E
- Related accepted implementation: #243 Browser Auth Runtime, #251 Gateway auth route, #255 Gateway runtime bootstrap, #257 Vault-bound candidate capture
- Task kind: implementation
- Base: `2e9d8246703681b5e57d29205f4c9d55e318f4e4`
- Preferred worker: Codex Cloud (`env:cloud`), `gpt-5.6-luna`, reasoning high
- Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test
- Hard publication dependencies: #255 and #257 accepted; #246 remains blocked and is not a prerequisite for this repository Task
- Hard boundary: no live login/target/tx-node/browser/VNC/CDP/phone/TV/deployment action; no local build/test/install; no #191/#195

## Goal

Provide a generic, server-owned authentication experience that lets a same-origin Control client start a Browser Worker auth attempt, observe only bounded/redacted lifecycle state, send validated generic input through a short-lived capability, request the internal Vault-bound candidate capture from #257, and invoke the existing authenticated playback handoff. Site Plugin interpretation remains site-owned; no Bilibili DOM, private API, credential, profile, page-body, or raw browser transport crosses the public surface.

This Task closes the repository interaction contract only. It does not claim live Bilibili login, target execution, media resolution, Web Display playback, or #246 publication.

## In Scope

1. Add or complete a generic same-origin Control auth surface that starts an attempt for a registered `site_id/account_ref`, exposes redacted state/events, and drives the existing auth attempt lifecycle.
2. Add bounded input/view/panel capability handling with explicit Host/Origin/CSRF checks, short TTL, request idempotency, stale operation handling, and attempt ownership checks.
3. Connect the Control flow to the server-internal #257 candidate capture seam and the existing authenticated playback route. HTTP DTOs contain only opaque IDs, bounded generic observations and status/error codes.
4. Preserve failure isolation: panel/view/input failure or auth cancellation cannot stop an already accepted PlaybackSession; expired/cancelled attempts cannot capture or swap a candidate.
5. Add deterministic fake-worker and HTTP tests for same-origin policy, duplicate/mismatch requests, stale operations, capability expiry, bounded input, redacted events, candidate capture/playback handoff and failure isolation.
6. Add focused GitHub Actions verification for the exact Candidate SHA, including fmt/clippy/tests and architecture/secret-boundary guards.

## Out of Scope

- Bilibili selectors, login-success rules, QR/CAPTCHA/password handling, private APIs or site-specific UI.
- VNC, raw CDP, open proxy, public browser debugging, clipboard/file upload, audio capture, profile download, page bodies, Cookie/Authorization, localStorage or profile paths.
- Playback authority changes, Display authority changes, EgressPolicy exceptions, DASH/remux/HLS implementation, phone/TV/Jellyfin deployment, tx-node execution or live account use.
- Changes to #68/#246 publication state and #191/#195.

## Architecture Invariants

- Gateway remains PlaybackSession authority; Control is a projection/command surface.
- Browser Worker is generic; Site Plugin interprets site semantics and never reads Vault.
- Session Vault remains the sole owner of persistent Secret/profile material; public routes expose only opaque references and redacted status.
- Same-origin Host/Origin/CSRF and SSRF/egress boundaries remain fail-closed.
- Native panel/auth UX failure cannot stop an already accepted PlaybackSession.
- Target runners and the browser runtime do not inherit Vault, production Secret, SSH, Tailscale, root or ADB privileges.

## Files Expected to Change

- `gateway-core/src/auth_route.rs`, `gateway-core/src/browser_auth.rs`, and generic Control/view modules as required.
- Focused HTTP/auth contract tests and architecture/security guards.
- `.github/workflows/issue-259-generic-browser-auth-experience.yml` or equivalent focused workflow.
- Do not add site-specific branches to Core or public Secret/profile DTOs.

## Implementation Requirements

1. Reuse #255 runtime composition and #257 capture seam; do not create a second auth or Vault state store.
2. Enforce same-origin Host/Origin/CSRF checks on every mutating JSON route and bound every request, event, input and capability field.
3. Preserve request idempotency and stale-result protections; old async events/results must not overwrite a newer attempt, candidate, PlaybackItem, display generation or authenticated handoff.
4. Keep all Secret/profile material crate-private/server-internal and redacted from Debug, errors, events, logs and artifacts.
5. Verify only through GitHub Actions; record exact Candidate SHA and actual run/job URLs.

## Verification Plan

### Claims

- C1: A same-origin Control client can start and drive a generic auth attempt using bounded opaque IDs/status/input, with no site-specific semantics.
- C2: Host/Origin/CSRF, capability TTL, request idempotency, stale operation and input bounds fail closed.
- C3: Candidate capture and authenticated playback use the #257/#248 server-owned handoff without exposing Secret/profile material.
- C4: Auth panel/input failures, cancellation and expiry preserve existing PlaybackSession and active session state.
- C5: Exact-Candidate hosted x64 fmt/clippy/tests and architecture/security guards pass.
- C6: Real account login, target evidence and playback remain BLOCKED/NOT RUN and belong to #246/#68.

### Verification Job Matrix

| Job | Claims | Execution plane | Runner | Required |
|---|---|---|---|---|
| J1 | C1-C5 | GitHub Actions | hosted x64 | yes |
| J2 | C2-C4 | GitHub Actions | hosted x64 | yes |
| J3 | C5 | GitHub Actions artifact/log guards | hosted x64 | yes |
| J4 | C6 | target | none | no; must remain NOT RUN |

Every required job must assert the exact Candidate SHA and record actual run/job URLs. Separate Implementation Result, Verification Result and Coordinator Decision. Unrelated aggregate workflow failures are not evidence for this Task.

## Freshness / Integration

Use dependency-aware freshness. Worker must report the exact current-main base and Candidate. If auth route, browser auth or Control files overlap with a later accepted change, reuse this Issue/PR and record the integration result; do not create a duplicate Task.

## Worker Completion

Worker claims only after Publication Gate sets `status:ready`; then posts `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, sets `status:review`, releases ownership and stops. Worker must not merge or close the Issue. Coordinator alone performs Review and Final Acceptance.