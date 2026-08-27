# Task — R002-TV Physical TV Remote Playback Verification

## Metadata

```text
GitHub Issue: #7
Parent Goal / Research Item: R002 / P0 TV Browser Remote Playback / Autoplay
Task / Research ID: R002-TV
Task kind: verification
Planning base commit: cacdb6b543b40612be8ba1014e3a0ed5331bd42b
Session bootstrap prompt: docs/tasks/7-r002-physical-tv-verification/prompt.md
Downstream handoff profile: docs/tasks/handoffs/manual-tv.md
Preferred worker: manual-tv
Eligible worker environments after publication: env:manual-tv
Required capabilities: physical-tv-access, remote-control-observation, audible-playback-observation, timing-observation, github-read-write-for-report
Linked implementation task: Issue #6 / R002-PREP
Hard publication dependencies: Issue #6 accepted candidate available on a reachable test deployment; target TV/browser and phone/remote Control path available
```

> This Task is the independent physical Evidence Authority for R002. Desktop browsers, simulators, hosted Chromium and theoretical policy analysis cannot replace it.

## Goal

Determine, on a real target television/browser, whether the Gateway Web Display supports an acceptable low-interaction **audible remote playback** flow and classify R002 as:

```text
PASS
CONDITIONAL PASS
FAIL
BLOCKED
```

The classification is Evidence, not a Worker/Coordinator lifecycle decision. Coordinator may ACCEPT this Verification Task even when the verified R002 hypothesis is `FAIL`, provided the required experiment was correctly completed and recorded.

## Hypothesis

A target TV browser has a low-interaction flow in which `/display` can remain waiting and later playback can be driven primarily by the phone:

- best case: no TV interaction after opening `/display`;
- acceptable conditional case: one explicit activation after first open/browser restart, then multiple later playback tasks need no repeated TV interaction.

Requiring TV interaction for every new playback task is a major product risk and does not qualify as PASS/acceptable CONDITIONAL PASS.

## Task Decomposition Decision

```text
Verification mode: separate-task
Linked implementation task: Issue #6
Linked verification task: Issue #7 (this Task)
Decision reason: physical TV behavior is an independent Manual Evidence Authority with a distinct owner, timing, device availability and P0 PASS/FAIL deliverable.
```

## Preconditions

Before publication/claim:

- Issue #6 has Coordinator ACCEPT and a specific candidate/deployment is available;
- the TV can reach the Gateway `/display` over the intended LAN path;
- a phone/Control or equivalent real remote trigger can create the probe playback attempt;
- a legal non-DRM media sample with audible audio is available;
- TV model, OS/platform and browser/WebView version can be recorded to the extent exposed by the device;
- no browser debug flag/policy override is enabled that would invalidate normal autoplay behavior.

## Claims

```text
C1: Never-activated /display has a recorded result for remote audible play, including rejection reason when it fails.
C2: If initial activation is required, one explicit TV interaction is enough for subsequent remote audible tasks during the same usable browser session.
C3: After playback end and 10-minute / 30-minute idle intervals, remote audible playback still has a recorded stable result without per-play TV interaction.
C4: Page refresh and browser/TV restart behavior is recorded, including whether activation must be repeated.
C5: Sleep/screen-off/resume behavior is recorded when the target TV/browser exposes a practical way to test it; otherwise explicitly N/A with reason.
C6: Fullscreen denial/unavailability does not make the viewport-immersive Display unusable.
C7: autoplay/play rejection and lifecycle/reconnect errors are observable rather than silently hidden.
C8: final R002 classification follows the frozen success criteria without weakening them after observation.
```

## Required Manual Scenarios

### Case A — never interacted

```text
open /display fresh
→ do not click/press remote key in page
→ wait 1–5 min
→ phone sends playback task
→ attempt audible play
```

Record success/failure, audible state, promise/browser error, muted-only behavior if any, startup latency where practical.

### Case B — one-time activation

Only if Case A fails or is blocked by autoplay policy:

```text
show explicit activation prompt
→ press TV OK/confirm once
→ phone triggers audible play
→ end playback
→ trigger at least two additional playback tasks without TV interaction
```

Record whether one interaction truly unlocks later tasks or whether each new play requires another action.

### Case C — long idle after playback

Test at minimum:

- 10 minutes;
- 30 minutes.

After each interval, trigger a new audible playback from phone without touching TV first.

### Case D — page refresh

Refresh/reload `/display`, then repeat the required remote trigger. Record activation persistence/loss.

### Case E — browser process / TV restart

Where feasible on the target device, restart browser or TV, reopen `/display`, and record whether one-time initialization must be repeated.

### Case F — sleep / screen-off / resume

Where feasible, record browser/network recovery and remote play outcome after resume. If the platform cannot expose a meaningful controllable case, record `N/A` rather than inventing evidence.

## Additional Observations

- Fullscreen request allow/deny and viewport-immersive fallback;
- `visibilitychange` or equivalent lifecycle observation;
- remote-command arrival;
- reconnect/recovery behavior;
- video element error/result;
- muted → audible transition only as diagnostic evidence;
- number of TV-side interactions required per new playback task.

## Architecture / Test Integrity Rules

- no desktop browser/simulator substitution for required target evidence;
- no Chrome autoplay bypass flags, DevTools permission injection, kiosk enterprise policy override, synthetic user activation or test-framework click used to manufacture acceptance;
- muted autoplay is not audible playback PASS;
- Fullscreen is not required if viewport immersive remains usable;
- HTTP LAN baseline and secure-context enhancements must be distinguished if behavior differs;
- do not expose Secrets, account data or sensitive media URLs in Issue evidence.

## Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Executor | Target | Required | Evidence |
|---|---|---|---|---|---|---|
| J1 | C1,C2,C6,C7 | manual | manual-tv verifier | physical target TV/browser | yes | step log + observations/screenshots/video if useful |
| J2 | C3 | manual | manual-tv verifier | same physical TV/browser | yes | timestamped 10/30-min replay observations |
| J3 | C4 | manual | manual-tv verifier | same physical TV/browser | yes | refresh + browser/TV restart result |
| J4 | C5 | manual | manual-tv verifier | same physical TV/browser | when feasible | resume result or explicit N/A |

All required jobs must use the same identified candidate/deployment unless the Coordinator explicitly records a rerun on a revised candidate.

## R002 Result Classification

### PASS

TV opens `/display` and, without additional TV-side interaction, reliably receives later remote **audible** playback tasks across the tested normal session/idle scenarios.

### CONDITIONAL PASS

First open or browser/TV restart requires one explicit initialization interaction, after which multiple playback tasks—including post-play idle replay—do not require another TV interaction until the browser session/restart boundary documented by the evidence.

### FAIL

Examples include:

- every new playback task requires another TV interaction;
- audible remote playback is unavailable while only muted autoplay works;
- remote play succeeds too inconsistently to support the intended TV experience;
- refresh/reconnect behavior makes the Display operationally unusable without repeated manual recovery.

### BLOCKED

Required physical device, deployment, remote trigger, audio-capable legal media or reliable observation path is unavailable. Do not convert BLOCKED into PASS/FAIL by theory.

## Task Success Criteria

The Verification Task itself is complete when:

1. a specific Issue #6 candidate/deployment is identified;
2. Cases A-D are executed and durably recorded;
3. 10- and 30-minute replay observations are present;
4. Case E is executed when practically available and its restart boundary recorded;
5. Case F is executed or explicitly marked N/A with reason;
6. Fullscreen/viewport degradation and browser error observability are recorded;
7. interaction count per playback task is unambiguous;
8. a final R002 `PASS | CONDITIONAL PASS | FAIL | BLOCKED` result is assigned using the frozen criteria;
9. limitations and exact TV/browser environment are recorded.

Coordinator `ACCEPT` means the verification was correctly performed and the result is trusted; it does not necessarily mean the R002 hypothesis passed.

## Evidence Contract

Record at least:

```text
Task / Claim: R002-TV / C1..C8
Attempt:
Candidate/deployment SHA:
TV manufacturer/model:
TV OS/platform/version:
Browser/WebView name/version if available:
Network path / HTTP vs HTTPS:
Media sample type:
Case A result / error:
Case B interaction count + subsequent plays:
Case C 10-min result:
Case C 30-min result:
Case D refresh result:
Case E restart result:
Case F resume result or N/A:
Fullscreen result / viewport fallback:
Reconnect/lifecycle observations:
R002 result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
Limitations:
```

Screenshots/video are optional supporting evidence; concise written observations with timestamps and reproducible steps are required.

## Failure / Blocked Handling

A negative product result is not a reason to alter Success Criteria. Report it as `FAIL` and let Coordinator decide whether the TV Web Display remains the primary route or needs a device-specific mode/other Adapter.

If the probe/candidate itself is broken, distinguish probe defect from TV autoplay failure and return to Coordinator; do not classify browser policy from an invalid probe.

## Deliverables

- durable Issue #7 manual execution report;
- R002 classification + condition/limit;
- environment/device metadata;
- reproducible Case A-F observations;
- optional screenshots/video references;
- recommended product/architecture follow-up only after evidence.

## Completion Protocol

Manual verifier follows `docs/tasks/issue-lifecycle-protocol.md`: claim only when `status:ready + env:manual-tv`, execute one Attempt, post `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, move to review/blocked, release ownership, and stop. Only Coordinator performs Final Acceptance/closure.