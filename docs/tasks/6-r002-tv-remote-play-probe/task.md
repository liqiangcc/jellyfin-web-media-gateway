# Task — R002-PREP TV Remote Playback Probe

## Metadata

```text
GitHub Issue: #6
Parent Goal / Research Item: R002 / P0 TV Browser Remote Playback / Autoplay
Task / Research ID: R002-PREP
Task kind: implementation
Planning base commit: cacdb6b543b40612be8ba1014e3a0ed5331bd42b
Session bootstrap prompt: docs/tasks/6-r002-tv-remote-play-probe/prompt.md
Downstream handoff profile: docs/tasks/handoffs/web-gpt.md
Preferred worker: web
Eligible worker environments after publication: env:web-gpt
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, browser-automation
Linked verification task: Issue #7 / R002-TV
Hard publication dependency: stable reusable R001 Web Display/media-path candidate from Issue #3, accepted or explicitly approved by Coordinator for downstream use
```

> Live status, owner, candidate and results belong in Issue #6. This Task is not R002 device acceptance; physical-TV Evidence belongs to Issue #7.

## Goal

Prepare the smallest Web Display probe needed to let a real television answer the R002 hypothesis without ambiguity.

The probe must let a remote Control/session trigger an **audible** playback attempt on `/display`, record the actual browser result/rejection, support a one-time user-activation initialization path when needed, expose lifecycle/reconnect/visibility evidence, and remain usable when Fullscreen is denied.

This Task must not claim R002 `PASS` or `CONDITIONAL PASS`; it only delivers a reproducible candidate for physical verification.

## Why / Context

R002 is Core-blocking because the product assumes a TV can remain on the Gateway Display while later playback is primarily driven from the phone. Browser autoplay/user-activation behavior is target-browser dependent and cannot be established by design reasoning or desktop Chromium alone.

R001 is currently building the minimal Web media path. R002-PREP must reuse that path rather than create a competing media/display stack.

## Task Decomposition Decision

```text
Verification mode: separate-task
Linked implementation task: Issue #6 (this Task)
Linked verification task: Issue #7
Decision reason: physical TV audible-playback behavior is an independent Manual Evidence Authority, has a different owner/lifecycle, and its PASS/FAIL result is itself a P0 research deliverable. Splitting is based on Evidence Authority and deliverables, not merely environment.
```

## Hard Dependency

R002-PREP may be planned now but must not be published until Issue #3 exposes a stable reusable candidate that contains the minimal Web Display/media path needed by this probe.

The dependency can be satisfied before R001 Final Acceptance only if the Coordinator explicitly records that a specific R001 candidate SHA is stable enough for downstream probe work.

An open/in-progress PR alone is not sufficient authority.

## Work Role

### Implementation

Build only the probe/instrumentation needed by Issue #7:

- reuse the R001 Web Display/media endpoint;
- expose a deterministic remote `play` trigger path suitable for a phone/Control or a minimal test controller;
- attempt audible `HTMLMediaElement.play()` without silently converting the acceptance condition into muted-only autoplay;
- record promise resolve/reject and browser error name/message (`NotAllowedError` or equivalent) in structured, non-secret diagnostics;
- provide a clear one-time activation UI such as “press OK/confirm to enable remote playback” without assuming the activation will persist;
- allow the same remote trigger to be repeated after playback end and long idle periods;
- expose visibility/page lifecycle/WebSocket-or-equivalent reconnect observations needed by manual verification;
- provide viewport-immersive TV layout that remains usable when Fullscreen is denied;
- make Fullscreen request result observable but never make Fullscreen success a prerequisite for playback;
- provide an explicit reset/reload path so manual Cases A-F are reproducible;
- keep secure-context APIs optional enhancements; ordinary LAN HTTP must still allow the baseline probe where browser policy permits.

### Verification

This Task verifies probe correctness on portable/desktop automation only. It does **not** verify target-TV autoplay policy.

Claims:

```text
C1: A remote event can reach the probe and invoke exactly one observable audible play attempt.
C2: play() resolve/reject/error telemetry is captured and correlated to the remote attempt.
C3: the activation/bootstrap interaction and retry path are deterministic and repeatable.
C4: viewport-immersive operation remains usable when Fullscreen is rejected/unavailable.
C5: lifecycle/visibility/reconnect events required by Issue #7 are observable without exposing Secrets.
C6: the probe reuses R001 media/display boundaries and does not create a second Playback/media authority.
```

## In Scope

- minimal `/display` probe changes required for R002;
- minimal remote trigger/control surface for the experiment;
- audible `play()` result/error instrumentation;
- one-time activation UX and retry;
- Fullscreen allow/deny degradation;
- viewport immersive behavior;
- page visibility/lifecycle/reconnect telemetry;
- deterministic fixture/hosted-browser tests for probe mechanics;
- documentation needed to run Issue #7.

## Out of Scope

- claiming real-TV autoplay success;
- replacing R001 media path;
- R007 concurrency/state-machine redefinition;
- full Control application;
- TV app packaging/native Android TV app;
- DRM/real site auth;
- Jellyfin;
- R003 performance/thermal proof;
- browser-specific hacks that bypass the intended audible-playback policy test.

## Architecture Invariants

- R001 remains media-path authority; R002-PREP consumes it.
- R007 remains Playback concurrency authority; this probe does not invent alternate command/revision/handoff semantics.
- muted autoplay is useful diagnostic evidence but is not equivalent to audible remote playback.
- a synthetic click, DevTools permission override, autoplay command-line flag, browser profile policy override, or test framework bypass must never be used as physical R002 acceptance evidence.
- Fullscreen is progressive enhancement; viewport immersive is baseline.
- browser rejection must be surfaced rather than hidden/retried until it looks successful.
- no upstream Cookie/Authorization/Vault material enters browser diagnostics.

## Files Expected to Change

Exact paths depend on the accepted R001 candidate. Expected categories:

- minimal Web Display/probe code;
- minimal remote-trigger/test controller code;
- hosted-browser probe tests;
- optional R002 probe workflow additions if no equivalent CI exists;
- R002 manual verification instructions/evidence schema docs when useful.

Do not duplicate the R001 media server or create an unrelated frontend framework.

## Verification Plan

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Evidence |
|---|---|---|---|---|---|---|
| J1 | C1-C6 | github-actions | github-hosted-x64 | runner browser | yes | build/test + headless browser logs |
| J2 | C1-C5 | github-actions | github-hosted-x64 + Chromium | desktop Chromium only | yes | remote trigger, promise result/error, activation retry, Fullscreen-deny/lifecycle artifacts |

Desktop/hosted browser results prove only that the probe works. They cannot classify R002.

## Success Criteria

1. A specific reusable R001 candidate is integrated without creating a duplicate media/display stack.
2. Remote trigger → one audible `play()` attempt is reproducible and correlated in diagnostics.
3. Promise rejection and browser-equivalent autoplay errors are visible to the verifier/Gateway-side diagnostics.
4. A clear one-time activation/retry path exists and can be reset for repeated tests.
5. Fullscreen denial does not break viewport-immersive playback UX.
6. Visibility/lifecycle/reconnect observations needed for Cases C-F are available.
7. Required hosted automation passes on the final candidate SHA.
8. Issue #7 can execute without inventing additional probe behavior.
9. No R002 physical result is claimed by this Task.

## Evidence Contract

Each Attempt records candidate SHA/PR, R001 base candidate, browser version, remote trigger method, commands/workflows, play resolve/reject telemetry, Fullscreen result, lifecycle/reconnect observations, and J1/J2 run/job evidence.

Do not store Secrets or sensitive media URLs.

## Failure / Blocked Handling

BLOCKED when:

- no stable reusable R001 Web Display/media candidate is approved;
- accepted R001 architecture cannot support a remote play probe without Contract revision;
- required browser automation cannot run after reasonable retry.

FAIL when the probe itself cannot reliably deliver/observe remote play attempts without violating architecture/security boundaries.

A target TV rejecting autoplay is **not** failure of Issue #6; that result belongs to Issue #7.

## Deliverables

- R002 probe candidate/PR;
- hosted J1/J2 evidence;
- reproducible manual entry/setup for Issue #7;
- no R002 product verdict.

## Completion Protocol

Worker follows `docs/tasks/issue-lifecycle-protocol.md`: report `[EXECUTION REPORT]`, move to `status:review`, release ownership, and stop. Coordinator acceptance of Issue #6 only means the probe is ready for physical verification; it does not mean R002 PASS.