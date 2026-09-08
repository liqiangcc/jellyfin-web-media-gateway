# Task — BROWSER-E2E-MEDIA-READINESS Fail closed on media readiness and playback progression

## Metadata

```text
GitHub Issue: #159
Parent Goal: browser end-to-end control on ordinary Linux
Task / Research ID: BROWSER-E2E-MEDIA-READINESS
Task kind: combined
Planning / Evidence Base: 7b4aab5ed8d6f694f727b2062711f1edcf0bf174
Session bootstrap prompt: docs/tasks/159-browser-media-readiness/prompt.md
Preferred worker: cloud-codex (Fast + gpt-5.6-luna, high reasoning)
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, github-actions-authoring, headless-chromium-verification
Hard dependencies: #154 ACCEPTED; #157 ACCEPTED
Explicitly independent: #67 real-site compatibility, #68 Bilibili Web E2E, phone/ADB, physical TV, Jellyfin, Native Site Panel
```

Realtime status, owner, Attempt, branch, candidate, verification and review live in Issue #159. Do not copy dynamic state into this contract.

## Goal

Make the browser Control/Display evidence path fail closed when media is not actually usable:

```text
reachable bounded generic-direct source
→ successful Gateway media response
→ browser media metadata/readiness
→ explicit user-equivalent Display activation
→ measurable media time progression
```

The workflow must not call a `/stream/` request count, a server `playing` state, or a rendered first frame proof of playback. The change must preserve the existing Gateway PlaybackSession authority and Display lease/generation contracts.

## Why this Task exists

The current hosted workflow configures `https://example.test/deterministic-generic-direct.mp4`, which is intentionally unresolvable from the target and produces `502 EGRESS_DNS_FAILED`. `scripts/web-mvp-e2e-prep.mjs` records a `/stream/` request but does not assert its response status, media readiness, or time progression after the Display's activation control. Chromium can therefore leave the Display paused on the first decoded frame while the workflow reports success. A tx-node reproduction with the tested Sintel trailer showed `readyState=4`, `duration=52.208333`, `paused=true`, and `currentTime=0` until `Press OK to play`; after activation, time advanced continuously with no media error.

## Claims

- **C1 — successful media transport:** the browser E2E fails on a non-success Gateway media response or an unexpected media request failure, while retaining the existing expected source-replacement cancellation allowance.
- **C2 — usable media state:** the browser E2E requires a media element with a usable ready state, finite positive duration (or an explicitly justified live-media alternative), and no media error before declaring the rendering path ready.
- **C3 — actual playback:** the browser E2E performs an explicit user-equivalent Display activation and requires measurable currentTime progression while the element is not paused and has no error.
- **C4 — reachable workflow input:** the hosted workflow uses a stable reachable public MP4 that is compatible with the existing generic-direct contract, or records a concrete evidence-backed reason for an equivalent replacement. It must not use the unresolvable `example.test` placeholder as the happy-path source.
- **C5 — authority/security preservation:** existing Gateway-owned rendering, PlaybackSession revision, stale/error, secret-boundary, and cleanup checks remain intact; no raw upstream URL/header, Cookie, Authorization, Vault data, lease token, or local path is emitted.
- **C6 — evidence separation:** the report distinguishes implementation result, verification claim result, and Coordinator task decision, and records exact Candidate/run/job/artifact/runner values.
- **C7 — scope boundary:** this Task proves hosted/ordinary-Linux browser media readiness only. It does not claim Bilibili extraction, phone deployment, physical-TV audibility, Jellyfin, or autoplay without a user gesture on every device.

## In Scope

- Update `scripts/web-mvp-e2e-prep.mjs` with bounded assertions for:
  - Gateway `/stream/` response status and request failures;
  - media readiness/metadata/error state;
  - explicit activation through the production Display control;
  - currentTime progression within a bounded timeout.
- Update `.github/workflows/browser-control-e2e.yml` so its happy-path source is reachable and its J1/J2/J4 invocations exercise the strengthened assertions.
- Add only minimal focused harness evidence fields or tests needed to diagnose a readiness/progression failure without exposing sensitive values.
- Run the required build, focused browser jobs, regression matrix, artifact consumption, and cleanup entirely through GitHub-hosted Actions.
- Update this Task package only when a contract correction is required before implementation; do not silently broaden the contract.

## Out of Scope

- Bilibili/generic-ytdlp compatibility, Site Plugin changes, real-site login, phone/ADB or Android deployment, physical TV/audio/autoplay proof, Jellyfin, Native Site Panel, or any product Playback/Display/Egress/Security contract redesign.
- Public Gateway or CDP exposure, proxy creation, SSRF relaxation, credential propagation, Vault access, or browser second-state authority.
- Replacing the deterministic browser evidence with a direct Bilibili page interaction.
- Local `cargo build`, `cargo test`, `cargo run`, FFmpeg/Chromium compilation, or any binary-producing command in a Codex shell or on tx-node. Required build/test evidence must come from GitHub Actions.

## Architecture Invariants

1. Gateway/R007 remains the only PlaybackSession, item, revision, media, Display and handoff authority.
2. Control remains View + Intent + bounded form/presentation state; the harness must not inject browser-local authority.
3. Source recognition/resolution continues through `SiteAdapterRegistry`; no site-specific branch is added to Core.
4. Display obtains only server-owned same-origin Gateway paths and keeps lease/generation callbacks bounded.
5. Browser evidence may observe media element state but must not expose or persist upstream secrets, raw signed URLs, cookies, or authorization headers.
6. The test source change must not widen EgressPolicy or create an open proxy.
7. Existing target-runner isolation and no-phone-deployment constraints remain unchanged.

## Expected implementation surface

- `scripts/web-mvp-e2e-prep.mjs`
- `.github/workflows/browser-control-e2e.yml`
- focused documentation/evidence fixtures only if justified by the implementation

If the required behavior needs changes outside these surfaces or contradicts a canonical invariant, stop and post a blocker for Coordinator Review.

## Required verification jobs

### J0 — contract and topology preflight

- Read Issue #159 and all relevant comments, this contract, `AGENTS.md`, lifecycle/recovery/freshness protocols, and accepted #154/#157 contracts.
- Confirm the Candidate is based on the current `main` and that no phone or physical-TV deployment is required.
- Confirm required build/test execution is routed to GitHub-hosted Actions; report topology and any unavailable capability explicitly.

### J1 — hosted baseline with real media assertions

- Checkout the exact Candidate SHA on `ubuntu-latest`.
- Build `r001-server` and create the existing deterministic artifact only on the GitHub-hosted runner.
- Run `scripts/web-mvp-e2e-prep.mjs` against the workflow source.
- Verify the run proves successful media response, metadata/readiness, explicit activation, and measurable progression; retain redacted evidence.
- Upload the existing artifact/proof bundle and record run/job/artifact IDs and digests.

### J2 — exact-artifact browser repeat

- Consume the exact J1 artifact without recompilation.
- Run the strengthened browser journey again and confirm currentTime progression and clean media state on the same Candidate.
- Preserve the established Control/Display lifecycle and no-secret checks.

### J3 — regressions and negative matrix

Run the existing accepted authority/security matrix, including at least:

- duplicate `request_id`;
- stale expected revision;
- stale item callback and stale display generation/handoff;
- two-Control concurrent mutation;
- missing/offline Display and malformed/unsupported source;
- stale/replaced media request handling;
- secret/header/raw URL/local path leakage checks.

A negative media source may be used to prove the new failure assertion, but its expected failure must be classified explicitly and must not be confused with happy-path PASS.

### J4 — cleanup and reproducibility

- Consume the same exact artifact a second time without compilation.
- Verify temporary browser/server state and artifacts are cleaned up and no credentials/profile data were copied.
- Record every unrun check as NOT RUN or BLOCKED; never turn a network or browser limitation into PASS.

## Freshness / integration contract

- Freshness policy: `strict-main` for implementation and hosted Jobs J1/J3; claim the current `main` SHA at Attempt start.
- The exact Candidate SHA, workflow/run/job/artifact, browser execution plane, and runner must be recorded for each evidence slice.
- A newer main commit does not silently invalidate prior evidence; Coordinator decides whether a rerun is required.
- If a prior Candidate/PR exists for Issue #159, reuse/rebase it rather than opening a parallel business Task.
- Contract changes require `status:draft`, updated `task.md`/`prompt.md`, GitHub read-back, and a new Publication Gate.

## Evidence contract

Every report must distinguish:

```text
Implementation Result
!= Verification Claim Result
!= Coordinator Task Decision
!= Parent Goal / Research Gate Decision
```

Reports must include real Candidate SHA, workflow/run/job/artifact IDs, execution plane, runner, target class, browser mechanism, media readiness/progression result, and cleanup. Redact stream capabilities, upstream URLs where required, lease tokens, cookies, headers, and private paths.

## Success criteria

1. A reviewed Candidate based on current main changes only the bounded harness/workflow surfaces unless a minimal justified readiness presentation fix is approved.
2. J1 hosted evidence passes on the exact Candidate and demonstrates response success, media readiness, activation, and time progression.
3. J2 exact-artifact repeat passes without local or tx-node compilation.
4. J3 authority/security/negative checks pass, with every exception explicitly classified.
5. J4 cleanup/reproducibility is recorded.
6. No phone/physical-TV deployment or public Gateway/CDP exposure is performed.
7. Worker posts `[EXECUTION REPORT]`, releases ownership, and stops. Coordinator separately reviews, merges if accepted, posts `[FINAL ACCEPTANCE]`, and closes the Issue.

## Worker stop conditions

Stop and post `[BLOCKER REPORT]` if:

- Issue is not `status:ready` for `env:cloud`, or another owner has claimed it;
- a reachable source would require weakening EgressPolicy, exposing a secret, or allowing arbitrary private-network access;
- a required assertion cannot be represented without changing a canonical contract;
- GitHub-hosted build/test/artifact execution is unavailable;
- phone, physical-TV, or real-site evidence is required to satisfy this contract.

Never claim playback PASS from a `/stream/` request count or a server `playing` field alone.
