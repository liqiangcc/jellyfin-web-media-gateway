# Task Contract — BILIBILI-WEB-E2E (Public / No-Login Contract Revision)

> **Contract Revision (2026-09-11):** This package replaces the stale generic-ytdlp-only route. The first product attempt is a public, non-DRM Bilibili video with no account or login. It composes the accepted production Bilibili Site Plugin, generic Browser Worker observation, server-owned media contracts, PlaybackSession, and the generic Web Display path. This revision is a contract/package change only; it does not authorize a live attempt or claim playback.

## Metadata

```text
GitHub Issue: #68
Task ID: BILIBILI-WEB-E2E
Task kind: combined (implementation seam + real-source functional verification)
Planning Base: d50de827e6b444e9476659b7165a8bbcc0b387b4
Candidate: n/a until a later Publication Gate
Preferred worker: Codex Cloud, env:cloud
Required capabilities after publication: github-read-write, repository-static-analysis,
  code-authoring, automated-build, automated-test, cloud-interactive,
  an explicitly approved ordinary-Linux browser/control route
Current package state: status:draft, owner-free
```

The revision is based on the accepted architecture and implementation chain through the production Bilibili Site Plugin (#237), generic Browser Observation Bridge (#240), generic Browser Worker/auth foundations (#243/#248/#251/#255/#257/#259), MediaShapeV1 (#268), and policy-bound media delivery (#271). #246 is a separate authorized-account target Task and remains independently `status:blocked`; it is not a dependency or alternate route for this public scope.

## Goal

Close one public, no-login product journey for the frozen sample `BV14V411W7r5`, without weakening any Site Plugin, EgressPolicy, Secret, Browser Worker, MediaShape, or PlaybackSession authority:

```text
Control submits the frozen public Bilibili URL
→ SiteAdapterRegistry recognizes a Bilibili SourceLocator
→ production Bilibili Site Plugin requests generic Browser Worker observation
→ Browser Worker performs bounded EgressPolicy-governed public-web work
→ plugin interprets bounded observation into server-owned ResolvedMedia / MediaShapeV1
→ SourceSession prepares and publishes one PlaybackSession / PlaybackItem
→ direct muxed/HLS delivery, or generic #271 remux delivery for separated A/V
→ Gateway same-origin media capability reaches Web Display
→ bounded play / pause / seek / stop / refresh / reconnect
```

The Browser Worker is generic infrastructure. Bilibili URL, page, media-selection and observation interpretation remain in `plugins/bilibili`; no Bilibili knowledge is added to Core or the generic worker. The Gateway remains the only `PlaybackSession` authority.

## Frozen public source scope

Unless the Coordinator performs a later documented Contract Revision at Publication Gate:

```text
site: Bilibili
mode: public / no-login / non-DRM
selector: BV14V411W7r5
account: none
Auth Mode: none
```

The sample may be replaced only by a live contract/evidence finding that makes it unusable. Such a replacement requires a Coordinator-recorded revision; a Worker must not silently substitute a different video. Full page, resolved, signed, Cookie-bearing or upstream URLs never enter durable Evidence.

## Contract route and authority boundaries

### Source and observation

1. Control submits only the public source input through the existing session API. It cannot submit `ResolvedMedia`, `SourceLocator`, an upstream URL, a header, a media generation, or an Egress decision.
2. `SiteAdapterRegistry` recognizes the input and routes it to the production Bilibili Site Plugin. Generic yt-dlp is not a Core fallback and is not the sole route of this revision.
3. The plugin owns Bilibili source semantics and requests a generic, bounded Browser Worker operation. The worker emits only versioned, size-limited, redacted observations and server-owned handoff references.
4. Every browser request and redirect is admitted by the central `EgressPolicy`. The public route cannot use an open proxy, private-network exception, arbitrary caller authority, fingerprint bypass, CAPTCHA bypass, DRM bypass, paywall bypass or region/access-control bypass.
5. The plugin interprets the generic observation and produces a server-owned `ResolvedMedia`/`MediaShapeV1`. Public headers remain free of Cookie, Authorization, bearer, profile and other Secret material.

### Playback and delivery

- `SourceSession` validates and prepares the plugin result; it owns no second playback state.
- `PlaybackSession` and `PlaybackItem` remain the sole playback authority, with session/item/revision/display-generation checks and stale-result rejection.
- Direct muxed HTTP-file or HLS uses the existing media path. A validated separated audio/video `MediaShapeV1` uses the generic policy-bound #271 delivery/remux path; this task does not add Bilibili-specific remux logic.
- Web Display receives only a short-lived same-origin Gateway media capability bound to the current session, item, revision, media generation and resource. It never receives upstream URLs, upstream headers, Browser Worker state, Vault material or profile data.
- Control commands retain existing R007 request-id/CAS semantics. Refresh and reconnect rebuild from Gateway authority; callbacks from old leases, sessions, items or generations cannot overwrite current state.

## In scope

1. Minimal production composition needed for the route above, while keeping site semantics in the Bilibili plugin and generic runtime boundaries unchanged.
2. Public/no-login observation and media handoff through existing server-owned APIs.
3. Browser playback viability for the accepted media shape, followed by bounded play, pause, seek and stop.
4. Control and Web Display refresh/reconnect on the same `PlaybackSession`.
5. Failure, stale-authority, cleanup, Egress and Secret-boundary evidence for the product path.
6. A future exact-Candidate runbook that another Worker can execute without raw media injection, direct store mutation, ad-hoc extractor CLI use or the old chat.

## Out of scope and hard prohibitions

- Any account, test account, login, QR/password/verification-code flow, Auth Mode, Cookie, Authorization, bearer token, profile, Vault session or token reuse/injection.
- CAPTCHA, DRM, paywall, region/access-control, fingerprint, proxy, TLS, SSRF or Egress bypass.
- Phone/TV deployment, VNC, CDP, physical autoplay/audibility, Jellyfin acceptance, #191 or #195.
- Live Bilibili requests, tx-node access, target browser activity, or target traffic from this docs revision. This package remains `status:draft` and does not itself authorize a live attempt.
- Changes to #246. #246 stays `status:blocked`, owner-free, and retains its separate written-authorization/disposable-account/approved-channel gate.
- Generic Browser Worker site knowledge, Bilibili branches in Core, a second Secret owner, open proxy behavior, or weakening R008/Egress/SSRF/Vault/PlaybackSession/#271 authority.
- Relabelling old diagnostics as success or repeating an old navigation probe without new product-path evidence.
- Local/tx-node build, test, package or install. Required build/test/package evidence is GitHub-hosted Actions; an approved ordinary-Linux host may only run a later verified artifact and bounded live route after Publication Gate.

## Historical evidence and failure interpretation

The following records remain append-only historical evidence: #67, #166, #188, #223, #226, #229 and #232. Their anonymous/diagnostic paths must not be reclassified as product playback. In particular, the old anonymous browser route stabilized at an upstream/navigation `4xx` with no media request. A new public product-path attempt that reaches the same boundary is an explicit `FAIL` or `BLOCKED` result with sanitized evidence; it is not permission to bypass Egress, TLS, access controls or to retry indefinitely.

A later Coordinator may revise the source contract only when new evidence establishes a different product-relevant condition. The revision must identify the exact Candidate, execution plane, failure phase and affected claim.

## Claims for a future execution

```text
P1 — Public source authority: the frozen input enters through the production
     Bilibili Site Plugin and existing session API; no raw media/URL injection.
P2 — Generic observation boundary: Browser Worker facts are generic, bounded,
     redacted and EgressPolicy-governed; Bilibili interpretation stays in the plugin.
P3 — Media contract: the plugin result is server-owned ResolvedMedia/MediaShapeV1;
     direct muxed/HLS and separated A/V through generic #271 are handled without
     leaking upstream URL/header/Secret material.
P4 — Playback authority: one PlaybackSession/PlaybackItem drives same-origin
     Web Display media and preserves R007 revision/generation semantics.
P5 — User controls: play/pause/seek/stop and Control/Display refresh/reconnect
     operate on that same authority with stale and duplicate commands rejected.
P6 — Security and cleanup: Egress/SSRF, capability binding, cancellation, expiry,
     bounded resources, cleanup and redacted evidence remain fail-closed.
P7 — Public-only scope: no account, Auth Mode, Cookie, profile, token, DRM,
     CAPTCHA, paywall, region or access-control bypass is used or claimed.
P8 — Historical honesty: old anonymous 4xx/no-media diagnostics remain negative;
     the new product path is reported FAIL/BLOCKED if it meets that boundary.
```

## Publication Gate (must remain unsatisfied in this revision PR)

`#68` stays `status:draft`, `env:cloud`, owner-free until the Coordinator independently reads back and records all of the following. A Worker cannot claim an Attempt before the gate is complete.

1. **Package and dependency read-back:** live Issue, this `task.md`, `prompt.md`, canonical docs and accepted authorities are mutually consistent. #237/#240/#243/#248/#251/#255/#257/#259, #268 and #271 are checked against current `main`; #246 remains separate and blocked.
2. **Exact implementation Candidate:** freeze one full Candidate SHA/branch/PR and classify movement from the current base as `NONE`, `UNRELATED`, `INTEGRATION_OVERLAP`, `SEMANTIC_AUTHORITY` or `CONTRACT_INVALIDATING`. Moving `main` is not an execution identity.
3. **GitHub-hosted freshness:** run the required hosted x64 fmt/clippy/workspace/security/regression jobs against that exact Candidate and read back every required job/artifact. No local compilation or target compilation substitutes for this evidence.
4. **Approved ordinary-Linux execution host/control route:** record where Gateway and browser run, the private/loopback or explicitly bounded control path, Host/Origin behavior, browser sandbox mode and the operator route. A tx-node route is not granted by this package; if later selected, the Coordinator must explicitly name it and record its isolation before publication. No public listener or uncontrolled Chrome/CDP route is allowed.
5. **Low-privilege identity and isolation:** record the dedicated non-root user, workspace, temporary profile, allowed filesystem/network scope, cleanup owner and confirmation that no Vault, production Secret, SSH key, GitHub token, Tailscale auth key, personal browser profile or account material is available to the runtime.
6. **Budgets and cancellation:** freeze wall-clock, navigation/request count, Browser Worker observation size/count, media/output size, concurrent process, storage, retry and cleanup deadlines. Abort on budget or cancellation; do not retry the known 4xx/no-media boundary indefinitely.
7. **Sanitized Evidence contract:** bind Evidence to Candidate/run/job/host/attempt and record only statuses, bounded phases, media shape/protocol, same-origin capability facts, playback/control/reconnect facts and cleanup. Exclude full URLs, signed queries, headers, cookies, tokens, profile paths, page/media bodies and raw worker stderr.
8. **Publication state:** only after the above read-back may the Coordinator set `status:ready` and issue a downstream execution entry. This docs revision itself performs no live request and launches no target job.

## Future verification matrix

| Job | Claims | Execution plane / runner | Required evidence |
|---|---|---|---|
| J1 | P2–P7 regressions | GitHub-hosted x64 Actions | exact SHA fmt, clippy, workspace tests, plugin/worker/SourceSession/Playback/Egress/security checks |
| J2 | P3–P5 product composition | GitHub-hosted x64 Actions | server-owned Control → Display route with bounded controlled observation/media fixture; no seed store mutation or raw media injection |
| J3 | P1–P8 public journey | Coordinator-approved ordinary-Linux execution route | same exact Candidate; real Control source creation for `BV14V411W7r5`, generic Browser Worker observation, media load/progress, controls, reconnect and sanitized failure/cleanup evidence |
| J4 | P6–P8 evidence hygiene | GitHub-hosted and/or approved execution route as explicitly frozen | capability/authority invalidation, expiry, cancellation, no Secret/profile leakage, bounded artifacts and cleanup |

J2 synthetic or controlled evidence cannot be promoted to a real Bilibili claim. J3 is not authorized or scheduled by this package revision.

## Success criteria for the later Task

1. The Coordinator accepts this Contract Revision and completes the Publication Gate.
2. P1–P8 are reported against one exact Candidate; each claim is separately marked `PASS`, `CONDITIONAL PASS`, `FAIL` or `BLOCKED`.
3. The real public source is created through the production Control/API and Bilibili Site Plugin route; no raw media or store injection is used.
4. Gateway same-origin Web Display playback viability is shown for the actual resolved media shape, with #271 generic remux only when required.
5. Controls and reconnect preserve one authoritative PlaybackSession and all stale/duplicate paths fail closed.
6. No account/Auth Mode/Secret/bypass/phone/TV/#246 scope is introduced.
7. Historical anonymous diagnostics remain negative and any repeated 4xx/no-media boundary is reported honestly.
8. Worker reports one bounded Attempt, releases ownership and stops; Coordinator alone reviews, accepts, revises, blocks or closes #68.

## Evidence contract

A future `[EXECUTION REPORT]` or `[BLOCKER REPORT]` must separate implementation result, verification result and Coordinator decision and include:

```text
Attempt / worker / environment
Contract revision and exact Candidate SHA/PR
GitHub-hosted freshness run/job/artifact references
Execution plane / runner / target host and low-privilege identity class
Frozen selector: BV14V411W7r5
Source creation phase and Site Plugin/Browser Worker phase statuses
ResolvedMedia/MediaShapeV1 protocol and stream-shape summary
Gateway same-origin media capability and PlaybackSession/item/revision facts
play/pause/seek/stop and Control/Display refresh/reconnect observations
Egress/SSRF, stale/expiry/cancellation/cleanup and Secret-leak negatives
P1-P8 result, historical-boundary result, unverified/out-of-scope items
```

Never publish source/resolved/signed URLs, Cookie/Authorization/bearer material, profile/Vault data, page or media payloads, raw worker stderr, arbitrary filesystem paths or unredacted browser/network storage.

## Freshness and integration rules

At every future Publication Gate, compare this package and all semantic authorities with current `main`. Relevant authorities include `site-adapter-api`, `plugins/bilibili`, generic Browser Worker, `gateway-core` SourceSession/Playback/Display/Control, R008 Egress/security, MediaShapeV1 and #271 delivery. A change to source interpretation, observation handoff, media shape, capability binding or Playback authority is `SEMANTIC_AUTHORITY` or `CONTRACT_INVALIDATING` and requires Coordinator review before execution.

## Completion protocol

```text
status:draft + owner-free
→ Contract Revision accepted by Coordinator
→ dependency/Candidate/host/budget/Evidence Publication Gate
→ status:ready + env:cloud + queue read-back
→ one bounded Worker Attempt
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ status:review or status:blocked
→ release owner → Coordinator review
```

The Worker cannot set `status:done`, close #68, modify #246, start an authenticated route, start #72, or silently broaden this public scope.
