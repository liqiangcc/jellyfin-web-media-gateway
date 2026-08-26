# Task — GENERIC-YTDLP-EXTRACT-PREP

## Metadata

```text
GitHub Issue: #66
Task ID: GENERIC-YTDLP-EXTRACT-PREP
Task kind: implementation + deterministic verification
Planning Base: 50a3c09126d31b1554a2f2f93c16ab9bd3586551
Preferred worker: cloud-codex
Eligible environment: env:cloud
Accepted upstream: #60 GENERIC-YTDLP-RUNTIME-PREP Final Accepted
Frozen upstream: yt-dlp 2026.08.19 @ 3a08beaf031ab68f966401ead017ac81fe8486cf
Downstream real-site verification: #67 GENERIC-YTDLP-BILIBILI-REAL
Downstream user-visible E2E: #68 BILIBILI-WEB-E2E
Freshness policy: dependency-aware
```

> #66 owns only the real extraction implementation and deterministic proof. It does not perform real Bilibili access and does not enable production generic-ytdlp by default.

## Context

#60 Final Acceptance proved the security/runtime architecture:

```text
frozen yt-dlp Python worker
→ inherited structured IPC
→ Gateway-owned R008 HTTP(S) broker
→ worker/descendants cannot open direct sockets
→ bounded lifecycle/diagnostics
```

However the current worker `probe` path is still a deterministic security fixture: it performs a brokered request to a fixture and returns static machine output. It does not yet execute normal yt-dlp site extraction with `extract_info()`.

#65 also proved that reviving the old #23 Bilibili-specific branch as an integration-only merge is contract-invalidating against accepted #39 SiteAdapter authority. First Bilibili playback therefore proceeds through the current generic-ytdlp fallback path rather than making old site-specific navigation APIs a prerequisite.

## Goal

Implement an actual, bounded, anonymous, broker-only yt-dlp extraction path that converts a public URL into the **current accepted SiteAdapter `ResolvedMedia` shape**.

```text
SourceLocator(URL)
→ GenericYtdlpAdapter explicit runtime-enabled construction
→ BrokerProcessRunner
→ worker action: extract
→ frozen yt_dlp.YoutubeDL.extract_info(download=False)
→ broker-only RequestHandler for all HTTP(S)
→ fixed format selection owned by plugin
→ bounded machine output
→ existing Rust parser + #39 conformance
→ current ResolvedMedia
```

Production `GenericYtdlpAdapter::default()` must remain fail-closed on `DisabledRunner`.

## Functional scope decision

The first playback milestone targets one **muxed audio+video stream** that Web Display/Gateway can consume without introducing a new DASH/remux contract.

The worker owns a fixed format policy equivalent in intent to:

```text
best muxed audio+video
→ supported direct HTTP(S) file or HLS
→ otherwise explicit unsupported
```

Caller cannot choose format selectors or yt-dlp options.

If yt-dlp returns only separate audio/video DASH formats for a source, #66 must not silently add old #23 `Dash` semantics to the current SiteAdapter API. Return a bounded unsupported classification; #67 will determine whether the frozen Bilibili sample is compatible with this first-playback policy.

## Accepted authority / invariants

1. #39 current `site-adapter-api` is implementation authority. Do not import #23-only `NavigationContext`, `ResolveContext`, `Dash`, duration/expiry or site-specific AdapterError variants merely to make Bilibili work.
2. #60 brokered runtime and no-direct-egress enforcement are preserved.
3. R008 remains DNS/public-IP/address-pinning/redirect/TLS/Secret authority.
4. No Cookie, Authorization, profile, netrc, caller proxy, browser state or credential input is admitted.
5. No `curl_cffi`, direct urllib/requests/websocket handler, external downloader, remote component, arbitrary plugin or runtime discovery is admitted.
6. No caller-controlled executable, argv, yt-dlp config or format selector.
7. `GenericYtdlpAdapter::default()` remains disabled and normal production registration must not gain network authority in this Task.
8. No real Bilibili/site smoke in #66.
9. No change to #23/#36/#37/#65 state or semantics.
10. No navigation/login/native-panel functionality in this Task.

## In Scope

### A. Actual extraction action

Add a worker action, expected `extract`, using the frozen yt-dlp API path rather than the current static `_probe` fixture.

Requirements:

- call normal frozen yt-dlp extraction semantics (`extract_info(..., download=False)` or equivalent API);
- use the existing broker-only `YoutubeDL` / RequestDirector path so all HTTP(S) extractor requests traverse fd 3 and R008;
- keep `simulate` / no-download behavior;
- no media payload download;
- no normal handler set/direct socket authority;
- fixed anonymous options and fixed format policy;
- playlists/collections are not expanded for this first playback Task unless needed only to select the requested single item safely;
- output only bounded machine JSON needed by the current Rust parser.

### B. Current ResolvedMedia normalization

Normalize only values supported by current `site-adapter-api`:

```text
ResolvedMedia
- title
- source_site = generic
- streams[]
- subtitles[] (may remain empty in this Task)
- protection
```

For the selected primary stream:

- audio and video must both be present for first-playback PASS;
- direct HTTP/HTTPS file maps to `http-file`;
- HLS/m3u8 maps to `hls` only if the existing Gateway/Web path supports it under current contract;
- separate DASH/requested-formats composition is rejected as unsupported here;
- output URL must be HTTP(S), bounded, and not logged in durable Evidence;
- public headers pass current Secret validation;
- `upstream_access_ref` remains absent in anonymous mode.

Do not place Cookie/Authorization in `public_headers`.

### C. Explicit runtime-enabled adapter construction

The accepted broker runtime currently exists behind `runtime-prep`, while normal `Default` stays disabled.

Provide the smallest explicit construction seam needed by verification and later #67/#68, for example a feature-gated constructor that accepts an already-created `Arc<dyn ProcessRunner>` / `BrokerProcessRunner`.

Requirements:

- explicit, non-default, feature-gated;
- cannot be reached merely by ordinary production registry registration;
- default regression proves `DisabledRunner` remains production behavior;
- no caller executable/argv/format options are exposed through this seam.

### D. Deterministic extraction fixtures

Add deterministic tests that exercise **actual `extract_info` control flow**, not only static `_probe` output.

At minimum cover:

- a broker-served direct media fixture that yt-dlp accepts as one muxed playable item;
- machine-output normalization to current `ResolvedMedia`;
- unsupported separate/non-muxed or unsupported protocol shape;
- malformed/oversized extraction output;
- Secret-header/output rejection;
- default DisabledRunner regression;
- direct socket/alternate handler denial remains intact;
- broker cancellation/timeout/descendant cleanup remains intact.

Fixtures must not depend on real public websites.

## Claims

```text
C1 — Real yt-dlp extraction code path
Frozen yt-dlp normal extraction API participates in the deterministic test path; the worker no longer fabricates the extraction result for this action.

C2 — Broker-only network authority
All HTTP(S) requests made during extraction use the inherited structured broker path and accepted R008; direct worker/descendant network remains denied.

C3 — First-playback format normalization
A muxed public HTTP/HLS extraction result maps to the current ResolvedMedia contract without reviving stale #23 API additions.

C4 — Unsupported formats fail closed
Separate DASH/non-muxed/unsupported transport shapes produce explicit bounded unsupported/error results rather than contract expansion or unsafe fallback.

C5 — Anonymous/Secret boundary
No Cookie/profile/netrc/proxy/Auth/Secret authority is introduced; output/public headers remain conformant.

C6 — Explicit verification runtime only
A non-default runtime-enabled adapter can be constructed for #67/#68, but production Default/normal registry remains DisabledRunner.

C7 — Existing security/lifecycle preserved
#60 sandbox, fd isolation, resolver/broker cancellation, descendant cleanup and diagnostics remain passing.

C8 — Current plugin architecture preserved
#39 conformance, SiteAdapter ownership, Core-site-boundary and current workspace composition remain passing.
```

## Verification Jobs

| Job | Claims | Runner | Required Evidence |
|---|---|---|---|
| J1 actual extraction + normalization | C1,C3,C4,C6 | GitHub-hosted Ubuntu | frozen yt-dlp API extraction against broker fixtures; exact Candidate |
| J2 broker/sandbox/Secret | C2,C5,C7 | GitHub-hosted Ubuntu | broker-only extraction requests + direct socket/Secret/escape negatives |
| J3 current architecture/workspace | C6,C8 | GitHub-hosted Ubuntu | #39 conformance, R008, workspace fmt/clippy/test, architecture guard |
| J4 lifecycle regressions | C2,C7 | GitHub-hosted Ubuntu | cancel/timeout/crash/overflow/descendant + resolver lifecycle on exact Candidate |

All required jobs must assert exact Candidate SHA.

## Success Criteria

1. C1-C8 PASS on one exact Candidate.
2. Deterministic Evidence proves actual frozen yt-dlp extraction API participates.
3. All extractor HTTP(S) remains brokered under #60/R008 authority.
4. One muxed supported fixture maps to current `ResolvedMedia` and existing parser/conformance.
5. Separate/non-muxed DASH or unsupported protocol fails closed without changing current SiteAdapter API.
6. A feature-gated explicit verification constructor exists for downstream #67/#68.
7. Production `GenericYtdlpAdapter::default()` remains disabled.
8. J1-J4 and affected current workspace/R008/#39 regressions pass on exact Candidate.
9. No real-site result is claimed; Bilibili compatibility remains #67.

## Evidence Contract

`[EXECUTION REPORT]` must include:

```text
Attempt / worker / environment
Base SHA
Candidate SHA / PR
Frozen yt-dlp version+commit
Actual extraction worker action/location
Fixed format policy summary
Runtime-enabled construction seam
Direct-media extract_info fixture result
ResolvedMedia protocol summary (no full media URL)
Unsupported separate-DASH result
Broker request path result
Direct-network denial result
Secret/output validation result
Default DisabledRunner proof
J1-J4 run/job IDs
#39/R008/workspace/architecture regressions
Claims C1-C8
Real-site execution: NOT RUN
Production enablement: disabled
Downstream readiness for #67
```

Never persist full signed media URLs, Cookie, Authorization, tokens, account data or media payloads.

## Freshness / Integration Contract

Semantic authorities:

- #60 broker runtime/security architecture;
- #39 current SiteAdapter/conformance authority;
- #14/R008 egress authority;
- frozen yt-dlp 2026.08.19 selector.

Semantic domains:

- `plugins/generic-ytdlp/**`;
- `gateway-egress/**` and R008 surfaces;
- `site-adapter-api/**`;
- production registry/default generic-ytdlp behavior.

Unrelated docs/task changes do not invalidate Evidence. Accepted semantic changes in these domains require Coordinator reclassification.

## Out of Scope

- real Bilibili/public-site access;
- enabling generic-ytdlp by default in production;
- separate audio/video DASH composition or FFmpeg remux;
- site-specific Bilibili plugin migration;
- navigation/next episode;
- login/account/profile/native panel;
- physical TV;
- performance/soak.

## Completion Protocol

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ implement + exact-SHA J1-J4
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Worker must not run #67/#68 automatically, merge its own PR, set done/close #66, or enable production real-network generic-ytdlp by default.
