# Task — WEB-MVP-E2E-PREP Hosted Web-only MVP product integration gate

## Metadata

```text
GitHub Issue: #49
Parent Goal / Research Item: first coherent hosted Web-only MVP integration gate
Task / Research ID: WEB-MVP-E2E-PREP
Task kind: combined
Planning / Evidence Base: d6c53d3b2d1b954132db90872afb8e7a63201442
Session bootstrap prompt: docs/tasks/49-web-mvp-e2e-prep/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, rust-code-authoring, browser-ui-authoring, headless-chromium-verification, github-actions-orchestration
Hard publication dependencies: #44 ACCEPTED; #45 ACCEPTED; #47 ACCEPTED; #48 ACCEPTED
Physical TV / phone / real-site Evidence Authorities: #7 / #9 / #23,#36 remain separate
```

Realtime status/owner/branch/PR/Evidence lives in Issue #49.

## Goal

Prove and, only where current accepted components still need bounded product-composition glue, complete one deterministic hosted Web-only MVP journey using the **actual production routes and accepted authorities**:

```text
GET /
→ explicit TV Display / Control choice
→ /display?profile=tv
→ #45 Web Display registration / lease / heartbeat
→ /control
→ choose an existing live Display selector
→ submit bounded generic-direct source through POST /api/v1/sessions
→ #44 Registry resolve + atomic PlaybackSession publication
→ server-owned safe rendering view for the attached Display
→ Display renders Gateway media + optional WebVTT
→ /control?session_id=<id>
→ #47 ControlView + #38/R007 play/pause/seek/stop
→ Control event reconnect + Display refresh/reconnect
→ authoritative session remains coherent
```

This Task is the first **hosted product integration Gate**. It may add only the minimal generic read/UI/rendering glue required to compose already accepted #44/#45/#47/#48 behavior. It must not redesign their semantics or introduce a second Playback, Display, Site, media, or frontend state authority.

Acceptance means the hosted Web-only MVP path is coherent and reproducible. It does **not** mean physical-TV autoplay, phone resource viability, real-site behavior, or Core Feasibility GO has been proven.

## Accepted dependency facts frozen at publication

Planning / Evidence Base is `d6c53d3b2d1b954132db90872afb8e7a63201442`, which contains Final Accepted #44/#45/#47/#48.

### Accepted live routes

```text
GET  /
GET  /display?profile=tv
GET  /control?session_id=<id>
GET  /api/v1/control/{session_id}
POST /api/v1/sessions
GET  /api/v1/sessions/{session_id}
POST /api/v1/sessions/{session_id}/commands
GET  /api/v1/sessions/{session_id}/events?after=<cursor>
POST /api/v1/displays/register
POST /api/v1/displays/{display_id}/heartbeat
GET  /api/v1/displays/{display_id}/context?session_id=<id>
POST /api/v1/displays/{display_id}/callback
GET  /stream/{token}/{session}/{item}/{revision}/{resource}
```

### Accepted #44 create contract

Public creation remains bounded to:

```text
CreateSessionRequest
├── request_id
├── source
└── display_id
```

The response carries opaque server-owned session/item identity plus safe media summary/Gateway paths. Caller cannot inject raw `ResolvedMedia`, locator payload, upstream headers, lease/page epoch, item/display generation, Egress/Vault fields or Playback revision authority.

### Accepted #45 Display contract

- browser registration/lease is server-owned and is not R007 display generation;
- a page may register while idle;
- `GET .../context?session_id=<id>` may attach an existing registration to an existing session, but item/revision/display-generation facts come from Playback authority;
- stale lease/session/item/generation callbacks are rejected before side effects;
- `DisplayContextResponse` currently carries authority/status facts and advertised media capabilities, but it does not itself carry #44 Gateway media paths.

### Accepted #47 Control contract

- `/control?session_id=<id>` consumes server-produced `ControlView`;
- play/pause/seek/stop use the accepted revision-aware command envelope;
- events are freshness/wakeup signals and refresh rebuilds from authoritative server state;
- current Control UI intentionally did not implement source/session creation.

### Accepted #48 Display UX contract

- `/` defaults to TV after five seconds while retaining explicit mode choice;
- `/display?profile=tv` registers/heartbeats/reconnects while idle;
- WebVTT reaches browser only through Gateway-safe same-origin capability paths;
- current deterministic media/subtitle proof path is a PREP harness and must not substitute for the product #44 session path in this Task.

## Integration gaps this Task is allowed to close

Current live main exposes three bounded composition gaps. They are **in scope only as generic integration glue**:

### G1 — Control source/session entry

The product Control surface must provide a bounded way to call accepted `POST /api/v1/sessions` without adding a second creation contract.

Allowed shape:

- minimal source input + existing live Display selector;
- use #44 DTO/error semantics unchanged;
- on success transition/navigate to `/control?session_id=<returned id>` and rebuild from server ControlView;
- local frontend state is form/presentation state only.

### G2 — Live Display selector discovery

If the current product has no usable way for Control to select a registered Display, add the smallest **read-only server-owned live Display view/list** needed for G1.

It may expose only bounded non-secret selector/label/capability/liveness metadata. It must not expose:

- lease token;
- page epoch as mutation authority;
- Playback display generation authority;
- attached Secret/media internals;
- arbitrary browser/storage identity as authority.

Do not redefine #45 registration/liveness semantics.

### G3 — Session → Display rendering view

The physical TV page must obtain the #44 prepared media/subtitle rendering facts from a server-owned view tied to the current accepted session/item/revision and #45 page lease.

A minimal solution may extend the existing Display context response or add a narrow Display rendering endpoint/view, but it must:

- derive current session/item/revision/display authority from accepted #45 + R007 server state;
- return only Gateway-safe media/subtitle paths and bounded metadata already prepared/owned server-side;
- never accept caller-supplied raw upstream URL/header/ResolvedMedia;
- never make browser state the media authority;
- reject stale/foreign lease/session/item/generation relationships;
- preserve the current #48 idle Display state when no session is attached.

If implementing G3 would require reopening #44 media preparation ownership or R007/#45 generation semantics, STOP and return for Coordinator semantic reclassification.

## Task Decomposition Decision

```text
Verification mode: inline hosted product E2E
Linked dependencies: #44 #45 #47 #48 Final Accepted
Linked physical TV verification: #7 remains separate
Linked phone resource verification: #9 remains separate
Linked Core Feasibility Gate: #22 remains separate
```

No new Issue is created merely because the integration uses browser + Rust + CI. Split only if execution reveals a genuinely independent scope/Evidence Authority.

## Claims

- **C1 — production entrypoints only:** the required success path begins at product `/`/`/control`/`/display` and uses accepted public/service APIs; no `seed_test_session`, arbitrary `ResolvedMedia` injection, direct session-store mutation, synthetic Display authority, or proof-only media path constructs the success path.
- **C2 — smart entry to real Display registration:** `/` → TV Display reaches #48 TV shell and a real #45 registration/heartbeat/lease; the same registered Display selector is discoverable/usable by Control without exposing lease/generation authority.
- **C3 — Control source to real session:** a bounded generic-direct source submitted from the product Control journey reaches accepted #44 Registry/resolve/media preparation/atomic publication and returns one real PlaybackSession bound to the selected live Display.
- **C4 — server-owned rendering composition:** the registered Display obtains only current Gateway-safe media/subtitle rendering facts for that session/item/revision; no raw upstream/Secret/local-file authority or browser-selected generation enters the rendering path.
- **C5 — visible media/subtitle render:** hosted Chromium Display consumes the product session rendering view, loads Gateway media, and when deterministic WebVTT is present loads it through a same-origin Gateway path; proof-only fixture injection is not the product authority.
- **C6 — Control command journey:** `/control?session_id=<id>` renders accepted server ControlView and executes play/pause/seek/stop through #38/R007 request-id/revision semantics; UI presentation state cannot overwrite server authority.
- **C7 — reconnect/refresh coherence:** Control refresh/event reconnect and Display refresh/lease reconnect rebuild from Gateway authority without resetting the session or allowing an old page lease/callback to regain rights.
- **C8 — failure seams:** invalid/no-match source, missing/offline/expired Display, stale command revision, incompatible request-id reuse, stale lease/callback, missing session and event resync produce bounded recoverable product behavior with no partial/duplicate authority.
- **C9 — security / hosted product scope:** DOM/console/network diagnostics/storage/artifacts contain no Cookie/Authorization/Vault/profile/lease token/raw protected upstream header or URL/arbitrary local path; hosted Chromium proves product integration mechanics only and does not claim physical-TV/phone/real-site/Core GO.

## In Scope

- minimal product Control source/session form/glue using accepted #44 API;
- minimal read-only live Display selector discovery if required;
- minimal server-owned session→Display rendering view/glue if required;
- wiring #48 TV page to the product rendering view rather than proof-only media authority;
- one deterministic generic-direct source and optional deterministic WebVTT through real Gateway paths;
- #47 Control journey over the newly created real session;
- Control/Display refresh/reconnect and bounded failure UX;
- deterministic hosted Chromium E2E harness/workflow;
- integration regressions across accepted #44/#45/#47/#48 and their authorities.

## Out of Scope

- physical TV autoplay/audible PASS/FAIL (#7);
- Ubuntu ARM64 phone resource/60-minute proof (#9);
- real Bilibili/site acceptance (#23/#36);
- real-network generic-ytdlp requirement (#50);
- Browser Worker/login/Native Site Panel runtime;
- Jellyfin client/DisplayAdapter proof;
- new Playback command/revision/item/media-generation/display-generation semantics;
- redesign of #44 creation DTO/idempotency/atomic publication;
- redesign of #45 lease/generation/callback authority;
- redesign of #47 ControlView projection or command semantics;
- advanced subtitle selection/styling/extraction;
- service restart persistence;
- Core Feasibility `GO | CONDITIONAL GO | NO-GO` decision.

## Architecture Invariants

1. Gateway/R007 remains the only PlaybackSession command/revision/item/media/display/handoff authority.
2. #45 page lease/registration state remains distinct from R007 display generation.
3. Control is View + Intent + bounded product form state, never a second playback/source/display state store.
4. Source recognition/resolution goes only through `SiteAdapterRegistry`; no concrete-site/yt-dlp branch in Core.
5. Product Display receives only server-owned Gateway-safe rendering paths; browser cannot inject raw upstream media/subtitle authority.
6. R001/R008 Egress/Secret/open-proxy boundaries remain intact.
7. Refresh/reconnect reconstructs from server authority; event/browser storage is not another source of truth.
8. Test/proof harnesses may support deterministic verification but may not be required to construct the product happy path.
9. Failure of hosted E2E must not be “fixed” by weakening accepted dependency semantics.

## Expected files

Likely integration surfaces only:

- `gateway-core/src/lib.rs` product routes/page composition;
- `gateway-core/src/source_session.rs` only if a **read-only safe rendering lookup/accessor** is required; do not change #44 creation semantics;
- `gateway-core/src/display_session.rs` only for bounded read-only Display discovery/rendering composition while preserving #45 authority;
- `gateway-core/src/control.rs` only for read-only server-safe accessors if needed; do not change R007 semantics;
- browser E2E harness/scripts;
- `.github/workflows/web-mvp-e2e-prep.yml`;
- focused tests.

Do not alter canonical architecture merely to make the test easy. If required behavior contradicts canonical docs, stop and return for Coordinator Review.

## Implementation Requirements

1. Start from current accepted main and reuse existing APIs before adding any new surface.
2. Implement only G1/G2/G3 that are actually missing after live code inspection.
3. Any new read DTO must be bounded, Secret-free, generation-safe and server-produced.
4. The E2E success path must not use `seed_test_session`, direct store mutation, arbitrary `ResolvedMedia`, proof-only `/proof/paths`, display-probe authority, or raw fixture URL injection as product state.
5. Deterministic upstream media/subtitle fixtures are allowed **behind accepted generic-direct/R001 Gateway paths** to keep required CI reproducible and network-independent.
6. Product Control may retain only form/presentation cache; after creation/command/error it rebuilds from authoritative API state.
7. Product Display must be able to remain idle and later attach/render the created session without reload-dependent hidden authority.
8. Preserve explicit bounded error codes and no-side-effect behavior from accepted components.
9. Produce one exact Task Candidate and a durable PR before required Evidence.

## Verification Plan

### Verification Job Matrix

| Job | Claims | Runner | Required | Intent |
|---|---|---|---|---|
| J1 | C1-C7 | github-hosted `ubuntu-latest` + headless Chromium | yes | complete hosted product happy path: `/` → Display register → Control live-display/source create → real session/render view → media/subtitle → play/pause/seek/stop → refresh/reconnect |
| J2 | C1,C4,C7-C9 | github-hosted `ubuntu-latest` + headless Chromium | yes | production-route guard + invalid source/display + stale revision/request-id/lease/callback + Display/Control reconnect + event resync + browser Secret/storage/network negatives |
| J3 | C1-C9 | github-hosted `ubuntu-latest` | yes | fmt/clippy/workspace + exact #44/#45/#47/#48/#40/#38/R007/R001/R008/plugin/media/security regression selectors |

All required jobs must checkout and assert the exact Task Candidate SHA.

### Hosted happy-path evidence must show

```text
GET /
→ /display?profile=tv
→ real #45 registration_id/display_id/lease lifecycle (lease value redacted)
→ product Control discovers/selects that Display
→ POST /api/v1/sessions {request_id, source, display_id}
→ returned opaque session_id
→ Display attaches to that session using server-owned context
→ Display receives safe Gateway media/subtitle rendering view
→ browser loads media + optional WebVTT from same-origin Gateway path
→ /control?session_id=<id>
→ play → pause → seek → stop
→ Control refresh/event reconnect
→ Display page refresh/reconnect
→ same authoritative session/item remains coherent
```

Evidence may record opaque IDs/hashes but must not record lease token or raw upstream Secret/media URL.

## Success Criteria

1. C1-C9 have exact-Candidate Evidence.
2. One deterministic hosted Chromium run constructs the success path entirely through product/public routes and accepted service boundaries.
3. Control can create a real session against a real registered Display without manual server-store/test-fixture mutation.
4. Display receives and renders the current session's server-owned Gateway media path; optional subtitle smoke uses a same-origin Gateway WebVTT path.
5. Play/pause/seek/stop and both Control/Display reconnect paths preserve one authoritative PlaybackSession.
6. Failure matrix proves stale/missing/invalid cases do not create duplicate/partial authority or Secret/browser leakage.
7. Workspace and accepted dependency regression suites pass on the exact Candidate.
8. No accepted #44/#45/#47/#48/R007/R001/R008 semantic boundary is weakened to obtain E2E PASS.

## Freshness / Integration Contract

Freshness policy: dependency-aware

Planning / Evidence Base:
- `d6c53d3b2d1b954132db90872afb8e7a63201442`

Semantic authorities:
- #44 source→session creation/idempotency/atomic publication + safe media preparation;
- #45 Display registration/lease/context/callback authority;
- #47/#40 ControlView + #38/R007 command/snapshot/events semantics;
- #48 smart entry/TV Display/subtitle/degradation behavior;
- R001/R008 media capability/Egress/Secret boundary;
- #39 SiteAdapter/ResolvedMedia conformance.

Semantic freshness domains:
- `gateway-core/src/source_session.rs` creation semantics;
- `gateway-core/src/display_session.rs` lease/generation/callback semantics;
- `gateway-core/src/control.rs`, `control_view.rs`, `playback.rs` command/read authority;
- `site-adapter-api/**` ResolvedMedia/conformance;
- `gateway-core/src/security.rs` and R001 capability path.

Task-owned integration surfaces:
- product source/create form glue;
- live Display read/discovery glue;
- safe session→Display rendering view;
- E2E browser harness/workflow.

Integration surfaces:
- `gateway-core/src/lib.rs` router/page/service composition;
- any bounded read-only accessor added to accepted services;
- browser product JS/UI wiring;
- workspace test composition.

Authority/domain → Claim mapping:
- #44: C1,C3,C4,C8,C9
- #45: C2,C4,C5,C7,C8,C9
- #47/#40/#38/R007: C6,C7,C8,C9
- #48: C2,C5,C7,C9
- R001/R008/#39: C3,C4,C5,C9

Integration verification if main advances after semantic Evidence:
- JI1: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --all-targets`
- JI2: hosted product happy-path smoke + targeted #44/#45/#47/#48 regression selectors on one Coordinator-frozen Integration Base.

Unrelated-main policy:
- preserve exact-Candidate semantic Evidence; no full rerun solely because main advanced.

Integration-overlap policy:
- preserve accepted semantic Evidence; compose with Coordinator-frozen Integration Base and run JI1/JI2 unless conflict changes Task semantics.

Semantic-authority-change policy:
- stop/reconcile changed accepted authority and rerun mapped affected Claims; Worker may not silently reinterpret a dependency contract as integration glue.

Strict-main reason: n/a

## Evidence Contract

Required `[EXECUTION REPORT]` must record at minimum:

```text
Attempt:
Worker / Environment:
Base / Candidate commit:
PR:
Execution outcome:
Production path used:
Display registration/selector evidence (lease redacted):
CreateSession request shape / result summary:
Session/item/revision identity summary:
Display rendering-view/Gateway-path evidence:
Media/subtitle browser result:
Control play/pause/seek/stop result:
Control refresh/event reconnect result:
Display refresh/lease reconnect result:
Failure matrix:
Browser DOM/console/network/storage Secret scan:
C1-C9:
J1/J2/J3 run/job IDs and exact checkout:
Freshness identities:
Problems / limitations:
```

Never include Cookie, Authorization, Vault/profile material, lease token, raw protected upstream URL/header, `upstream_access_ref`, local filesystem path, password/code/QR data or unnecessary copyrighted media payload.

## Failure / Blocked Handling

- **FAIL:** required product path is executable but accepted components cannot compose without violating an Architecture Invariant, or the candidate fails a required Claim/Job.
- **BLOCKED:** required GitHub Actions/browser capability, dependency authority, or reproducible deterministic fixture is unavailable and no safe in-scope substitute exists.
- A normal implementation/test bug is `REVISE` on the same Issue, not a new Task.
- If a true independent blocker/evidence authority appears, Coordinator may SPLIT; Worker does not self-split or lower Success Criteria.

## Deliverables

- minimal product integration glue required by G1/G2/G3;
- deterministic hosted browser E2E harness;
- `.github/workflows/web-mvp-e2e-prep.yml` or equivalent exact-Candidate workflow;
- Candidate commit + PR;
- J1/J2/J3 Evidence;
- standard Issue `[EXECUTION REPORT]`.

## Completion Protocol

```text
claim → status:in-progress → Attempt N
→ minimal integration implementation + exact-SHA J1/J2/J3
→ [EXECUTION REPORT] → status:review → release owner → STOP
```

Worker must not merge its own PR, set `status:done`, close #49, execute #7/#9/#22/#23/#36/#50, or automatically start another Task/Attempt.

Coordinator acceptance of #49 means only: **hosted Web-only MVP integration mechanics are coherent and reproducible**. It is not physical-TV acceptance, phone-resource acceptance, real-site acceptance, production readiness, or Core Feasibility GO.
