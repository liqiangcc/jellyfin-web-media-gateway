# Task — R005-AUTH-PREP Session Vault / Scoped Site Access

## Metadata

```text
GitHub Issue: #28
Parent Goal / Research Item: R005 / Authenticated Site Session Gate
Task ID: R005-AUTH-PREP
Task kind: implementation / deterministic verification
Planning base: 2034e7c4e13d5eb7f92983ed9f0856d21b71d3e5
Session bootstrap: docs/tasks/28-r005-auth-prep/prompt.md
Downstream handoff: docs/tasks/handoffs/cloud.md
Preferred worker: cloud-codex
Eligible environment: env:cloud
Execution plane: github-actions / GitHub-hosted runners
Accepted authorities: Issue #14 / R008, Issue #2 / R007
Hard publication dependencies: none beyond accepted canonical/security authorities
Parallel task: Issue #23 / R005-PUBLIC
```

## Goal

Implement the deterministic server-side authentication infrastructure needed by future authenticated Site Plugin resolution without using a real source-site account, Browser Worker, phone, TV, production Secret, or site-specific login logic.

Required architecture:

```text
SiteAccount metadata
→ SiteSessionRef
→ Session Vault ownership
→ AccountState
→ non-secret PendingIntent
→ scoped SiteAccessCapability
→ central R008-controlled HTTP injection boundary
→ atomic candidate-session validation/swap primitives
```

This Task is preparation only. It must not claim that authenticated Bilibili or any other real site works.

## Frozen boundaries

- `vault/` is the unique owner of Cookie/token/localStorage/profile Secret material.
- Site Plugin, Control, Display and Target Runner never receive raw Vault filesystem authority.
- SiteAccessCapability is metadata/capability only; plugin-facing data contains no raw Cookie jar/token/profile.
- SourceLocator and ResolvedMedia public headers remain Secret-free.
- Core receives no Bilibili/YouTube-specific auth key, DOM selector or account endpoint.
- Browser Worker, real login, QR flow, CAPTCHA/code input and password persistence are out of scope.
- Accepted R008 `EgressPolicy` remains the network authority.
- Accepted R007 remains Playback command/revision authority; PendingIntent is retry metadata, not a second Playback state authority.
- Current public resolution and Jellyfin paths must remain behaviorally independent.

## Required implementation capabilities

### C1 — SiteAccount / SiteSessionRef / AccountState

Provide stable generic server-side contracts for MVP one-active-account-per-site behavior.

Account state must represent semantic equivalents of:

```text
unknown
checking
valid
expired
login_required
error
```

Account/session identifiers exposed outside Vault must be non-secret references.

### C2 — Session Vault ownership

Provide a Vault API/storage boundary for structured Secret material.

Requirements:

- production Vault path is not required by CI;
- deterministic tests use isolated temporary storage;
- fake Secret sentinels are used to prove redaction/non-leakage;
- Debug/error output never reveals Secret values;
- plugin/control/display-facing APIs expose only refs/capabilities.

This Task may establish storage interfaces and a safe test implementation. It does not need production migration/deployment tooling.

### C3 — Scoped SiteAccessCapability

Capability must bind at minimum:

```text
site_id
account_ref?
allowed_hosts
expiry
capability_id
```

Deterministically reject:

- wrong site;
- wrong account;
- expired capability;
- disallowed host;
- cross-site session access;
- stale capability after active-session replacement when the contract requires invalidation.

The capability itself contains no raw Secret.

### C4 — Controlled authenticated HTTP boundary

Provide or complete a server-side capability-consumption path that integrates accepted R008 Egress behavior.

Requirements:

- Cookie/Authorization injection occurs only inside trusted server-side infrastructure;
- plugin code supplies no arbitrary auth headers;
- public/private/configured-local egress semantics remain owned by R008;
- redirect/host scope remains centrally validated;
- no plugin-local private-network bypass is introduced.

### C5 — PendingIntent

Define a non-secret, serializable/recoverable retry-intent contract sufficient for later auth retry.

It may retain generic references such as:

- original SourceLocator;
- requested display reference/action;
- generic playback intent metadata;
- expected/current authority references needed to detect staleness.

It must contain no Cookie/token/profile/password/verification-code material and must not independently mutate R007 revision state.

### C6 — Candidate-session validation / atomic swap

Provide deterministic primitives for:

```text
old active session (optional)
→ create/store candidate session
→ validate candidate result supplied by the future auth flow
→ atomic active-session ref swap
→ cleanup old material after successful swap
```

Required negative behavior:

- validation failure preserves previous valid active session;
- cancellation preserves previous valid active session when present;
- partial write cannot expose a half-swapped active session;
- cleanup behavior is bounded/observable.

### C7 — Expiry / logout / rotation

Provide deterministic state-transition primitives for:

- expired session;
- login-required state;
- logout;
- invalid/revoked session;
- active-session replacement;
- capability/session rotation;
- stale access after replacement.

Observable results must remain non-secret.

### C8 — Security regression

Must prove:

- fake Secret values are absent from logs/errors/artifacts;
- accepted R008 relevant tests remain PASS;
- no real account/profile/production Secret is required;
- no phone/TV/self-hosted target execution is required.

### C9 — Integration boundary

Final Candidate must preserve:

- R001 media/Secret boundary;
- R007 Playback authority;
- R008 Egress/security boundary;
- R004/Jellyfin optional adapter behavior;
- R005-PUBLIC ability to proceed without a new site-specific Core dependency.

If the implementation reveals a real contradiction with the generic SourceLocator/ResolvedMedia contracts, report it to Coordinator rather than silently redefining them.

## Verification jobs

### J1 — Auth-domain deterministic suite

GitHub-hosted exact-Candidate tests covering C1/C2/C3/C5/C6/C7.

### J2 — Security / failure suite

GitHub-hosted exact-Candidate tests covering:

- wrong site/account;
- expired/stale capability;
- cross-site access;
- disallowed host/redirect;
- failed/cancelled swap preserving old active session;
- Secret sentinel/redaction scan.

### J3 — Affected regressions

Run exact-Candidate relevant regression suites for current workspace plus accepted R008 and affected R001/R007 surfaces.

Do not use Ubuntu ARM64 Target Runner for this Task.

## Task Success Criteria

Task execution is complete when:

1. C1-C9 have explicit PASS/FAIL/BLOCKED evidence;
2. Vault ownership is executable/tested, not documentation-only;
3. scoped SiteAccess rejects wrong-site/account/host/expiry cases;
4. Secret injection remains server-side behind R008 egress validation;
5. PendingIntent is non-secret and authority-safe;
6. atomic replacement failure preserves old valid session;
7. expiry/logout/rotation primitives have deterministic tests;
8. J1/J2/J3 exact-Candidate Evidence is recorded;
9. no real login/Browser Worker/authenticated-site PASS is claimed.

## Evidence Contract

Worker `[EXECUTION REPORT]` must include:

```text
Attempt:
Base commit:
Candidate commit:
PR:
Implementation summary:
Storage/test-Vault strategy:
Claims C1-C9:
J1/J2/J3 run + job IDs:
Secret sentinel/redaction result:
Atomic-swap failure/cancel result:
Cross-site/account/expiry/host-scope results:
Affected R008/R001/R007 regressions:
Limitations:
Result: COMPLETED | BLOCKED
```

## Out of scope

- real Bilibili or other site login;
- authenticated real-site media proof;
- Browser Worker / Native Site Panel;
- production Vault migration/deployment;
- Gateway user identity/RBAC;
- more than one active account per site;
- password/verification-code/QR persistence;
- DRM/paywall/region/access-control bypass;
- changing R007 concurrency/revision semantics;
- changing generic SourceLocator/ResolvedMedia semantics without Coordinator review.

## Completion protocol

Worker follows `docs/tasks/issue-lifecycle-protocol.md`:

```text
status:ready + env:cloud + no owner
→ claim
→ status:in-progress
→ Attempt N
→ implementation + exact-SHA Evidence
→ [EXECUTION REPORT] + status:review + release owner
```

If blocked, post `[BLOCKER REPORT]`, set `status:blocked`, release ownership and STOP.

Worker must not mark `status:done`, close #28, start R005-AUTH-REAL, or start another Task.