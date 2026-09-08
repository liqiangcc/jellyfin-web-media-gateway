# Task — WEB-DISPLAY-CONTROL Apply Gateway playback commands on TV Display

## Metadata

```text
GitHub Issue: #162
Parent Goal: browser end-to-end control on ordinary Linux
Task / Research ID: WEB-DISPLAY-CONTROL
Task kind: combined
Planning / Evidence Base: 3ad1940f4abb6b940a990beb34e0516139d726ed
Session bootstrap prompt: docs/tasks/162-web-display-control/prompt.md
Preferred worker: cloud-codex (Fast + gpt-5.6-luna, high reasoning)
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, github-actions-authoring, headless-chromium-verification
Hard dependencies: #154 ACCEPTED; #157 ACCEPTED; #159 ACCEPTED
Explicitly independent: #67 real-site compatibility, #68 Bilibili Web E2E, phone/ADB, physical TV, Jellyfin, Native Site Panel
```

Realtime status, owner, Attempt, branch, candidate, verification and review live in Issue #162. Do not copy dynamic state into this contract.

## Goal

Make the production TV Web Display consume the Gateway's server-owned rendering context and apply accepted same-session PlaybackSession commands to its browser media element:

```text
phone/browser Control intent
→ Gateway PlaybackSession command authority
→ Display rendering context
→ TV media play / pause / seek / stop
→ bounded Display callback and telemetry
```

A browser autoplay rejection must remain explicit. A server `playing` state alone is not playback proof.

## Why this Task exists

The #159 ordinary-Linux path exposed a product gap. A live Web Display rendered and played the Gateway media after an explicit TV activation gesture. A Control `pause` command changed the authoritative Gateway state to `paused`, while the TV video continued advancing because the Display page only refreshed its context and media path; it did not apply state transitions to the media element.

## Claims

- **C1 — command application:** after a required one-time TV activation where Chromium needs it, Gateway `pause`, `play`, `seek`, and `stop` transitions are reflected by the TV media element within bounded polling/command timeouts.
- **C2 — media and context freshness:** commands are applied only for the current server-owned session/item/display generation; stale rendering or callback data cannot mutate the TV media state or Gateway authority.
- **C3 — autoplay boundary:** a fresh TV page does not falsely claim audible autoplay. If `play()` is rejected without a user gesture, the overlay/status and bounded callback identify the condition and the page remains recoverable through the explicit activation control.
- **C4 — telemetry and recovery:** accepted playback observations/position samples are bounded, redacted, and stale-safe; reconnect, item replacement, and stop leave no old media command able to overwrite the current item.
- **C5 — regression/security preservation:** existing authority, Display lease/generation, EgressPolicy, stale/error, secret-boundary, HTTP security, and cleanup checks remain green.
- **C6 — evidence separation:** the report records exact Candidate, Actions run/job/artifact, execution plane/runner, and distinguishes implementation result, verification claims, and Coordinator decision.
- **C7 — scope boundary:** this proves ordinary-Linux hosted browser Control→TV Display behavior only. It does not prove Bilibili extraction, phone deployment, physical-TV audibility, Jellyfin, or universal autoplay.

## In Scope

- Update the production `TV_DISPLAY_PAGE` behavior in the existing Gateway surface, or the smallest supporting Display/browser surface required, to:
  - observe context revision/item/display-generation changes;
  - apply server-authoritative play/pause/seek/stop transitions without rewinding ordinary playback on every poll;
  - keep explicit user activation for audible playback when required;
  - report bounded observation/error and position telemetry through the existing lease/callback boundary.
- Extend the focused browser harness/evidence to exercise Control command → Display media state for pause, play, seek, and stop, with bounded tolerances and no sensitive values.
- Preserve all existing stale callback, lease, generation, concurrency, security, and cleanup assertions.
- Run all required builds/tests/browser jobs and artifact consumption through GitHub-hosted Actions.

## Out of Scope

- Bilibili/generic-ytdlp compatibility, Site Plugin changes, real-site login, phone/ADB or Android deployment, physical TV/audio proof, Jellyfin, Native Site Panel, public Gateway/CDP exposure, proxy/SSRF relaxation, credential propagation, Vault changes, and PlaybackSession authority redesign.
- Treating a direct Bilibili page or VNC interaction as Gateway evidence.
- Local `cargo build`, `cargo test`, `cargo run`, FFmpeg/Chromium compilation, or any binary-producing command in a Codex shell or on tx-node.

## Architecture Invariants

1. Gateway/R007 remains the only PlaybackSession, item, revision, media, Display, and handoff authority.
2. Control remains View + Intent; the browser must not invent session, item, display-generation, or lease authority.
3. Display accepts only server-owned same-origin Gateway paths and uses the existing lease token/callback boundary.
4. SiteAdapterRegistry and EgressPolicy boundaries remain unchanged; no site-specific branch or open proxy is added.
5. A stale item, session revision, media generation, display generation, or lease must not apply a delayed browser result.
6. Browser errors and autoplay rejection are bounded observations; they cannot silently rewrite authoritative playback.
7. Target-runner and no-phone-deployment constraints remain unchanged.

## Expected Implementation Surface

- `gateway-core/src/lib.rs` production TV Web Display page and any minimal Display callback helper.
- Focused browser verification script/workflow and tests only where needed to prove C1–C5.
- If implementation requires canonical contract or security changes, stop and post a blocker for Coordinator Review instead of broadening this Task.

## Required Verification Jobs

### J0 — contract/topology preflight

Read Issue #162 and all relevant comments, this contract, `AGENTS.md`, lifecycle/recovery/freshness protocols, accepted #154/#157/#159 contracts, and applicable canonical docs. Confirm current `main`, `env:cloud`, GitHub-hosted Actions routing, and no phone/physical-TV requirement.

### J1 — hosted Control→Display command journey

On `ubuntu-latest`, checkout the exact Candidate, build the existing runtime on the hosted runner, register a Web Display, create a bounded generic-direct session, and use the Control surface to verify:

- explicit TV activation is required/recorded when audible autoplay is blocked;
- `pause` pauses the TV media and time remains stable;
- `play` resumes and currentTime advances;
- `seek` moves to the requested bounded position;
- `stop` pauses/stops and does not resume from a stale command;
- evidence contains only redacted paths/identities.

### J2 — exact-artifact browser repeat

Consume the exact J1 runtime artifact without recompilation and repeat the command journey, including reconnect or item/session refresh where the harness already supports it.

### J3 — authority/security/negative matrix

Retain the accepted matrices, including duplicate request IDs, stale expected revisions, stale callbacks/display generations, overlapping handoff/concurrent Control, offline/malformed source, HTTP Origin/CSRF/SSRF, secret/header/raw URL/local path scans, and no stale browser result overriding current media.

### J4 — cleanup/reproducibility

Consume the exact artifact again without compilation; verify temporary browser/server state and sensitive profiles are cleaned up and record all unrun checks explicitly.

## Freshness / Integration Contract

- Freshness policy: strict-main for implementation and hosted jobs; claim the current `main` SHA at Attempt start.
- Candidate, run/job/artifact, execution plane, runner, target class, and browser mechanism must be recorded for each evidence slice.
- Existing Candidate/PR for this Issue must be reused across retries; do not create parallel business Tasks.
- Contract changes require `status:draft`, task/prompt updates, GitHub read-back, and a new Publication Gate.

## Evidence Contract

Every report must distinguish:

```text
Implementation Result
!= Verification Claim Result
!= Coordinator Task Decision
!= Parent Goal / Research Gate Decision
```

Never claim actual TV playback from a Gateway state field alone. Record media element state and progression at the Display page, plus the corresponding Gateway command/revision context. Redact stream tokens, upstream URLs, cookies, headers, lease tokens, local protected paths, and browser profiles.

## Success Criteria

1. Candidate is based on current `main` and changes only the bounded TV Display/browser verification surfaces.
2. J1 and J2 demonstrate Control command → TV media pause/play/seek/stop behavior with bounded evidence after the required activation gesture.
3. Autoplay rejection remains explicit and recoverable; no false playback PASS.
4. J3 authority/security/negative checks and J4 cleanup/reproducibility pass.
5. Worker posts `[EXECUTION REPORT]`, releases ownership, and stops; Coordinator separately reviews, merges, posts `[FINAL ACCEPTANCE]`, and closes Issue.
6. No phone/physical-TV deployment or public Gateway/CDP exposure occurs.

## Worker Stop Conditions

Stop and post `[BLOCKER REPORT]` if Issue is not `status:ready` for `env:cloud`, another owner has claimed it, required hosted Actions execution is unavailable, the change would weaken security/authority boundaries, or a canonical contract change is required. Do not lower the activation or progression evidence threshold to avoid a blocker.
