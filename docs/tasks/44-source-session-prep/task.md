# Task — SOURCE-SESSION-PREP Source request → atomic PlaybackSession publication

## Metadata

```text
GitHub Issue: #44
Parent Goal / Research Item: Phase 0A-2 / close Web-only source → session gap
Task / Research ID: SOURCE-SESSION-PREP
Task kind: combined
Planning base: d0cd647d8965c03c412d67a7f6cea9b33fa2ec38
Session bootstrap prompt: docs/tasks/44-source-session-prep/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, rust-code-authoring, rust-build-test, github-actions-orchestration
Hard publication dependencies: #39 accepted; #45 accepted; #38/R007 accepted; R001/R008 accepted authority available
```

Realtime status/owner/branch/PR/Evidence lives in Issue #44. Attempt history follows `docs/tasks/issue-lifecycle-protocol.md`.

## Goal

Implement the smallest production source-to-session path that accepts bounded user source input, resolves it only through `SiteAdapterRegistry`, validates an already-registered live Web Display from accepted #45, prepares R001-scoped media capabilities, constructs a server-owned PlaybackSession, and publishes the session atomically through the existing #38/R007 control authority.

```text
bounded CreateSessionRequest
→ #45 registered/live display selector validation
→ SiteAdapterRegistry.recognize
→ opaque SourceLocator
→ SiteAdapterRegistry.resolve
→ validated ResolvedMedia
→ reserve server-owned session/item identity
→ prepare safe session-bound media view/capabilities
→ construct PlaybackSession
→ atomic publish
→ opaque session_id + safe summary
→ existing snapshot/command/events APIs
```

## Current accepted facts

- #45 Web Display registration/liveness is independent of PlaybackSession existence; page lease and display generation remain separate.
- `DisplaySessionService` currently has no production read-only idle-display selector accessor; this Task may add only a narrow internal/read-only accessor that returns safe registration/liveness facts and never lease token, session attachment, display generation or Playback authority.
- `ControlService` has no production session-create entry; `seed_test_session` remains test-only.
- R001 exposes server-side `Binding`, `resource_from_resolved()` and scoped `issue_path()`; browser-safe media paths must not reveal raw upstream URL/header/Secret material.
- R007 remains sole command/session-revision/item/display-generation/handoff authority.

## Task Decomposition Decision

```text
Verification mode: inline
Linked verification task: n/a
Physical TV / phone / real-site verification: separate existing tasks
Reason: source/session mechanics and security boundaries are deterministic and GitHub-hosted.
```

## API / service contract

Add the product session-creation surface:

```text
POST /api/v1/sessions

CreateSessionRequest
├── request_id: bounded identifier
├── source: bounded source input
└── display_id: bounded selector for an existing #45 registration

CreateSessionResponse
├── request_id
├── session_id
├── item_id
├── item_revision
├── session_revision
├── display_id
├── source_site
└── media: safe SessionMediaView
```

`SessionMediaView` is browser-safe and contains only generic metadata plus Gateway-owned paths/capabilities. It must not expose `SourceLocator.opaque_payload`, raw upstream URL, Cookie/Authorization, `upstream_access_ref`, Egress scope, Vault material, page lease, or caller-controlled header authority.

A minimal safe shape may include:

```text
SessionMediaView
├── session_id / item_id / item_revision / media_generation
├── title / source_site
└── streams[]
    ├── id
    ├── protocol
    └── gateway_path
```

Do not invent quality-selection semantics beyond what current accepted `ResolvedMedia` can safely expose. Preserve resolved stream order; browser/display selection policy remains a later UX concern unless only one playable stream exists.

## Claims

- **C1 — bounded ingress:** public creation accepts only `request_id + source + display_id`; it cannot accept `ResolvedMedia`, raw headers, `SourceLocator`, Vault/Egress fields, revision/generation, lease or arbitrary media path.
- **C2 — registered Display selector:** `display_id` is only a selector. Gateway validates an unexpired accepted #45 registration through a narrow server-side read-only API; caller cannot mint registration identity, lease, page epoch, display generation or active-display authority.
- **C3 — Registry-only routing:** source recognition and resolution use `SiteAdapterRegistry` only. Stable Core gains no Bilibili/YouTube/yt-dlp branch and never parses locator opaque payload.
- **C4 — safe resolved-media preparation:** only conformance-validated `ResolvedMedia` enters preparation. R001 capabilities bind to reserved `session_id + item_id + item_revision + resource_id`; public media view contains only Gateway paths and safe metadata.
- **C5 — atomic publication:** externally visible session publication occurs only after display validation, recognize/resolve, media preparation and PlaybackSession construction all succeed.
- **C6 — deterministic cleanup:** any pre-publication failure leaves no visible session and no reusable orphan media capability/resource. Add internal revoke/rollback support if required; do not rely only on eventual TTL for the required test.
- **C7 — creation idempotency:** same creation `request_id` + same fingerprint returns the same successful/error outcome without creating a second session/capability set; incompatible reuse returns stable `CREATE_REQUEST_ID_MISMATCH` (or documented equivalent). This namespace is separate from R007 playback-command request IDs.
- **C8 — R007/#38 compatibility:** successful creation immediately supports existing GET snapshot, POST command and event reconnect APIs; no new command/revision semantics are introduced.
- **C9 — security/public boundary:** R008 Host/Origin/body limits apply; source/locator/upstream Secret data is redacted from Debug/log/Evidence and is never reflected into browser-visible DTOs.

## In Scope

- source/session service and HTTP `POST /api/v1/sessions`;
- bounded creation request validation and idempotency store;
- dependency-injected `SiteAdapterRegistry` composition boundary without concrete-site Core branches;
- narrow internal #45 registered/live display selector lookup;
- server-owned session/item IDs;
- safe R001 media capability preparation and safe `SessionMediaView`;
- atomic `ControlService` publication through a crate-internal validated/prepared constructor path;
- deterministic rollback/cleanup;
- tests/workflow and architecture guards.

## Out of Scope

- real Bilibili/YouTube/login or generic-ytdlp runtime networking;
- Control UI (#47);
- TV viewport/subtitles/fullscreen/autoplay UX (#48/#7);
- new Display registration/lease/generation semantics;
- new R007 command/revision/handoff semantics;
- next/previous item navigation;
- persistence across Gateway restart;
- arbitrary public raw-media/session constructor.

## Architecture Invariants

1. Core uses Registry; concrete plugin behavior stays outside Core.
2. `SourceLocator.opaque_payload` remains plugin-owned.
3. #45 owns page registration/liveness; R007 owns Playback/display generation.
4. A read-only display selector lookup must not expose or validate via browser lease possession.
5. Session creation may reserve identities before publication, but they are not externally authoritative until atomic publish.
6. R001 media capabilities remain short-lived, scoped and server-side; no open proxy.
7. `ControlService` production creation must be crate-internal/validated and must not reintroduce a general public `seed_session(resolved_media, ...)` path.

## Expected files

Likely:

- `gateway-core/src/source_session.rs` or equivalent;
- `gateway-core/src/lib.rs` composition/router/media view wiring;
- `gateway-core/src/control.rs` narrow internal prepared-session publication only;
- `gateway-core/src/display_session.rs` narrow safe read-only registration/liveness lookup only;
- tests + `.github/workflows/source-session-prep.yml`;
- composition root/tests for Registry injection.

Do not change canonical architecture to fit an implementation shortcut.

## Verification Plan

| Job | Claims | Runner | Required | Intent |
|---|---|---|---|---|
| J1 | C1-C5,C8 | github-hosted ubuntu-latest | yes | generic-direct source → locator → resolve → registered display → media prep → atomic session → snapshot/command happy path |
| J2 | C2,C5-C7,C9 | github-hosted ubuntu-latest | yes | invalid/offline display, no-match, ambiguous/invalid adapter output, duplicate/mismatched create request, injected raw media/headers/generation/lease, mid-creation rollback/Secret negatives |
| J3 | C1-C9 | github-hosted ubuntu-latest | yes | workspace + #45/#39/#38/R007/R001/R008/security/architecture regressions |

All required jobs assert exact Task Candidate SHA.

## Success Criteria

1. C1-C9 have executable exact-Candidate Evidence.
2. A deterministic generic-direct input creates one real production ControlService session bound to an existing live #45 display.
3. The browser-safe creation result contains only Gateway media paths/safe metadata and works with existing snapshot/command/events APIs.
4. Duplicate creation is idempotent; incompatible reuse is rejected.
5. Injected raw media/Secret/display-generation/lease authority is impossible through the public DTO.
6. A forced failure after capability preparation but before publication proves rollback: no visible session and no surviving issued capability for the failed attempt.
7. Stable Core contains no concrete-site/yt-dlp branch.

## Freshness / Integration Contract

Freshness policy: dependency-aware

Semantic authorities:
- #45 accepted `DisplaySessionService` registration/liveness ownership;
- #39 `SiteAdapterRegistry` / `SourceLocator` / `ResolvedMedia` conformance;
- #38/R007 `ControlService` / `PlaybackSession` command, revision and display authority;
- R001/R008 media capability, Egress and Secret boundaries.

Semantic freshness domains:
- `gateway-core/src/display_session.rs` registration/liveness semantics;
- `gateway-core/src/control.rs`, `gateway-core/src/playback.rs` creation/authority semantics;
- `site-adapter-api/**` Registry/locator/media contract;
- `gateway-core/src/security.rs` and R001 media capability contract.

Integration surfaces:
- `gateway-core/src/lib.rs` router/state composition;
- `gateway-core/src/control.rs` shared service wiring;
- `Cargo.toml` / `Cargo.lock` / workspace test closure;
- HTTP authority middleware and common API routing.

Task-owned surfaces:
- source/session creation service, creation DTO/idempotency, safe media view, narrow validated publication/rollback path and related tests/workflow.

Authority/domain → Claim mapping:
- #45 display registration/liveness: C2,C5,C8
- SiteAdapter/ResolvedMedia: C3,C4,C9
- R007/#38: C5,C7,C8
- R001/R008: C4,C6,C9

Integration verification:
- JI1: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --all-targets`
- JI2: targeted source-session + HTTP security + #45 display-session regression selectors from the accepted workflow/tests.

Unrelated-main policy:
- existing exact-Candidate semantic Evidence remains valid; no rebase/full rerun solely because main advanced.

Integration-overlap policy:
- preserve accepted semantic Evidence; compose with Coordinator-frozen Integration Base and run only JI1/JI2 unless conflict changes Task semantics.

Semantic-authority-change policy:
- reconcile current accepted authority and rerun only mapped affected Claims when impact is safely bounded.

Strict-main reason: n/a

## Evidence Contract

`[EXECUTION REPORT]` must include Task Candidate SHA/PR, Evidence Base/observed current main, source-session API location, display-selector proof, Registry path, atomic-publication/rollback proof, idempotency result, safe media-view/Secret result, C1-C9, J1/J2/J3 run+job IDs and limitations.

No source-site Secret, raw sensitive URL/query, Cookie/Authorization, `upstream_access_ref`, lease token, raw locator payload or production account data may appear in Evidence.

## Completion Protocol

```text
claim → status:in-progress → Attempt N
→ candidate + exact-SHA J1/J2/J3
→ [EXECUTION REPORT] → status:review → release owner → STOP
```

Worker never merges its own PR, sets `status:done`, closes #44, executes #47/#48, or silently changes this Contract.
