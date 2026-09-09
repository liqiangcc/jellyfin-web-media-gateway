# Task — Gateway authenticated Browser Worker playback seam

## Metadata

```text
GitHub Issue: #248
Parent Goal / Research Item: #68 Bilibili Web E2E; R005 authenticated browser playback
Task / Research ID: R005-GATEWAY-SEAM
Task kind: implementation
Base commit: 60c9eb5c560ba12751f890b5c22af1fa42b9e5f4
Candidate commit: n/a until Worker attempt
Session bootstrap prompt: docs/tasks/248-gateway-auth-playback-seam/prompt.md
Preferred worker: cloud
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, cloud-interactive
Hard publication dependencies: accepted #240 and #243; Coordinator Publication Gate
```

## Goal

Wire the accepted generic `BrowserAuthRuntime`, Bilibili Site Plugin handoff and `SourceSessionService` into a production Gateway orchestration seam so a later authorized target can start browser auth, drive bounded generic browser operations, submit a server-owned authenticated candidate/observation, and create the normal `PlaybackSession`/Web Display session. The HTTP/control surface must never receive or return cookies, Authorization, profile paths, signed URLs, raw bodies or site-private semantics.

## Why / Context

#240 and #243 establish generic observation, Vault-bound profile capability and atomic candidate-session replacement, but `GatewayService.router()` still exposes only context-free `POST /api/v1/sessions`. The accepted Bilibili plugin can resolve only with a server-owned `ResolveContext`, and no production route currently composes that context with SourceSession/Playback/Web Display. This seam is required before #246 can perform a real target proof.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: #248
Linked verification task: #246 (future, separate target; remains blocked)
Decision reason: repository API/orchestration and deterministic tests share one Candidate; live target evidence has independent authorization/runner lifecycle.
```

## Preconditions

- Read `AGENTS.md`, all canonical docs listed there, #248 history, and accepted #240/#243/#237 contracts.
- Preserve Gateway PlaybackSession authority, SiteAdapterRegistry routing, SourceLocator opacity, Vault ownership, Browser Worker generic boundary, Display isolation and R008 EgressPolicy.
- Use existing `BrowserAuthRuntime`/`BrowserObservationHandoff`/`ResolveContext` APIs; do not invent a second auth/session store.
- No live login/playback, no target/phone/TV/VNC/CDP, no #191/#195, and no local build/test/package/install.

## In Scope

1. A server-owned Gateway orchestration component that owns a configured generic Browser Worker, Session Vault and registry without leaking them through HTTP. It may use a test/fake worker in deterministic tests and a deployment-injected Chromium worker in production composition.
2. Bounded structured HTTP/API or equivalent service seam for:
   - start/inspect/cancel an auth attempt bound to site/account and expiry;
   - submit only generic browser navigation/input/control operations already authorized by R008;
   - read redacted auth/browser events by bounded sequence;
   - accept a plugin-produced opaque candidate handoff and server-owned media observation exactly once;
   - create the normal SourceSession/PlaybackSession/Web Display response using `ResolveContext`.
3. Candidate/session binding and race safety: request idempotency, session/attempt identity, expiry/cancellation, stale handoff rejection, no duplicate publication, and no overwrite of newer PlaybackItem/revision/display generation.
4. Error/status mapping that is deterministic and secret-safe; Native Panel/display failures must not prevent Web Display publication when the normal display contract permits it.
5. Deterministic fake/HTTP/security/architecture tests and a focused GitHub Actions workflow. Test fixtures may use server-owned opaque refs and synthetic public media, never real credentials or signed URLs.
6. Documentation/comments for target worker invocation and later #246 handoff, without changing #246's blocked authorization gate.

## Out of Scope

- Real Bilibili login, account use, CAPTCHA/DRM/paywall/region bypass, media extraction from a live site or playback claim.
- Cookie/Authorization/profile/token injection, open proxy, SSRF bypass, raw VNC/CDP, copied profiles, phone/TV deployment or tx-node execution.
- New site-specific knowledge in Core/HTTP handlers; Bilibili semantics stay in `plugins/bilibili`.
- Reopening/processing #191/#195 or closed navigation diagnostics; no local compile/test/install.

## Architecture Invariants

- Gateway is PlaybackSession authority; SourceSession prepares and publishes through existing Control/Display services.
- Browser Worker is generic Chromium/Auth infrastructure; plugin interprets Bilibili auth/media facts.
- Session Vault is sole Secret owner. HTTP clients, Control, Display, logs and artifacts get opaque IDs/capabilities only.
- R008 EgressPolicy validates every browser request/redirect and resolved media destination.
- `SourceLocator` remains opaque/versioned; Core does not parse Bilibili URL/DOM/private API/login rules.
- Display Adapter never reads Vault; Web Display remains usable if Native Panel fails.

## Files Expected to Change

- `gateway-core/src/lib.rs`, `gateway-core/src/browser_auth.rs`, `gateway-core/src/source_session.rs` or focused new orchestration module;
- `gateway-core/src/bin/*` only if a deployment composition seam is required;
- focused HTTP/fake/security tests and `.github/workflows/issue-248-gateway-auth-playback-seam.yml`;
- no unrelated plugin/site or target infrastructure changes.

## Claims

```text
C1: Gateway can start and manage a bounded generic auth attempt using the accepted BrowserAuthRuntime without exposing Vault/profile secrets.
C2: A plugin-owned authenticated candidate plus server-owned observation can be consumed exactly once through ResolveContext and normal SourceSession/Playback/Web Display publication.
C3: Request/attempt/session/display/revision races fail closed; stale or duplicate handoffs cannot overwrite newer authority.
C4: HTTP status/error/event projections are bounded and redacted; R008, Vault, SourceLocator, Native Panel and Web Display invariants remain intact.
C5: Exact Candidate hosted x64 Actions pass deterministic API, lifecycle, security, architecture and workspace regressions.
C6: Live Bilibili login/playback remains BLOCKED/NOT RUN and belongs only to separately authorized #246.
```

## Verification Job Matrix

| Job | Claims | Plane | Runner | Required |
|---|---|---|---|---|
| J1 | C1-C5 | GitHub Actions | hosted x64 | yes |
| J2 | C2-C4 | GitHub Actions | hosted x64 | yes |
| J3 | C6 | target | tx-node via #246 | no |

Record exact Candidate SHA and run/job IDs. No local build/test/install is verification evidence. Do not run target jobs from #248.

## Success Criteria

- One focused Candidate/PR adds the Gateway seam and tests without weakening any invariant.
- J1/J2 pass on GitHub-hosted x64 for exact Candidate SHA.
- Worker posts `[EXECUTION REPORT]`, sets `status:review`, releases ownership and stops; Coordinator reviews/merges/accepts.
- C6 is explicitly `BLOCKED/NOT RUN`; no playback success is claimed by this Task.
