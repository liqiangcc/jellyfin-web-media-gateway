# Task — CONTROL-VIEW-PREP canonical ControlView projection

## Metadata

```text
GitHub Issue: #40
Parent Goal / Research Item: Phase 0A-2 / unified Control experience
Task / Research ID: CONTROL-VIEW-PREP
Task kind: combined
Base commit: ac425ef56b420993a07d915f746c71c775262c71
Candidate commit: n/a until Worker execution
Session bootstrap prompt: docs/tasks/40-control-view-prep/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, rust-build, rust-test, github-actions-orchestration
Hard publication dependencies: Issue #38 / CONTROL-SHELL-PREP Final Acceptance — satisfied
```

Realtime status/owner/branch/PR/Evidence lives only in Issue #40. Attempt history follows `docs/tasks/issue-lifecycle-protocol.md`.

## Goal

Implement a canonical **read-only** `ControlView` projection that composes accepted domain snapshots/status into the unified Control experience without creating a second mutable authority for Playback, Site Account, Display or Browser state.

```text
accepted #38 ControlSnapshot/cursor ─┐
#28 AccountState/PendingIntent       ─┤
generic DisplayInstance/Status       ─┼→ ControlView projection
#33 BrowserStatus/NativePanel status ─┘

ControlView
├── now_playing
├── playback_controls
├── playback_context
├── active_display
├── site
├── site_account_state
├── native_site_panel
└── action_required
```

## Context / Accepted Inputs

Canonical authority:

- `docs/control-experience-architecture.md`
- accepted #38 / Control shell and safe `ControlSnapshot` / reconnect cursor semantics
- accepted #2 / R007 Playback authority
- accepted #28 `AccountState` / non-secret auth/PendingIntent contracts
- accepted #33 `BrowserStatus`, generic Browser failure taxonomy and `NativePanelSession` boundary
- accepted generic `DisplayContext`, `DisplayInstance`, `DisplayStatus` contracts from R004-PREP
- accepted R008 Secret/log boundary

Current accepted facts include:

- `ControlSnapshot` is safe and contains Playback state/item revision/media generation/position/telemetry/active-display/handoff summary but not raw `resolved_media`;
- `AccountState` is generic: Unknown/Checking/Valid/Expired/LoginRequired/Error;
- Browser status/error types are generic and NativePanel token/profile material is not publicly readable;
- Display API uses generic adapter-neutral `DisplayContext`/`DisplayInstance`/`DisplayStatus`.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: projection/failure-isolation/rebuild/Secret Claims are deterministic and can be verified with accepted DTOs + fake domain snapshots in GitHub-hosted CI.
```

## Worker Routing

```text
implementation: cloud-codex / env:cloud
verification: GitHub Actions / github-hosted ubuntu-latest x64
```

No real Browser Worker, login, TV, phone, Jellyfin client or real site is required.

## Claims

- C1 — `ControlView` is read-model-only: derived from authoritative inputs, with no command executor, persistence authority or independent mutable playback truth.
- C2 — Playback projection consumes accepted #38 `ControlSnapshot` and exposes stable safe Now Playing/control context without raw media/upstream data.
- C3 — Site account projection consumes generic #28 `AccountState`/non-secret PendingIntent metadata and maps login-required/error/action-required state without Vault/profile Secret.
- C4 — Browser/NativePanel projection consumes generic #33 status/failure/session availability without DOM/API/site-specific semantics or panel token exposure.
- C5 — Display projection consumes generic Playback active-display authority plus optional generic Display status; Web vs Jellyfin is represented only as adapter metadata, not projection branches.
- C6 — independent failure isolation is executable: Browser failure leaves Playback projection usable; Display offline leaves Site/Panel projection available; Site auth required does not reset Playback; one domain failure does not erase unrelated accepted state.
- C7 — refresh/reconnect rebuild from fresh authoritative snapshots yields the same current projection without needing local mutable history or replaying commands.
- C8 — freshness metadata remains domain-specific: R007 session/item/media/display generations and #38 event cursor are carried only for reconciliation; ControlView does not invent a global revision that overrides them.
- C9 — serialization/Debug/error output is Secret-safe: no raw `resolved_media`, Cookie/Authorization, Vault/profile contents, Jellyfin API key, signed upstream URL query, password/code/QR, Browser text input or panel control token.

## Preconditions

- #38 Final Accepted Candidate `6563cbdea574b49de1f4705fb6611363ee77a04c`, merged as `aebf921b6876616dcf791ecae7d894ea0bb847c7`.
- #28, #33 and R004 generic adapter contracts remain accepted/current.
- Worker re-reads live main before claim and integrates newer accepted main before exact-Candidate Evidence if necessary.

## In Scope

- `ControlView` and explicitly named safe sub-view DTOs;
- pure/deterministic projection functions/builders from accepted domain inputs;
- generic `ActionRequired` taxonomy based on accepted domain states/errors;
- safe playback-control availability projection derived from playback state/transition context, not a second command implementation;
- generic active-display view and optional online/observation metadata;
- generic site account status view;
- generic NativePanel availability/failure view, with no token/profile/input content;
- source/domain freshness markers sufficient for stale-input rejection or deterministic rebuild;
- deterministic failure-isolation and reconnect/rebuild tests;
- exact-Candidate GitHub-hosted verification.

## Out of Scope

- full `/control` frontend, CSS/mobile UX or visual design;
- HTTP command semantics already owned by #38/R007;
- source URL recognition/resolution or public create-session flow;
- real Chromium/Browser Worker lifecycle or real login;
- real TV/Jellyfin/phone verification;
- Bilibili/YouTube-specific fields;
- persistence/database/cache making ControlView authoritative;
- new WebSocket/SSE/event transport semantics;
- new global `control_revision` or replacement for R007/domain revisions.

## Architecture Invariants

1. Projection consumes authority; it does not become authority.
2. Playback mutations continue through accepted #38/R007 paths only.
3. ControlView may carry `session_revision`, `item_revision`, `media_generation`, `display_generation`, telemetry sequence and event cursor, but cannot reinterpret them into one global commit authority.
4. Site account/Vault, Browser Worker, Display and Playback can fail independently.
5. No projection branch may contain concrete Bilibili/YouTube/Jellyfin business semantics; adapter/site labels are metadata only.
6. NativePanel session/token/profile inputs remain opaque; ControlView exposes availability/capability state only.
7. Safe projection excludes raw media URL/headers/Secrets and sensitive Browser inputs.
8. Reconnect means fetch current authoritative snapshots then rebuild projection; it does not replay old mutations.

## Files Expected to Change

Likely:

- `gateway-core/src/control_view.rs` or equivalent;
- `gateway-core/src/lib.rs` for safe public projection DTOs only;
- `gateway-core/src/control.rs` only for non-authoritative safe input/accessor reuse if needed;
- deterministic `control_view` tests;
- `.github/workflows/*control-view*`.

Avoid changing accepted domain contracts unless a genuine contradiction is found and reported.

## Implementation Requirements

1. Prefer a pure projection API such as `ControlView::project(inputs)` / `project_control_view(...)`; do not create an authoritative ControlView store.
2. Define explicit input wrappers for accepted domain snapshots/status if needed, without adding mutable domain truth.
3. Playback section must derive from #38 `ControlSnapshot`; do not expose a `resolved_media` field or arbitrary media URL.
4. Site account section must consume `AccountState` and only non-secret identifiers/labels/action metadata; never expose `SiteSessionRef` secret material or Vault internals.
5. Browser/NativePanel section must expose safe availability/status/error code only; no `PanelControlToken`, profile ref value, raw navigation query, page text/title, password/code/QR or `BrowserInput::Text` value.
6. Display section must use generic display identity/type/online/observation metadata; no `if jellyfin` / `if web_display` behavior branch in projection logic.
7. Define deterministic precedence for `action_required` when multiple domains require attention. Precedence must be presentation-oriented only and must not mutate/override domain authority. Tests must cover multiple simultaneous conditions.
8. Stale input handling must use the source domain's own revision/generation/sequence; do not introduce a global Control revision.
9. Rebuild test must construct the current view from fresh snapshots and prove equality with the view after applying equivalent accepted newer domain inputs, without command replay/local truth dependency.
10. Add Secret/debug sentinel tests with fake values in every sensitive source boundary.
11. Final Candidate integrates live accepted main and required Jobs run on exact final SHA.

## Verification Plan

| Job | Claims | Execution plane | Runner | Required | Selector / intent |
|---|---|---|---|---|---|
| J1 | C1-C5,C7 | GitHub Actions | ubuntu-latest | yes | fmt/clippy + projection/rebuild deterministic contracts |
| J2 | C6,C8,C9 | GitHub Actions | ubuntu-latest | yes | failure isolation, stale/freshness, action precedence, Secret/debug negatives |
| J3 | C1-C9 regressions | GitHub Actions | ubuntu-latest | yes | workspace/all-targets + affected #38/R007/#28/#33/display/R008 regressions |

```text
Target proof required: no
Interactive external debugging: no by default
```

## Success Criteria

1. C1-C9 have explicit exact-Candidate Evidence.
2. `ControlView` can be rebuilt deterministically from accepted authoritative inputs and has no independent mutation/store authority.
3. Browser/Display/Site failures are isolated as required by canonical Control architecture.
4. No new global Control revision is introduced.
5. No raw media/Secret/browser-input/panel-token material appears in DTO/serialization/Debug/errors.
6. No concrete-site/display-adapter business branch enters projection Core.
7. J1/J2/J3 green on exact final Candidate.
8. Candidate is in a reviewable PR and Worker stops at `status:review`.

## Evidence Contract

`[EXECUTION REPORT]` must include:

```text
Attempt:
Base commit:
Candidate commit:
PR:
ControlView DTO/projection location:
Authoritative input types consumed:
Read-model-only proof/result:
Playback/site/browser/display projection results:
Failure-isolation matrix result:
Reconnect/rebuild result:
Freshness/revision separation result:
Secret/debug sentinel result:
Claims C1-C9:
J1/J2/J3 run + job IDs:
Exact-Candidate assertion:
Affected #38/R007/#28/#33/display/R008 regressions:
Limitations:
Result: COMPLETED | BLOCKED
```

No Secret, Cookie, Authorization, API key, profile/token content, password/code/QR or sensitive URL may appear in Evidence.

## Failure / Blocked Handling

- projection/test defects are same-Task revision work;
- if accepted domain APIs cannot express required safe read state without violating authority boundaries, report a Coordinator-visible architecture conflict rather than adding duplicate mutable state;
- missing GitHub Actions exact-SHA Evidence is BLOCKED;
- phone/TV/real-site unavailability is not a blocker.

## Deliverables

- canonical ControlView projection implementation/tests;
- exact-Candidate J1/J2/J3 Evidence;
- Candidate commit + PR;
- `docs/tasks/40-control-view-prep/prompt.md` bootstrap.

## Completion Protocol

```text
claim → status:in-progress → Attempt N
→ candidate + exact-SHA J1/J2/J3
→ [EXECUTION REPORT] → status:review → release owner → STOP
```

Blocker path: `[BLOCKER REPORT] → status:blocked → release owner → STOP`.
Worker never sets `status:done`, closes #40, merges its own PR, starts another Task, or silently changes this Contract.