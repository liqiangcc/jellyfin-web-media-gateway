# Task — BROWSER-CONTROL-E2E Browser Control to Web Display loop

## Metadata

```text
GitHub Issue: #154
Parent Goal: browser end-to-end control on ordinary Linux
Task / Research ID: BROWSER-CONTROL-E2E
Task kind: combined
Planning / Evidence Base: 8562bf39b3af4f5146ccad8710a4904a1a445072
Session bootstrap prompt: docs/tasks/154-browser-control-e2e/prompt.md
Preferred worker: cloud-codex (Fast + gpt-5.6-luna, high reasoning)
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, github-actions-authoring, headless-chromium-verification, ssh-target-diagnostics
Hard dependencies: #44 ACCEPTED; #45 ACCEPTED; #47 ACCEPTED; #48 ACCEPTED; #49 ACCEPTED
Explicitly independent: #67 real-site compatibility, physical phone/TV, Jellyfin, Native Site Panel
```

Realtime status, owner, Attempt, branch, candidate, verification and review live in Issue #154. Do not copy dynamic state into this contract.

## Goal

Prove one browser end-to-end control loop in the ordinary-Linux Web product:

```text
browser Display registers with Gateway
→ browser Control discovers the live Display
→ Control submits a bounded deterministic generic-direct source
→ Gateway creates the authoritative PlaybackSession
→ Web Display obtains Gateway-owned same-origin rendering paths
→ browser Control executes play / pause / seek / stop
→ Control and Display refresh/reconnect from Gateway authority
→ stale and invalid operations are rejected without corrupting the session
```

The deterministic generic-direct fixture is only an auditable input for the product path. This Task proves browser Control/Display composition and Playback command behavior; it does not claim that Bilibili extraction works. #67 remains a separate source-compatibility task and must not gate this Task.

The browser evidence target is an ordinary Linux browser session reachable through the approved Chrome DevTools MCP or equivalent isolated headless Chromium evidence path. No physical phone, physical TV, Android/ADB deployment, or Jellyfin client is required.

## Why this Task exists

#49 already accepted the hosted Web-only MVP mechanics using a deterministic generic-direct fixture. The current goal needs an independently reproducible browser Control-to-Display evidence loop, with the browser execution topology and artifact provenance recorded separately. The old #68 contract incorrectly made #67 real-site resolution a hard dependency for browser control; this contract removes that accidental dependency while preserving the architecture boundary between product control and Site Plugin compatibility.

## Claims

- **C1 — real product entrypoints:** the success path starts at the product Web Display/Control pages and accepted public APIs. It does not use `seed_test_session`, direct store mutation, proof-only media paths, synthetic Display authority, arbitrary `ResolvedMedia` injection, or browser-only playback state.
- **C2 — live Display authority:** the Display registers and heartbeats through the accepted #45 lease contract; Control uses a bounded server-owned live Display selector. Lease tokens, page epochs, and R007 display generations are never exposed as caller authority.
- **C3 — authoritative session creation:** Control submits the accepted bounded #44 source contract and receives one server-owned PlaybackSession bound to the selected Display. Duplicate request IDs and invalid sources retain accepted no-side-effect semantics.
- **C4 — server-owned rendering:** the Display obtains current Gateway-safe same-origin media/subtitle paths for the authoritative session/item/revision. Raw upstream URLs, headers, cookies, Vault data, local paths, or caller-selected revisions never enter the rendering contract.
- **C5 — browser command loop:** the Control page executes play, pause, seek, and stop through the accepted #47/R007 revision-aware command path, and the visible Display follows the authoritative state.
- **C6 — refresh/reconnect coherence:** Control event reconnect/refresh and Display refresh/lease reconnect rebuild from Gateway authority. An old page, callback, or event cannot overwrite a newer PlaybackItem/revision/display generation.
- **C7 — bounded failure behavior:** stale revisions, duplicate request IDs, missing/offline Display, malformed source, missing session, and stale callback/lease cases return bounded recoverable errors with no partial or duplicate authority.
- **C8 — evidence and security boundary:** the exact Candidate SHA, hosted Actions run/job/artifact, browser environment, target host, and cleanup are recorded. Evidence contains no Cookie, Authorization, Vault/profile data, lease token, signed protected URL, raw upstream header, or arbitrary local path.
- **C9 — scope boundary:** all claims are ordinary-Linux browser Control/Display claims. No claim is made for physical-TV autoplay/audibility, phone resources, Jellyfin, or real Bilibili/site extraction.

## In Scope

- Inspect the current product composition and reuse accepted #44/#45/#47/#48/#49 routes and DTOs.
- Add only the smallest missing product glue for a real Control→Session→Display loop:
  - bounded Control source/session entry;
  - bounded read-only live Display selector if required;
  - bounded server-owned session-to-Display rendering view if required.
- Replace any proof-only media authority in the happy path with accepted Gateway-owned paths.
- Add or update a deterministic browser E2E harness and GitHub-hosted workflow.
- If target evidence needs a runnable server, add a compile-free artifact publication/consumption path:
  - build and package on GitHub-hosted Actions;
  - verify Candidate SHA and artifact manifest/digest;
  - consume the exact artifact on the approved Linux target;
  - never compile on the coordinator, Codex shell, or tx-node.
- Execute the browser path through isolated Chrome DevTools MCP or an equivalent isolated headless Chromium run. Keep Gateway listeners loopback-only or behind a narrow approved SSH tunnel; never expose a public Gateway/CDP listener.
- Record cleanup, resource bounds, and any topology limitation as evidence.

## Out of Scope

- Any real Bilibili/generic-ytdlp compatibility repair or #67 reopening.
- Physical phone deployment, ADB, Android/ARM64 phone runtime, physical TV, audible autoplay, or Jellyfin.
- Direct manipulation of a Bilibili page as a substitute for Gateway Control authority.
- New Playback, Display lease, display generation, handoff, source locator, Secret, or Egress semantics.
- Browser-local second state authority, raw URL/header injection, open proxy, SSRF relaxation, Cookie/Authorization propagation, or Vault access.
- Service restart persistence, Native Site Panel, site login automation, advanced subtitle features, or Core Feasibility GO.
- Local `cargo build`, `cargo test`, `cargo run`, FFmpeg/Chromium compilation, or any equivalent binary-producing command. Local read-only inspection and lint-free diagnostics are allowed; required build/test Evidence must come from GitHub Actions.

## Architecture Invariants

1. Gateway/R007 is the only PlaybackSession command, revision, item, media, display and handoff authority.
2. Jellyfin remains an optional DisplayAdapter; it is not used by this Task.
3. Control is View + Intent + bounded presentation/form state, never a second business state store.
4. Source recognition/resolution goes through SiteAdapterRegistry. Core has no concrete Bilibili or yt-dlp branch.
5. Site Browser Worker and Site Plugin boundaries remain intact; no plugin reads Vault or bypasses EgressPolicy.
6. Display receives only server-owned Gateway-safe rendering capability paths.
7. Refresh/reconnect reconstructs from Gateway authority; browser storage, DOM, events, and Chrome MCP are not authority.
8. Native Site Panel failure and Jellyfin failure are independent of this Web path.
9. Trusted artifact and target runner are isolated from Gateway Vault, production secrets, root/ADB privileges and long-lived credentials.

## Expected implementation surface

The Worker must inspect before editing and touch only justified files. Likely surfaces are:

- `gateway-core/src/` product route/composition modules;
- browser E2E scripts/harness;
- `.github/workflows/` for hosted build/test/artifact jobs;
- `docs/tasks/154-browser-control-e2e/` evidence/runbook updates only when needed.

Do not modify canonical architecture/security docs for convenience. If the required behavior contradicts them, stop and return a blocker for Coordinator Review.

## Required verification jobs

### J0 — contract and topology preflight

- Read Issue #154, all relevant comments, this contract, lifecycle/recovery/freshness protocols, and accepted #44/#45/#47/#48/#49 contracts.
- Confirm the candidate base is current main and no active owner exists.
- Determine whether the isolated browser can reach a loopback Gateway through the approved target/tunnel topology. Do not public-bind to make it work.
- Report topology as PASS or BLOCKED with the exact non-secret environment/runner labels.

### J1 — GitHub-hosted implementation and baseline

- Build/test/package only on a GitHub-hosted runner.
- Run the focused browser E2E and relevant accepted regressions against the exact Candidate SHA.
- If an artifact is produced, upload a manifest and digest; record run ID, job IDs and artifact ID.
- Do not claim target/browser proof from a compile result alone.

### J2 — browser Control/Display E2E

Using the exact accepted artifact/Candidate and deterministic generic-direct fixture:

1. open isolated Display and Control pages;
2. register/heartbeat the Display;
3. discover/select that Display from Control;
4. create one session through the public bounded source contract;
5. verify same-origin Gateway media/subtitle rendering;
6. execute play, pause, seek, stop;
7. refresh/reconnect Control and Display and verify authoritative state;
8. capture only redacted, bounded browser/HTTP evidence.

The browser may be Chrome DevTools MCP or isolated headless Chromium, but the evidence must identify the actual execution plane, runner and target. A direct Bilibili page interaction may be recorded as a diagnostic observation only; it is not C1–C8 evidence.

### J3 — stale/error/security matrix

Cover at least:

- duplicate `request_id`;
- stale expected revision;
- stale item callback;
- stale re-resolve result;
- stale display generation/handoff;
- two-Control concurrent mutation;
- offline/missing Display;
- malformed or unsupported generic-direct source;
- secret/header/raw URL/local path leakage checks.

All failures must preserve accepted no-side-effect and authority semantics.

### J4 — cleanup and reproducibility

- Stop the server/browser/tunnel and remove temporary target files.
- Verify no long-lived credentials or browser profile data were copied.
- Confirm a second run can consume the same exact artifact without local compilation.
- Record any unrun check as NOT RUN or BLOCKED, never PASS.

## Freshness / integration contract

- Freshness policy: `strict-main` for implementation and hosted Jobs J1/J3; the Worker must base the Candidate on the current main SHA recorded at claim time.
- For J2 target/browser evidence, the exact Candidate SHA, artifact digest, target hostname class, and browser execution plane must be recorded. A newer main commit does not silently rewrite old Evidence; Coordinator decides whether a rerun is required.
- If a prior Candidate/PR exists for this Issue, reuse/rebase it rather than creating a parallel business Task.
- Contract changes require `status:draft`, an updated `task.md`/prompt, read-back and a new Publication Gate.

## Evidence contract

Every report must distinguish:

```text
Implementation Result
!= Verification Claim Result
!= Coordinator Task Decision
!= Parent Goal / Research Gate Decision
```

The Worker must post an append-only `[EXECUTION REPORT]` or `[BLOCKER REPORT]` before changing Issue status. Evidence must include real values for Candidate SHA, workflow/run/job/artifact, execution plane, runner, target class, browser mechanism, and cleanup. Do not include cookies, tokens, signed URLs, raw upstream headers, or private profile paths.

## Success criteria

The Task is complete only when all are true:

1. A reviewed Candidate exists on a branch/PR based on the current main and changes only the bounded product/workflow/harness surfaces.
2. J1 hosted Actions build/test Evidence passes for the exact Candidate SHA.
3. J2 browser Control/Display loop passes with server-owned Gateway authority and deterministic generic-direct input.
4. J3 stale/error/security checks pass, or each exception is explicitly classified as FAIL/CONDITIONAL PASS/BLOCKED by the Coordinator.
5. J4 cleanup/reproducibility evidence is recorded.
6. No phone/physical TV deployment was performed or required.
7. Worker posts `[EXECUTION REPORT]`, releases ownership, and stops. Coordinator must separately review, merge if accepted, post `[FINAL ACCEPTANCE]`, then close the Issue.

## Worker stop conditions

Stop and post `[BLOCKER REPORT]` if:

- Issue is not `status:ready` for `env:cloud`, or another owner has claimed it;
- the browser topology requires public binding, credential sharing, raw signed URL exposure, or unsafe SSH privileges;
- a required behavior needs a new architecture/security contract;
- required GitHub-hosted runner/artifact or target/browser capability is unavailable;
- a real-site result would be needed to make a browser Control claim.

Never lower the criteria or claim PASS from a local compile or a direct site-page click.
