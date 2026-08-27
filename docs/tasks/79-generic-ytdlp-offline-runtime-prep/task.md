# Task — GENERIC-YTDLP-OFFLINE-RUNTIME-PREP

## Metadata

```text
GitHub Issue: #79
Task ID: GENERIC-YTDLP-OFFLINE-RUNTIME-PREP
Task kind: implementation + supply-chain artifact verification
Planning Base: e8d292ebaa58e66f8ad737c9ddb643b9d8aacfaf
Preferred worker: cloud-codex
Eligible environment: env:cloud
Accepted upstream: #73 R2 Final Accepted; #60/#66 runtime/security authority
Downstream: #67 Attempt 3
Freshness policy: dependency-aware
```

> #79 owns only the reproducible offline distribution/verification boundary for the frozen generic-ytdlp runtime. It does not execute Bilibili, change R008 policy, enable production generic-ytdlp, or implement media-format fixes.

## Trigger

#67 Attempt 2 again proved the public-site route was reachable while the real extractor never started:

```text
direct public HTTPS: HTTP 200
direct Bilibili page: HTTP 200
result: BLOCKED
process_error: FROZEN_RUNTIME_SETUP
broker_request_count: 0
```

#73 R2 reduced repeated setup work with a user-owned cache, but a cold target still has to perform `pip git+https` acquisition. That supply-chain dependency remains too sensitive to target network/proxy/TLS/git/pip behavior and prevents environment-consistent Evidence.

## Goal

Move frozen yt-dlp acquisition/build out of Target execution and into a repository-owned supply-chain build:

```text
frozen source identity
2026.08.19 @ 3a08beaf031ab68f966401ead017ac81fe8486cf
        ↓
GitHub Actions supply-chain build
        ↓
immutable offline runtime bundle
+ manifest
+ SHA256SUMS
        ↓
verify the same bundle on hosted Linux x86_64
and hosted Linux ARM64
        ↓
Target receives bundle by transfer/download
        ↓
non-root verify + offline install/reuse
        ↓
extractor runtime starts with setup network unavailable
        ↓
R008Broker remains sole extractor HTTP(S) authority
```

Normal Target verification must not resolve/build the frozen dependency from source.

## Artifact contract

Prefer a platform-independent Python wheel/bundle if the frozen source actually supports it. The Worker must prove this rather than assume it.

Canonical bundle shape should be equivalent to:

```text
generic-ytdlp-offline-runtime/
├── artifacts/
│   └── <frozen runtime artifact>
├── manifest.json
└── SHA256SUMS
```

`manifest.json` must contain only bounded repository-owned identity, including at least:

```text
schema_version
runtime_name
yt_dlp_version
source_commit
artifact_filename
artifact_sha256
artifact_format
python_compatibility
platform_compatibility
build_candidate_sha
```

Rules:

- exact yt-dlp version remains `2026.08.19`;
- exact source commit remains `3a08beaf031ab68f966401ead017ac81fe8486cf`;
- no caller-selected source URL/ref/version;
- no secrets, proxy values, user paths or machine-specific temp paths in manifest;
- artifact content is immutable and SHA256-addressed;
- manifest/hash/provenance mismatch fails closed;
- no global/system yt-dlp fallback.

If the frozen source cannot produce one architecture-neutral artifact, create an explicit supported architecture matrix instead of silently producing host-specific output. At minimum the current product route requires Linux `x86_64` and `aarch64` verification.

## Target-side offline install contract

Add the smallest repository-owned install/verify path that consumes an already-supplied bundle.

Required semantics:

```text
bundle path
→ validate manifest schema/fixed identity
→ validate SHA256
→ verify artifact format/compatibility
→ install into staging under user-owned cache
→ verify imported yt_dlp version/provenance/location
→ atomic promotion
→ warm reuse
```

Requirements:

- non-root only;
- user-owned destination only;
- setup/install must succeed with `HTTP_PROXY/HTTPS_PROXY/ALL_PROXY` unusable and package indexes/network unavailable;
- use offline pip semantics such as `--no-index --no-deps` when pip is the installer;
- no system package installation;
- no writable global Python environment;
- corrupt/partial/mismatched bundle or cache fails closed and does not leave a promoted runtime;
- staging cleanup is deterministic;
- already verified cache is reusable without network;
- bundle may be transferred/downloaded before the install step, but install/verification itself has zero supply-chain network authority.

The existing real-site smoke entrypoint may be refined so it prefers/accepts the repository-owned verified offline runtime source. It must not re-enable arbitrary caller executable/source/version authority.

## GitHub Actions verification

Use GitHub-hosted Linux runners to prove environment consistency.

### J1 — Build canonical frozen bundle

On a hosted Linux build runner:

- acquire only the repository-fixed source commit;
- build the offline artifact/bundle;
- create manifest + SHA256SUMS;
- verify exact version/commit provenance;
- assert deterministic/bounded artifact contract;
- expose the bundle as CI artifact for downstream jobs without rebuilding it separately per architecture.

A single build artifact must be consumed by validation jobs unless architecture-specific output is explicitly justified by Evidence.

### J2 — x86_64 offline consume

On GitHub-hosted Linux x86_64:

- download/use the J1 artifact only;
- disable package-index/setup-network access;
- install into a fresh non-system user-owned/temp destination using the offline path;
- verify version/provenance/import location;
- verify warm reuse;
- run affected generic-ytdlp deterministic parser/runtime tests without real-site traffic.

### J3 — ARM64 offline consume

On GitHub-hosted Linux ARM64:

- use the **same J1 bundle** when the artifact is declared architecture neutral, otherwise use the explicit verified aarch64 member of the artifact matrix;
- disable setup/package-index network;
- perform the same offline install/provenance/import verification;
- run the same affected generic-ytdlp deterministic tests that are valid on ARM64;
- prove no target-side source build/git/pip network resolver is required.

Do not report this as phone performance/capacity Evidence.

### J4 — security/lifecycle/regressions

Exact Candidate:

- existing #60/#66/#73 R008/broker/sandbox/secret/lifecycle regressions PASS;
- production `GenericYtdlpAdapter::default()` remains `DisabledRunner`;
- setup proxy/site proxy values cannot enter artifact metadata or extractor runtime;
- corrupted hash/manifest/artifact tests fail closed;
- interrupted install leaves no promoted partial cache;
- workspace fmt/clippy/tests and architecture guards PASS.

Every required job must assert the exact Candidate SHA.

## Distribution boundary

This Task must make the artifact contract durable and reproducible. For CI Evidence, GitHub Actions artifacts are acceptable as the transfer mechanism between J1/J2/J3 jobs.

For downstream Target use, expose a deterministic repository-owned way to obtain or provide the exact bundle without rebuilding it on the Target. Acceptable implementation shapes include:

- a release asset with manifest/hash identity;
- a Coordinator-materialized artifact from the accepted exact Candidate;
- another immutable repository-owned distribution path with equivalent provenance.

Do not make #67 depend on an expiring opaque artifact ID without a durable identity/hash contract.

## Claims

```text
O1 — Frozen immutable artifact
One repository-owned artifact contract represents exact yt-dlp 2026.08.19 / frozen commit and is hash-addressed.

O2 — Target build independence
Normal Target setup does not invoke git or network dependency resolution/build from source.

O3 — Offline deterministic install
A fresh user-owned runtime can be installed and verified with package/setup network unavailable.

O4 — Cross-architecture consistency
The same declared artifact identity is verified on hosted Linux x86_64 and ARM64, or an explicit justified architecture matrix is verified.

O5 — Provenance fail-closed
Manifest/hash/version/commit/location/ownership mismatch or interruption cannot promote an invalid runtime.

O6 — Runtime security unchanged
R008Broker/BrokerProcessRunner and production DisabledRunner boundaries remain unchanged.

O7 — Safe distribution
Artifact metadata contains no Secret/proxy/user-path/site payload and downstream Target can consume a durable immutable identity.
```

## Success criteria

1. O1-O7 PASS on one exact Candidate.
2. One canonical immutable bundle contract exists with manifest + SHA256.
3. Hosted x86_64 offline consume PASS.
4. Hosted ARM64 offline consume PASS.
5. Target-side install path demonstrably requires no setup network/source build.
6. Existing generic-ytdlp/R008/security/lifecycle regressions PASS.
7. No real Bilibili request occurs.
8. Worker reports and STOPs; it does not execute #67 Attempt 3.

## Evidence contract

`[EXECUTION REPORT]` must include bounded Evidence:

```text
Attempt / worker / environment
Base SHA
Candidate SHA / PR
artifact format
artifact filename or bounded artifact identity
artifact SHA256
manifest schema/version/source commit
platform-neutral yes/no; if no, explicit matrix
J1 build result
J2 x86_64 offline consume result
J3 ARM64 offline consume result
network-disabled install proof
corrupt/mismatch/interruption fail-closed proof
production DisabledRunner result
R008/security/lifecycle regression result
O1-O7
freshness classification
unverified/out-of-scope
```

Do not post raw dependency setup logs containing URLs/credentials, proxy values, user filesystem paths, source/resolved media URLs, Cookie/Auth/token or site payload.

## Out of scope

- real Bilibili/site extraction;
- Cookie/login/profile/auth;
- R008 limit/policy weakening;
- DASH/remux/FFmpeg;
- #68 Web E2E;
- #72 Bilibili Navigation;
- Browser/Native Panel/Auth;
- phone CPU/RSS/thermal/performance;
- production generic-ytdlp enablement.

## Completion protocol

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ J1-J4
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ status:review or status:blocked
→ release owner
→ STOP
```

Worker cannot set `status:done`, close #79, execute #67, or weaken security/runtime authority.
