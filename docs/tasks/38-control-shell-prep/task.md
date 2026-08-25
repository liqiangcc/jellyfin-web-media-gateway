# Task — CONTROL-SHELL-PREP PlaybackSession HTTP / snapshot / reconnect contracts

## Metadata

```text
GitHub Issue: #38
Parent Goal / Research Item: Phase 0A-2 / PlaybackSession + Control Shell
Task / Research ID: CONTROL-SHELL-PREP
Task kind: combined
Base commit: 81d08f02b6928543f06d43e9f6a7a2cfa54fbdd1
Candidate commit: n/a until Worker execution
Session bootstrap prompt: docs/tasks/38-control-shell-prep/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, rust-build, rust-test, github-actions-orchestration
Hard publication dependencies: Issue #2 / R007 Final Acceptance; Issue #14 / R008 Final Acceptance — both satisfied
```

Realtime status/owner/branch/PR/Evidence remains in GitHub Issue #38. Attempt history follows `docs/tasks/issue-lifecycle-protocol.md`.

## Goal

Implement the smallest server-side Control shell around the already accepted R007 `PlaybackSession` authority so a Control client can:

```text
read canonical PlaybackSession snapshot
→ submit request_id / expected_session_revision command
→ receive deterministic R007 result/error
→ reconnect using snapshot + monotonic later events
```

without Control owning a second playback truth or exposing media/session Secrets.

## Why / Context

R007 already proves command CAS, request-id idempotency, item/media freshness, display generation and handoff concurrency in an in-memory executable model. The canonical Control architecture requires Control to be View + Command, and refresh/reconnect to rebuild from Gateway snapshot rather than local UI state.

Current proof server paths do not yet provide a generic server-managed session/snapshot/command/reconnect shell. This Task creates that cloud-verifiable boundary before any physical TV, source-site, Chromium or production Control UI work.

Canonical inputs:

- `docs/control-experience-architecture.md`
- `docs/implementation-contracts.md`
- `docs/security.md`
- `docs/mvp-plan.md`
- accepted Issue #2 / R007 semantics
- accepted Issue #14 / R008 security semantics

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: all required Claims are deterministic repository/HTTP/concurrency contracts that can be proven by GitHub-hosted CI; no independent device Evidence Authority is required.
```

## Worker Routing Decision

```text
ordinary Rust repository implementation + HTTP contract tests + GitHub Actions
→ cloud-codex
→ env:cloud

Verification
→ GitHub Actions / github-hosted ubuntu-latest x64
```

No phone, TV, self-hosted runner, real site, real Jellyfin or Chromium runtime capability is required.

## Work Role

### Implementation

Produce a candidate/PR that adds a generic Control service/router boundary around accepted `gateway_core::playback::PlaybackSession` semantics.

The implementation may refactor serialization/accessors needed to expose safe snapshots, but must not change accepted R007 authority semantics merely to simplify HTTP handling.

### Verification

Claims to verify:

- C1 — session store isolation and deterministic lookup/not-found behavior.
- C2 — canonical non-secret snapshot is derived from live `PlaybackSession`, not duplicated mutable Control state.
- C3 — HTTP command envelope maps to accepted R007 `CommandEnvelope` semantics including request-id idempotency and `expected_session_revision` CAS.
- C4 — rejected/replayed/conflicting commands have deterministic error mapping and no unintended mutation/event side effect.
- C5 — monotonic event cursor plus snapshot/reconnect reconstructs current state without resetting or replaying destructive commands.
- C6 — concurrent Control mutations preserve accepted R007 concurrency: two commands against the same old revision cannot both commit.
- C7 — position telemetry can update snapshot/event observation without advancing command `session_revision`; stale telemetry remains rejected by R007 rules.
- C8 — HTTP/log/security boundary is bounded and non-secret; Control snapshot/events never expose raw `resolved_media`, Cookie/Authorization/Vault material or arbitrary upstream URL authority.
- C9 — R001/R007/R008/R004/R005/R006 authority boundaries remain intact.

## Preconditions

- Issue #2 / R007 is Final Accepted and merged.
- Issue #14 / R008 is Final Accepted and merged.
- Current planning base is `main@81d08f02b6928543f06d43e9f6a7a2cfa54fbdd1`.
- Worker must integrate current accepted `main` before final exact-Candidate verification if main advances.
- No external service/device is required.

## In Scope

- server-managed in-memory `PlaybackSession` store or equivalent deterministic authority container;
- server-generated/opaque session identity and isolated per-session mutation locking;
- safe snapshot DTO/view containing at least session revision, playback state, item identity/revision/media generation, position/telemetry sequence, active display authority and applicable handoff summary;
- snapshot must omit raw `resolved_media` and Secret-bearing upstream details;
- HTTP endpoint equivalent to `GET /api/v1/sessions/{id}`;
- HTTP endpoint equivalent to `POST /api/v1/sessions/{id}/commands`;
- command DTOs for accepted R007 stable commands: play, pause, seek, stop, begin handoff;
- deterministic mapping of R007 semantic errors to stable HTTP/application error codes;
- bounded monotonic per-session event journal/cursor or equivalent deterministic reconnect mechanism;
- trusted/internal test/service hook for accepted R007 position telemetry so C7 is executable;
- deterministic tests for snapshot, command CAS/idempotency, concurrency and reconnect;
- GitHub Actions workflow/equivalent exact-Candidate CI.

## Out of Scope

- source URL recognition/resolution or creating a session directly from arbitrary caller-selected media URL;
- public source-to-session creation UX; tests may seed a `PlaybackSession` through internal/service fixture APIs;
- actual Display network transport, WebSocket implementation, production SSE, TV autoplay or audible playback;
- full `/control` web UI or ControlView product UX;
- Site Plugin, Bilibili, Browser Worker, Native Site Panel or real login;
- Vault production persistence/encryption;
- Jellyfin real client/server behavior;
- R003 phone resource/thermal measurement;
- redefining R007 command/revision/item/media-generation/display-generation/handoff semantics.

## Architecture Invariants

1. `PlaybackSession` / accepted R007 remains Playback authority; Control shell serializes and routes commands only.
2. Control has no second mutable truth for playback state.
3. `session_revision` is command CAS authority; position telemetry alone does not advance it.
4. duplicate `request_id` with the same fingerprint returns the recorded semantic outcome; incompatible reuse is rejected.
5. stale `expected_session_revision` is rejected before mutation.
6. event sequence/cursor is reconnect transport metadata, not a replacement for `session_revision` or `item_revision`.
7. safe snapshots/events do not expose raw media upstream details or source-site/Jellyfin/Vault Secrets.
8. this Task does not make Browser Worker, Site Plugin, Jellyfin or DisplayAdapter authoritative for playback.
9. HTTP security/error logging follows accepted R008/canonical non-secret boundaries.

## Files Expected to Change

Implementation-owned, likely including:

- `gateway-core/src/control.rs` or equivalent new module;
- `gateway-core/src/lib.rs`;
- `gateway-core/src/playback.rs` only for safe snapshot/accessor/refactor needs that preserve R007 semantics;
- `gateway-core/tests/*control*`;
- `.github/workflows/*control*`.

Do not modify canonical architecture merely to fit an implementation shortcut. Genuine contradictions must be reported to Coordinator.

## Implementation Requirements

1. Provide a concurrency-safe session store; mutation of one session must not serialize unrelated sessions unnecessarily.
2. Session IDs are opaque/server-issued in production-facing shapes; route input cannot grant caller-selected filesystem/network authority.
3. Snapshot is generated from the live `PlaybackSession` under a coherent read/lock and is internally self-consistent.
4. Do not serialize `PlaybackItem.resolved_media` into Control-facing snapshot/event output.
5. Command endpoint accepts a bounded structured command body containing `request_id`, optional `expected_session_revision`, and a stable command shape.
6. Preserve R007 exact idempotency/CAS semantics; do not implement a second dedupe table in HTTP code that can disagree with `PlaybackSession`.
7. Map semantic errors to stable non-secret application error codes; HTTP status mapping must be deterministic and tested.
8. Successful state-changing commands append a monotonic event after the accepted mutation; rejected semantic commands must not produce a fake successful state event.
9. Event cursor behavior must define unknown/expired cursor behavior explicitly. Bounded retention may return a stable `snapshot_required`/equivalent result rather than pretending history is complete.
10. Provide reconnect tests: get snapshot at S, apply later mutations, read events after cursor, reconstruct current state without reissuing commands.
11. Provide telemetry test path proving newer valid telemetry updates position/telemetry sequence while leaving command revision unchanged; stale item/revision/sequence telemetry is ignored.
12. Add deterministic two-Control concurrent mutation test through the service/HTTP boundary.
13. Request body size/content-type/diagnostic behavior must be bounded and non-secret; no raw upstream URL/proxy parameter is introduced.
14. Final Candidate must integrate current accepted `main` and rerun all required jobs on that exact SHA.

## Verification Plan

### Claims

```text
C1 session store isolation/lookup
C2 canonical safe snapshot
C3 R007 HTTP command mapping/idempotency/CAS
C4 deterministic reject/no-side-effect behavior
C5 event cursor + reconnect
C6 concurrent Control mutation
C7 telemetry/revision separation
C8 HTTP/security/Secret boundary
C9 cross-domain regression preservation
```

### Verification Job Matrix

| Job ID | Claims | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1-C5,C7 | github-actions | ubuntu-latest x64 | runner-self | yes | fmt/clippy + deterministic Control/session HTTP contract tests | exact SHA + run/job |
| J2 | C4,C6,C8 | github-actions | ubuntu-latest x64 | runner-self | yes | concurrency, revision/idempotency negatives, oversized/content-type/redaction/no-raw-media tests | exact SHA + run/job |
| J3 | C9 + affected C3/C7 | github-actions | ubuntu-latest x64 | runner-self | yes | `cargo test --workspace --all-targets` + accepted R007/R008/R001 affected regressions | exact SHA + run/job |

### Execution Plane

```text
Execution plane: github-actions
Target proof required: no
Interactive external debugging required: no by default
```

Cloud shell checks may aid development but do not replace required Actions Evidence.

## Success Criteria

### Task success

1. All C1-C9 have explicit exact-Candidate Evidence.
2. A reusable generic Control/session service boundary exists and is exercised through deterministic HTTP/service tests.
3. Snapshot/reconnect demonstrates Control recovery without local second truth.
4. Command semantics remain exactly compatible with accepted R007 behavior.
5. No raw `resolved_media`, source-site/Jellyfin/Vault Secret or arbitrary upstream URL authority is exposed to Control.
6. Exact-Candidate J1/J2/J3 are green.
7. Candidate is reviewable in a PR and Worker stops at `status:review`.

### Verification claim success

```text
C1 PASS when two seeded sessions remain isolated and missing IDs fail deterministically.
C2 PASS when snapshot fields match one coherent live PlaybackSession state and omit raw resolved-media/Secret details.
C3 PASS when HTTP play/pause/seek/stop/handoff produce the same revision/idempotency semantics as direct accepted R007 execution.
C4 PASS when stale revision, incompatible request-id reuse and other semantic rejects mutate neither authoritative state nor successful-state event history.
C5 PASS when snapshot + after-cursor events recover later current state; expired/unknown cursor has an explicit snapshot-required behavior.
C6 PASS when concurrent requests using one old expected revision cannot both commit.
C7 PASS when valid newer telemetry updates position/telemetry sequence without session_revision bump and stale telemetry is rejected.
C8 PASS when negative HTTP/security/redaction tests show bounded structured input and no Secret/raw media URL leakage.
C9 PASS when required workspace/R001/R007/R008 affected regressions remain green.
```

## Evidence Contract

Worker `[EXECUTION REPORT]` must include at minimum:

```text
Attempt:
Base commit:
Candidate commit:
PR:
Implementation summary:
Session store/snapshot result:
Command/idempotency/CAS result:
Concurrent-Control result:
Reconnect/event cursor result:
Telemetry/revision result:
Secret/raw-media redaction result:
Claims C1-C9:
J1/J2/J3 workflow run + job IDs:
Exact-Candidate checkout assertion:
Affected R001/R007/R008 regressions:
Limitations / unverified:
Result: COMPLETED | BLOCKED
```

No Secret, Cookie, Authorization, API key, full signed upstream URL or production account data may appear in artifacts/comments.

## Failure / Blocked Handling

- deterministic implementation/test defect is not a blocker; fix within the same Issue/Attempt when practical or report for Coordinator `REVISE`;
- inability to preserve accepted R007 semantics is a substantive conflict and must be reported, not papered over with a second authority;
- missing GitHub Actions/execution permission that prevents required exact-Candidate Evidence is `BLOCKED`;
- no device availability is needed, so phone/TV unavailability is not a blocker for this Task;
- do not weaken concurrency/security criteria to manufacture PASS.

## Deliverables

- generic Control/session implementation and tests;
- candidate commit + PR;
- exact-Candidate J1/J2/J3 Actions Evidence;
- `docs/tasks/38-control-shell-prep/prompt.md` bootstrap already supplied by Coordinator;
- no separate target Verification Task.

## Completion Protocol

Worker follows `docs/tasks/issue-lifecycle-protocol.md`:

```text
status:ready + env:cloud + no owner
→ claim
→ status:in-progress
→ Attempt N
→ implementation + exact-SHA J1/J2/J3
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

If blocked: `[BLOCKER REPORT] → status:blocked → release owner → STOP`.

Worker must not set `status:done`, close #38, start another Task, publish device work, or silently change this Contract.