# Task Contract — BILIBILI-WEB-E2E (Public / No-Login Contract Revision)

> **Contract Revision (2026-09-11, after Attempt 1 blocker):** This package replaces the stale generic-ytdlp-only route and adds the missing generic Site Plugin-owned Browser acquisition target seam. The next implementation attempt remains public, non-DRM Bilibili with no account or login. This revision is a contract/package change only; it does not authorize a live attempt or claim playback.

## Metadata

```text
GitHub Issue: #68
Task ID: BILIBILI-WEB-E2E
Task kind: combined (implementation seam + real-source functional verification)
Execution Base: n/a until post-revision Publication Gate
Final Candidate: n/a until the bounded combined Attempt
Preferred worker: Codex Cloud, env:cloud
Required capabilities after publication: github-read-write, repository-static-analysis,
  code-authoring, automated-build, automated-test, cloud-interactive,
  an explicitly approved ordinary-Linux browser/control route
Current package state: status:draft, owner-free
```

### Attempt 1 blocker and recovery

Attempt 1 was blocked after its focused hosted run and did not consume the live
public-source budget. The cancelled superseded J3 never started its bounded fresh
consumer, so no Gateway public session creation or Bilibili source POST occurred.
The reusable implementation branch remains PR #278 at Candidate
`e8e214dddb4252bd4869085c7a01c0c1c8e03397` (DRAFT); it must be resumed after this
contract revision rather than replaced or merged here.

The architectural blocker was a missing generic Site Plugin-owned Browser acquisition
target contract. `SiteAdapter::navigation()` is the existing previous/next/collection
capability and is not an acquisition API. Core must not parse the caller's Bilibili
URL after recognition. Attempt 1's ordinary hosted defects (formatting and the test
trait import) remain historical implementation evidence and are not reclassified as
success. This revision defines the API seam; it does not repair or verify PR #278.

The revision extends the accepted ordinary-Linux browser authority from #154 rather than redefining it. Issue #154 was Final Accepted and merged to `main` as `836e220e6ba4e38377a4e40cff677c9549aa7798`; its accepted Control → Gateway `PlaybackSession` → Web Display route, same-origin media, play/pause/seek/stop, refresh/reconnect, stale/error/concurrency/security behavior and Candidate-bound artifact consumption are the baseline for this Task. #68 adds the real public Bilibili source path to that route.

The direct source-path authorities are the production Bilibili Site Plugin (#237) and generic Browser Observation Bridge (#240). MediaShapeV1 (#268) and policy-bound media delivery (#271) are the direct media-path authorities. #255 may supply the production composition root/adapter registration where the current executable needs it, but this scope may make only the smallest public/no-account composition adjustment required; it must not configure or use an account. The auth-specific implementation surfaces (#243/#248/#251/#257/#259) are code present on `main` and regression surfaces only. They are not hard dependencies for this public/no-login route and must not be invoked. #246 is a separate authorized-account target Task and remains independently `status:blocked`; it is not a dependency or alternate route for this public scope.

### Accepted #154 baseline

The following behavior is inherited from #154 Final Acceptance and must remain unchanged while #68 extends the source ingress:

```text
ordinary-Linux Control
→ server-owned PlaybackSession / PlaybackItem
→ same-origin Gateway media capability
→ Web Display
```

The accepted baseline covers the `/display` and `/control` product entrypoints, Display registration/heartbeat and selection, request-id/CAS command authority, play/pause/seek/stop, refresh/reconnect, stale lease/item/revision/display-generation rejection, error/concurrency/security regressions, and exact Candidate-bound runtime/artifact consumption. #68 must exercise those existing authorities with a real public Bilibili source; it must not add a second state store, redefine command semantics, or replace the baseline with an ad-hoc browser or extractor path.

## Goal

Close one public, no-login product journey for the frozen sample `BV14V411W7r5`, without weakening any Site Plugin, EgressPolicy, Secret, Browser Worker, MediaShape, or PlaybackSession authority:

```text
Control submits the frozen public Bilibili URL
→ SiteAdapterRegistry recognizes a Bilibili SourceLocator
→ normal direct resolve is attempted on that locator
→ only on explicit ObservationRequired, owning Site Plugin produces a BrowserAcquisitionTarget
→ Core binds the target to operation/session and admits it through R008/Egress
→ generic Browser Worker performs the bounded acquisition
→ the same locator is resolved with the one-shot observation/handoff
→ plugin interprets bounded observation into server-owned ResolvedMedia / MediaShapeV1
→ SourceSession prepares and publishes one PlaybackSession / PlaybackItem
→ direct muxed/HLS delivery, or generic #271 remux delivery for separated A/V
→ Gateway same-origin media capability reaches Web Display
→ bounded play / pause / seek / stop / refresh / reconnect
```

The Browser Worker is generic infrastructure. Bilibili URL, page, part, media-selection
and observation interpretation remain in `plugins/bilibili`; no Bilibili knowledge is
added to Core or the generic worker. The Gateway remains the only `PlaybackSession`
authority.

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
3. The owning plugin derives a server-owned, bounded `BrowserAcquisitionTarget` from the opaque locator. HTTP callers, Control and Display cannot provide or override it. The existing collection `navigation()` method remains separate and unchanged.
4. Core validates only the generic target shape, binds it to the original locator/operation/session and admits it through R008/Egress. Every initial target, redirect and request is re-authorized; the target never widens SSRF, TLS, host or access-control authority.
5. The generic Browser Worker consumes only the plugin-produced target and generic policy. It emits only versioned, size-limited, redacted observations and server-owned handoff references; it does not interpret Bilibili identifiers or page rules.
6. The plugin interprets the generic observation and produces a server-owned `ResolvedMedia`/`MediaShapeV1`. Public headers remain free of Cookie, Authorization, bearer, profile and other Secret material.

### Playback and delivery

- `SourceSession` validates and prepares the plugin result; it owns no second playback state.
- `PlaybackSession` and `PlaybackItem` remain the sole playback authority inherited from #154, with session/item/revision/display-generation checks and stale-result rejection.
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
- Core-side parsing or reconstruction of a Bilibili acquisition URL, BVID, part, site path/query or private API rule.
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
     redacted and EgressPolicy-governed; the owning plugin supplies the bounded
     BrowserAcquisitionTarget and retains Bilibili interpretation.
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

## Execution Base and final Candidate lifecycle

The Publication Gate freezes the starting identity, not a Worker-produced implementation Candidate:

1. The Coordinator freezes one exact **Execution Base**: the current accepted `main`/integration SHA, branch or ref identity, and freshness classification (`NONE`, `UNRELATED`, `INTEGRATION_OVERLAP`, `SEMANTIC_AUTHORITY` or `CONTRACT_INVALIDATING`). Existing hosted evidence is recorded as fresh or stale against that base.
2. The Gate does not require a final Worker Candidate, implementation branch or PR before `status:ready`. A combined Task may still need a focused implementation change.
3. After `status:ready`, one bounded combined Attempt starts from the frozen Execution Base. If the public route needs implementation, the Worker creates one focused Candidate/PR from that base. If no implementation is needed, the final Candidate is the Execution Base itself.
4. Before the Worker reports, all required hosted J1/J2/J4 verification, Candidate-bound package/manifest/digest/fresh-consumer admission, and the real public J3 must target the same final Candidate SHA. No mixed-base or moving-main evidence is accepted.

For the next implementation attempt, the final Candidate must consume this
`BrowserAcquisitionTarget` contract and reuse PR #278/its branch. The docs revision
must be merged and read back before #68 can be republished through the Publication
Gate; this package does not start Attempt 2 or authorize live traffic.

## Publication Gate (must remain unsatisfied in this revision PR)

`#68` stays `status:draft`, `env:cloud`, owner-free until the Coordinator independently reads back and records all of the following. A Worker cannot claim an Attempt before the gate is complete.

The gate must verify that the canonical `BrowserAcquisitionTarget` contract and its
documentation are merged and read back on current `main`, and that PR #278 is the
only reusable implementation branch for the next Attempt. It must not require the
API implementation itself before `status:ready`: implementing that seam is a required
Attempt 2 Candidate criterion, together with its hosted verification. Attempt 1's
blocker and negative live-budget audit remain append-only history.

1. **Package and dependency read-back:** live Issue, this `task.md`, `prompt.md`, canonical docs and accepted authorities are mutually consistent. The Coordinator checks #154's Final Acceptance/merge baseline, direct source authorities #237/#240, direct media authorities #268/#271, and the composition root where #255 is actually needed against current `main`. Auth-specific surfaces #243/#248/#251/#257/#259 are regression inputs only, not public-route dependencies; #246 remains separate and blocked.
2. **Execution Base and freshness classification:** freeze the exact accepted `main`/integration identity from which the Attempt will start and classify its movement using `NONE`, `UNRELATED`, `INTEGRATION_OVERLAP`, `SEMANTIC_AUTHORITY` or `CONTRACT_INVALIDATING`. Record whether existing hosted evidence is fresh for this base. This Gate does not require a final Worker Candidate or PR; moving `main` is never an execution identity.
3. **Final-Candidate verification plan:** record the required hosted J1/J2/J4 jobs, Candidate-bound package/manifest/digest/fresh-consumer admission and real public J3 that must all run against one final Candidate before the report. No local compilation or target compilation substitutes for that evidence.
4. **Approved ordinary-Linux execution host/control route:** record where Gateway and browser run, the private/loopback or explicitly bounded control path, Host/Origin behavior, browser sandbox mode and the operator route. A tx-node route is not granted by this package; if later selected, the Coordinator must explicitly name it and record its isolation before publication. No public listener or uncontrolled Chrome/CDP route is allowed.
5. **Artifact admission and low-privilege isolation:** record the exact Candidate-bound artifact/package, manifest and digest, target platform/ABI, runtime asset and helper/worker identity, and a fresh-consumer admission check that starts the same product without the Actions build tree. Then record the dedicated non-root user, workspace, temporary profile, allowed filesystem/network scope, cleanup owner and confirmation that no Vault, production Secret, SSH key, GitHub token, Tailscale auth key, personal browser profile or account material is available to the runtime. Missing, tampered, wrong-platform or wrong-Candidate assets fail closed. The target must never compile, build or install to compensate for missing assets. Reuse the accepted #154/#146 provenance and admission principle where applicable, but do not reuse an old artifact as the new #68 runtime.
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

## Artifact admission for a later ordinary-Linux live run

The future live execution must consume a freshly admitted, exact-Candidate package produced by GitHub-hosted Actions. The package manifest must bind at least:

```text
Candidate SHA / workflow run / required job IDs
artifact names, digests and provenance
target platform / ABI / runtime layout
Gateway binary and helper/worker hashes
required static assets and configuration schema/version
fresh-consumer start result from a directory without the Actions build tree
```

The ordinary-Linux host receives only this verified package and runs compile-free commands as the dedicated low-privilege identity. It may not run Cargo, install a compiler/dependency/FFmpeg/Chromium package to repair an incomplete package, or fall back to source/fixture injection. The host-side start, stop, cleanup and artifact paths must be bounded and recorded without leaking local paths or secrets. This reuses the provenance/admission principle accepted by #154 and #146 while requiring a new #68 Candidate package; neither old #154 artifacts nor old #146 artifacts are the #68 runtime.

## Success criteria for the later Task

1. The Coordinator accepts this Contract Revision and completes the Publication Gate.
2. P1–P8 are reported against one exact final Candidate; each claim is separately marked `PASS`, `CONDITIONAL PASS`, `FAIL` or `BLOCKED`.
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
Candidate-bound package/manifest/digest/platform/runtime asset admission and fresh-consumer result
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
→ dependency/Execution-Base/host/budget/Evidence Publication Gate
→ status:ready + env:cloud + queue read-back
→ one bounded Worker Attempt
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ status:review or status:blocked
→ release owner → Coordinator review
```

The Worker cannot set `status:done`, close #68, modify #246, start an authenticated route, start #72, or silently broaden this public scope.
