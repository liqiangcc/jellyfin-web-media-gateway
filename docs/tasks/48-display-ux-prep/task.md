# Task — DISPLAY-UX-PREP Smart TV entry + subtitle + degraded browser UX

## Metadata

```text
GitHub Issue: #48
Parent Goal / Research Item: Phase 0A-3 / first usable Web-only TV Display UX PREP
Task / Research ID: DISPLAY-UX-PREP
Task kind: combined
Planning base: d0cd647d8965c03c412d67a7f6cea9b33fa2ec38
Session bootstrap prompt: docs/tasks/48-display-ux-prep/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, rust-code-authoring, browser-ui-authoring, headless-chromium-verification
Hard publication dependencies: #45 accepted; R001/#39/R008 accepted authorities available
Physical TV Evidence Authority: #7 remains separate
```

Realtime status/owner/branch/PR/Evidence lives in Issue #48.

## Goal

Build the browser-only Phase 0A-3 UX PREP on accepted #45 Web Display semantics: deterministic smart `/` entry, a TV-style `/display?profile=tv` shell that can register/reconnect/heartbeat while idle, minimal additive subtitle contract closure, remote-focus basics, and explicit Fullscreen/secure-context/autoplay degradation in hosted browsers.

```text
GET /
→ explicit Control / TV choices
→ default 5-second countdown to TV
→ /display?profile=tv
→ #45 register / reconnect / heartbeat
→ waiting or safe Gateway media fixture
→ optional Gateway-safe WebVTT track
→ viewport immersive UX
→ fullscreen / secure-context / play() degradation reporting
```

This is hosted/browser PREP. It does not prove target-TV autoplay and it does not define the product source→session media endpoint owned by #44. Full production source/session→Display composition is #49.

## Current accepted facts

- #45 allows a Web Display to register before a PlaybackSession exists and uses server-issued registration/lease authority; page lease is not R007 display generation.
- Reconnect can rotate a lease using previous registration identity/token; browser page state must not log/expose it.
- Current #45 safe capability allowlist is only `video|audio`; this Task may add `subtitles` only after real subtitle rendering support exists.
- Canonical `ResolvedMedia` already includes `subtitles[]`, but current Rust `ResolvedMedia` has only `title/source_site/streams/protection`.
- R001 already provides scoped Gateway capability paths; subtitle browser URLs must use the same server-side authority pattern rather than raw upstream/local file URLs.

## Task Decomposition Decision

```text
Verification mode: inline hosted-browser PREP
Linked target verification: Issue #7 physical TV/manual remains separate
Linked product E2E: Issue #49 after #44/#47/#48 acceptance
```

## Smart entry contract

`GET /` renders a small deterministic mode chooser:

- explicit `TV Display` and `Control` actions;
- if there is no explicit choice, countdown defaults to `/display?profile=tv` after **5 seconds**;
- user interaction cancels/restarts only according to one documented bounded rule; no navigation loop or UA sniffing;
- a direct `/control` or `/display?profile=tv` navigation never waits for the countdown.

Do not infer product mode from User-Agent as authority.

## Web Display UX contract

`/display?profile=tv`:

- obtains/reuses a bounded non-secret `display_id` selector without claiming generation authority;
- registers through #45 with `video`, `audio`, and after C3 is complete, `subtitles` capability;
- keeps registration/lease only in memory/session-scoped browser state needed for refresh continuity; never logs it or persists it as long-lived product identity/credential;
- heartbeat interval is derived from/comfortably below lease TTL and stops cleanly on page teardown;
- refresh/reconnect reuses #45 previous-registration semantics so old page lease loses callback authority;
- can remain in a clear `waiting for session/media` state; it must not manufacture PlaybackSession state.

A deterministic test harness may provide a **Gateway-safe same-origin media descriptor** to exercise rendering. No production public raw-media seed or arbitrary URL parameter is allowed.

## Minimal subtitle contract closure

Add the smallest generic subtitle path required by MVP.

### Site-domain internal type

Extend `site-adapter-api` additively:

```text
ResolvedMedia
+ subtitles: Vec<ResolvedSubtitle>

ResolvedSubtitle
├── id
├── url                 # upstream http/https only, server-side
├── content_type        # MVP: text/vtt
├── language?
├── label?
├── public_headers?     # non-Secret only
└── upstream_access_ref? # opaque server-side handle only
```

Rules:

- existing plugins/fixtures return `subtitles=[]` unless they already have a deterministic generic subtitle fixture;
- no site-specific subtitle extraction in this Task;
- `file:`, local filesystem paths and unsupported schemes/content types are rejected;
- shared Secret-header validation is reused for subtitle headers;
- `upstream_access_ref` is never browser-visible and cannot be caller-forged through public UI input;
- #39 conformance is extended, not weakened.

### Display-safe type

Convert resolved subtitle to a browser-safe view only through R001-style Gateway capability binding:

```text
SubtitleTrackView
├── id
├── language?
├── label?
├── format: webvtt
└── gateway_path
```

The browser receives only same-origin Gateway `gateway_path`, never raw upstream URL/header/Secret/local path.

At least one deterministic external WebVTT fixture must travel through the real Gateway capability/HTTP path into an HTML `<track>` (or equivalent native browser text-track API) in hosted Chromium.

This Task may use a test-only safe media descriptor/provider for the browser fixture. It must not define #44's source/session creation API or a production arbitrary-media injection route.

## Claims

- **C1 — smart entry:** `/` offers explicit modes and defaults after exactly 5 seconds to TV Display with deterministic cancellation/no-loop behavior.
- **C2 — viewport/idle Display:** TV shell registers/heartbeats with #45 even with no session, supports 720p/1080p/4K-like viewport tests, and presents readable waiting/recovery overlays without Fullscreen success.
- **C3 — additive subtitle contract:** generic Rust subtitle types/conformance are additive; all existing plugins compile with empty subtitle lists; Secret/scheme/content-type validation is bounded.
- **C4 — subtitle Gateway path:** deterministic WebVTT reaches browser through scoped same-origin Gateway capability and renders as a text track; raw upstream/local/Secret authority never reaches browser.
- **C5 — remote-focus basics:** essential activation/recovery controls have deterministic keyboard/remote-like focus traversal and Enter/OK behavior in hosted tests.
- **C6 — Fullscreen degradation:** allow/deny/unsupported paths are explicit; viewport immersive remains usable if Fullscreen fails.
- **C7 — secure-context degradation:** Wake Lock/Service Worker or similar secure-context-only features are optional enhancement; trusted-LAN HTTP baseline does not fail without them.
- **C8 — play rejection + reconnect:** forced `video.play()` rejection produces bounded #45-compatible Display error/status reporting; refresh rotates/supersedes old lease and old page callback cannot regain authority.
- **C9 — browser/Secret safety:** DOM/console/storage/artifacts contain no source-site Secret, raw protected upstream headers/URLs, arbitrary local file authority, password/code/QR data or hidden bypass.

## In Scope

- `/` smart entry and TV display shell;
- #45 registration/heartbeat/reconnect client behavior;
- responsive TV viewport and focus/recovery UI;
- minimal generic WebVTT subtitle types/conformance/Gateway capability/view;
- deterministic Gateway-safe media/subtitle test harness only;
- Fullscreen/secure-context/play rejection feature detection/degradation;
- hosted Chromium tests/workflow.

## Out of Scope

- physical TV PASS/CONDITIONAL PASS/FAIL (#7);
- claim that headless Chromium predicts audible TV autoplay;
- #44 product source/session creation or product session media endpoint;
- #47 Control UI implementation;
- real site subtitle extraction/login;
- advanced subtitle preferences/translation/styling/OCR;
- Jellyfin UX;
- R007 revision/display-generation redesign;
- secure-context API as a LAN playback requirement.

## Architecture Invariants

1. #45 owns page registration/lease; R007 owns display generation/Playback authority.
2. Browser `display_id` is only an accepted bounded selector; it cannot choose generation/active authority.
3. Subtitle internal upstream metadata remains server-side; browser sees only Gateway-safe paths.
4. Subtitle addition is generic/additive and must keep generic-direct/generic-ytdlp/Core site-neutral.
5. Existing R008 Secret/Egress checks are reused; no open proxy/local-file escape.
6. Hosted PREP does not manufacture physical-TV Evidence.

## Expected files

Likely:

- `site-adapter-api/src/lib.rs` + conformance tests;
- plugin constructors/tests (`generic-direct`, `generic-ytdlp`, fake/conformance fixtures) for additive empty subtitles;
- `gateway-core/src/lib.rs` Display/smart-entry/subtitle Gateway-safe view/harness wiring;
- `gateway-core/src/display_session.rs` only to truthfully allow `subtitles` capability after implementation;
- browser assets/tests + `.github/workflows/display-ux-prep.yml`.

Do not silently alter #44/#47 contracts.

## Verification Plan

| Job | Claims | Runner | Required | Intent |
|---|---|---|---|---|
| J1 | C1-C5,C8 | github-hosted ubuntu-latest + headless Chromium | yes | smart entry, idle registration/heartbeat/reconnect, viewport/focus, Gateway WebVTT render happy paths |
| J2 | C3-C9 | github-hosted ubuntu-latest + headless Chromium | yes | invalid subtitle scheme/type/Secret, fullscreen allow-deny-unsupported, insecure-context degradation, forced play rejection, old-lease callback, DOM/console/storage negatives |
| J3 | C1-C9 | github-hosted ubuntu-latest | yes | workspace + #45/#39/#38/R007/R001/R008/media/plugin/security regressions |

Required Evidence asserts exact Task Candidate SHA.

## Success Criteria

1. C1-C9 have exact-Candidate Evidence.
2. `/` defaults to the TV profile after 5 seconds while retaining explicit Control/TV choice.
3. `/display?profile=tv` can register idle, heartbeat and refresh/reconnect without inventing a session.
4. Existing plugins remain conformant with `subtitles=[]` and at least one deterministic WebVTT fixture renders through a scoped Gateway path.
5. Fullscreen and secure-context failure do not make viewport playback UX unusable.
6. Forced play rejection is visible through bounded Display status/error reporting.
7. No raw upstream/local/Secret subtitle/media authority reaches browser state or Evidence.

## Freshness / Integration Contract

Freshness policy: dependency-aware

Semantic authorities:
- #45 Display registration/lease/context/callback semantics;
- #39 SiteAdapter/ResolvedMedia conformance and shared Secret-header semantics;
- R001 media capability path and R008 Egress/Secret boundary;
- R007 Display generation authority.

Semantic freshness domains:
- `gateway-core/src/display_session.rs`;
- `site-adapter-api/**` ResolvedMedia/conformance/security;
- R001 capability path in `gateway-core/src/lib.rs`;
- `gateway-core/src/security.rs`;
- `gateway-core/src/playback.rs` display authority only as read-only invariant.

Integration surfaces:
- `gateway-core/src/lib.rs` router/page/media composition;
- `site-adapter-api` public struct additions consumed by all plugins;
- `plugins/generic-direct/**`, `plugins/generic-ytdlp/**`;
- `Cargo.toml` / `Cargo.lock` / workspace tests.

Task-owned surfaces:
- smart entry/TV browser shell, additive subtitle contract/conformance, subtitle Gateway-safe capability/view and browser degradation tests.

Authority/domain → Claim mapping:
- #45 Display: C2,C5,C8,C9
- #39/ResolvedMedia: C3,C4,C9
- R001/R008: C4,C9
- R007 display authority: C8

Integration verification:
- JI1: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --all-targets`
- JI2: targeted SiteAdapter conformance + generic-direct/generic-ytdlp + #45 display-session + media-gateway/security regressions and one hosted smart-entry/subtitle smoke.

Unrelated-main policy:
- existing exact-Candidate semantic Evidence remains valid; no rebase/full rerun solely because main advanced.

Integration-overlap policy:
- preserve accepted semantic Evidence; compose with Coordinator-frozen Integration Base and run JI1/JI2 unless conflict changes Task semantics.

Semantic-authority-change policy:
- reconcile changed #45/#39/R001/R008/R007 authority and rerun mapped affected Claims; `ResolvedMedia` changes are semantic, not merely path overlap.

Strict-main reason: n/a

## Evidence Contract

Report Task Candidate SHA/PR, Evidence Base/observed main, smart-entry/Display UI paths, #45 reconnect evidence, subtitle type/conformance/Gateway path, headless viewport/focus/fullscreen/secure-context/play-rejection results, browser Secret scan, C1-C9 and exact J1/J2/J3 job IDs.

No lease token value, raw upstream media/subtitle URL/header, `upstream_access_ref`, source-site Secret, local filesystem path or password/code/QR data may appear in Evidence.

## Completion Protocol

```text
claim → status:in-progress → Attempt N
→ candidate + exact-SHA J1/J2/J3
→ [EXECUTION REPORT] → status:review → release owner → STOP
```

Worker never merges its own PR, sets `status:done`, closes #48, executes #44/#47/#7, or automatically starts another Task.
