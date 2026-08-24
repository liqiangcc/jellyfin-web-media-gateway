# Task — R005-PUBLIC Real Site Public Resolution PoC

## Metadata

```text
GitHub Issue: #23
Parent Goal / Research Item: R005 / Real Site Resolution PoC
Task / Research ID: R005-PUBLIC
Task kind: combined / research
Planning base commit: 02f7ca1373867010f50a56f7495a736ae56fad98
Session bootstrap prompt: docs/tasks/23-r005-public-real-site/prompt.md
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Preferred worker: cloud
Eligible worker environments after publication: env:cloud
Required capabilities: github-read-write, repository-static-analysis, rust-code-authoring, automated-test, real-http-smoke, plugin-contract-design, evidence-authoring
Hard publication dependencies: Issue #14/R008 Final Acceptance merged; Coordinator freezes one concrete legal public non-DRM target site/sample before publication
Accepted upstream authority: Issue #2/R007, Issue #3/R001
```

> Live status, selected target site/sample, Attempt, candidate/PR, runs and R005 result belong in Issue #23.
>
> This Task proves only the public/no-login phase of R005. Authenticated content is a later Task and must not be pulled into this Attempt.

## Goal

Prove that one real priority source site can map naturally into the existing Site Plugin contracts without requiring Core to understand concrete site URL structure, Cookie names, DOM/private API details, content identifiers, episode/season semantics, or site-specific navigation rules.

Required path:

```text
real public URL
→ SiteAdapterRegistry
→ plugins/<site>/ concrete adapter
→ versioned opaque SourceLocator
→ ResolvedMedia
→ accepted R001 Media Gateway boundary
→ generic navigation / re-resolve / PlaybackItem refresh semantics
```

## Why / Context

Canonical R005 asks whether a real source site validates the generic Plugin Boundary rather than merely working with synthetic/public generic fixtures. The canonical order explicitly requires public content first; authenticated content begins only after the public chain succeeds.

R005 is not a Web-only Core P0 publication blocker, but it is required before claiming that the Site Plugin Boundary has been validated by a real site.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Future child task: R005-AUTH after this Task is accepted
Decision reason: plugin implementation plus deterministic contract tests and bounded public-site smoke share one scope and Evidence Authority. Authenticated/session behavior has a different prerequisite/security lifecycle and is intentionally deferred.
```

## Worker Routing

```text
plugin implementation / fixture / real-site smoke orchestration
→ cloud-codex
→ env:cloud

portable deterministic verification
→ GitHub Actions github-hosted x64

real public-site smoke
→ GitHub Actions when practical, otherwise bounded Worker diagnostic followed by durable reproducible Evidence
```

No phone/TV runner is required.

## Publication Preconditions

Before Issue #23 may become `status:ready`:

1. Issue #14 / R008 has Coordinator Final Acceptance and its accepted public-web Egress/Secret implementation is merged to main.
2. Coordinator records one exact target site and one or more evidence-safe public sample locators/URLs.
3. Selected content is legal to access, public/no-login, non-DRM, and does not require bypassing access controls, regional restrictions or paid authorization.
4. The sample is suitable for bounded repeatable resolution without committing copyrighted media payloads to the repository.
5. Current main still contains accepted R001/R007 contracts and no unresolved contradiction makes this Task Contract stale.

The target site/sample is live publication metadata and must be recorded in Issue #23; do not guess it from chat history.

## In Scope

- one concrete `plugins/<site>/` public-content adapter;
- Registry routing for the selected site;
- versioned opaque SourceLocator encoding/decoding owned by the plugin;
- public real-site resolution to current generic `ResolvedMedia` or explicit unsupported/error result;
- protocol/public-header/media-expiry mapping;
- previous/next or equivalent continuous-content navigation when the selected sample naturally supports it;
- same-locator retry/re-resolve after expiry/refresh condition;
- R007 freshness integration so stale async resolve cannot replace newer media/item state;
- accepted R008 `public_web` egress boundary and Secret-header validation;
- deterministic local fixtures for parser/schema/error/navigation behavior;
- bounded real-site smoke with exact candidate identity and evidence-safe logging;
- durable `docs/research/r005-real-site.md` result/evidence mapping.

## Out of Scope

- login, Cookie/session/profile/Vault material;
- authenticated or paid content;
- CAPTCHA/anti-bot bypass, DRM bypass, region/access-control bypass;
- Site Browser Worker / Native Site Panel / remote Chromium interaction (R006);
- storing passwords, login input or account state;
- implementing site-specific logic in Core;
- changing R007 command/revision/media-generation ownership;
- weakening R008 EgressPolicy for site compatibility;
- adding a generic open proxy or exposing upstream Secret to Display;
- bulk crawling/downloading or retaining media payloads.

## Architecture Invariants

1. All concrete site knowledge remains in `plugins/<site>/` plus site-specific fixtures/docs.
2. Core routes through `SiteAdapterRegistry`; no `if site == ...` or concrete domain/content-ID parsing in Core.
3. `SourceLocator.opaque_payload` is plugin-owned/versioned and contains no Cookie/Authorization/account Secret.
4. `ResolvedMedia.public_headers` contains no Secret; sensitive access is not invented for this public-only Task.
5. All real network egress uses accepted R008 `public_web` policy; plugin cannot declare private exceptions.
6. URL expiry/re-resolve consumes accepted R007 item/media freshness semantics; an old resolve result cannot overwrite a newer item/media generation.
7. Failure/unsupported/DRM states are explicit; no fallback silently bypasses Plugin or security boundaries.

## Claims

```text
C1 — Registry/plugin boundary
The selected real-site URL is recognized/routed by SiteAdapterRegistry to the concrete plugin without concrete-site branches in Core.

C2 — SourceLocator opacity and recoverability
The plugin produces a versioned opaque SourceLocator that can round-trip/retry and does not contain Secret/account material. Core does not parse the opaque payload.

C3 — Real ResolvedMedia mapping
A real public sample resolves to the current generic ResolvedMedia shape with protocol, public headers, media-expiry and explicit unsupported/error semantics preserved. DRM/access-control conditions are rejected rather than bypassed.

C4 — Navigation contract
For a selected sample with natural previous/next/queue semantics, the plugin returns SourceLocator-based navigation without Core understanding site identifiers. If the frozen sample/site cannot meaningfully exercise navigation, Publication Gate must predeclare this C4 as N/A and require a second public sample that can, or split navigation proof before acceptance.

C5 — Re-resolve / freshness
A refresh/expiry path can re-resolve the same SourceLocator, and deterministic interleaving tests prove stale old resolve results cannot overwrite the current R007 item/media generation.

C6 — Egress / Secret boundary
Real-site and fixture resolution uses accepted R008 public-web egress; no private/loopback exception, Cookie/Authorization in locator/public headers, or browser-visible upstream Secret is introduced.

C7 — Failure observability
Representative invalid URL/content-not-found/upstream-denied/unsupported/parse/schema errors map to stable plugin/Core error semantics without leaking sensitive full URLs/query material.

C8 — Real-site Plugin Contract result
Evidence is sufficient to classify R005-PUBLIC PASS | CONDITIONAL PASS | FAIL | BLOCKED and state whether the existing SiteAdapter/SourceLocator/ResolvedMedia/navigation contracts can continue unchanged or need a contract revision.
```

## Verification Plan

### J1 — Deterministic plugin/contract suite

GitHub-hosted x64, exact Candidate SHA.

At minimum cover:

- Registry matching/non-matching;
- locator version/round-trip/unsupported version;
- locator Secret sentinel rejection;
- deterministic public resolution fixture;
- ResolvedMedia schema/public headers/expiry;
- navigation mapping;
- error mapping;
- re-resolve freshness/stale result behavior integrated with accepted R007 semantics;
- no concrete-site knowledge added outside allowed plugin/test/doc paths.

### J2 — Security/integration regression

GitHub-hosted x64, exact Candidate SHA.

Required:

- accepted R008 public-web/Secret relevant regressions;
- accepted R001 media capability regressions affected by the new plugin output;
- accepted R007 freshness regressions;
- sentinel scan showing no Cookie/Authorization/account/signature value in accepted logs/artifacts.

### J3 — Bounded real-site smoke

Use only the Coordinator-frozen public sample.

Record:

- UTC time;
- exact Candidate SHA;
- selected site/plugin/version;
- evidence-safe input identifier (redact unnecessary query/signature material);
- resolution outcome/protocol;
- SourceLocator version/opaque hash or redacted representation, never account Secret;
- ResolvedMedia protocol/expiry/capability-relevant summary without publishing unnecessary signed media URL;
- navigation result where applicable;
- retry/re-resolve result where safely reproducible;
- HTTP/status/error classification;
- tool/runtime versions used by plugin implementation.

A transient public-site outage may produce BLOCKED/failed smoke Evidence; it must not be rewritten as contract PASS from fixtures alone.

## Success Criteria

Task is complete when:

1. one real public site is implemented through `plugins/<site>/` and Registry with no Core site special case;
2. C1-C7 have required deterministic Evidence and C3 has real-site smoke Evidence;
3. SourceLocator supports retry/re-resolve and required navigation without leaking site semantics into Core;
4. accepted R007 stale refresh protection remains valid;
5. accepted R008 Egress/Secret boundary remains valid;
6. affected R001 media regressions remain passing;
7. `docs/research/r005-real-site.md` records result, maintenance risk, exact Evidence and architecture impact;
8. Worker posts standard `[EXECUTION REPORT]`, moves to `status:review`, releases ownership and stops;
9. Worker does not start R005-AUTH.

## R005-PUBLIC Result Classification

### PASS

Real public-site evidence and deterministic tests show the existing Plugin/SourceLocator/ResolvedMedia/navigation/re-resolve contracts work without Core site knowledge or security exceptions.

### CONDITIONAL PASS

The contracts remain valid but the selected site requires a clearly bounded plugin-only limitation (for example an optional navigation/media shape is unavailable). Conditions must not weaken Core/security invariants.

### FAIL

Real evidence requires concrete-site knowledge in Core, breaks the generic SourceLocator/ResolvedMedia/navigation model, or requires weakening R007/R008 boundaries. The next action is contract/architecture review, not a site-specific Core exception.

### BLOCKED

Required real public sample/site is unavailable, required upstream behavior cannot be legally/reliably exercised, or an accepted dependency/capability is unavailable. Missing real evidence is not PASS.

## Evidence Contract

Issue #23 report must include:

```text
Attempt:
Target site/sample selector:
Plugin path/version:
Base/Candidate SHA:
PR:
J1/J2/J3 runs/jobs:
Runtime/tool versions:
Registry result:
SourceLocator version/redacted evidence:
ResolvedMedia protocol/expiry summary:
Navigation evidence:
Re-resolve/freshness evidence:
R001/R007/R008 regression evidence:
Secret/sensitive URL scan result:
Claims C1-C8:
R005-PUBLIC result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
Architecture impact: Continue | Change | Defer | Drop
Limitations:
```

Never publish Cookie, Authorization, account data, full signed media URLs or copyrighted media payloads.

## Completion Protocol

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ implement + exact-SHA J1/J2 + bounded J3
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Only Coordinator may ACCEPT/close and decide whether/when to create or publish R005-AUTH.
