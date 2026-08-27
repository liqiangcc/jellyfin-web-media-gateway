# Task — WEB-DISPLAY-SESSION-PREP Web Display registration / lease / generation-safe callbacks

## Metadata

```text
GitHub Issue: #45
Parent Goal / Research Item: Phase 0A-2 / Web Display session integration
Task / Research ID: WEB-DISPLAY-SESSION-PREP
Task kind: combined
Base commit: cb3e205e8b4caa4657b00f83f9e32c0f68777f1b
Candidate commit: n/a until Worker execution
Session bootstrap prompt: docs/tasks/45-web-display-session-prep/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, rust-build, rust-test, github-actions-orchestration
Hard publication dependencies: accepted #38/R007; #40 accepted for projection compatibility
```

Realtime status/owner/branch/PR/Evidence lives only in Issue #45. Attempt history follows `docs/tasks/issue-lifecycle-protocol.md`.

## Goal

Implement the generic Web Display runtime session boundary needed for a browser display to register with Gateway, maintain a bounded liveness lease, obtain current server-owned Playback display context, and submit callbacks that are accepted only when both the browser lease and the accepted R007 item/display generation context are current.

```text
Web Display page
→ register / renew opaque display lease
→ Gateway-owned display_id + page-instance lease
→ read current Playback display context
→ media capability / rendering context
→ heartbeat + status / position callback
→ lease validation
→ R007 session/item/item_revision/display_generation validation
→ accept current callback or reject stale callback
```

## Why / Context

Accepted #38 intentionally exposes Playback snapshot/command/reconnect but does not provide a production Display registry. Accepted #40 can project `DisplayInstance` / `DisplayStatus`, but those inputs must come from a real generic display authority rather than caller-attested state. This Task supplies that missing Web Display runtime boundary without changing R007 semantics.

A browser page instance is ephemeral. Reconnect/refresh therefore needs a separate opaque **page lease / attachment epoch**. That lease is not `display_generation` and must never replace R007 display authority.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: Issue #7 remains physical-TV acceptance authority
Decision reason: registration/liveness/stale-callback contracts are deterministic and GitHub-hosted; real audible autoplay remains a separate manual/device claim.
```

## Worker Routing Decision

```text
implementation: cloud-codex / env:cloud
verification: GitHub Actions / github-hosted ubuntu-latest x64
```

No phone, physical TV, Jellyfin client, real source site or Browser Worker is required.

## Claims

- C1 — generic Web Display registration issues/records a server-owned display identity and opaque page lease; caller cannot choose R007 generation authority.
- C2 — page lease/attachment epoch is explicitly separate from R007 `display_generation` and `session_revision`.
- C3 — bounded heartbeat/TTL produces deterministic online/offline state without incrementing Playback command revision.
- C4 — current display context is derived from accepted PlaybackSession/ControlSnapshot authority; browser cannot self-assert a newer item/display generation.
- C5 — status/position/error callback is accepted only when lease + session + item + item_revision + display_id + display_generation match current authority.
- C6 — refresh/reconnect invalidates or supersedes an old page lease so an old page cannot keep callback rights even if display identity remains the same.
- C7 — handoff candidate callbacks remain candidate observations only; callback cannot commit `active_display` or generation.
- C8 — HTTP registration/heartbeat/context/callback surfaces use accepted trusted-LAN Host/Origin/body-limit security and expose no raw upstream/Vault/source-site Secret authority.
- C9 — accepted #40 `DisplayViewInput` can consume current generic instance/status/error facts without weakening its stale-context checks; no Web/Jellyfin branch is added to projection Core.

## Preconditions

- `main@cb3e205e8b4caa4657b00f83f9e32c0f68777f1b` contains accepted #39 and #40.
- R007 remains the sole item/display-generation/handoff commit authority.
- #7 remains the physical-TV Evidence Authority.
- Worker must integrate live accepted main before final exact-Candidate Evidence if main advances.

## In Scope

- generic Web Display registry/service;
- server-issued bounded `display_id` and/or registration identity;
- opaque short-lived page lease/token/epoch with redacted Debug;
- heartbeat/lease expiry and deterministic online/offline state;
- safe current Playback display context lookup;
- current-generation status/position/error callback validation;
- integration with existing R007 position telemetry where applicable;
- safe generic `DisplayInstance`, `DisplayStatus`, contextual display error production for #40 projection;
- HTTP endpoints or equivalent service contract for register/heartbeat/context/callback;
- deterministic fake-browser tests and exact-SHA GitHub Actions.

## Out of Scope

- physical TV autoplay verdict or remote-control UX acceptance (#7 owns this);
- TV viewport/subtitles/fullscreen polish (#48 later);
- source recognition/session creation (#44 later);
- Jellyfin special cases;
- Browser Worker / Native Site Panel;
- real source-site login;
- changing R007 command/revision/item/media-generation/display-generation/handoff semantics;
- a global display revision that replaces R007 generation.

## Architecture Invariants

1. Registration authority and Playback authority are separate: registry owns liveness/lease, R007 owns active/candidate display generation.
2. A browser lease/token is opaque, short-lived and non-secret in logs; possessing an old lease never grants new generation authority.
3. Heartbeat does not mutate `session_revision`.
4. Callback validation occurs before side effects.
5. Old session/item/item revision/display generation/page lease cannot overwrite current state.
6. Candidate handoff observation cannot commit active display.
7. Display-facing responses contain Gateway media capabilities/safe metadata only, never source Cookie/Authorization/Vault material.
8. R008 Host/Origin/body-limit/security semantics are reused rather than redefined.

## Files Expected to Change

Likely:

- `gateway-core/src/display_session.rs` or equivalent;
- `gateway-core/src/lib.rs` router/service wiring;
- `gateway-core/src/control.rs` only for narrow trusted telemetry/context accessors if required;
- deterministic tests;
- `.github/workflows/*web-display-session*`.

Do not change canonical architecture just to fit an implementation shortcut.

## Implementation Requirements

1. Define a generic display registry/lease contract; no concrete browser/TV brand logic.
2. Server must generate the page lease and any registration identity. Caller-selected `display_generation` is forbidden.
3. Keep lease epoch separate from R007 `display_generation`; name/types/tests must make the distinction explicit.
4. Heartbeat renews only registry liveness and must not call an R007 command or bump `session_revision`.
5. Context lookup must bind to an existing PlaybackSession snapshot; it cannot create sessions or mutate active display.
6. Callback input must carry enough accepted context to validate current session/item/item_revision/display_id/display_generation plus lease identity before mutation.
7. Position callbacks must route through accepted R007/Control telemetry semantics; observation/error storage must be generic and context-bound.
8. Reconnect/refresh must supersede prior page-instance authority. Add a regression where an old page callback is rejected after a new lease is established.
9. Provide contextual error input compatible with accepted #40 projection; raw protocol/error details must remain redacted.
10. Apply existing `HttpAuthorityPolicy`, same-origin expectations, body limits and bounded identifiers to new HTTP routes.
11. Required tests use deterministic fake browser clients only; no target-TV claim.
12. Final Candidate integrates live accepted main and all required jobs assert exact final SHA.

## Verification Plan

| Job | Claims | Execution plane | Runner | Required | Intent |
|---|---|---|---|---|---|
| J1 | C1-C4,C6 | GitHub Actions | ubuntu-latest | yes | registration, lease, heartbeat, reconnect, current-context contracts |
| J2 | C5,C7,C8,C9 | GitHub Actions | ubuntu-latest | yes | stale lease/session/item/generation, candidate handoff, HTTP/security/redaction negatives |
| J3 | C1-C9 | GitHub Actions | ubuntu-latest | yes | workspace + affected #38/R007/#40/R001/R008/display regressions |

```text
Target proof required: no
Interactive external debugging: no by default
```

## Success Criteria

1. C1-C9 have exact-Candidate executable Evidence.
2. A fake browser can register, heartbeat, obtain current context and send a current callback.
3. Old page lease, wrong session/item/item revision/display generation callbacks are deterministically rejected before side effects.
4. Heartbeat does not change Playback command revision.
5. Handoff candidate callback cannot commit active display.
6. #40 can consume safe generic current display facts without adapter-specific branching.
7. No upstream/Vault/source-site Secret or raw protocol detail reaches public DTO/Debug/Evidence.
8. J1/J2/J3 pass on the exact final Candidate and Candidate is in a reviewable PR.

## Evidence Contract

`[EXECUTION REPORT]` must include:

```text
Attempt:
Base commit:
Candidate commit:
PR:
Display registry/lease API location:
Lease vs R007 display_generation separation proof:
Registration/heartbeat/reconnect result:
Current-context result:
Stale callback matrix result:
Handoff candidate isolation result:
#40 projection integration result:
HTTP/security/redaction result:
Claims C1-C9:
J1/J2/J3 run + job IDs:
Exact-Candidate assertion:
Affected regressions:
Limitations:
Result: COMPLETED | BLOCKED
```

No Secret, Cookie, Authorization, raw media upstream URL, lease token value or production account data may appear in Evidence.

## Failure / Blocked Handling

- implementation/test defects stay in the same Task/next Attempt;
- if correct registration requires redefining R007 active display/generation semantics, report an architecture conflict instead of doing so;
- inability to run exact-SHA GitHub Actions Evidence is BLOCKED;
- phone/TV unavailability is not a blocker for this Task.

## Deliverables

- generic display registry/lease/callback implementation;
- deterministic tests/workflow;
- exact-Candidate PR/Evidence;
- `docs/tasks/45-web-display-session-prep/prompt.md`.

## Completion Protocol

```text
claim → status:in-progress → Attempt N
→ candidate + exact-SHA J1/J2/J3
→ [EXECUTION REPORT] → status:review → release owner → STOP
```

Blocker path: `[BLOCKER REPORT] → status:blocked → release owner → STOP`.
Worker never sets `status:done`, closes #45, executes #7/#44/#48, merges its own PR, or silently changes this Contract.