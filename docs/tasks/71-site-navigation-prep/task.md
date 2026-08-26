# Task — SITE-NAVIGATION-PREP

## Metadata

```text
GitHub Issue: #71
Task ID: SITE-NAVIGATION-PREP
Task kind: implementation / generic contract + deterministic verification
Planning Base: 826d02c22105ee1877ae79706d2cb03112f995a9
Preferred worker: cloud-codex
Eligible environment: env:cloud
Accepted authorities: #39 SiteAdapter conformance; #2/R007 Playback authority; #44 SourceSession preparation; canonical docs/implementation-contracts.md
Downstream: #72 BILIBILI-NAVIGATION
Freshness policy: dependency-aware
```

> #71 owns only the generic continuous-content/navigation capability. It contains no concrete Bilibili semantics and requires no real-site access.

## Context

Canonical `docs/implementation-contracts.md` already defines the conceptual authority:

```text
SiteAdapter.navigation(locator, access)
→ NavigationContext
   ├ previous: SourceLocator?
   ├ next: SourceLocator?
   ├ collection_id?
   └ current_index?

PlaybackContext
├ previous: SourceLocator?
├ next: SourceLocator?
├ queue: SourceLocator[]
└ autoplay_policy

Playback command
├ NextItem
└ PreviousItem
```

The current accepted `site-adapter-api` implementation, however, exposes only `recognize + resolve`; current `PlaybackSession::Command` also lacks NextItem/PreviousItem. The old #23/PR #37 experiment is historical reference only and is not API authority.

## Goal

Implement the smallest current-main generic capability that lets a future concrete Site Plugin provide previous/next content and lets Playback consume it without learning concrete-site identifiers or creating a second revision/state model.

Target shape:

```text
current SourceLocator
→ SiteAdapterRegistry.navigation(...)
→ generic previous/next SourceLocator
→ select direction
→ Registry.resolve(target locator)
→ prepare bounded Gateway media capability / item candidate
→ freshness/CAS validation against current item + session
→ commit current PlaybackItem through existing R007 authority
```

The implementation may refine exact Rust names, but must preserve the semantics below.

## Required generic contract

### A. Navigation value

Add a current-authority navigation result equivalent to:

```text
NavigationContext
- previous: Option<SourceLocator>
- next: Option<SourceLocator>
- collection_id: Option<String>
- current_index: Option<u32/u64>
```

Rules:

- previous/next absence is normal edge state, not an error;
- collection metadata is optional and non-authoritative;
- Core does not parse `opaque_payload`;
- no concrete-site IDs appear in shared types;
- result metadata is bounded.

### B. SiteAdapter / Registry navigation

Extend current SiteAdapter authority with a backward-compatible navigation capability.

Requirements:

- existing plugins that do not implement navigation continue to compile and fail closed as unsupported;
- Registry routes navigation strictly by locator `plugin_id/site_id` ownership;
- previous/next returned locators must belong to the same owning adapter unless canonical authority explicitly permits otherwise; no caller-selected cross-plugin jump;
- malformed/foreign/unsupported locators return stable generic errors;
- conformance tests cover navigation ownership and edge semantics;
- do not restore #23-only ResolveContext, DASH/expiry, site-specific errors or concrete-site logic.

### C. Navigation preparation and freshness

The source/session layer must retain enough server-owned source identity to prepare the next/previous item safely.

Required semantics:

1. navigation starts from the authoritative current item/source locator;
2. target locator is obtained from Registry navigation, not caller payload;
3. target media resolves through normal Registry resolve;
4. media capability issuance uses existing Gateway security/egress rules;
5. preparation is not itself a committed item switch;
6. prepared result/ticket binds at least:
   - session identity;
   - expected current item id + item_revision;
   - expected session revision or equivalent accepted R007 CAS authority;
   - direction/target locator identity;
7. if current item/session changes before commit, stale prepared result is rejected with no side effect;
8. successful current-item commit increments item_revision exactly once and session_revision according to existing R007 authority;
9. position/telemetry are reset/isolated according to existing item-switch semantics;
10. old display/media callbacks cannot overwrite the newly committed item.

Do not introduce a second navigation-owned session revision.

### D. Playback command integration

Canonical commands include `NextItem` and `PreviousItem`; implement the smallest generic integration needed to make them current authority.

Requirements:

- command ingress remains the existing Playback command endpoint/CommandEnvelope authority;
- `request_id` idempotency and `expected_session_revision` CAS semantics remain exactly R007-owned;
- a repeated identical navigation command cannot cause two item switches;
- reuse of request_id with a different direction/fingerprint is rejected;
- two navigation/control mutations racing on one old revision cannot both commit;
- unsupported/no-target is a bounded stable result and does not mutate the session;
- navigation resolution/preparation failure does not partially mutate PlaybackSession;
- do not change handoff authority or allow NextItem to become a display handoff.

If current code structure requires a source-session coordinator around the pure `PlaybackSession`, keep resolution outside the pure state object and commit through one explicit R007-compatible boundary.

## Primary file ownership

Expected #71 ownership is primarily:

```text
site-adapter-api/**
gateway-core/src/source_session.rs
gateway-core/src/playback.rs
navigation-specific tests / workflows
```

Minimal adjacent Control/server plumbing is allowed only for `NextItem/PreviousItem` command integration.

To preserve Cloud parallelism with the R006 Chromium runtime task:

- do **not** modify `gateway-core/src/browser.rs`;
- do not add Chromium/browser-runtime dependencies;
- do not change BrowserWorker/NativePanel contracts.

If an unexpected hard dependency requires browser contract changes, BLOCK/SPLIT rather than crossing ownership silently.

## Architecture / security invariants

1. Core never parses BVID/page/episode/playlist/site DOM semantics.
2. SiteAdapterRegistry remains the only route from Core to concrete Site Plugins.
3. `SourceLocator` remains opaque/versioned plugin-owned identity; CDN/HLS URL is not content identity.
4. #39 current SiteAdapter conformance remains authority; do not revive stale #23 types verbatim.
5. #2/R007 command idempotency/CAS/session_revision/item_revision/handoff semantics remain authority.
6. #44 session/media capability preparation and R008 security remain authority.
7. No login/Vault/Browser Worker/Native Panel/DASH/remux work.
8. No real Bilibili/site request.
9. No performance/capacity claim.

## Claims

```text
N1 — Generic navigation contract
Current SiteAdapter authority can express previous/next/edge metadata using opaque SourceLocator values without concrete-site knowledge.

N2 — Registry ownership/conformance
Navigation dispatch and returned locators are validated against plugin/site ownership and malformed/foreign results fail closed.

N3 — Safe preparation
Navigation target resolve/media preparation occurs before authoritative item commit and cannot partially mutate the session on failure.

N4 — Stale-result protection
Prepared navigation bound to an older item/session cannot commit after current item/revision changes.

N5 — R007 command authority preserved
NextItem/PreviousItem use existing request_id/CAS/session revision authority; identical retries are idempotent and racing old-revision commands cannot double-switch.

N6 — Item transition semantics preserved
Successful navigation commits exactly one new current item, advances item_revision/session_revision correctly, resets item-local telemetry/position, and rejects old callbacks.

N7 — Architecture boundary preserved
No concrete-site logic, Browser runtime semantics, Vault/Secret authority or stale #23 API expansion enters Stable Core.
```

## Deterministic verification

### J1 — SiteAdapter navigation/conformance

GitHub-hosted Ubuntu, exact Candidate.

Prove:

- fake adapter previous/next/middle/start/end;
- unsupported navigation default;
- foreign plugin/site locator denial;
- malformed returned locator denial;
- Registry does not depend on registration order for ownership;
- existing SiteAdapter resolve/recognize conformance remains passing.

### J2 — Preparation/stale-result matrix

Prove deterministic interleavings without sleep-based races:

- prepare next → commit success;
- prepare previous → commit success;
- prepare → current item changes → stale commit rejected;
- prepare → session revision changes incompatibly → commit rejected;
- target resolve/capability failure → zero item mutation;
- old media/display callback after switch → rejected.

### J3 — Command/CAS/idempotency

Prove:

- NextItem/PreviousItem through existing command authority;
- identical request retry returns same outcome/no second switch;
- request_id mismatch direction rejected;
- two commands using same old expected revision: at most one item commit;
- navigation vs Pause/Seek/Handoff interleavings preserve R007 authority and no double mutation.

### J4 — Workspace/architecture regressions

Run exact-Candidate fmt/clippy/test plus affected R001/R007/#44/R008/site-boundary guards. Include bounded repeated/sharded R007 race tests where existing protocol requires them.

All required Jobs assert exact Candidate SHA.

## Success Criteria

1. N1-N7 PASS on one exact Candidate.
2. Existing plugins remain source compatible unless they opt into navigation.
3. Generic start/middle/end navigation works in deterministic fake adapter tests.
4. No stale navigation preparation can replace a newer current item.
5. NextItem/PreviousItem preserve R007 idempotency/CAS semantics.
6. No Bilibili-specific or Browser runtime logic enters this Task.
7. J1-J4 and affected workspace/security/architecture regressions pass.
8. Worker reports and STOPs; it does not start #72.

## Evidence Contract

`[EXECUTION REPORT]` must include:

```text
Attempt / worker / environment
Base SHA
Candidate SHA / PR
Navigation API shape
Default unsupported behavior
Registry ownership validation
Preparation/freshness ticket shape
Successful next/previous transition summary
start/end no-target summary
stale prepare/commit rejection
request_id retry/mismatch result
same-revision race result
item/session revision transition summary
old callback rejection
J1-J4 run/job IDs
Claims N1-N7
Concrete-site execution: NOT RUN
Browser runtime work: NOT RUN
Downstream readiness for #72
```

Do not persist concrete-site secrets or signed media URLs in Evidence.

## Freshness

Semantic authorities:

- `site-adapter-api/**` / #39 conformance;
- `gateway-core/src/playback.rs` / #2 R007;
- `gateway-core/src/source_session.rs` / #44;
- canonical `docs/implementation-contracts.md` navigation/Playback sections.

Accepted changes in these domains while the Task is executing require Coordinator freshness classification. Browser-only/runtime-only changes outside these domains are normally `UNRELATED`.

## Out of Scope

- Bilibili multipart implementation or real Evidence (#72);
- login/Vault/Auth;
- Browser Worker/Native Panel/Chromium;
- DASH/remux/FFmpeg;
- #9 performance/resource work;
- production autoplay policy tuning;
- concrete-site playlist semantics.

## Completion Protocol

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ J1-J4
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Worker cannot set status:done, close #71, start #72, or merge its own PR.