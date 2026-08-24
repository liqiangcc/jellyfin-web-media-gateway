# Task — R007 Playback Concurrency Contract Closure

## Metadata

```text
GitHub Issue: #2
Parent Goal / Research Item: R007 / Core Feasibility / Phase 0A-2 PlaybackSession + Control Shell
Task / Research ID: R007
Task kind: combined
Base commit: 81965ef709de8a52e961d35045cdc26568bdd165
Candidate commit: n/a (live state belongs in Issue)
Session bootstrap prompt: docs/tasks/2-r007-playback-concurrency-closure/prompt.md
Preferred worker: web
Eligible worker environments: env:web-gpt
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, rust-build, rust-test
```

> Live status, owner, candidate, runs and results belong in Issue #2. A Worker may claim only when the Issue is `status:ready`, the current environment is eligible, and no active owner exists. This file does not store live Task state.
>
> GitHub Actions / Runner are execution backends, not Workers. Attempt / Review / Acceptance follows `docs/tasks/issue-lifecycle-protocol.md`.

## Goal

Close R007 before R001 by turning the Playback concurrency contract into a minimal executable Rust model plus deterministic race/interleaving tests that prove stale commands, callbacks, resolves and handoff candidates cannot overwrite newer Playback authority.

The result must leave `docs/implementation-contracts.md` with unambiguous revision/generation semantics that later Phase 0A-1/0A-2 implementation can reuse without redesigning concurrency ownership.

## Why / Context

`docs/technical-feasibility-validation.md` defines R007 as an encoding-before-implementation contract gate, not a large external PoC. It requires closure of:

```text
session revision vs high-frequency position telemetry
same-item async re-resolve / media refresh staleness
handoff candidate generation before active_display commit
```

`AGENTS.md` additionally requires the minimum concurrency suite:

```text
duplicate request_id
stale expected revision
stale item callback
stale re-resolve result
stale display generation
overlapping handoff
two-Control concurrent mutation
```

The repository currently has no Rust workspace or executable tests, so this Task includes only the smallest workspace/model needed to prove these contracts. It must not grow into Media Gateway, Control UI, Site Plugin, FFmpeg, Jellyfin, TV or target-device work.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: R007 is a code/test contract-closure unit whose required verification is portable, deterministic and suitable for standard GitHub-hosted CI. No independent target Evidence Authority is required.
```

Do not split x64 / ARM64 / repeated-race into separate Tasks. They are Verification Jobs for the same Claim set.

## Contract Decisions to Prove

The Task freezes the following semantic direction. Exact Rust type/field names may differ only if the same invariants are demonstrably preserved.

### D1 — command revision is not telemetry revision

`session_revision` is the CAS revision for authoritative Playback mutations.

High-frequency position telemetry **must not increment `session_revision` merely because position changed**.

Position callbacks must still be scoped to the current item identity so stale item telemetry cannot update the new item. If an observable telemetry sequence is required to reject out-of-order current-item telemetry, it must remain separate from command CAS semantics.

Required invariant:

```text
many position callbacks
+ a valid Pause / Seek using the current command revision
→ no REVISION_CONFLICT caused solely by telemetry churn
```

### D2 — `item_revision` remains item-switch identity; media refresh gets its own freshness ticket

`item_revision` continues to advance when `current_item` changes.

A same-item async re-resolve / media refresh must use a per-item **media resolve generation/ticket** (name may vary) so a stale earlier resolve completion cannot overwrite the latest `ResolvedMedia` while `item_revision` remains the same.

Required invariant:

```text
resolve generation N starts
resolve generation N+1 starts
N+1 commits first
N completes later
→ N is discarded with zero current-media mutation
```

Do not overload `item_revision` by pretending a same-item media refresh is a new PlaybackItem unless the canonical item semantics are intentionally changed through a reviewed Contract revision.

### D3 — handoff has candidate authority before committed display authority

`active_display` and committed `display_generation` remain Playback Coordinator authority.

A handoff must have an explicit transition/reservation identity (for example `transition_id` + reserved/candidate generation). Target callbacks received after target start but before commit belong to candidate transition state and **cannot mutate global committed Playback authority**.

First implementation should permit at most one active handoff transition per Session. A concurrent second handoff must receive a stable conflict/in-progress outcome rather than create a second independently committable authority path.

Required invariant:

```text
prepare target
→ target start
→ target callback before commit
→ no active_display/global committed state switch
→ confirm/commit current transition only
```

Timeout/cancel must invalidate candidate authority; callbacks from the old source generation and expired candidate transition must be ignored.

### D4 — request idempotency precedes side effects

For a Session, repeated delivery of the same `request_id` for the same logical command must not execute the mutation twice.

A reused `request_id` with incompatible command content must be rejected deterministically; it must never become a second mutation under the old idempotency key.

### D5 — expected revision CAS is authoritative

A command carrying stale `expected_session_revision` must fail before side effects and return the current revision/snapshot according to the command contract.

Two Controls racing with the same expected revision may not both commit incompatible authoritative mutations.

## Work Role

### Implementation

Build only the minimal executable contract harness needed for R007:

- create a root Rust workspace if none exists;
- add a minimal `gateway-core` crate (or equivalent canonical Core crate) with Playback model/state transition code;
- implement only enough `PlaybackSession`, `PlaybackItem`, command envelope, idempotency state, media-refresh ticketing and handoff transition state to exercise R007;
- keep external Site/Media/Display implementations fake/in-memory; do not implement real adapters;
- update canonical contract documentation after the executable model proves the selected semantics;
- add the first meaningful GitHub-hosted workflow only if code/tests now exist; do not create an empty workflow.

### Verification

Claims to verify:

```text
C1: High-frequency position telemetry cannot churn command CAS revision or create telemetry-only command conflicts.
C2: Duplicate request_id is idempotent and incompatible request-id reuse cannot execute a second mutation.
C3: stale expected_session_revision fails before side effects; two Controls cannot both commit against the same old revision.
C4: stale item callbacks cannot mutate a newer current item.
C5: stale same-item re-resolve/media-refresh results cannot overwrite the latest ResolvedMedia.
C6: stale committed display-generation callbacks cannot mutate current display/session state.
C7: handoff candidate callbacks before commit cannot become global authority; timeout/cancel/old callbacks are rejected.
C8: overlapping handoff cannot produce two committable transitions or ambiguous active_display ownership.
C9: the final canonical docs and executable tests encode the same revision/generation semantics.
```

## Routing Rationale

Implementation stays Web-first. Verification is portable and has no target-device claim:

```text
Web Worker
→ Rust model / tests / docs
→ Candidate commit / PR
→ GitHub-hosted x64 deterministic suite
→ GitHub-hosted repeated stress
→ optional generic ARM64 regression
→ Coordinator Review
```

Ubuntu ARM64 Target Runner is explicitly not required for R007 acceptance.

## Preconditions

- Worker must re-read Issue #2 immediately before claim and proceed only if live state is `status:ready + env:web-gpt` with no active owner.
- Canonical inputs are `AGENTS.md`, `architecture.md`, `implementation-contracts.md`, `technical-feasibility-validation.md`, `mvp-plan.md`.
- No existing Rust workspace is assumed.
- Rust stable should be used unless the repository later freezes a toolchain for a separate reason.
- Tests must be deterministic enough to reproduce a failing interleaving; `sleep()` timing races alone are insufficient evidence.

## In Scope

- minimal Rust workspace bootstrap needed by R007;
- minimal Playback Core model and fake collaborators;
- session command revision semantics;
- request idempotency;
- item identity/revision callback rejection;
- per-item media resolve generation/ticket;
- display generation callback rejection;
- handoff transition/candidate reservation/commit/timeout semantics;
- deterministic concurrency/interleaving tests;
- bounded repeated race/stress verification;
- canonical contract updates driven by the accepted executable model;
- an R007 evidence summary if a durable research result needs preservation.

## Out of Scope

- HTTP server/API implementation beyond types needed by tests;
- Media Gateway proxy / HLS / MP4 / Range;
- `ResolvedMedia` real source resolution;
- SiteAdapter implementations or yt-dlp;
- FFmpeg;
- `/control` UI or `/display` player;
- WebSocket networking;
- real Jellyfin / TV / Chromium;
- Site auth / Vault implementation;
- R001/R002/R003/R008 acceptance;
- target phone Runner or device Evidence;
- production persistence/recovery.

## Architecture Invariants

- Gateway/Playback Coordinator remains the sole `PlaybackSession` authority.
- old `item_revision`, media resolve generation, display generation or handoff transition callbacks cannot overwrite current state.
- Control outcome is based on server committed snapshot, not optimistic local authority.
- handoff does not change `current_item`; it changes committed display ownership only at commit.
- position telemetry is observational state and must not create command-CAS conflict storms.
- Site-specific knowledge must not enter `gateway-core`.

## Files Expected to Change

Expected, not mandatory exact layout:

```text
Cargo.toml
gateway-core/Cargo.toml
gateway-core/src/lib.rs
gateway-core/src/playback/...
gateway-core/tests/playback_concurrency.rs
.github/workflows/portable-ci.yml        # only if this becomes the first real runnable CI
docs/implementation-contracts.md
docs/technical-feasibility-validation.md # R007 wording/result linkage only when evidence exists
docs/research/r007-playback-concurrency.md # optional durable Evidence summary
```

Do not create unrelated crates merely to satisfy the future full workspace shape.

## Implementation Requirements

1. **Minimal state machine, not fake assertions**
   - tests must exercise real mutation/rejection code paths;
   - do not encode expected outcomes only in test-local mock logic.

2. **Atomic command acceptance boundary**
   - request-id duplicate detection and expected-revision validation happen before authoritative side effects;
   - successful authoritative mutation advances command revision exactly according to the accepted contract.

3. **Telemetry separation**
   - position updates do not advance command CAS revision;
   - position callback carries current item identity/revision;
   - stale-item telemetry is ignored/rejected.

4. **Media freshness reservation**
   - start of each same-item resolve/refresh yields a generation/ticket;
   - only the latest still-valid ticket may commit media;
   - item switch invalidates prior media tickets.

5. **Handoff reservation**
   - prepare/start candidate state is distinguishable from committed active display state;
   - only the current transition can commit;
   - timeout/cancel invalidates the reservation;
   - callbacks are scoped to committed generation or candidate transition as appropriate;
   - no callback can infer authority merely because target playback started.

6. **Deterministic interleavings first**
   - use explicit barriers/channels/manual scheduling/model checking (for example Loom if justified) to force relevant event order;
   - sleeping and hoping for races is not sufficient;
   - repeated stress supplements deterministic tests but does not replace them.

7. **Error stability**
   - stale revision must expose `REVISION_CONFLICT` or the canonical equivalent already defined;
   - overlapping handoff / request-id mismatch may introduce stable errors if needed, but update `implementation-contracts.md` so tests and docs agree.

8. **No architecture creep**
   - real network/media/display behavior is represented by fake collaborators/events;
   - do not implement R001 or R002 while closing R007.

## Verification Plan

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1-C9 | github-actions | github-hosted-x64 | runner-self | yes | `cargo fmt --check` + R007 deterministic/unit/contract suite | run/job/log |
| J2 | C1-C8 | github-actions | github-hosted-x64 matrix | runner-self | yes | bounded repeated/stress execution of concurrency scenarios, aggregate target >= 1000 scenario iterations | run/job summary/artifact |
| J3 | C1-C9 | github-actions | github-hosted-arm64 | generic Linux ARM64 | no | same deterministic suite when practical | run/job/log |

J3 is compatibility evidence only and does not make phone Target Runner a dependency.

### Automated verification

Required suite must include named scenarios equivalent to:

```text
duplicate_request_id_is_idempotent
request_id_reuse_with_different_command_is_rejected
stale_expected_revision_has_no_side_effects
position_telemetry_does_not_advance_command_revision
high_frequency_position_plus_pause_seek_has_no_telemetry_conflict
stale_item_callback_is_ignored
stale_media_resolve_result_is_ignored
newer_media_resolve_wins_when_old_result_arrives_late
stale_display_generation_callback_is_ignored
handoff_candidate_callback_before_commit_has_no_global_authority
handoff_timeout_invalidates_candidate
old_source_callback_after_handoff_commit_is_ignored
overlapping_handoff_has_single_authority_path
two_controls_same_expected_revision_only_one_authoritative_mutation_commits
```

Names may differ; semantic coverage may not.

### Repeated verification

J2 should repeat the concurrency scenarios with bounded matrix/sharding. Aggregate target: at least 1000 scenario iterations with zero invariant violation for the candidate being reviewed.

If deterministic tests reveal the full state space and repetition adds no information, the Worker may propose reducing J2, but the Coordinator must explicitly approve that change before R007 acceptance; the Worker cannot silently remove required Evidence.

### Runner Selection

```text
Primary runner: github-hosted-x64
Optional compatibility: github-hosted-arm64
Target device proof: no
Trust gate: normal-ci
```

### Interactive debugging

```text
WSL / external Codex required: no by default
Reason: Web Worker + Actions should close portable R007.
```

Use WSL only if an Actions-only failure requires interactive diagnosis; final Evidence still returns to GitHub.

## Success Criteria

### Task success

1. A real minimal Rust Playback model exists and the R007 suite executes against it.
2. `session_revision` no longer has ambiguous position-telemetry semantics: position churn does not advance command CAS revision.
3. duplicate `request_id` cannot produce duplicate side effects; incompatible reuse is deterministically rejected.
4. stale expected revision cannot mutate session state, and concurrent Controls cannot both commit against one stale revision.
5. stale item callbacks cannot mutate a newer current item.
6. same-item media refresh has an explicit generation/ticket mechanism and stale resolve completion cannot overwrite newer media.
7. handoff candidate state is distinct from committed `active_display`; pre-commit/timeout/stale/old-generation callbacks cannot seize authority.
8. overlapping handoff has a single deterministic authority path and cannot double-commit.
9. deterministic R007 tests pass on GitHub-hosted x64.
10. required bounded repeated verification completes with zero invariant violation.
11. canonical `implementation-contracts.md` matches the semantics actually proven by code/tests.
12. no phone/TV/Jellyfin/media-path Evidence is required or falsely claimed.

### Verification claim success

```text
C1 PASS when: telemetry-heavy deterministic/repeated tests show no command-revision churn/conflict caused solely by position callbacks.
C2 PASS when: duplicate/mismatched request-id tests prove exactly-once logical mutation semantics.
C3 PASS when: stale revision and two-Control tests prove CAS loser has zero authoritative side effects.
C4 PASS when: forced late old-item callback leaves the new item snapshot unchanged.
C5 PASS when: forced N+1-before-N resolve ordering leaves N+1 media authoritative and discards N.
C6 PASS when: stale committed display-generation callbacks leave current display/session unchanged.
C7 PASS when: candidate callback before commit and timeout/cancel paths cannot mutate committed authority.
C8 PASS when: overlapping handoff tests prove only one current transition/commit path exists.
C9 PASS when: docs and executable semantics are reviewed as equivalent.
```

## Evidence Contract

Each Attempt must record in Issue #2:

```text
Role: implementation | verification | combined
Task / Claim: R007 / C1..C9
Attempt:
Job ID: J1 | J2 | J3
Orchestrator:
Execution plane:
Runner class / image:
Execution host:
Target:
OS / architecture:
Rust toolchain:
Base commit:
Candidate commit:
Workflow / run / job:
Commands / test selector:
Duration / repetitions / shards:
Artifact / raw evidence:
Claim results:
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

Do not call static review or local-only reasoning runtime PASS.

## Failure / Blocked Handling

FAIL examples:

- any stale callback/result mutates newer authority;
- position telemetry increments command revision and creates telemetry-only command conflicts;
- two commands with the same accepted expected revision both commit incompatible mutations;
- request-id duplicate executes side effects twice;
- overlapping handoff can create two committable transitions;
- deterministic or required repeated suite exposes an invariant violation.

BLOCKED examples:

- GitHub-hosted test execution is unavailable after reasonable retry;
- the minimal Rust workspace cannot be established without a previously unknown repository/toolchain constraint;
- canonical contracts conflict in a way that requires Coordinator design revision before code can proceed.

Do not lower Success Criteria to manufacture PASS.

If the concurrency model itself must change beyond D1-D5, stop and request a Contract revision rather than hiding the design change in implementation.

## Deliverables

- minimal Rust workspace/Core Playback model;
- deterministic R007 concurrency suite;
- bounded repeated verification workflow/job;
- candidate commit/PR;
- updated `docs/implementation-contracts.md` with accepted semantics;
- optional durable `docs/research/r007-playback-concurrency.md` Evidence summary;
- Issue #2 `[EXECUTION REPORT]` and Coordinator Review history.

## Issue Feedback / Iteration Protocol

Follow `docs/tasks/issue-lifecycle-protocol.md`.

Normal implementation/test bugs or missing Evidence keep this same Task Contract:

```text
Attempt N
→ [EXECUTION REPORT]
→ Coordinator REVISE
→ status:ready
→ Attempt N+1
```

If Scope/Claims/Success Criteria/contract semantics themselves require change, return to `status:draft`, update this Task Contract/canonical docs and run Publication Gate again.

## Completion Protocol

Worker never closes this Issue.

R007 completes only when Coordinator reviews the candidate + J1/J2 required Evidence, accepts C1-C9, posts `[FINAL ACCEPTANCE]`, sets `status:done`, and closes Issue #2.

Closing R007 proves the concurrency contract is ready for later Core implementation; it does not prove R001 media path, R002 TV behavior, R003 resource baseline, or the overall Core Feasibility Gate.