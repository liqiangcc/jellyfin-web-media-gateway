# Task — GENERIC-YTDLP-REAL-HARNESS-PREP

## Metadata

```text
GitHub Issue: #73
Task ID: GENERIC-YTDLP-REAL-HARNESS-PREP
Task kind: implementation + deterministic verification
Contract Revision: R2
Planning Base: 32f5461b18fbba44991727bc0cbeae15cfdad328
Preferred worker: cloud-codex
Eligible environment: env:cloud
Accepted upstream: #66 GENERIC-YTDLP-EXTRACT-PREP Final Accepted; #60 runtime/security Final Accepted
Triggering target Evidence: #67 Attempt 1 BLOCKER / FROZEN_RUNTIME_SETUP
Downstream: #67 GENERIC-YTDLP-BILIBILI-REAL Attempt 2
Freshness policy: dependency-aware
```

> #73 owns the durable real-site verification harness and its frozen yt-dlp runtime preparation. It does not access Bilibili and does not enable generic-ytdlp in production.

## Why R2 exists

#73 Attempt 1 was previously Final Accepted from hosted deterministic Evidence. #67 then executed the accepted harness on the approved Ubuntu ARM64 phone and produced new contradictory Evidence:

```text
exact Candidate: 826d02c22105ee1877ae79706d2cb03112f995a9
direct public HTTPS: HTTP 200
direct frozen Bilibili page: HTTP 200
harness result: BLOCKED
process_error: FROZEN_RUNTIME_SETUP
broker_request_count: 0
```

Therefore no Bilibili extractor behavior was observed. The failure happened before `R008Broker` received any request.

The accepted R1 script coupled every smoke invocation to a fresh temporary:

```text
pip install --target <temp>
  yt-dlp @ git+https://github.com/yt-dlp/yt-dlp.git@3a08beaf...
```

and removed proxy variables before that supply-chain acquisition. This works in hosted CI but is not a reliable target-runtime preparation strategy.

R2 separates:

```text
supply-chain dependency acquisition
!=
formal extractor/site network authority
```

A setup route may use the target/operator's ordinary outbound route, including an explicitly present setup proxy, solely to acquire the exact frozen dependency. Formal extractor traffic must still run with proxy variables scrubbed and must still use only `R008Broker`.

## Goal

Make the repository-owned smoke command genuinely reusable on the approved target without ad-hoc code or security weakening:

```text
one public URL
→ verify/reuse exact user-owned frozen yt-dlp cache
   or atomically prepare it through setup-only supply-chain networking
→ scrub proxy/setup environment
→ verification smoke entrypoint
→ R008Broker::default()
→ BrokerProcessRunner
→ frozen yt-dlp worker / extract
→ GenericYtdlpAdapter explicit runtime seam
→ recognize + resolve
→ safe summary only
```

The normal production registry and `GenericYtdlpAdapter::default()` remain DisabledRunner.

## Frozen dependency identity

Exactly:

```text
yt-dlp version: 2026.08.19
source commit: 3a08beaf031ab68f966401ead017ac81fe8486cf
```

No moving release/tag/global package may substitute for this identity.

## In Scope

### A. Durable user-owned frozen runtime cache

Replace the mandatory per-run temporary install with a fixed, deterministic user-owned cache selected by repository code/script, expected shape:

```text
${XDG_CACHE_HOME:-$HOME/.cache}/jellyfin-web-media-gateway/
  generic-ytdlp/
    2026.08.19-3a08beaf031ab68f966401ead017ac81fe8486cf/
      site-packages/...
      verified.json or equivalent bounded provenance marker
```

Exact naming may differ, but the following semantics are required:

1. cache lives only in user-writable non-root storage;
2. cache identity is bound to exact yt-dlp version + commit;
3. incomplete preparation uses a Task-owned staging directory and is atomically promoted only after verification;
4. a valid cache is reusable across smoke invocations and does not require fresh network access;
5. every smoke invocation re-validates sufficient provenance before executing Python code from the cache;
6. invalid/mismatched cache is rejected or rebuilt; it is never silently trusted;
7. no global/site-system yt-dlp fallback;
8. no root/sudo/package-manager requirement.

A repository-owned `prepare` helper may be added, but the normal `scripts/generic-ytdlp-real-smoke.sh <url>` path must be able to prepare-or-reuse the cache without the Target Worker writing code.

### B. Setup-only supply-chain network route

Dependency acquisition is **not extractor traffic**.

The preparation step may use the process's ordinary setup network environment, including HTTP(S)_PROXY if explicitly present in that setup environment, because it only downloads the frozen yt-dlp source/dependency from its known supply-chain origin.

Rules:

- no proxy URL/credential may be written to cache metadata, logs or Evidence;
- no caller-supplied yt-dlp source URL, commit, package name or executable;
- acquisition source and frozen commit are repository-owned constants;
- the setup route must not be reused as extractor/network authority;
- before launching the smoke binary/worker, remove `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY` and lowercase equivalents from the runtime environment;
- R008's pinned client remains `.no_proxy()` independently;
- tests must prove a sentinel setup proxy can be used during preparation without becoming visible/usable by extractor runtime.

If setup cannot acquire or verify the exact frozen dependency, return bounded `FROZEN_RUNTIME_SETUP`/`FROZEN_RUNTIME_VERIFY` and fail closed.

### C. Frozen runtime provenance verification

Before promotion/use, verify at least:

- imported `yt_dlp.version.__version__ == 2026.08.19`;
- installation/source metadata proves commit `3a08beaf031ab68f966401ead017ac81fe8486cf` or an equally strong repository-approved digest/provenance binding;
- imported module resolves under the fixed cache, not global Python state;
- cache/staging ownership is the current non-root user;
- no root-owned dependency tree is required for normal use.

If VCS metadata representation differs across platforms, use a deterministic equivalent but do not reduce provenance to version string alone.

### D. Verification-only smoke runtime

Preserve the accepted R1 runtime path:

```text
scripts/generic-ytdlp-real-smoke.sh <one HTTP(S) URL>
→ feature-gated generic-ytdlp smoke binary
→ SafeBroker::new(R008Broker::default())
→ BrokerProcessRunner
→ repository-owned worker/sandbox
→ GenericYtdlpAdapter explicit runtime seam
→ recognize + resolve
```

Requirements:

- accepts exactly one source URL as content input;
- caller cannot choose yt-dlp argv/config/plugin/profile/Cookie/Auth/format selector/browser state;
- caller cannot choose arbitrary Python/browser/worker/sandbox executable paths;
- no production registry/default mutation;
- no alternate/direct/open-proxy extractor networking.

### E. Safe output / diagnostics

Preserve R1's safe summary contract. Durable stdout/stderr may contain only bounded fields such as:

```text
result: PASS | UNSUPPORTED | BLOCKED | FAIL
plugin: generic-ytdlp
protocol: http-file | hls | n/a
stream_count: N
title_length: N | n/a
broker_status_class: bounded class | n/a
broker_error_code: allowlisted code | n/a
broker_request_count: N
process_error: bounded enum
runtime_cache: hit | prepared | n/a
```

Never durable-log:

- full source/resolved media URL;
- signed query;
- Cookie / Authorization / token / profile/account data;
- raw worker stderr;
- setup proxy URL/credentials;
- page/media payload;
- arbitrary pip/git setup logs.

Setup diagnostics needed for failure classification must be sanitized before Evidence.

### F. Cleanup semantics

R2 intentionally distinguishes durable verified dependency cache from temporary Task state:

```text
verified fixed cache
→ may persist for future target runs

staging/install temp/log/process files
→ must be removed on success/failure/cancel
```

No orphan staging directory, worker process, sandbox descendant or setup log remains after completion.

Provide a bounded repository-owned cleanup/invalidation path for the frozen cache; do not require root.

## Deterministic / Hosted Verification

No real Bilibili/site access in #73.

Required tests:

1. cold-cache preparation from a controlled/frozen fixture or approved supply-chain setup path;
2. warm-cache reuse with network setup deliberately unavailable;
3. corrupted/version-mismatched/commit-mismatched cache rejection;
4. interrupted preparation cannot promote partial cache;
5. setup proxy sentinel is permitted only for dependency acquisition and is scrubbed before extractor runtime;
6. real extractor path still constructs `R008Broker` and no other HTTP client becomes extractor authority;
7. safe output leak tests cover setup success/failure and runtime success/failure;
8. production Default remains DisabledRunner;
9. #60/#66 lifecycle/cancellation/descendant/security regressions remain passing.

## Architecture / Security Invariants

1. #60 sandbox/no-direct-egress remains unchanged.
2. R008 remains the sole extractor HTTP(S) authority.
3. No change to R008 DNS/public-IP/pinning/redirect/TLS/Secret policy or 96 KiB response limit.
4. Setup networking is supply-chain-only and cannot flow into the extractor runtime.
5. No caller-controlled yt-dlp source/version/commit/executable/argv/config/plugin/profile/Cookie/Auth/format selector.
6. No global yt-dlp fallback.
7. Raw worker stderr remains suppressed.
8. No Bilibili request in #73.
9. No SiteAdapter/Navigation/DASH/remux/login/Native Panel contract expansion.
10. Production generic-ytdlp remains disabled by default.

## Parallel Ownership

#71 and #75 may continue in parallel.

#73 R2 primarily owns:

```text
scripts/generic-ytdlp-real-smoke.sh
optional frozen-runtime prepare/cleanup helper scripts
plugins/generic-ytdlp smoke/runtime-prep support
generic-ytdlp prep workflow/tests
```

#73 must not modify:

```text
site-adapter-api navigation semantics
source_session/playback NextItem/PreviousItem work owned by #71
browser.rs / Chromium runtime work owned by #75
```

## Claims

```text
H1 — Durable target executable path
The repository-owned command can reach the accepted extraction runtime on the approved target without ad-hoc code.

H2 — Extractor network authority unchanged
All extractor HTTP(S) traffic still uses R008Broker with no ambient/setup proxy authority.

H3 — Frozen runtime provenance
Exact yt-dlp version+commit is prepared/reused from user-owned storage and revalidated before use.

H4 — Setup/runtime separation
A setup-only proxy/network route cannot leak into extractor runtime or formal site Evidence.

H5 — Evidence-safe output
Setup and runtime paths expose only bounded non-secret classifications.

H6 — Fail-closed cache lifecycle
Partial/corrupt/mismatched cache never executes; staging/temporary state is cleaned deterministically.

H7 — Production boundary unchanged
Default/normal registry remains DisabledRunner and no production network enablement is added.

H8 — Existing security/lifecycle preserved
#60/#66/R008 sandbox, cancellation, descendants, Secret and conformance regressions remain passing.
```

## Verification Jobs

| Job | Claims | Runner | Evidence |
|---|---|---|---|
| J1 cold/warm cache + provenance | H1,H3,H6 | GitHub-hosted Ubuntu | exact-Candidate cold prepare, warm offline reuse, mismatch/interruption tests |
| J2 setup/runtime network separation | H2,H4,H5 | GitHub-hosted Ubuntu | setup proxy sentinel + runtime scrub + R008-only authority tests |
| J3 safe output/cache cleanup | H5,H6 | GitHub-hosted Ubuntu | leak scans, failure classifications, staging/cache lifecycle |
| J4 workspace/security regressions | H7,H8 | GitHub-hosted Ubuntu | fmt/clippy/test + #60/#66/R008/conformance regressions |

All jobs must assert exact Candidate SHA.

## Success Criteria

1. H1-H8 PASS on one exact Candidate.
2. One repository command can prepare-or-reuse the frozen runtime and then drive the accepted extraction path without ad-hoc target code.
3. A valid cache can be reused when setup network is unavailable.
4. Exact version+commit provenance is checked before executing cached yt-dlp.
5. Setup network/proxy state is scrubbed before extractor runtime and cannot become R008 authority.
6. Partial/corrupt cache fails closed.
7. Output remains evidence-safe across preparation and runtime failure paths.
8. Production Default/registry remains disabled.
9. J1-J4 pass on exact Candidate.
10. No real-site/Bilibili claim is made by #73.

## Evidence Contract

Report:

```text
Attempt / worker / environment
Base SHA
Candidate SHA / PR
Harness entry command/path
Cache path shape (sanitized; no user-specific secrets)
Cold prepare result
Warm/offline reuse result
Frozen yt-dlp version/commit proof
Setup proxy sentinel proof
Runtime proxy scrub proof
R008Broker construction path
Corrupt/partial cache rejection proof
Safe output/leak scan
Temporary cleanup proof
Production DisabledRunner proof
J1-J4 run/job IDs
Claims H1-H8
Real-site execution: NOT RUN
Downstream readiness for #67 Attempt 2
```

## Freshness / Integration Contract

Semantic domains:

- `plugins/generic-ytdlp/**` runtime-prep/smoke;
- `scripts/generic-ytdlp-real-smoke.sh` and setup/cleanup helpers;
- `gateway-egress/**` / R008 only for regression/read authority, not redesign;
- production generic-ytdlp registration/default behavior.

Unrelated #67/#71/#75 task docs and their non-overlapping implementation surfaces do not invalidate #73. Semantic changes in generic-ytdlp/R008 require Coordinator classification.

## Out of Scope

- real Bilibili/public-site verification;
- changing R008 policy/body limit for compatibility;
- production generic-ytdlp enablement;
- account/login/Cookie/profile;
- Navigation / Bilibili multipart;
- Chromium/Native Panel runtime;
- DASH/remux/transcode;
- performance/capacity claims.

## Completion Protocol

```text
status:ready
→ claim / Attempt 2
→ implementation + J1-J4
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

If R2 cannot make target-reusable frozen runtime preparation work without weakening the invariants:

```text
[BLOCKER REPORT]
→ status:blocked
→ release owner
→ STOP
```

Worker must not execute #67/#68, set status:done, merge own PR or close #73.
