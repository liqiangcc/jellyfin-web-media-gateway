# Task — GENERIC-YTDLP-REAL-HARNESS-PREP

## Metadata

```text
GitHub Issue: #73
Task ID: GENERIC-YTDLP-REAL-HARNESS-PREP
Task kind: implementation + deterministic verification
Planning Base: 9fb6b25bc7781e1396c4e979454df962de43090d
Preferred worker: cloud-codex
Eligible environment: env:cloud
Accepted upstream: #66 GENERIC-YTDLP-EXTRACT-PREP Final Accepted; #60 runtime/security Final Accepted
Downstream: #67 GENERIC-YTDLP-BILIBILI-REAL
Freshness policy: dependency-aware
```

> #73 owns only the durable real-site verification harness. It does not access Bilibili or enable generic-ytdlp in production.

## Goal

Add a repository-owned, bounded, evidence-safe entrypoint that a later Target Runner can invoke to exercise the accepted real extraction path without ad-hoc code:

```text
one public URL
→ verification smoke entrypoint
→ R008Broker::default()
→ BrokerProcessRunner
→ frozen yt-dlp worker / extract
→ GenericYtdlpAdapter explicit runtime seam
→ recognize + resolve
→ safe summary only
```

The normal production registry and `GenericYtdlpAdapter::default()` remain DisabledRunner.

## Context

#66 Final Acceptance proved the library/runtime path but added no standalone real-network smoke harness. #67 must not invent throwaway Rust code or bypass the accepted broker merely to run on the phone.

The harness must therefore be reusable on Cloud/hosted deterministic fixtures and later on the approved phone Target Runner.

Current R008 implementation details that must remain authoritative:

- `R008Broker` owns public-web validation;
- DNS/public-IP/address pinning/TLS/redirect semantics stay unchanged;
- pinned reqwest clients use `.no_proxy()`;
- broker response body limit is currently `96 KiB`;
- Secret-classified request/response headers are rejected.

## In Scope

### A. Verification-only executable / script

Add the smallest durable entrypoint, expected shape:

```text
scripts/generic-ytdlp-real-smoke.sh
+
feature-gated Rust binary/example/test harness
```

Requirements:

- accepts exactly one HTTP(S) source URL as content input;
- does not accept caller format selectors, yt-dlp argv/config/plugin/profile/Cookie/Auth/proxy/executable authority;
- constructs `R008Broker` as the real BrokerBackend;
- constructs `BrokerProcessRunner` using repository-owned worker/sandbox paths and the frozen runtime;
- uses `GenericYtdlpAdapter::with_runtime_runner(...)` or an equivalent already-accepted #66 explicit seam;
- calls normal recognize + resolve;
- no production registry/default mutation.

### B. Frozen yt-dlp setup

Provide a bounded, non-root setup path for exact:

```text
yt-dlp 2026.08.19
commit 3a08beaf031ab68f966401ead017ac81fe8486cf
```

Rules:

- user-writable isolated location only;
- verify the frozen version/source before execution;
- temporary dependency state is cleaned by the Task-owned script/harness;
- no root/sudo/package-manager requirement is introduced;
- if the local Python environment lacks a capability required for isolated install, the later target run must report BLOCKED rather than silently use a different/global yt-dlp.

### C. Safe output contract

Stdout/stderr that may enter CI/Evidence must never print:

- full resolved media URL;
- signed query parameters;
- Cookie / Authorization / token / account data;
- media payload;
- raw worker stderr containing source URLs/secrets.

Allowed safe summary should be bounded and machine-readable, for example:

```text
result: PASS | UNSUPPORTED | BLOCKED | FAIL
plugin: generic-ytdlp
protocol: http-file | hls | n/a
stream_count: N
title_length/title_hash: optional safe metadata
broker_status_class: optional
broker_error_code: optional bounded code
process_error: optional bounded enum
```

Do not use title text in durable Evidence when a hash/length is sufficient.

### D. Safe broker diagnostic seam

The harness must preserve #60's raw-stderr suppression while making real-site failures actionable.

A site-neutral wrapper around `R008Broker` may record only bounded metadata such as:

- last/first broker error code;
- status class;
- request count;
- timeout/cancel classification.

It must not retain/log full request URLs, query strings, headers or bodies.

This is especially required so #67 can distinguish a current broker limit/policy failure (for example `BROKER_RESPONSE_TOO_LARGE`) from generic process failure without weakening the policy.

### E. Deterministic proof

No real-site access in #73. Deterministic tests/fixtures must prove:

- success summary contains no URL/query/Secret;
- unsupported format summary is bounded;
- broker egress rejection / response-too-large / timeout or equivalent errors can be surfaced as safe codes;
- ambient HTTP(S)_PROXY values are not used as network authority because R008 pinned client remains no-proxy;
- production Default remains disabled;
- cleanup is bounded and Task-owned.

## Architecture / Security Invariants

1. #60 sandbox/no-direct-egress remains unchanged.
2. R008 remains network authority; #73 does not add a second HTTP client for extractor traffic.
3. No change to R008 public/private IP policy, DNS pinning, redirect handling, TLS, Secret handling or 96 KiB body limit merely for compatibility.
4. No caller-controlled executable/argv/config/plugin/profile/Cookie/Auth/proxy/format selector.
5. The harness is verification-only and cannot be reached by normal production SiteAdapter registration.
6. Raw worker stderr remains suppressed.
7. No real Bilibili access in #73.
8. No SiteAdapter/Navigation/DASH/remux/login/Native Panel contract expansion.

## Claims

```text
H1 — Durable executable path
A repository-owned smoke entrypoint can drive the accepted #66 real extraction path without ad-hoc target code.

H2 — Real broker authority
The harness uses R008Broker as the BrokerBackend and preserves the accepted no-proxy/pinning/security path.

H3 — Frozen runtime provenance
The harness can prepare/verify exact frozen yt-dlp in an isolated non-root location with bounded cleanup.

H4 — Evidence-safe output
Success and failure output cannot expose full source/resolved URLs, signed query, Cookie/Auth/token or media payload.

H5 — Actionable safe diagnostics
Broker/process failures needed by #67 are surfaced as bounded site-neutral classifications without raw stderr/URL leakage.

H6 — Production boundary unchanged
Default/normal registry remains DisabledRunner and no production network enablement is added.

H7 — Existing security/lifecycle preserved
#60/#66 sandbox, fd/descendant cleanup, cancellation, Secret and conformance regressions remain passing.
```

## Verification Jobs

| Job | Claims | Runner | Evidence |
|---|---|---|---|
| J1 harness success/unsupported fixtures | H1,H4,H6 | GitHub-hosted Ubuntu | exact-Candidate deterministic smoke output assertions |
| J2 broker diagnostics/no-proxy/security | H2,H4,H5 | GitHub-hosted Ubuntu | real R008Broker fixture/negative harness path, proxy env sentinel, no URL/Secret leakage |
| J3 frozen runtime/setup/cleanup | H3,H4 | GitHub-hosted Ubuntu | exact yt-dlp provenance + isolated setup/cleanup |
| J4 workspace/security lifecycle regressions | H6,H7 | GitHub-hosted Ubuntu | fmt/clippy/test + #60/#66/R008/conformance relevant regressions |

All required jobs must assert exact Candidate SHA.

## Success Criteria

1. H1-H7 PASS on one exact Candidate.
2. A later target can run one durable command/script against a public URL without writing code.
3. Real extractor traffic uses `R008Broker`, not a fixture/open proxy/direct client.
4. Exact frozen yt-dlp preparation is reproducible without root and cleanup is explicit.
5. Output is evidence-safe by test, including failure paths.
6. Safe broker error classification is sufficient to diagnose policy/limit/timeout classes without raw URLs.
7. Production Default/registry remains disabled.
8. J1-J4 pass on exact Candidate.
9. No real-site/Bilibili claim is made.

## Evidence Contract

Report:

```text
Attempt / worker / environment
Base SHA
Candidate SHA / PR
Harness entry command/path
Frozen yt-dlp setup/provenance path
R008Broker construction path
Repository-owned worker/sandbox path
Success fixture safe output
Unsupported fixture safe output
Broker failure safe classification
Proxy-env no-authority proof
Secret/URL leak scan
Cleanup proof
Production DisabledRunner proof
J1-J4 run/job IDs
Claims H1-H7
Real-site execution: NOT RUN
Downstream readiness for #67
```

## Freshness / Integration Contract

Semantic domains:

- `plugins/generic-ytdlp/**`;
- `gateway-egress/**` / R008;
- verification script/harness files;
- production generic-ytdlp registration/default behavior.

Unrelated planning/docs changes do not invalidate Evidence. Accepted semantic changes in these domains require Coordinator reclassification.

## Out of Scope

- real Bilibili/public-site verification;
- increasing R008 body/timeout limits for site compatibility;
- production generic-ytdlp enablement;
- account/login/Cookie/profile;
- Navigation / Bilibili multipart;
- DASH/remux/transcode;
- performance/capacity claims.

## Completion Protocol

```text
status:ready
→ claim / Attempt N
→ implementation + J1-J4
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

If the required safe harness cannot be built without weakening #60/R008 or exposing sensitive diagnostics:

```text
[BLOCKER REPORT]
→ status:blocked
→ release owner
→ STOP
```

Worker must not execute #67/#68, set status:done, merge own PR or close #73.
