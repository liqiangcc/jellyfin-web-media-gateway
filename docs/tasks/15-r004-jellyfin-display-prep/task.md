# Task — R004-PREP Jellyfin DisplayAdapter PoC

## Metadata

```text
GitHub Issue: #15
Parent Goal / Research Item: R004 / Jellyfin Display Adapter PoC
Task / Research ID: R004-PREP
Task kind: combined
Planning / integration base commit: 84b3a2d4d8b48fdb66a3317a8555e35c30f5cf16
Candidate commit: n/a (live candidate belongs in Issue)
Session bootstrap prompt: docs/tasks/15-r004-jellyfin-display-prep/prompt.md
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Preferred worker: cloud
Eligible worker environments after publication: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, rust-build, rust-test, failure-injection, protocol-fixture-authoring
Accepted upstream authority: Issue #2 / R007, Issue #3 / R001
Hard publication dependencies: none; R002/R003/R008 may proceed in parallel
Linked physical verification task: Issue #16 / R004-TV
```

> Live status, owner, Attempt, branch/PR, candidate SHA and Evidence belong in Issue #15.
>
> This Task prepares the adapter candidate and hosted proof mechanics. It does not classify R004 on a real Jellyfin Android TV/client; that Evidence belongs to Issue #16.

## Goal

Build the smallest auditable `JellyfinDisplayAdapter` PoC and verification harness that can consume the accepted Gateway media/playback boundaries without making Jellyfin part of Playback Core.

The candidate must provide enough server-side behavior and diagnostics for Issue #16 to test:

```text
client/session discovery
→ probe/prepare
→ remote start from a requested position
→ confirm actual playback state
→ pause/resume/seek/stop/status
→ position/handoff observation
→ explicit failure handling
```

## Why / Context

Canonical R004 asks whether a temporary non-DRM Gateway `PlaybackItem` can be consumed through Jellyfin Server/Android TV with reliable remote control and acceptable position handoff.

Jellyfin is optional. R004 failure must not block the Web-only Core. Gateway remains `PlaybackSession` authority and accepted R007 handoff/concurrency semantics remain authoritative.

## Task Decomposition Decision

```text
Verification mode: separate-task
Linked implementation task: Issue #15 (this Task)
Linked verification task: Issue #16 / R004-TV
Decision reason: ordinary adapter/protocol implementation and deterministic failure tests are repository work, while the R004 research verdict depends on an independent real Jellyfin Server + Android TV/client Evidence Authority.
```

Hosted/mock evidence proves adapter mechanics only. It cannot be used as the final R004 device verdict.

## Worker Routing Decision

```text
Adapter implementation / fixture / tests / CI
→ cloud-codex
→ env:cloud

Portable verification
→ GitHub Actions
→ GitHub-hosted x64

Optional real Jellyfin Server hosted smoke
→ GitHub-hosted x64 when practical
→ still not Android-TV Evidence

Final Jellyfin client / audible-visible playback / position-handoff verdict
→ Issue #16
→ real Jellyfin Server + real Jellyfin Android TV/client
→ manual/capability-driven device verification
```

Cloud Codex is the repository Worker, not the final Jellyfin client Evidence Authority.

## Canonical Sources to Read

Before implementation read at least:

- `AGENTS.md`
- Issue #15 and relevant comments
- `docs/requirements.md` — DisplayAdapter, Handoff, Media Gateway, Egress/Secret rules
- `docs/architecture.md` — Display Domain, Media Gateway, configured local service
- `docs/implementation-contracts.md` — DisplayAdapter and accepted R007 handoff authority
- `docs/technical-feasibility-validation.md` — R004
- `docs/mvp-plan.md` — Phase 0B
- `docs/security.md` — Jellyfin API Key / configured local service / Display security
- `docs/research/r001-media-path.md`
- Issue #2 R007 Final Acceptance / accepted concurrency semantics
- Issue #3 R001 Final Acceptance / accepted media candidate
- `docs/tasks/issue-lifecycle-protocol.md`

## P0 Parallelism / Integration Rule

R004-PREP has no hard dependency on R002, R003 or R008 and may execute in parallel.

However:

- do not redefine generic EgressPolicy/configured-local-service security semantics owned by canonical security/R008 work;
- do not redefine R007 Playback command, revision, handoff generation or commit authority;
- do not replace the R001 media path with a Jellyfin-specific media core;
- final candidate must integrate current `main` before exact-SHA verification;
- if parallel work changes an interface actually consumed by this adapter, integrate/rebase and rerun required Evidence instead of declaring an artificial hard dependency in advance.

## Work Role

### Implementation

Implement only what is needed for the R004 PoC, expected to include repository equivalents of:

- a `JellyfinDisplayAdapter` / Jellyfin client module behind the generic DisplayAdapter boundary;
- explicit configured Jellyfin endpoint/service reference supplied from server-side configuration, never arbitrary browser/plugin URL input;
- server-side Jellyfin authentication reference/key handling that never reaches Display/media/public logs;
- client/session discovery and stable selection/probe behavior;
- prepare/start/pause/resume/seek/stop/status operations;
- position unit conversion and requested-start-position handling;
- confirmation logic that distinguishes `command accepted` from `client actually playing`;
- timeouts/cancellation/stable adapter errors;
- deterministic Jellyfin protocol fixture/server for hosted tests;
- manual/deployment entry sufficient for Issue #16.

The exact Jellyfin API endpoints and protocol objects are implementation-defined from the currently supported Jellyfin API, but must remain isolated inside the adapter/client layer.

### Security / configured local service boundary

Jellyfin is an explicit `configured_local_service` integration.

The adapter must not introduce a generic bypass such as:

```text
user_url + allow_private=true
plugin says private is okay
browser chooses Jellyfin base URL
```

If current Core does not yet expose a reusable configured-local-service policy API, keep the integration point narrow/injected and do not invent a second central EgressPolicy in Jellyfin code. R008 may later supply/centralize the generic policy implementation.

Jellyfin API Key/credential must stay server-side and adapter-scoped. Do not expose it in:

- Web Display URLs;
- Gateway media capabilities;
- Control response bodies;
- logs/artifacts;
- test fixtures except fake sentinel values that are explicitly asserted absent from public evidence.

## Verification Claims

```text
C1 — Adapter boundary
Jellyfin-specific API/session/media behavior is isolated behind the DisplayAdapter/Jellyfin client boundary; Playback Core contains no Jellyfin special-case authority.

C2 — Config/Secret boundary
Only an explicitly configured Jellyfin service target is usable; adapter credentials remain server-side and are absent from public media/control/log evidence.

C3 — Discovery/probe
Deterministic fixture proves client/session discovery, unavailable/offline filtering and stable probe/selection outcomes without relying on UI scraping.

C4 — Command mapping
Start, pause, resume, seek, stop and status map to deterministic Jellyfin client operations with bounded timeout/cancellation and stable errors.

C5 — Start/position semantics
Requested start position is converted consistently to/from Jellyfin units; rounding/error is measurable and test-covered rather than silently ignored.

C6 — Command accepted != playback confirmed
An HTTP/API success response to a start command is not treated as successful playback until the adapter observes the required client/session playback state or times out with an explicit failure.

C7 — R007 handoff authority preserved
prepare/start/status callbacks remain candidate/adapter evidence only. The adapter cannot directly mutate global `active_display` or bypass accepted R007 generation/commit authority.

C8 — R001 media path reused
The PoC consumes a Gateway-controlled temporary media entry/capability suitable for Jellyfin testing; it does not expose upstream Cookie/Authorization or create an arbitrary open proxy.

C9 — Failure semantics
Deterministic tests cover at least Server unavailable, auth failure, target session missing/offline, command accepted but no playback confirmation, media incompatibility/error representation, timeout/cancellation and stale/late status behavior where applicable.

C10 — Issue #16 readiness
A reproducible real-device setup/evidence procedure exists so the physical verification Task does not need to invent new adapter semantics.
```

## In Scope

- minimal Jellyfin DisplayAdapter/client implementation;
- explicit configured Jellyfin endpoint/service handling;
- adapter-scoped fake/real-key plumbing with no public leakage;
- Jellyfin session/device discovery/probe;
- remote start/pause/resume/seek/stop/status;
- start-position and reported-position conversion;
- actual-playing confirmation state/timeout;
- deterministic fake Jellyfin server/API fixture;
- contract/failure tests;
- optional hosted Jellyfin Server smoke when practical;
- R001/R007 regression as required;
- Issue #16 run/setup/evidence instructions.

## Out of Scope

- declaring R004 PASS/FAIL from hosted mocks;
- implementing full Jellyfin library/catalog management;
- making Jellyfin the PlaybackSession authority;
- changing R007 command/revision/handoff semantics;
- replacing R001 Media Gateway with Jellyfin-specific proxy logic;
- Jellyfin user/account management beyond the explicit adapter credential needed for the PoC;
- public Internet exposure;
- DRM bypass;
- R002 TV browser autoplay;
- R003 phone resource classification;
- generic R008 security redesign;
- production-grade multi-server Jellyfin discovery.

## Architecture Invariants

- Gateway remains PlaybackSession authority.
- Jellyfin remains optional `DisplayAdapter`.
- Web Display continues working with Jellyfin absent/down.
- Adapter does not read Session Vault.
- Jellyfin credential is adapter-scoped and server-side.
- Jellyfin local-network access is explicit configured service access, not arbitrary private egress.
- candidate display/status does not become committed `active_display` without Playback Coordinator/R007 commit authority.
- old/stale adapter callbacks cannot overwrite a newer item/display generation.
- media entry does not expose upstream source-site Secret.
- R004 failure produces a bounded adapter/result change, not Core contamination.

## Files Expected to Change

Implementation-defined; expected categories:

- `gateway-core/` generic display integration only when required;
- a Jellyfin adapter/client module or crate;
- deterministic Jellyfin fixture/tests;
- `.github/workflows/` R004 hosted verification when useful;
- `docs/research/r004-jellyfin-display.md` or equivalent PoC/run guide/evidence skeleton;
- minimal dependency/config wiring.

Avoid unrelated Control UX or Site Plugin work.

## Implementation Requirements

1. Inventory the current DisplayAdapter/Web Display implementation before creating new abstractions; extend the accepted boundary rather than duplicating it.
2. Keep all concrete Jellyfin endpoint/API response interpretation inside the Jellyfin adapter/client layer.
3. Use an explicit configured endpoint/service ref; no user/plugin-controlled private-network opt-in.
4. Keep credential injection server-side. Tests use fake sentinel credentials and assert no leakage to public/log evidence.
5. Model session discovery and target selection explicitly enough that offline/missing/ambiguous target states are stable errors.
6. Make start a two-stage observable process: command submission followed by bounded playback confirmation/status observation.
7. Define/test position conversion and record expected vs reported position for later Issue #16 handoff accuracy.
8. Preserve R007 candidate/commit authority. Adapter callbacks include/bind the identity/generation data required by the current generic display contract.
9. Preserve R001 capability/Secret boundary. A media URL accepted by Jellyfin must remain Gateway-controlled and scoped.
10. Add deterministic failure injection for the required C9 cases.
11. Generate a concise Issue #16 setup/manual evidence procedure including what to record when Jellyfin says command accepted but the TV does not play.
12. Required CI must bind to the exact final Candidate SHA.

## Verification Plan

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Evidence |
|---|---|---|---|---|---|---|
| J1 | C1-C10 | github-actions | github-hosted-x64 | runner-self | yes | fmt/clippy/workspace + deterministic adapter/fixture/failure tests + R007 regression |
| J2 | C2-C6,C8,C9 | github-actions | github-hosted-x64 | deterministic Jellyfin API fixture | yes | protocol integration logs/artifact with fake secret sentinel redaction |
| J3 | C2-C6,C8 | github-actions | github-hosted-x64 | real Jellyfin Server on hosted Linux | optional | server API/media smoke; never substitutes Android-TV Evidence |

Physical/client R004 Evidence is intentionally not a Job here; it is Issue #16.

### Required Evidence rules

- exact Candidate SHA in every required run;
- fake secret sentinel must not appear in public/log artifact surfaces;
- fixture must prove accepted-command-without-play-confirmation failure;
- hosted run must not be described as a real Android TV result;
- if R001/R007-sensitive code changes, rerun their affected regressions on the final candidate.

## Success Criteria

1. Jellyfin-specific code is isolated behind the DisplayAdapter/client boundary.
2. Explicit configured-local-service endpoint and server-side credential boundary are preserved.
3. Client/session discovery/probe has deterministic success and failure behavior.
4. Start/pause/resume/seek/stop/status operations have deterministic fixture coverage.
5. Start-from-position conversion and reported-position evidence hooks are implemented/tested.
6. `command accepted` cannot be mistaken for `TV is actually playing`.
7. R007 handoff/active-display authority remains unchanged.
8. R001 media/Secret/open-proxy boundary remains unchanged.
9. Required failure cases are observable as stable adapter outcomes.
10. J1/J2 pass on the exact final Candidate SHA.
11. Issue #16 can execute a real-device R004 verification without inventing new behavior.
12. No R004 real-device PASS/FAIL is claimed by this Task.

## Evidence Contract

Each Attempt records:

```text
Attempt:
Base commit:
Candidate commit:
PR:
Worker / Orchestrator:
Execution plane:
Jellyfin fixture/server version when applicable:
J1/J2/J3 run/job IDs:
Configured service mechanism:
Credential redaction sentinel result:
Discovery/probe result:
Start confirmation method/timeout:
Position conversion units/tests:
Command results:
Failure-injection results:
R001/R007 regression:
Issue #16 manual entry/doc:
Claims C1-C10:
Unverified real-device scope:
```

Do not store real Jellyfin API keys, site credentials, signed sensitive media URLs or account data in repository/artifacts.

## Failure / Blocked Handling

### REVISE / FAIL findings

- Jellyfin-specific state leaks into Playback Core;
- adapter can directly commit `active_display`;
- arbitrary private URL can become Jellyfin configured service;
- Jellyfin key reaches public/browser/log output;
- start API 2xx is treated as playback success without confirmation;
- position conversion is unspecified/unbounded;
- hosted fixture is represented as physical R004 PASS;
- R001 media path is bypassed by a new open proxy.

### BLOCKED

Use BLOCKED only when required repository/Actions capability is unavailable or a concrete accepted interface cannot be consumed without a Contract/architecture revision.

Real Android TV absence does not block Issue #15; it keeps Issue #16 draft/blocked later.

## Deliverables

- Jellyfin DisplayAdapter/client PoC candidate + PR;
- deterministic protocol/failure fixture;
- J1/J2 exact-SHA Actions Evidence;
- optional J3 hosted Jellyfin Server smoke;
- Issue #16 real-device run/setup/evidence instructions;
- no R004 product verdict.

## Completion Protocol

Worker follows `docs/tasks/issue-lifecycle-protocol.md`:

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ implementation + exact-SHA J1/J2
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Worker must not execute Issue #16, merge/close itself, set `status:done`, or reinterpret hosted evidence as the real Jellyfin client verdict.