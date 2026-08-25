# Task — CONTROL-UI-PREP Real `/control` on accepted ControlView + command APIs

## Metadata

```text
GitHub Issue: #47
Parent Goal / Research Item: Phase 0A-2 / first usable Control experience
Task / Research ID: CONTROL-UI-PREP
Task kind: combined
Planning base: d0cd647d8965c03c412d67a7f6cea9b33fa2ec38
Session bootstrap prompt: docs/tasks/47-control-ui-prep/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, rust-code-authoring, browser-ui-authoring, github-hosted-headless-browser-verification
Hard publication dependencies: #40 accepted; #45 accepted; #38/R007 accepted
```

Realtime status/owner/branch/PR/Evidence lives in Issue #47.

## Goal

Replace the R002 probe-only `/control` page with the first real product Control surface driven by server-produced accepted #40 `ControlView`, accepted #45 Display facts and existing #38/R007 command/events APIs, without creating a second frontend authority.

```text
/control?session_id=<id>
→ GET server-produced ControlView
→ render Now Playing / Display / Site / action_required
→ command with request_id + expected_session_revision
→ existing #38/R007 command endpoint
→ events/reconnect signal
→ fetch fresh ControlView
```

#44 source/session creation is not a hard dependency. The UI may operate on any existing real session; deterministic browser tests may use an internal test harness to create one, but production must not expose a fixture/session-seed endpoint.

## Current accepted facts

- #40 `ControlView::project(ControlViewInput)` is pure/read-only and retains domain-owned freshness; no global Control revision exists.
- #45 `DisplaySessionService::display_view_input()` supplies generic safe `DisplayInstance/Status/Error` facts and preserves stale-context filtering.
- #38 exposes GET snapshot, POST commands and event reconnect/cursor semantics.
- Current `/control` is only an R002 probe UI and must no longer be the product Control implementation after this Task.

## Product/API contract

Add a server-side read-model surface, for example:

```text
GET /api/v1/control/{session_id}
→ ControlView
```

The exact path may use the sessions namespace if cleaner, but there must be exactly one documented product read-model endpoint and `/control` must consume it.

Server projection flow:

```text
ControlService.snapshot(session)
+ event cursor
+ #45 display_view_input(active display)
+ safe Site/Browser domain facts when available
→ ControlView::project
→ JSON
```

Missing Site/Browser domains are represented as their accepted unavailable/default states; do not fabricate login/panel/display truth. A domain failure must not prevent otherwise valid Playback/Display projection unless the frozen #40 contract requires it.

`/control` takes a bounded session selector (`session_id` query/path or another explicit safe selector). It must not guess an arbitrary session when more than one exists.

## Claims

- **C1 — server-side projection authority:** browser renders serialized accepted `ControlView`; it does not independently derive Playback/Display/Site action semantics from raw snapshots.
- **C2 — real Display facts:** active display online/status/error comes from accepted #45 `display_view_input` and #40 context filtering; frontend cannot self-assert liveness/generation.
- **C3 — command envelope:** Play/Pause/Seek/Stop use existing #38 `request_id + expected_session_revision` envelope and existing command endpoint. No new mutation API or optimistic success authority.
- **C4 — reconnect/resync:** refresh and event disconnect/cursor truncation rebuild from a fresh server `ControlView`. Event delivery is a wake-up/freshness signal, not a second state store.
- **C5 — conflict UX:** `REVISION_CONFLICT`, `REQUEST_ID_MISMATCH`, session-not-found, display-domain unavailable/error and command rejection have explicit bounded UI recovery; stale local view is discarded/resynced rather than patched as authority.
- **C6 — failure isolation:** Site/Browser/NativePanel failure does not stop already-started Playback; Display failure is shown through accepted ControlView/action-required semantics rather than a UI-only rule.
- **C7 — presentation-only local state:** frontend state is limited to current safe ControlView, request-in-flight state and presentation preferences; it does not create a local session revision, display generation or authoritative playback state.
- **C8 — browser/Secret safety:** DOM, console, network diagnostics, local/session storage and test artifacts contain no raw `ResolvedMedia`, upstream URL/header Secret, Vault/profile material, panel token, password/code/QR/browser text or source-site credentials.
- **C9 — deterministic browser proof:** hosted headless browser verifies rendering, commands, conflict resync, event reconnect and current display flows through production routes; no physical-TV claim is made.

## In Scope

- real `/control` page shell and accessible rendering of Now Playing, controls, active Display, Site/account state and action-required;
- one server-side ControlView endpoint/provider;
- integration of #45 display facts into projection;
- existing #38 command/events transport consumption;
- deterministic session selector and resync UX;
- browser-safe error/status presentation;
- headless browser tests/workflow.

## Out of Scope

- public source/session creation (#44); do not silently absorb it;
- Native Site Panel transport/runtime;
- real site login;
- TV remote-focus/fullscreen/autoplay (#48/#7);
- concrete Bilibili/Jellyfin branches;
- new Playback/R007 mutation/revision semantics;
- persistence/offline-first frontend state.

## Architecture Invariants

1. Control is a projection/interaction layer, not a second domain store.
2. `ControlView` stays server-produced; frontend cache is disposable.
3. Existing R007 request/revision semantics remain the only Playback mutation authority.
4. #45 remains Display liveness/context authority.
5. UI must not interpret raw protocol errors or source-site Secret data.
6. No production fixture/session-seed endpoint may be added to make E2E convenient.

## Expected files

Likely:

- `gateway-core/src/lib.rs` `/control` and ControlView endpoint wiring;
- a focused ControlView provider/helper module if needed;
- browser UI assets embedded or served by Gateway;
- deterministic browser tests + `.github/workflows/control-ui-prep.yml`.

Avoid modifying `control.rs`, `playback.rs`, `control_view.rs` or `display_session.rs` semantics unless a narrow additive accessor is genuinely required; any such change is semantic and must be justified/tested.

## Verification Plan

| Job | Claims | Runner | Required | Intent |
|---|---|---|---|---|
| J1 | C1-C4,C7,C9 | github-hosted ubuntu-latest + headless Chromium | yes | render authoritative view, play/pause/seek/stop, display facts, refresh and event reconnect |
| J2 | C5,C6,C8,C9 | github-hosted ubuntu-latest + headless Chromium | yes | revision conflict/request mismatch/session missing/display failure/domain isolation/DOM-console-storage-Secret negatives |
| J3 | C1-C9 | github-hosted ubuntu-latest | yes | workspace + #45/#40/#38/R007/R008/HTTP security regressions |

Required Evidence runs on exact Task Candidate SHA.

## Success Criteria

1. `/control` no longer operates the R002 probe command store as its product authority.
2. A hosted browser renders a real server-generated `ControlView` for an existing session and controls it through accepted command APIs.
3. Refresh/event reconnect reconstructs authoritative UI state from Gateway.
4. Same-old-revision competing command/conflict path visibly resyncs instead of claiming success.
5. Current #45 display liveness/status/error is rendered without frontend-generated display truth.
6. No public session seed or Secret-bearing browser state is introduced.
7. C1-C9 and J1/J2/J3 pass.

## Freshness / Integration Contract

Freshness policy: dependency-aware

Semantic authorities:
- #40 `ControlView` projection/freshness contract;
- #38/R007 Control command/snapshot/events/revision semantics;
- #45 Web Display registration/liveness/contextual DisplayViewInput;
- R008 HTTP authority/Secret boundary.

Semantic freshness domains:
- `gateway-core/src/control_view.rs`;
- `gateway-core/src/control.rs` command/snapshot/events contract;
- `gateway-core/src/display_session.rs` display-view facts;
- `gateway-core/src/security.rs` / shared HTTP guard semantics.

Integration surfaces:
- `gateway-core/src/lib.rs` router/state/static page composition;
- shared HTTP routes and browser assets;
- `Cargo.toml` / `Cargo.lock` / workspace test closure.

Task-owned surfaces:
- product `/control` UI, server ControlView provider/endpoint, frontend resync/command glue, browser tests/workflow.

Authority/domain → Claim mapping:
- #40 projection: C1,C5,C6,C7
- #38/R007: C3,C4,C5,C7
- #45 Display: C2,C5,C6
- R008/browser safety: C8,C9

Integration verification:
- JI1: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --all-targets`
- JI2: targeted Control UI headless smoke + accepted #45 display-session and HTTP-security regression selectors.

Unrelated-main policy:
- existing exact-Candidate semantic Evidence remains valid; no rebase/full rerun solely because main advanced.

Integration-overlap policy:
- preserve accepted semantic Evidence; compose with Coordinator-frozen Integration Base and run JI1/JI2 only unless conflict changes Task semantics.

Semantic-authority-change policy:
- reconcile changed #40/#38/R007/#45/R008 authority and rerun mapped affected Claims.

Strict-main reason: n/a

## Evidence Contract

Report Task Candidate SHA/PR, Evidence Base/observed main, ControlView endpoint/provider, browser route and session selector, command/reconnect result, conflict/failure matrix, browser Secret scan, C1-C9 and exact J1/J2/J3 job IDs.

No source-site Secret, raw media upstream URL/header, lease token, Vault/profile/panel token, password/code/QR or sensitive browser text may appear in Evidence.

## Completion Protocol

```text
claim → status:in-progress → Attempt N
→ candidate + exact-SHA J1/J2/J3
→ [EXECUTION REPORT] → status:review → release owner → STOP
```

Worker never merges its own PR, sets `status:done`, closes #47, executes #44/#48, or starts another Task automatically.
