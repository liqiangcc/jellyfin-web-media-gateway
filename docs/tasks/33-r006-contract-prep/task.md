# Task — R006-CONTRACT-PREP Generic Browser Worker Contracts

## Metadata

```text
GitHub Issue: #33
Parent Goal / Research Item: R006 / Generic Site Browser Worker + Native Site Panel
Task ID: R006-CONTRACT-PREP
Task kind: implementation / deterministic verification
Planning base: ebe74af4b8735b1fa0f438243833eb7fc9de83a6
Session bootstrap: docs/tasks/33-r006-contract-prep/prompt.md
Downstream handoff: docs/tasks/handoffs/cloud.md
Preferred worker: cloud-codex
Eligible environment: env:cloud
Execution plane: github-actions / GitHub-hosted runners
Accepted authority: Issue #14 / R008 + canonical Site Plugin/Vault/Playback boundaries
Hard publication dependencies: none beyond accepted canonical/security authority
Future target/runtime dependency: Issue #9 / R003-TARGET
```

## Goal

Implement the smallest **target-neutral** Browser Worker / BrowserEvent / ProfileAttachment / Native Panel contract layer and deterministic fake-worker harness required for later R006 runtime work.

This Task deliberately does not decide whether Chromium runs on the phone, how many workers exist, or whether the runtime is always-on/on-demand.

Required boundary:

```text
Gateway / Site Plugin
→ BrowserWorker contract
→ BrowserCommand / BrowserEvent
→ ProfileAttachmentRef
→ NativePanelSession / short-lived control token
→ deterministic fake worker / hosted harness
```

## Frozen boundaries

- Browser Worker contains no concrete-site DOM/API/login-success/next-item logic.
- Site Plugin remains the interpretation authority for BrowserEvent.
- Browser Worker cannot mutate PlaybackSession, active_display, session_revision or display_generation.
- source-changing browser interpretation still produces SourceLocator and follows normal Resolution/Playback Item Transition.
- Vault uniquely owns profile/session Secret material; contracts expose opaque refs only.
- R008 remains Egress/security authority.
- Native Panel contracts cannot become arbitrary remote desktop, server filesystem access, profile download or DRM/media-capture path.
- no real Chromium/Playwright process is launched in this Task.
- no phone-specific concurrency/idle-timeout/resource defaults are frozen.
- no claim is made that Chromium is viable on Ubuntu ARM64 phone.

## Required capabilities

### C1 — BrowserWorker interface

Provide a target-neutral lifecycle contract with minimal operations for equivalents of:

```text
create/open session
attach/detach profile reference
navigate
send generic input
query/status
subscribe/poll events
close/cancel
```

Exact method names are implementation-owned, but the public contract must not contain site-specific concepts.

Cancellation, timeout and closed/crashed session behavior must be explicit.

### C2 — BrowserEvent

Define a versioned/generic event surface sufficient for later Site Plugin interpretation.

Required semantic categories include:

- navigation/url/title change;
- loading/ready/error;
- user-input/result lifecycle where relevant;
- worker opened/closed/crashed/timed-out;
- generic browser/network denial/error class.

Do not put Bilibili/YouTube selectors/private API names in BrowserEvent/Core.

### C3 — ProfileAttachmentRef

Define an opaque short-lived profile/session attachment reference.

Requirements:

- no raw Vault path in plugin/control-facing representation;
- no Cookie DB/profile contents;
- explicit attach/detach/expiry/cleanup semantics;
- stale/expired attachment rejected;
- future runtime can materialize safely without changing the public contract.

### C4 — Generic Auth Mode boundary

Represent a generic worker/session mode usable by future interactive login without embedding account-site interpretation.

- Site Plugin interprets AccountState/login success;
- Browser Worker only exposes generic runtime events/input;
- password/code/QR content must not be persisted in normal contract/log artifacts;
- no real login is executed in this Task.

### C5 — Native Panel session/control boundary

Define a contract for a short-lived panel/control session bound to a specific Browser Worker session.

Required semantics:

- unpredictable/short-lived token or equivalent capability;
- bound worker/session identity;
- expiry/stale-session rejection;
- disconnect/reconnect outcome;
- default-deny clipboard/file upload/audio unless a later contract explicitly enables them;
- transport abstraction may exist, but production remote-frame streaming is not required here.

### C6 — Security / navigation boundary

Browser navigation/control contract must consume R008-compatible policy abstractions.

Prove that contract use does not grant:

- `file://` / local file access;
- loopback/private/link-local/metadata navigation through public-web scope;
- arbitrary configured-local-service access;
- profile/Cookie download;
- arbitrary server shell/filesystem authority.

### C7 — Deterministic fake worker / harness

Implement a fake/in-memory worker sufficient to test contracts without Chromium.

Harness must cover:

- normal lifecycle/event ordering;
- attach/navigate/input/close;
- cancellation;
- crash;
- timeout;
- stale/expired profile ref;
- stale/expired panel session/token;
- cleanup/isolation across two fake worker sessions.

### C8 — Stable failure taxonomy

Provide generic stable outcomes for semantic equivalents of:

```text
WORKER_UNAVAILABLE
PROFILE_ATTACH_FAILED
NAVIGATION_DENIED
WORKER_CRASHED
WORKER_TIMEOUT
SESSION_EXPIRED
PANEL_DISCONNECTED
INTERPRETATION_UNAVAILABLE/UNSUPPORTED
```

Names may vary but semantics must be deterministic and non-secret.

### C9 — Target strategy independence

Final Candidate must contain no phone-specific resource policy decision.

Do not freeze:

- always-on vs on-demand;
- worker pool size;
- target-specific idle timeout;
- CPU/RSS/process limits tuned for the phone;
- phone vs external Browser Worker host.

Those are decided by later R006 runtime/target work after #9 Evidence.

## Verification jobs

### J1 — Contract deterministic suite

GitHub-hosted exact-Candidate tests for C1-C5/C7/C8.

### J2 — Security/failure suite

GitHub-hosted exact-Candidate tests for C3/C5/C6/C7:

- navigation denial;
- profile non-exposure;
- stale/expired refs/tokens;
- crash/timeout/cancel cleanup;
- cross-session isolation.

### J3 — Affected regressions

Run exact-Candidate relevant current workspace + accepted R008/R007 regression suites as appropriate.

No Ubuntu ARM64 Target Runner is required.

## Task Success Criteria

Task execution is complete when:

1. C1-C9 have explicit evidence;
2. generic contracts compile and are exercised by deterministic fake worker;
3. no concrete-site knowledge appears in Browser Worker/Core surfaces;
4. profile/session Secrets remain opaque refs;
5. navigation/security boundaries integrate R008-compatible policy;
6. Native Panel token/session stale/expiry behavior is tested;
7. fake worker crash/cancel/timeout cleans up deterministically;
8. J1/J2/J3 exact-Candidate Evidence is recorded;
9. no real Chromium/phone/runtime viability claim is made.

## Evidence Contract

Worker `[EXECUTION REPORT]` must include:

```text
Attempt:
Base commit:
Candidate commit:
PR:
Contract/API summary:
Claims C1-C9:
J1/J2/J3 run + job IDs:
Fake-worker lifecycle result:
Profile non-exposure/stale result:
Navigation/Egress denial result:
Panel session/token stale/expiry result:
Crash/cancel/timeout cleanup result:
Affected regressions:
Limitations:
Result: COMPLETED | BLOCKED
```

## Out of scope

- launching Chromium/Playwright;
- phone resource/lifecycle verification;
- always-on/on-demand/concurrency decision;
- concrete-site DOM/API logic;
- real login/account/profile acquisition;
- production remote desktop/frame streaming;
- unrestricted clipboard/file upload/audio;
- DRM/protected-content capture;
- Gateway user/RBAC;
- changing R007 Playback semantics.

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

If blocked: `[BLOCKER REPORT] → status:blocked → release owner → STOP`.

Worker must not mark done, close #33, start target/runtime verification, or auto-start another Task.