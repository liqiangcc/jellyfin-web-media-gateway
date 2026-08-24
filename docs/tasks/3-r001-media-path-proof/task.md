# Task — R001 Media Path Proof

## Metadata

```text
GitHub Issue: #3
Parent Goal / Research Item: R001 / Core Feasibility / Phase 0A-1 Media Path Proof
Task / Research ID: R001
Task kind: combined
Publication / integration base commit for Codex-first republish: ad18bd35ac17d28abc0af1d4a1c7d1c2b10950df
Candidate commit: n/a (live state belongs in Issue)
Session bootstrap prompt: docs/tasks/3-r001-media-path-proof/prompt.md
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Preferred worker: cloud
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, rust-build, rust-test, browser-automation
Hard publication dependencies: none
Accepted concurrency authority: Issue #2 / R007 is status:done and merged to main
```

> Live status, owner, candidate, runs and results belong in Issue #3. This file does not store live Task state.
>
> R001 owns media-path proof. R007 owns Playback concurrency/revision/handoff authority semantics and is already accepted on `main`; R001 must integrate with, not redefine or bypass, that domain.

## Goal

Prove the first Web-only Media Gateway path with public/non-DRM media:

```text
Source input
→ SiteAdapterRegistry
→ SourceLocator
→ SiteAdapter.resolve
→ ResolvedMedia
→ scoped Gateway media capability
→ Media Gateway
→ Web Display
```

Required primary proof: direct HTTP MP4 plays end-to-end in a real browser through the Gateway with usable play/pause/seek and correct byte-serving behavior.

Required secondary result: HLS must have concrete manifest/segment/query/redirect/failure evidence; it may not remain assumed support.

The proof must also show that upstream secrets stay server-side, `/stream` cannot become an arbitrary open proxy, resources are bounded/cleaned up, and Jellyfin is not required.

R001 is a media-path feasibility Task, not a complete product implementation.

## Why / Context

Canonical R001 success requires:

- at least one of MP4/HLS to play stably, with an explicit result for the other;
- pause/play/seek usability;
- Range/segment semantics preserved;
- upstream Secret not exposed to Display;
- `/stream` not degrading into an arbitrary open proxy;
- no concrete-site special case in Core;
- Jellyfin disabled without breaking the path;
- no evidence of unbounded media buffering/cache growth.

`docs/mvp-plan.md` distinguishes Task scheduling from Gate aggregation. R001 was allowed to execute before R007 closure; now R007 is accepted and any final R001 Candidate must integrate with the accepted current-main Playback contract before acceptance.

## Dependency Decision

### Hard dependency: none

R001 did **not** require R007 Final Acceptance to begin its own media-path proof, and R007 is now completed.

Authority separation remains:

```text
R001 authority
= SourceLocator / ResolvedMedia / generic-direct / media capability / proxy / Web Display media consumption

R007 authority
= Playback command CAS / telemetry revision / item refresh freshness / display generation / handoff transition
```

R001 may use stable/test-only values for:

```text
session_id
item_id
item_revision
resource_id
```

only as opaque capability-binding identities. It must not implement or redefine the Playback mutation state machine merely to obtain those identifiers.

### Integration rule after R007 acceptance

The final R001 Candidate must be based on / integrated with current `main`, including accepted R007 semantics:

- rebase/adapt the existing R001 candidate as needed;
- do not lower R001 Success Criteria;
- do not rewrite R007 concurrency semantics inside R001;
- shared root `Cargo.toml`/workspace conflicts are normal integration work, not a business Task blocker;
- any previous R001 Evidence remains historical Evidence for its exact Candidate SHA and must be rerun when integration changes the Candidate.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: R001 implementation and required browser/HTTP integration evidence are portable and can be produced by Codex Cloud Worker + GitHub-hosted CI. Phone thermal/resource proof is R003 and physical-TV behavior is R002, so no independent target Evidence Authority is required for R001 acceptance.
```

Do not split MP4, HLS, browser, x64 and generic ARM64 into separate business Tasks merely because they use different Jobs.

## Primary and Secondary Media Proof

### Primary required browser path — direct HTTP MP4

```text
public/non-DRM MP4 source
→ generic-direct SiteAdapter
→ ResolvedMedia
→ Gateway media capability
→ Media Gateway Range proxy
→ Web Display <video>
→ play / pause / seek
```

The browser proof must exercise actual media-element behavior; `HTTP 200` alone is not PASS.

### Secondary required result — HLS

At minimum R001 must implement/verify enough HLS semantics for a concrete result covering applicable items:

- master/variant playlist;
- relative segment URI resolution or capability rewriting;
- query parameters;
- redirect handling under allowed egress rules;
- segment requests;
- interrupted/missing segment behavior;
- seek-relevant playlist/segment behavior.

Full browser HLS playback is required only if the selected browser/runtime supports it directly or a justified player dependency can be added without expanding R001 into a frontend-framework project.

R001 may PASS with stable MP4 browser playback plus a fully explicit HLS result/follow-up, consistent with canonical criteria.

## Work Role

### Implementation

Build the minimum real media path needed to prove R001.

Expected components/capabilities:

- minimal stable `SourceLocator` / `ResolvedMedia` types needed by the media path;
- `SiteAdapter` / `SiteAdapterRegistry` boundary sufficient for `generic-direct`;
- `generic-direct` recognition/resolution for public/non-DRM direct media inputs;
- Media Gateway HTTP service or testable server surface;
- scoped short-lived media capability mapping to server-approved session/item/resource identity;
- HTTP file proxy with correct Range behavior;
- HLS manifest/segment proxy/rewriting sufficient for the required secondary result;
- minimal Web Display page/player for browser proof;
- deterministic upstream fixture server for Range, redirect, failure and Secret-boundary tests;
- public/non-DRM acceptance source smoke;
- meaningful GitHub-hosted CI/browser jobs.

Use the current accepted root workspace/Core state rather than establishing a competing workspace. If a prior R001 candidate was based before R007 landed, integrate/rebase it normally and preserve both domains.

### Verification Claims

```text
C1: A public/non-DRM direct MP4 resolves through Registry/SiteAdapter/ResolvedMedia and plays through Media Gateway in a browser without Jellyfin.
C2: HTTP byte Range/re-request semantics are preserved well enough for browser seek behavior.
C3: HLS manifest/segment proxy semantics produce a concrete verified result, including relevant relative URI/query/redirect/failure behavior.
C4: Browser/Display-visible requests and public response data contain no upstream Cookie, Authorization, bearer token or Vault material.
C5: `/stream` or equivalent media endpoint cannot be used to select an arbitrary caller-supplied upstream URL.
C6: invalid/expired/cross-session/cross-item/cross-resource media capability use is rejected deterministically.
C7: upstream 403/404, redirect, Range-unsupported and interrupted media/segment failures are bounded/explainable and do not mutate unrelated authority.
C8: stable Core contains no concrete-site URL/DOM/Cookie special case; `generic-direct` remains behind Registry/adapter boundary.
C9: streaming is bounded: active connections/resources are released after abort/seek/reconnect/end and retained structures do not grow with cumulative bytes streamed.
C10: canonical docs and executable behavior agree on the proven R001 path and unsupported/deferred boundaries.
```

## Routing Rationale

```text
Implementation / repository integration / orchestration
→ Codex Cloud Worker (`env:cloud`)

Portable HTTP/unit/contract verification
→ GitHub-hosted x64

Browser integration
→ GitHub-hosted x64 + headless Chromium (or equivalent supported browser)

Bounded repeated/cleanup verification
→ GitHub-hosted x64

Generic ARM64 compatibility
→ optional GitHub-hosted ARM64

Phone CPU/RSS/temperature
→ NOT R001 acceptance; belongs to R003

TV autoplay/remote UX
→ NOT R001 acceptance; belongs to R002
```

Codex Cloud is the Worker, not the verification Runner. Required runtime/browser Evidence must remain tied to exact Candidate SHA through GitHub Actions.

## Preconditions

- current canonical docs and this Task Contract are readable from GitHub;
- current `main` includes accepted R007 semantics and must be integrated before final R001 acceptance;
- Worker reads all Issue #3 history and evaluates any existing candidate PR before creating replacement work;
- Rust stable/toolchain rules from current repository state are followed;
- public acceptance source is legal, non-DRM and suitable for automated/recorded verification;
- deterministic fixture coverage exists for protocol/security/failure semantics; external public host behavior cannot be the only proof;
- R001 does not modify Playback command/revision/media-refresh/handoff semantics accepted by R007.

## In Scope

- minimal SiteAdapter API/Registry surface required by `generic-direct`;
- `generic-direct` direct media recognition/resolution;
- minimal `ResolvedMedia` shape required by direct MP4/HLS;
- Media Gateway streaming endpoint and scoped capability model;
- HTTP GET/HEAD behavior required by browser/media clients where applicable;
- Range behavior required by selected browser/source, with unsupported variants explicitly rejected rather than silently corrupted;
- MP4 browser playback, pause/play/seek;
- HLS master/variant/segment/URI rewriting result;
- redirect handling through the repository's egress direction;
- deterministic protected upstream fixture to prove server-side auth injection boundary;
- invalid/expired/cross-session/cross-item/cross-resource token replay tests;
- bounded streaming/abort cleanup tests;
- public source acceptance smoke;
- integration of R001 work with the accepted current-main workspace/Playback contract;
- docs updates driven by Evidence.

## Out of Scope

- PlaybackSession command/revision/handoff implementation owned by accepted R007;
- R002 TV audible autoplay / physical remote UX;
- R003 target-phone CPU/RSS/temperature/60-minute resource acceptance;
- Jellyfin DisplayAdapter;
- software video transcoding;
- FFmpeg remux unless Coordinator revises Scope based on concrete R001 Evidence;
- DASH full playback;
- concrete Bilibili/YouTube plugin logic;
- Site Auth / real account cookies;
- Native Site Panel / Browser Worker;
- full Control application;
- production persistence/restart recovery;
- full R008 security matrix.

R001 must still obey existing Egress/Secret invariants; “R008 is separate” is not permission to disable SSRF or expose Secrets.

## Architecture Invariants

- Core accesses source implementations only through `SiteAdapterRegistry` / SiteAdapter contract.
- `generic-direct` is an adapter/plugin, not a Core special case.
- `SourceLocator` represents stable source identity/re-location; transient media URLs are not promoted into stable content identity.
- upstream Cookie/Authorization/bearer material never enters `ResolvedMedia.public_headers` or browser-visible media capability URLs.
- Display consumes Gateway media capability, not raw upstream Secret.
- media capability identifies server-approved resources; caller input cannot turn it into `GET /stream?url=<arbitrary>`.
- redirect/egress handling must not silently bypass central EgressPolicy direction.
- Jellyfin absence cannot block Web Display.
- no large-media default disk cache or full-object buffering.
- R001 must not create a second Playback authority or redefine accepted R007 concurrency semantics.

## Media Capability Contract to Prove

Exact token representation is implementation-defined, but semantics must be equivalent to:

```text
MediaCapability
├── opaque random/signed identifier
├── session_id binding
├── item_id / item_revision binding
├── resource/stream identity binding
├── expiry
└── server-side mapping to approved upstream resource + access capability
```

For R001 these identity values may come from a deterministic test/fixture context; they do not require a complete PlaybackSession implementation.

Required properties:

- browser cannot replace upstream URL by editing a query parameter;
- token does not embed upstream Cookie/Authorization;
- token expires;
- wrong session/item/resource replay fails;
- invalid token fails without unauthorized upstream request;
- logs/errors do not reveal Secret or unnecessary signed upstream query values.

## Protected Upstream Secret Fixture

Public media alone cannot prove Secret injection safety.

Add a deterministic fixture:

```text
fixture requires server-side Authorization/Cookie
→ public ResolvedMedia/client data carries no Secret
→ Media Gateway injects fixture credential server-side through test-scoped access provider
→ browser/client receives media bytes
→ browser request/log/public response never contains fixture Secret
```

This is a contract fixture, not production Vault implementation and not real site auth.

## Failure Cases Required

At minimum verify:

- upstream 404;
- upstream 403;
- redirect;
- redirect to a target rejected by currently available egress rules when applicable;
- Range-supported source;
- Range-unsupported source;
- client abort/reconnect;
- HLS segment interruption/failure;
- expired media capability;
- invalid media capability;
- cross-session replay;
- cross-item replay;
- cross-resource replay;
- repeated media request for the same valid capability.

Failures must not create/modify Playback command authority. If an integration adapter to Playback is later added, it must consume—not redefine—the R007 contract.

## Verification Plan

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C2-C10 | github-actions | github-hosted-x64 | runner-self | yes | fmt/clippy/unit/contract + deterministic upstream fixture integration | run/job/log |
| J2 | C1,C2,C4,C5,C6 | github-actions | github-hosted-x64 + Chromium | browser on runner | yes | start fixture/gateway/display; browser MP4 play/pause/seek; capture network/errors | run/job/artifact |
| J3 | C1,C3 | github-actions/manual-dispatch | github-hosted-x64 | public non-DRM source | yes for acceptance | selected public source MP4 smoke + HLS concrete result | run/job/log/result summary |
| J4 | C7,C9 | github-actions | github-hosted-x64 matrix | runner-self | yes | bounded abort/seek/reconnect/repeat scenarios; resource counters return to baseline | run/job summary/artifact |
| J5 | C1-C10 | github-actions | github-hosted-arm64 | generic Linux ARM64 | no | portable deterministic suite when practical | run/job/log |

J5 is compatibility evidence only. Do not route ordinary R001 CI to the phone Target Runner.

### Browser Evidence

J2 must prove actual media-element behavior:

- metadata/load succeeds;
- playback time advances;
- pause stops advancement within normal test tolerance;
- seek changes playback position and produces expected Range/re-request behavior when applicable;
- browser console/player errors are captured;
- browser network inspection shows no upstream Secret.

### HLS Evidence

At minimum record:

- manifest before/after Gateway rewriting or equivalent resource mapping;
- resolved segment requests;
- relative/query URI correctness;
- failure behavior for missing/interrupted segment;
- whether browser playback was tested, supported, deferred or unnecessary for the primary MP4 R001 path.

No HLS result may remain implicit.

### Bounded Resource / Cleanup Evidence

Instrument/test at least:

- active stream/request count;
- retained media-resource/capability entries;
- temporary buffers/cache objects introduced by R001;
- cleanup after client abort, seek/reconnect and normal end;
- repeated scenario count >= 100 cycles or an equivalent deterministic bounded test set.

PASS requires resource counts to return to their defined baseline and retained structures to be bounded by explicit limits rather than cumulative bytes streamed.

Target-phone thermal/resource acceptance remains R003.

## Success Criteria

1. A real public/non-DRM MP4 source traverses Registry → generic-direct → ResolvedMedia → Media Gateway → Web Display and plays in browser with Jellyfin absent.
2. Browser play/pause/seek is demonstrably usable for the primary MP4 path.
3. HTTP Range/re-request semantics required by the tested source/browser are preserved or explicitly rejected where unsupported; Gateway does not silently corrupt byte ranges.
4. HLS has a concrete verified manifest/segment/seek-related result and explicit browser support/defer result.
5. Browser-visible requests/public headers/log artifacts contain no upstream Cookie, Authorization, bearer token or Vault material.
6. Media capability cannot select arbitrary caller-provided upstream URLs; invalid/expired/cross-session/cross-item/cross-resource replay is rejected.
7. Required failure cases are deterministic and bounded.
8. Stable Core contains no concrete-site special case; `generic-direct` remains behind Registry/adapter boundary.
9. Repeated abort/seek/reconnect/end scenarios release active resources; no retained buffer/cache/resource structure grows with cumulative bytes streamed.
10. Public-source acceptance Evidence and deterministic fixture Evidence are both present.
11. Canonical docs are updated to match the path actually proven, including explicit unsupported/deferred HLS/DASH/remux boundaries.
12. R001 does not modify/claim R007 Playback concurrency semantics and does not falsely mark R002/R003/Jellyfin/real-site work PASS.

### Claim success

```text
C1 PASS when: J2 + J3 prove public MP4 end-to-end browser playback through Gateway with Jellyfin absent.
C2 PASS when: deterministic Range tests + browser seek Evidence show preserved byte-serving semantics.
C3 PASS when: HLS fixture/public Evidence proves concrete manifest/segment behavior and records browser support status.
C4 PASS when: protected fixture and browser network/log artifacts show Secret remains server-side.
C5 PASS when: API/token tests prove caller cannot choose arbitrary upstream.
C6 PASS when: invalid/expired/cross-session/cross-item/cross-resource tests reject unauthorized use before unauthorized upstream access.
C7 PASS when: required upstream/protocol failures produce bounded errors.
C8 PASS when: architecture/static tests show no concrete-site knowledge added to stable Core.
C9 PASS when: repeated cleanup tests return active/retained resource counters to baseline and bounded limits hold.
C10 PASS when: docs and executable behavior are reviewed as equivalent.
```

## Evidence Contract

Each Attempt must record in Issue #3:

```text
Role: implementation | verification | combined
Task / Claim: R001 / C1..C10
Attempt:
Worker / Orchestrator:
Job ID: J1 | J2 | J3 | J4 | J5
Execution plane:
Runner class / image:
Execution host:
Target:
OS / architecture:
Rust toolchain:
Browser/version (J2):
Public source type/host (J3; no sensitive signed query):
Integration base commit:
Candidate commit:
Workflow / run / job:
Commands / selectors:
Duration / repetitions:
HTTP/Range/HLS summary:
Browser playback/seek summary:
Secret-boundary artifact/result:
Resource cleanup counters/artifact:
Claim results:
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

Do not store Cookies, Authorization, bearer tokens, account data, or unnecessary signed media URLs in Issue/artifacts/logs.

## Failure / Blocked Handling

FAIL examples:

- browser cannot play the required primary MP4 path and no canonical-allowed R001 path succeeds;
- Range/seek semantics are corrupted by Gateway;
- browser receives upstream Secret;
- caller can use `/stream` to proxy arbitrary URL;
- cross-session/item/resource replay succeeds;
- HLS remains untested/assumed;
- repeated abort/seek/reconnect leaks retained resources cumulatively;
- implementation bypasses Registry/`generic-direct` boundary;
- Jellyfin becomes required for Web-only path;
- R001 introduces competing Playback authority.

BLOCKED examples:

- GitHub-hosted browser/runtime required for J2 is unavailable after reasonable retry;
- no suitable legal public non-DRM acceptance source is available at execution time;
- canonical Egress/Secret contracts conflict in a way requiring Coordinator design revision;
- current-main integration exposes a concrete contract conflict that cannot be resolved without changing R001 Scope/Claims.

Do not lower Success Criteria to manufacture PASS.

## Deliverables

- R001 media path implementation integrated with current main;
- minimal SiteAdapter/Registry + `generic-direct` path;
- Media Gateway direct HTTP proxy and scoped media capability;
- HLS concrete-result implementation/tests;
- minimal Web Display browser proof surface;
- deterministic upstream fixture/failure/Secret tests;
- GitHub-hosted browser integration Evidence;
- public non-DRM acceptance Evidence;
- bounded cleanup/repetition Evidence;
- updated canonical docs;
- optional `docs/research/r001-media-path.md` durable Evidence summary;
- Issue #3 Attempt / Review history.

## Issue Feedback / Iteration Protocol

Follow `docs/tasks/issue-lifecycle-protocol.md`.

Normal implementation/test/integration bugs or insufficient Evidence keep the same Task:

```text
Attempt N
→ [EXECUTION REPORT]
→ Coordinator REVISE
→ status:ready
→ next env:cloud / Codex handoff
→ Attempt N+1
```

If concrete Evidence changes Scope/Claims/Success Criteria/Evidence Authority, return to `status:draft`, revise Contract/canonical docs and republish.

## Completion Protocol

Worker never closes Issue #3.

R001 completes only when Coordinator reviews the final integrated Candidate and required J1-J4 Evidence, accepts C1-C10, posts `[FINAL ACCEPTANCE]`, sets `status:done`, and closes Issue #3.

Closing R001 proves Web-only direct media-path feasibility. It does not prove R002 TV autoplay, R003 phone resource baseline, R004 Jellyfin, R005 real site plugin behavior, R006 Native Panel, R008 full security boundary, or the overall Core Feasibility Gate. R007 Playback concurrency is an already-accepted separate prerequisite authority, not a result claimed by R001.