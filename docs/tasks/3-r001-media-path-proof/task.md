# Task — R001 Media Path Proof

## Metadata

```text
GitHub Issue: #3
Parent Goal / Research Item: R001 / Core Feasibility / Phase 0A-1 Media Path Proof
Task / Research ID: R001
Task kind: combined
Planning base commit: 9bfffdb864f98082714c154330726e58a9f27702
Publication base commit: refresh required after R007 Final Acceptance
Candidate commit: n/a (live state belongs in Issue)
Session bootstrap prompt: docs/tasks/3-r001-media-path-proof/prompt.md
Downstream handoff profile: docs/tasks/handoffs/web-gpt.md
Preferred worker: web
Eligible worker environments: env:web-gpt
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, rust-build, rust-test, browser-automation
Publication dependency: Issue #2 / R007 Final Acceptance
```

> Live status, owner, candidate, runs and results belong in Issue #3. This draft is not executable until the Coordinator republishes it after R007 Final Acceptance.
>
> R007 may change the executable Playback/revision surface that R001 must build on. Before publication, refresh the base commit and reconcile this Contract against the accepted R007 code/docs. Do not publish R001 against an obsolete concurrency contract.

## Goal

Prove the first Web-only Media Gateway path with public/non-DRM media:

```text
Source input
→ SiteAdapterRegistry
→ SourceLocator
→ SiteAdapter.resolve
→ ResolvedMedia
→ Media Gateway
→ scoped Gateway media capability
→ Web Display
```

The proof must establish that direct HTTP MP4 can be consumed end-to-end by a browser through the Gateway with correct Range/seek behavior, while HLS receives a concrete manifest/segment result, upstream secrets remain server-side, `/stream` cannot become an arbitrary open proxy, and Jellyfin is not required.

R001 is a media-path feasibility Task, not a complete product implementation.

## Why / Context

`docs/technical-feasibility-validation.md` makes R001 a Core-blocking P0 feasibility item. The canonical success conditions require:

- at least one of MP4/HLS to play stably, with an explicit result for the other;
- pause/play/seek usability;
- Range/segment semantics preserved;
- upstream Secret not exposed to Display;
- `/stream` not degrading into an arbitrary open proxy;
- no concrete-site special case in Core;
- Jellyfin disabled without breaking the path;
- no evidence of unbounded media buffering/cache growth.

`docs/mvp-plan.md` maps R001 to Phase 0A-1 and requires the first real Rust media-path implementation, not another design-only document.

## Publication Dependency on R007

R001 may be planned while R007 is pending, but it must not be published until Issue #2 has Coordinator Final Acceptance.

Before publication the Coordinator must:

1. read Issue #2 and accepted Evidence;
2. read the final R007 `gateway-core` implementation and `docs/implementation-contracts.md`;
3. refresh `Publication base commit` to the actual accepted main HEAD;
4. confirm R001 media refresh/callback behavior does not reintroduce stale-authority semantics rejected by R007;
5. update this Contract only where the accepted R007 interface requires it;
6. run the normal Publication Gate.

R007 implementation/test bugs do not change R001 Goal. A true R007 Contract revision may require R001 Contract reconciliation before publication.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: R001 implementation and its required browser/HTTP integration evidence are portable and can be produced by Web Worker + GitHub-hosted CI. Phone thermal/resource proof is R003 and physical-TV behavior is R002, so no independent target Evidence Authority is required for R001 acceptance.
```

Do not split MP4, HLS, browser, x64 and generic ARM64 into separate business Tasks merely because they use different Jobs.

## Primary and Secondary Media Proof

### Primary required browser path — direct HTTP MP4

R001 uses direct HTTP MP4 as the required browser end-to-end proof because it can exercise native browser playback without introducing an HLS JavaScript player dependency merely to satisfy the first media-path Gate.

Required path:

```text
public/non-DRM MP4 source
→ generic-direct SiteAdapter
→ ResolvedMedia
→ Gateway stream capability
→ Media Gateway Range proxy
→ Web Display <video>
→ play / pause / seek
```

### Secondary required result — HLS

HLS must not remain “assumed supported”. At minimum R001 must implement and verify enough HLS proxy semantics to produce a concrete result for:

- master/variant playlist fetch where applicable;
- relative segment URI resolution/rewriting through Gateway capability;
- query parameters;
- redirect handling within the allowed egress model;
- segment requests;
- interrupted/failed segment behavior;
- seek-relevant playlist/segment behavior.

Full browser HLS playback is required only if the chosen browser/runtime natively supports the selected HLS path or an already justified player dependency is introduced without expanding R001 into a separate frontend framework project.

R001 may PASS with MP4 browser playback as the stable primary path and a fully documented HLS contract/result plus follow-up plan, as allowed by the canonical R001 criteria.

## Work Role

### Implementation

Build the minimum real media path needed to prove R001, reusing the accepted R007 workspace/Core rather than recreating it.

Expected capabilities/components:

- the minimal stable `SourceLocator` / `ResolvedMedia` types needed by the path;
- `SiteAdapter` / `SiteAdapterRegistry` boundary sufficient for a `generic-direct` adapter;
- `generic-direct` recognition/resolution for public/non-DRM direct media inputs;
- a Media Gateway HTTP service or testable server surface;
- scoped, short-lived media capability/token mapping to server-known session/item/resource identity;
- HTTP file proxy with correct Range behavior;
- HLS manifest/segment proxy/rewriting sufficient for the secondary result;
- minimal Web Display page/player required for browser proof;
- deterministic upstream fixture server for failure, Range, redirect and Secret-boundary tests;
- public/non-DRM source smoke used as acceptance Evidence;
- meaningful GitHub Actions jobs if the repository does not already contain equivalent CI after R007.

### Verification

Claims to verify:

```text
C1: A public/non-DRM direct MP4 resolves through Registry/SiteAdapter/ResolvedMedia and plays through Media Gateway in a browser without Jellyfin.
C2: HTTP byte Range semantics are preserved well enough for browser seek/re-request behavior.
C3: HLS manifest/segment proxy semantics produce a concrete verified result, including relative URI/query/redirect/segment behavior required by the chosen fixture/source.
C4: Browser/Display-visible requests and public response data contain no upstream Cookie, Authorization, bearer token or Vault material.
C5: `/stream` or equivalent media endpoint cannot be used to select an arbitrary caller-supplied upstream URL or replay a capability across the wrong session/item/resource.
C6: invalid/expired/cross-session/cross-item media capability use is rejected deterministically.
C7: upstream 403/404, redirect, Range-unsupported and interrupted media/segment failures map to bounded, explainable Gateway failures rather than corrupting another Playback authority.
C8: Core contains no concrete-site URL/DOM/Cookie special case; generic-direct remains a plugin/adapter boundary.
C9: streaming is bounded: active connections/resources are released after abort/seek/end and no cache/buffer/retained-resource structure grows with total bytes streamed.
C10: canonical docs and executable behavior agree on the proven R001 media path and its unsupported/deferred boundaries.
```

## Routing Rationale

```text
Implementation / orchestration
→ Web ChatGPT Worker + GitHub connector

Portable HTTP/unit/contract verification
→ GitHub-hosted x64

Browser integration
→ GitHub-hosted x64 + headless Chromium (or equivalent supported browser)

Bounded repeated/soak verification
→ GitHub-hosted x64

Generic ARM64 compatibility
→ optional GitHub-hosted ARM64

Phone CPU/RSS/temperature
→ NOT R001 acceptance; belongs to R003

TV autoplay/remote UX
→ NOT R001 acceptance; belongs to R002
```

INFRA-001 is not a publication or acceptance dependency for R001.

## Preconditions

- R007 Issue #2 has `[FINAL ACCEPTANCE]` and is closed/done as accepted work.
- Publication base commit is refreshed after that acceptance.
- Accepted R007 revision/media-refresh semantics are reused rather than bypassed.
- Rust stable/toolchain rules from the accepted repository state are followed.
- Public acceptance source is legal, non-DRM and suitable for automated/recorded verification.
- Tests must not depend only on an external public host; deterministic fixture coverage is required for protocol/security/failure semantics.

## In Scope

- minimal SiteAdapter API/Registry surface required by generic-direct;
- generic-direct direct media recognition/resolution;
- minimal ResolvedMedia shape required by direct MP4/HLS;
- Media Gateway streaming endpoint and scoped capability model;
- HTTP GET/HEAD behavior required by browser/media clients where applicable;
- single/multi Range behavior to the extent required by selected browser/source, with unsupported cases explicitly rejected rather than silently corrupted;
- MP4 browser playback, pause/play/seek;
- HLS master/variant/segment/URI rewriting result;
- redirect handling through central/compatible egress checks available at this phase;
- deterministic fixture upstream including optional protected endpoint to prove server-side auth injection boundary;
- invalid/expired/cross-session/cross-item token replay tests;
- bounded streaming/abort cleanup tests;
- public source acceptance smoke;
- docs updates driven by Evidence.

## Out of Scope

- R002 TV audible autoplay / physical remote UX;
- R003 target-phone CPU/RSS/temperature/60-minute resource acceptance;
- Jellyfin DisplayAdapter;
- software video transcoding;
- FFmpeg remux unless a minimal non-video-transcoding remux becomes strictly necessary to explain an HLS/DASH boundary and Coordinator explicitly revises Scope;
- DASH full playback;
- concrete Bilibili/YouTube plugin logic;
- Site Auth / real account cookies;
- Native Site Panel / Browser Worker;
- full Control application;
- production persistence/restart recovery;
- full R008 security proof (DNS rebinding/all private ranges/metadata matrix etc.).

R001 must still obey existing Egress/Secret invariants; “R008 is later” is not permission to disable SSRF or expose Secrets.

## Architecture Invariants

- Core accesses source implementations only through `SiteAdapterRegistry` / SiteAdapter contract.
- `generic-direct` is an adapter/plugin, not a Core special case.
- `SourceLocator` identifies content/re-resolution semantics; transient CDN/media URLs are not promoted into stable content identity.
- upstream Cookie/Authorization/bearer material never enters `ResolvedMedia.public_headers` or browser-visible capability URLs.
- Display consumes Gateway media capability, not raw upstream Secret.
- media capability identifies server-approved resources; caller input cannot turn it into `GET /stream?url=<arbitrary>`.
- redirect/egress handling must not silently bypass the repository's central EgressPolicy direction.
- Jellyfin failure/absence cannot block Web Display.
- no large-media default disk cache or full-object buffering.
- accepted R007 stale async/media-refresh semantics remain authoritative.

## Media Capability Contract to Prove

The exact token format is implementation-defined, but the semantics must be equivalent to:

```text
MediaCapability
├── opaque random/signed identifier
├── session_id binding
├── item_id / item_revision binding
├── resource/stream identity binding
├── expiry
└── server-side mapping to approved upstream resource + access capability
```

Required properties:

- browser cannot replace the upstream URL by editing a query parameter;
- token does not embed upstream Cookie/Authorization;
- token expires;
- wrong session/item/resource replay fails;
- invalid token fails without upstream request;
- logs/errors do not reveal Secret or unnecessary signed upstream query values.

Full cryptographic deployment design may evolve later; R001 must prove the authority boundary, not invent a permanent security subsystem unrelated to the first path.

## Upstream Secret Boundary Fixture

Public source smoke alone cannot prove Secret injection safety because public media needs no auth.

Add a deterministic local fixture representing a protected upstream resource:

```text
fixture requires server-side Authorization/Cookie
→ ResolvedMedia/public client data contains only non-secret metadata + opaque access ref
→ Media Gateway injects fixture credential server-side via test-scoped access provider
→ browser/client receives media bytes
→ browser request/log/public response never contains fixture Secret
```

This is a contract fixture, not production Vault implementation and not real site auth.

## Failure Cases Required

At minimum verify:

- upstream 404;
- upstream 403;
- redirect;
- redirect to a target rejected by currently available egress policy, if that policy surface exists after R007/base refresh;
- Range-supported source;
- Range-unsupported source;
- client abort/reconnect;
- HLS segment interruption/failure;
- expired media capability;
- invalid media capability;
- cross-session replay;
- cross-item replay;
- repeated browser/media request for the same valid capability.

Failures must not create a new global Playback authority or mutate a newer item/media generation contrary to accepted R007 semantics.

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

J2 must prove actual media element behavior rather than only HTTP responses:

- media metadata/load succeeds;
- playback time advances;
- pause stops advancement within normal test tolerance;
- seek changes playback position and causes expected Range/re-request behavior when applicable;
- browser console/player errors are captured;
- browser network inspection shows no upstream Secret.

Do not treat “HTTP 200 from /stream” as browser-playback PASS.

### HLS Evidence

At minimum record:

- manifest before/after Gateway rewriting or equivalent resource mapping;
- resolved segment requests;
- relative/query URI correctness;
- failure behavior for missing/interrupted segment;
- whether browser playback was tested, supported, deferred or unnecessary for the selected R001 primary path.

No HLS result may remain implicit.

### Bounded Resource / Cleanup Evidence

R001 does not set phone thermal thresholds. It must still prove bounded server resource ownership.

Instrument/test at least:

- active stream/request count;
- retained media-resource/capability entries;
- temporary buffers/cache objects introduced by R001;
- cleanup after client abort, seek/reconnect and normal end;
- repeated scenario count >= 100 cycles or an equivalent deterministic bounded test set.

PASS requires resource counts to return to their defined baseline after the scenario and retained structures to be bounded by explicit limits rather than cumulative bytes streamed.

Process RSS may be recorded as supplemental hosted evidence; target-phone CPU/RSS/temperature acceptance remains R003.

## Success Criteria

### Task success

1. A real public/non-DRM MP4 source traverses Registry → generic-direct → ResolvedMedia → Media Gateway → Web Display and plays in browser with Jellyfin absent.
2. Browser pause/play/seek is demonstrably usable for the primary MP4 path.
3. HTTP Range/re-request semantics required by the tested MP4 source/browser are preserved or explicitly rejected where unsupported; Gateway does not silently corrupt byte ranges.
4. HLS has a concrete verified manifest/segment/seek-related result and explicit browser support/defer result.
5. browser-visible requests/public headers/log artifacts contain no upstream Cookie, Authorization, bearer token or Vault material.
6. media capability cannot select arbitrary caller-provided upstream URLs; invalid/expired/cross-session/cross-item replay is rejected.
7. required failure cases are deterministic and do not corrupt newer Playback/media authority.
8. Core contains no concrete-site special case; generic-direct remains behind Registry/adapter boundary.
9. repeated abort/seek/reconnect/end scenarios release active resources; no retained buffer/cache/resource structure grows with total bytes streamed.
10. public source acceptance Evidence and deterministic fixture Evidence are both present; external public host behavior is not the only proof.
11. canonical docs are updated to match the path actually proven, including explicit unsupported/deferred HLS/DASH/remux boundaries.
12. no R002/R003/Jellyfin/real-site claim is falsely marked PASS by R001.

### Claim success

```text
C1 PASS when: J2 + J3 prove public MP4 end-to-end browser playback through the Gateway with Jellyfin absent.
C2 PASS when: deterministic Range tests + browser seek Evidence show preserved byte-serving semantics.
C3 PASS when: HLS fixture/public Evidence proves concrete manifest/segment behavior and records browser support status.
C4 PASS when: protected fixture and browser network/log artifacts show Secret remains server-side.
C5 PASS when: API/token tests prove caller cannot choose arbitrary upstream and capability is correctly resource-bound.
C6 PASS when: invalid/expired/cross-session/cross-item replay tests fail before unauthorized upstream access.
C7 PASS when: required upstream/protocol failures produce bounded errors without newer-authority mutation.
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
Job ID: J1 | J2 | J3 | J4 | J5
Orchestrator:
Execution plane:
Runner class / image:
Execution host:
Target:
OS / architecture:
Rust toolchain:
Browser/version (J2):
Public source type/host (J3; no sensitive signed query):
Planning/publication base commit:
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

Do not store Cookie, Authorization, bearer tokens, fixture secrets beyond deliberately non-sensitive test values, account data, or unnecessary full signed media URLs in Issue/artifacts/logs.

## Failure / Blocked Handling

FAIL examples:

- browser cannot play either required primary MP4 path and no canonical-allowed alternative media path succeeds;
- Range/seek semantics are corrupted by Gateway;
- browser receives upstream Secret;
- caller can use `/stream` to proxy arbitrary URL;
- cross-session/item capability replay succeeds;
- HLS remains untested/assumed;
- repeated abort/seek/reconnect leaks retained resources cumulatively;
- implementation bypasses Registry/generic-direct plugin boundary;
- Jellyfin becomes required for the Web-only path.

BLOCKED examples:

- R007 has not reached Final Acceptance when publication is attempted;
- accepted R007 changes require a Contract reconciliation not yet reviewed;
- GitHub-hosted browser/runtime needed for J2 cannot be made available after reasonable retry;
- no suitable legal public non-DRM acceptance source is available at execution time;
- canonical Egress/Secret contracts conflict in a way requiring Coordinator design revision before implementation.

Do not lower Success Criteria to manufacture PASS.

## Deliverables

Expected final deliverables after publication/execution:

- R001 media path implementation in the accepted Rust workspace;
- minimal SiteAdapter/Registry + generic-direct path required by R001;
- Media Gateway direct HTTP file proxy and scoped media capability;
- HLS contract/result implementation and tests;
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

Normal implementation/test bugs or insufficient Evidence keep the same Task:

```text
Attempt N
→ [EXECUTION REPORT]
→ Coordinator REVISE
→ status:ready
→ next env:web-gpt handoff
→ Attempt N+1
```

If accepted R007 or R001 Evidence changes Scope/Claims/Success Criteria/Evidence Authority, return to `status:draft`, revise the Contract/canonical docs and republish.

## Completion Protocol

Worker never closes Issue #3.

R001 completes only when Coordinator reviews required J1-J4 Evidence, accepts C1-C10, posts `[FINAL ACCEPTANCE]`, sets `status:done`, and closes Issue #3.

Closing R001 proves the Web-only direct media path feasibility claim. It does not prove R002 TV autoplay, R003 phone resource baseline, R004 Jellyfin, R005 real site plugin behavior, R006 Native Panel, R008 full security boundary, or the overall Core Feasibility Gate.