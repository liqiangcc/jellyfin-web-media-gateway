# Task — GENERIC-YTDLP-BILIBILI-REAL

## Metadata

```text
GitHub Issue: #67
Task ID: GENERIC-YTDLP-BILIBILI-REAL
Task kind: verification-only / real public network
Contract Revision: R4
Next Attempt: 4
Exact Execution Candidate: c23b49adbe1cad8a93ff4377dfeba3f12aac7ffe
Preferred worker: ubuntu-arm64
Eligible environment after publication: env:ubuntu-arm64
Accepted extraction upstream: #66 Final Accepted
Accepted harness authority: #73 R2 Final Accepted
Accepted offline runtime authority: #79 Attempt 2 Final Accepted
Accepted ARM64 sandbox authority: #83 Final Accepted
Accepted target environment: #63 Final Accepted
Accepted security/runtime authority: #60 + R008
Downstream: #68 BILIBILI-WEB-E2E
Freshness policy: dependency-aware / exact Candidate
```

> #67 owns only real-site compatibility Evidence for the frozen public Bilibili sample. It does not implement fixes, weaken security, add DASH/remux, enable production generic-ytdlp, or start #68.

## Trigger / why Attempt 4

Attempt 3 made decisive progress over Attempts 1/2:

```text
offline bundle transfer: PASS
repository trust-anchor verification: PASS
offline install: PASS
runtime_cache: offline-prepared
direct public HTTPS: HTTP 200
direct frozen Bilibili page: HTTP 200
```

It then stopped immediately before extractor execution:

```text
result: BLOCKED
process_error: SANDBOX_UNAVAILABLE
broker_request_count: 0
```

The blocker was not Bilibili compatibility and not the offline runtime. The accepted `ytdlp-sandbox` at that Candidate had an x86_64-only seccomp audit-architecture gate while the Target is Linux `aarch64`.

#83 is now Final Accepted and merged, providing target-bound Linux x86_64 + AArch64 seccomp support while preserving the fail-closed security model.

Accepted #83 identities:

```text
Accepted Candidate: a26995dd96d4765185b5c7c428c19ad2b56ba854
Merged main / #67 Attempt-4 Execution Candidate: c23b49adbe1cad8a93ff4377dfeba3f12aac7ffe
PR: #84
Exact-Candidate workflow: 32961265996
x86_64 J1: 98154582893 PASS
AArch64 J2: 98154583068 PASS on ubuntu-24.04-arm
J3: 98154583117 PASS
J4: 98154583090 PASS
```

Accepted security result:

```text
Linux x86_64 -> target-bound AUDIT_ARCH_X86_64
Linux aarch64 -> target-bound AUDIT_ARCH_AARCH64
unsupported target -> compile-time fail closed
no_new_privs -> preserved
seccomp -> preserved
new socket/socketpair -> denied
inherited broker IPC -> usable
R008/BrokerProcessRunner -> unchanged authority
production DisabledRunner -> preserved
```

## Accepted offline runtime identities

#79 remains the immutable runtime authority:

```text
Accepted Candidate: 3a3de8ee2f9ac8b0e1e312735a9305db7569baef
Merged main containing trust anchor: 290268c3cabe5ac16022b1ae5e4fa7716ee5deae
Artifact: yt_dlp-2026.8.19-py3-none-any.whl
Wheel SHA256: 86a521c89017200d7cc20173b9f1d04c6588dda4eabad324b5c76d5269ee1bf9
Trust anchor: scripts/generic-ytdlp-offline-runtime.lock.json
Manifest schema: 1
yt-dlp version: 2026.08.19
Source commit: 3a08beaf031ab68f966401ead017ac81fe8486cf
Platform: py3-none-any verified on hosted Linux x86_64 + ARM64
```

Accepted CI transfer convenience:

```text
workflow run: 32956386626
artifact name: generic-ytdlp-offline-runtime-3a3de8ee2f9ac8b0e1e312735a9305db7569baef
artifact id: 9602124791
```

The Actions artifact ID/name is transport only, not the trust root. Durable trust remains the exact repository lock + wheel SHA + manifest/provenance checks.

Attempt 3 intentionally removed the transferred bundle after execution but retained the verified final user-owned cache. Because the accepted smoke harness still requires `YTDLP_OFFLINE_BUNDLE` even for an `offline-hit`, Attempt 4 must provision the exact #79 bundle again or receive an operator-provided exact local copy. This is transport, not dependency resolution.

## Frozen sample

```text
site: Bilibili
mode: public / no-login / non-DRM
selector: BV14V411W7r5
source: https://www.bilibili.com/video/BV14V411W7r5/
formal site network class: normal direct / no bypass proxy
```

The source URL is Task input. Durable Evidence must not publish full resolved/signed media URLs, query tokens, Cookie, Authorization, profile/account state, artifact-transfer credentials, raw worker stderr, page body, or media payload.

## Goal

Determine whether the accepted generic-ytdlp path can resolve the frozen public Bilibili sample on the accepted Ubuntu ARM64 phone/network to the current first-playback muxed HTTP/HLS `ResolvedMedia` contract.

Required path:

```text
exact #79 offline bundle
→ transfer/provision to accepted Ubuntu ARM64 Target
→ repository lock + wheel SHA verification
→ offline install/reuse in gateway-runner user cache
→ runtime_cache: offline-hit | offline-prepared
→ direct/no-proxy Bilibili reachability
→ scripts/generic-ytdlp-real-smoke.sh
→ accepted ARM64 ytdlp-sandbox
→ BrokerProcessRunner
→ R008Broker
→ frozen yt_dlp.extract_info(download=False)
→ GenericYtdlpAdapter
→ current ResolvedMedia
→ evidence-safe summary only
```

The decisive Attempt-4 signal is:

```text
runtime_cache: offline-hit | offline-prepared
process_error != SANDBOX_UNAVAILABLE
broker_request_count > 0
```

Only after broker traffic occurs may #67 classify actual Bilibili compatibility.

## Exact Candidate

Execute runtime/product code exactly at:

```text
c23b49adbe1cad8a93ff4377dfeba3f12aac7ffe
```

This Candidate contains the accepted #66 extraction path, #73 safe harness, #79 offline runtime/trust anchor, and #83 Linux ARM64 seccomp sandbox support.

Task/prompt documentation is newer than the runtime Candidate by design. Do not substitute moving `main`. If an accepted semantic change touches `plugins/generic-ytdlp/**`, `scripts/generic-ytdlp-*`, `gateway-egress/**`/R008 or material SiteAdapter output authority before claim, stop for Coordinator freshness review.

## Host / environment authority

Use the Final Accepted #63 Ubuntu ARM64 phone environment:

- Linux ARM64/aarch64;
- low-privilege `gateway-runner` uid999, non-root/no-sudo/no-admin;
- Python 3.12, pip, git, curl available;
- user Rust toolchain available;
- direct/no-proxy public HTTPS and frozen Bilibili page previously HTTP 200;
- FFmpeg/Chromium/Node are not required.

No root/sudo/system package installation is permitted.

## J0 — Exact identity + bundle provisioning

Record only bounded safe Evidence:

```text
UTC time
uname -m
uid / privilege class
exact checkout SHA
python3 version
cargo/rustc bounded versions
bundle transfer class
```

Requirements:

1. checkout equals `c23b49adbe1cad8a93ff4377dfeba3f12aac7ffe`;
2. runtime user matches accepted low-privilege Target class;
3. obtain the exact #79 offline bundle without rebuilding/resolving it on Target;
4. permitted transfer shapes:
   - authenticated download of accepted #79 artifact from run `32956386626`, or
   - Coordinator/operator-provided exact local copy;
5. transfer credentials are transport-only and must be removed before extraction;
6. do not use Target-side git/source/package-index resolution to create or replace yt-dlp.

If the exact bundle cannot be placed on Target, report BLOCKED as `OFFLINE_BUNDLE_TRANSFER`.

## J1 — Trust anchor + offline runtime/cache verification

Before real-site extraction:

```text
python3 scripts/generic-ytdlp-offline-runtime.py verify "$YTDLP_OFFLINE_BUNDLE"
```

Then exercise the exact offline install/reuse path through the accepted helper/harness.

Required Evidence:

```text
trust anchor present: yes
expected wheel SHA256: 86a521c89017200d7cc20173b9f1d04c6588dda4eabad324b5c76d5269ee1bf9
bundle verification: PASS
runtime provenance: yt-dlp 2026.08.19 / accepted source commit
runtime cache: offline-hit | offline-prepared
```

A retained Attempt-3 cache should normally produce `offline-hit`; `offline-prepared` remains acceptable only when the exact accepted bundle verifies and a fresh user-owned cache is atomically prepared.

Rules:

- install consumes supplied wheel only;
- no package index/source network for install;
- no global/system yt-dlp fallback;
- lock/manifest/hash/provenance mismatch is BLOCKED;
- no same-version replacement wheel.

## J2 — Direct/no-bypass site reachability

Independently re-confirm formal site Evidence:

- clear upper/lowercase proxy variables for the reachability checks;
- use `curl --noproxy '*'` with bounded timeouts;
- record only public HTTPS status class and frozen Bilibili page status/error class.

No Cookie/Auth/login, proxy rotation, fingerprint spoofing, CAPTCHA automation or access-control bypass.

Artifact-transfer routing is not site Evidence.

## J3 — Accepted real-site smoke on ARM64 sandbox

Run only:

```text
YTDLP_OFFLINE_BUNDLE=<verified-bundle-path> \
  scripts/generic-ytdlp-real-smoke.sh \
  'https://www.bilibili.com/video/BV14V411W7r5/'
```

Do not replace the harness with ad-hoc yt-dlp/Python/Rust/CLI code.

Network/security separation:

```text
Artifact transfer network
!= Offline runtime install
!= Formal Bilibili direct reachability
!= Extractor network authority
```

Before extractor execution the accepted harness scrubs proxy state. Extractor HTTP(S) remains R008Broker + BrokerProcessRunner authority; the worker has no direct socket authority.

Capture only safe fields:

```text
result
plugin
runtime_cache
broker_status_class
broker_error_code
broker_request_count
protocol
stream_count
title_length
process_error
```

Attempt 4 specifically proves the accepted ARM64 sandbox reaches the extractor/broker path. A repeated `SANDBOX_UNAVAILABLE` is BLOCKED and contradicts the expected #83 downstream effect; do not bypass the sandbox.

## J4 — Post-run safety / cleanup

Verify:

- no cache staging directory remains;
- verified final user-owned cache may remain;
- no smoke/worker/sandbox/descendant process remains;
- no media payload/file was downloaded;
- checkout remains exact/unmodified;
- no production Vault/profile/Secret state was touched;
- safe-output scan contains no full resolved URL, signed query, Cookie/Auth/token/account/profile/transfer credential data.

## Result semantics

### PASS

All must hold:

- exact Candidate `c23b49ad...` used;
- accepted #79 bundle/trust anchor/provenance verifies;
- runtime cache is `offline-hit` or `offline-prepared`;
- direct/no-proxy sample is normally reachable;
- ARM64 sandbox starts without `SANDBOX_UNAVAILABLE`;
- harness reaches brokered extraction and returns `result: PASS`;
- `broker_request_count > 0`;
- protocol is `http-file` or `hls`;
- at least one current-contract accepted muxed stream is represented;
- no security/Secret/policy violation;
- J4 PASS.

PASS means the frozen source is compatible with the current generic-ytdlp first-playback resolution contract. It does not prove #68 browser playback.

### CONDITIONAL PASS

Only if brokered extraction produces a valid current `ResolvedMedia` with a bounded non-security condition that still permits an explicit #68 route. Coordinator decides.

### FAIL

Use only after accepted runtime + sandbox + broker path execute correctly and site is normally reachable, but the result cannot be represented by the current first-playback contract, for example stable `UNSUPPORTED_FORMAT` / separate A/V.

Do not implement DASH/remux/FFmpeg inside #67.

### BLOCKED

Examples:

- `OFFLINE_BUNDLE_TRANSFER`;
- `FROZEN_RUNTIME_VERIFY`;
- repeated `SANDBOX_UNAVAILABLE`;
- direct site no longer normally reachable;
- R008 policy/limit prevents compatibility determination;
- safe Evidence cannot be produced.

A bounded broker error such as `BROKER_RESPONSE_TOO_LARGE` must remain a bounded code/count; do not change R008 limits here.

## Claims

```text
R1 — Exact accepted runtime
#79 offline runtime + #83 ARM64 sandbox identities are present in exact Candidate c23b49ad... and execute without fallback.

R2 — Target build independence
Target consumes the locked bundle; no source/package-index resolution.

R3 — Normal-network public accessibility
Frozen Bilibili sample is reachable on accepted direct/no-bypass route independently of artifact transfer.

R4 — ARM64 sandbox + broker integrity
Accepted AArch64 seccomp sandbox starts fail-closed, direct socket creation remains denied, inherited broker IPC remains usable, and extractor HTTP(S) is under R008Broker/BrokerProcessRunner authority.

R5 — Current ResolvedMedia compatibility
Safe result establishes whether the sample maps to current muxed HTTP/HLS first-playback contract.

R6 — Secret/evidence boundary
No Secret, signed media URL, raw page/media payload, proxy/transfer credential or profile/account state enters durable Evidence.

R7 — Cleanup / target safety
No staging/process/media payload persists; verified cache may persist; low-privilege Target boundaries remain unchanged.
```

## Success criteria

1. J0-J4 execute or a concrete bounded blocker is preserved.
2. Exact Candidate, #79 trust anchor/wheel SHA/provenance and #83 sandbox authority are verified.
3. No Target-side source/package-index resolution occurs.
4. Direct/no-proxy Bilibili reachability is separated from artifact transfer.
5. `SANDBOX_UNAVAILABLE` is cleared by the accepted ARM64 sandbox unless a new concrete regression is proven.
6. Brokered traffic reaches R008 (`broker_request_count > 0`) unless a new pre-broker blocker is explicitly classified.
7. Safe result classifies PASS / CONDITIONAL PASS / FAIL / BLOCKED.
8. R1-R7 are explicitly reported.
9. No implementation/security-policy modification occurs.
10. Worker reports, releases ownership and STOPs; it does not execute #68.

## Evidence contract

`[EXECUTION REPORT]` or `[BLOCKER REPORT]` must contain only bounded Evidence:

```text
Attempt / worker / environment
UTC time
Host class / arch / uid privilege class
Exact Candidate SHA
Frozen selector: BV14V411W7r5
#83 accepted Candidate / merged SHA
Offline bundle transfer class
Repository trust-anchor result
Accepted wheel SHA verification
Frozen yt-dlp version/source identity
runtime_cache: offline-hit | offline-prepared | blocked
formal site network class: direct/no-proxy | blocked
Direct public HTTPS status class
Direct Bilibili page status class
ARM64 sandbox result
Harness result
protocol: http-file | hls | n/a
stream_count
safe title length/hash if emitted
broker_status_class
broker_error_code
broker_request_count
process_error
staging/process cleanup result
safe-output leak scan
Claims R1-R7
Overall: PASS | CONDITIONAL PASS | FAIL | BLOCKED
Downstream #68 readiness: yes/no + reason
```

Never publish transfer credentials, full resolved/signed media URLs, signed query parameters, Cookie, Authorization, tokens, profile/account state, setup logs, raw worker stderr, page body, or media payload.

## Freshness

Semantic authorities:

- exact Execution Candidate `c23b49adbe1cad8a93ff4377dfeba3f12aac7ffe`;
- #79 offline-runtime helper + lock;
- `scripts/generic-ytdlp-real-smoke.sh`;
- `plugins/generic-ytdlp/**`, especially `ytdlp-sandbox.rs`;
- `gateway-egress/**` / R008;
- `site-adapter-api/**` only if an accepted change materially alters extraction output/conformance before claim.

#75 Browser work is normally unrelated. Planning/doc-only changes are normally `UNRELATED`. Any accepted semantic change in the authorities above before claim requires Coordinator re-freeze.

## Out of scope

- code changes/fixes;
- sandbox bypass;
- R008 policy/limit weakening;
- Cookie/login/profile/auth/access-control bypass;
- DASH/separate A/V composition/remux/FFmpeg;
- Bilibili navigation/multipart (#72);
- Browser Worker/Native Panel;
- Web Display/control E2E (#68);
- production generic-ytdlp enablement;
- performance/capacity/thermal/soak (#9).

## Completion protocol

```text
status:ready
→ claim / Attempt 4
→ status:in-progress
→ J0-J4
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ status:review or status:blocked
→ release owner
→ STOP
```

Worker cannot set `status:done`, close #67, execute #68, or modify product/security policy.
