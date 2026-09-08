# Task — BILIBILI-WEB-E2E

> Coordinator planning note (2026-09-08): #165/#166 is an independent browser-source research branch. This existing unpublished generic-ytdlp contract is retained and must not consume browser-derived media. After accepted source portability and any required generic media/plugin implementation, formally revise this contract and repeat Publication Gate. See `docs/research/bilibili-browser-acquisition.md`. No current Claim/dependency is silently relaxed.

## Metadata

```text
GitHub Issue: #68
Task ID: BILIBILI-WEB-E2E
Task kind: implementation + real-source functional E2E verification
Planning Base: b17a27ca5d8c2f76cddc4c7cf3fdaa239169593a
Preferred worker: cloud-codex with authenticated SSH tx-node and accepted #146 browser path
Eligible environment after publication: env:cloud
Hard publication dependencies: #67 R20-or-later Final Acceptance PASS; #49 Web MVP Final Accepted; #146 R3-or-later execution authority remains valid
Accepted authorities: #44 SourceSession; #45 Web Display; #47 Control; #49 hosted Web MVP; #60/#66/#73 generic-ytdlp runtime/security; #71 Navigation authority remains independent
Freshness policy: dependency-aware
```

> #68 is the first real user-visible Bilibili playback closure. It composes already accepted Gateway/Web authorities with one accepted real Bilibili `ResolvedMedia` result. It does not add login, navigation, Native Site Panel, DASH/remux, production enablement or performance claims.

## Publication gate — unresolved until #67 PASS

This Task Package is deliberately materialized early as a planning buffer, but **must remain `status:draft`** until #67 is Final Accepted.

Before publication the Coordinator must freeze all of the following from actual #67 Evidence and accepted #146 runbook. No phone deployment/readiness/Runner/management task is a dependency for this functional stage:

```text
#67 Final Acceptance / exact accepted Candidate
frozen selector: BV14V411W7r5
accepted real-source protocol: http-file | hls
accepted stream_count / first-playback shape
accepted offline runtime artifact identity from #79, if still relevant to execution
exact #68 Execution Candidate / Planning Base after freshness classification
real-source Evidence routing: external-codex/ssh tx-node + identified isolated browser host/access path; #146 accepted runbook SHA/uid/gid; no public listener
```

If #67 returns `FAIL`, `BLOCKED`, or a result requiring DASH/separate A/V/remux, **do not publish #68**. Route the evidence-driven generic repair Task first.

## Goal

Prove that the current accepted Web-only product path can consume the exact Bilibili source shape accepted by #67 and deliver one coherent user journey:

```text
Control enters frozen public Bilibili URL
→ SiteAdapterRegistry
→ generic-ytdlp
→ accepted real-source ResolvedMedia
→ SourceSession preparation
→ Gateway same-origin media capability
→ registered Web Display <video>
→ Control play / pause / seek / stop
→ refresh / reconnect
→ one authoritative PlaybackSession remains coherent
```

This is **not** a second extraction verification. #67 owns the real-site extraction compatibility conclusion; #68 proves product composition and browser playback/control over that accepted source shape.

## Frozen source scope

Unless Coordinator revises it at Publication Gate:

```text
site: Bilibili
mode: public / no-login / non-DRM
selector: BV14V411W7r5
source URL: task input only; do not persist full source/resolved/signed URLs in durable Evidence
```

Only the exact media protocol/shape accepted by #67 may be claimed by this Task.

## Accepted product authorities

### #49 Web MVP

The accepted hosted product journey already proves the generic composition:

```text
GET /
→ /display?profile=tv
→ real Display registration/heartbeat
→ /control
→ live Display selector
→ POST /api/v1/sessions
→ SourceSession publication
→ server-owned rendering view
→ Gateway media path
→ /control?session_id=<id>
→ play/pause/seek/stop
→ Control + Display refresh/reconnect
```

#68 must reuse these production routes/authorities. It may add only the minimal real-source glue required by the accepted generic-ytdlp result.

### #44 / SourceSession

- source input remains `CreateSessionRequest { request_id, source, display_id }`;
- Registry recognition/resolution remains server-owned;
- caller cannot inject `ResolvedMedia`, `SourceLocator`, upstream URL/header, item/session revision or Egress authority;
- prepared media paths remain Gateway capabilities.

### #45 / Web Display

- page registration/lease remains distinct from Playback display generation;
- Display rendering derives current session/item/revision from server authority;
- stale lease/session/item/generation relationships fail closed.

### #47 / Control + R007

- Control is View + Intent, not a second playback state store;
- command ingress remains request-id + expected-session-revision authority;
- play/pause/seek/stop semantics are unchanged.

### #60/#66/#73/#79 / generic-ytdlp runtime

- production default remains disabled unless separately enabled by an explicit production gate;
- this Task may use the accepted verification/product composition seam needed to reproduce the #67 accepted source shape;
- R008 remains the only extractor/upstream network authority;
- no Cookie/Auth/profile/proxy bypass or alternate extractor path.

## Required product journey

### P1 — Display readiness

1. Start exact Candidate Gateway in an isolated test environment.
2. Open `/display?profile=tv` through the product route.
3. Register/heartbeat one real Display session.
4. Confirm Control can discover/select that Display using the accepted live-display view.

No synthetic Display authority or direct store mutation may construct the success path.

### P2 — Real Bilibili source creation

From product Control, submit the frozen public Bilibili URL using the accepted creation API.

Required path:

```text
POST /api/v1/sessions
{ request_id, source, display_id }
→ SiteAdapterRegistry
→ generic-ytdlp
→ exact accepted #67 media shape
→ SourceSession
→ PlaybackSession
```

The user-visible journey must not require ad-hoc Python/yt-dlp CLI execution, raw `ResolvedMedia` injection, or proof-only seed APIs.

### P3 — Gateway rendering path

The attached Display must obtain the server-owned current rendering view and load only Gateway same-origin media paths.

Required:

- current `session_id` / `item_id` / `item_revision` match Playback authority;
- media protocol matches the #67 accepted first-playback shape;
- browser does not receive arbitrary caller-selected upstream URLs/Secret headers;
- stale/foreign rendering context is rejected.

### P4 — Browser playback

In hosted/headless Chromium where deterministic browser media behavior is supportable, or in a Coordinator-approved real browser Evidence step if network/browser policy requires it, prove the accepted media path reaches the `<video>` element far enough to establish product playback viability.

Evidence should prefer bounded browser facts such as:

```text
media element source is Gateway same-origin
readyState / loadedmetadata / canplay class
currentTime progresses or equivalent bounded playback observation
no browser network request bypasses Gateway media authority
```

Do not claim physical-TV autoplay/audible behavior; that remains separate physical-target Evidence.

### P5 — Control commands

On the same session:

```text
play
→ pause
→ seek
→ play
→ stop
```

Prove:

- accepted R007 request-id/CAS semantics;
- item/session revision coherence;
- Display callbacks/telemetry cannot overwrite newer authority;
- no duplicate command mutation from retries.

### P6 — Refresh / reconnect

Prove both:

- Control refresh / event reconnect;
- Display page refresh / lease reconnect.

Both rebuild from Gateway authority and preserve the current session/item unless an accepted command changed it.

## Claims

```text
B1 — Real source enters only through accepted product source/session authority.
B2 — #67 accepted Bilibili media shape is reproduced without alternate extractor or raw-media injection.
B3 — Web Display receives only current server-owned Gateway media paths.
B4 — Browser media path is product-viable for the accepted http-file/HLS shape.
B5 — play/pause/seek/stop preserve existing R007 authority.
B6 — Control/Display refresh and stale callbacks cannot create a second state authority.
B7 — no Cookie/Auth/profile/signed URL/raw worker stderr/upstream payload leaks into durable Evidence.
B8 — first real Bilibili Web playback is closed without navigation/login/BrowserWorker/DASH/remux/performance scope creep.
```

## Deterministic verification

### J1 — Product composition / exact Candidate

GitHub-hosted Ubuntu unless Coordinator records another execution plane.

Prove:

- exact Candidate identity;
- product Display registration and live selector;
- product Control source creation;
- real accepted SiteAdapter path selected;
- session/rendering path coherence;
- no proof-only/store-injection success path.

### J2 — Browser journey

Use product `/control` + `/display` routes and the exact accepted media shape. The live ordinary-Linux step uses accepted #146 low-privilege tx-node execution and its isolated browser access path; record SSH/actual browser host separately from hosted Actions. Do not inspect or reuse the user browser profile, disable Chromium sandbox/autoplay policy, or expose Gateway/CDP publicly.

Prove:

- same-origin Gateway media path reaches `<video>`;
- bounded media readiness/playback observation;
- Control play/pause/seek/stop;
- Control and Display refresh/reconnect;
- browser console/network/storage leak negatives.

If public-network access is required for the exact real source, separate deterministic product mechanics from the permitted real-source Evidence step, but both must use the same exact Candidate and accepted product path.

### J3 — failure / security matrix

Cover:

- invalid/no-match source;
- offline/expired/missing Display;
- stale expected session revision;
- request-id mismatch;
- stale lease/callback/context;
- missing session/media projection;
- upstream/extractor bounded failure propagation;
- no partial session/media authority on failed creation.

### J4 — regressions

Exact Candidate:

- fmt / clippy / workspace tests;
- #44 SourceSession;
- #45 Web Display;
- #47 Control/R007;
- #49 hosted Web MVP;
- R001/R008/security;
- generic-ytdlp conformance/runtime boundaries;
- #71 navigation regressions where current main integration requires them.

All required jobs assert the exact Candidate SHA.

## Success criteria

1. #67 was Final Accepted PASS before publication and the exact accepted media protocol/shape is recorded.
2. B1-B8 PASS on one exact Candidate.
3. Product Control creates the real Bilibili session through accepted public/service APIs.
4. Display consumes the current server-owned Gateway media rendering path.
5. Browser playback viability is demonstrated for the #67 accepted first-playback media shape.
6. play/pause/seek/stop and reconnect paths preserve one authoritative PlaybackSession.
7. failure/security matrix shows no partial/duplicate authority or Secret leakage.
8. no navigation/login/Native Panel/DASH/remux/performance/production-enable scope is pulled in.
9. Worker reports and STOPs; it does not auto-start #72/Auth/performance work.

## Evidence contract

`[EXECUTION REPORT]` must include bounded Evidence:

```text
Attempt / worker / environment
Planning/Evidence Base
Candidate SHA / PR
#67 accepted source Evidence reference
frozen selector: BV14V411W7r5
accepted protocol: http-file | hls
accepted stream shape summary
Display registration/rendering result
session creation result
browser media readiness/playback observation
Control command results
refresh/reconnect result
security/leak scan
B1-B8
freshness classification
unverified/out-of-scope
```

Never publish:

- full resolved/signed media URL;
- Cookie / Authorization / bearer token;
- profile/account/Vault material;
- raw worker stderr;
- page/media payload;
- lease token;
- arbitrary local filesystem paths.

## Freshness / Integration Contract

Semantic authorities include:

```text
#67 accepted generic-ytdlp Bilibili result
site-adapter-api/**
plugins/generic-ytdlp/**
gateway-core/src/source_session.rs
gateway-core/src/display_session.rs
gateway-core/src/control.rs
gateway-core/src/playback.rs
gateway-core/src/lib.rs
R001/R008 security/media capability surfaces
```

At Publication Gate, Coordinator must compare the eventual #67 accepted Candidate and current main. Classify movement as `NONE | UNRELATED | INTEGRATION_OVERLAP | SEMANTIC_AUTHORITY | CONTRACT_INVALIDATING` and freeze an exact #68 Candidate/Base.

This early Task Package intentionally leaves the exact execution identity unresolved. Do not silently substitute moving `main`.

## Out of scope

- Bilibili multipart/previous/next (#72);
- login/Cookie/profile/Auth;
- BrowserWorker/Native Site Panel;
- DASH/separate audio-video composition/remux/FFmpeg;
- Jellyfin-specific DisplayAdapter work;
- physical-TV audible/autoplay certification;
- phone CPU/RSS/thermal/soak/performance (#9);
- production generic-ytdlp enablement/hardening.

## Completion protocol

```text
status:draft
→ #67 Final Acceptance PASS
→ Coordinator fills unresolved Evidence/Candidate fields
→ Publication Gate
→ status:ready
→ Worker claim / Attempt N
→ J1-J4
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ status:review or status:blocked
→ release owner
→ STOP
```

Worker cannot set `status:done`, close #68, start #72, or weaken accepted security/runtime authority.


### Non-phone revision and integration selectors

Required capabilities: github-read-write, code-authoring, automated-build/test, authenticated SSH tx-node, accepted isolated browser path (MCP or repository-owned Playwright). Scope remains product composition; Worker produces a new exact Candidate and all real/hosted Evidence targets it. #147 is independent; a missing required navigation run is a concrete verification dependency, not permission to skip it.

Freshness policy: dependency-aware; strict-main reason: n/a. Semantic authorities/domains and B1–B8 mapping: source/session/plugin/runtime → B1/B2/B7/B8; Playback/Control → B5/B6; Display/media → B3/B4/B6/B7; #146 privilege/browser boundary → B4/B7. Integration surfaces: Cargo workspace/dependencies, router, shared workflows/toolchain. Task-owned: minimal real-source registration/composition and relevant product browser harness/tests; no source-specific Core semantics.

JI1: workspace fmt/clippy/tests plus source-session/display/Control/R007 product regression on exact Integration Candidate. JI2: #49 browser composition/reconnect and R001/R008/plugin-boundary checks when router/build overlap. A conflict changing live source/rendering semantics requires affected B1–B8/live journey reruns. Unrelated main/doc changes preserve exact-Candidate evidence; no moving-main rule.

User-visible runbook delivery is required: exact build/start/stop, explicit verification-only plugin enablement, loopback/private browser access, one normal activation if required, source input and clear cleanup. It must permit another Codex to reproduce the journey without raw media injection or the old chat. Cleanup leaves only approved #146 resources; persistent production services and phone deployment remain out of scope.

### Artifact admission for the new product Candidate

All #68 binaries and test executables are built on GitHub-hosted Actions and transferred as verified artifacts; tx-node only runs compile-free commands. Reuse #146 R3's accepted packaging/admission mechanism, but regenerate its manifests and binaries for the exact #68 product Candidate. The old #67 runtime artifact is a source-compatibility baseline, not the #68 product executable.

Inspect the actual product binary/assets for compile-time paths (including CARGO_MANIFEST_DIR or embedded fixture/worker paths), package required runtime assets, and prove a fresh consumer without the original Actions build tree can start the same product used by the real browser journey. Bind Candidate/run/artifact/ABI/worker/helper hashes and target layout before launch. Missing, tampered or wrong-Candidate assets fail without execution. No fallback to cargo/source compilation or fixture injection on the target.
