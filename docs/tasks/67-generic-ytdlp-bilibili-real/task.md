# Task — GENERIC-YTDLP-BILIBILI-REAL

## Metadata

```text
GitHub Issue: #67
Task ID: GENERIC-YTDLP-BILIBILI-REAL
Task kind: verification-only / real public network
Contract Revision: R5
Next Attempt: 5
Exact Execution Candidate: 76b2032410b19ee18cfb14f00317b97f84e3b691
Preferred worker: ubuntu-arm64
Eligible environment after publication: env:ubuntu-arm64
Accepted extraction upstream: #66 Final Accepted
Accepted harness authority: #73 R2 Final Accepted
Accepted offline runtime authority: #79 Attempt 2 Final Accepted
Accepted ARM64 sandbox authority: #83 Final Accepted
Accepted legacy-kernel fd isolation authority: #85 Final Accepted / merge 76b2032410b19ee18cfb14f00317b97f84e3b691
Accepted target environment: #63 Final Accepted
Accepted security/runtime authority: #60 + R008
Downstream: #68 BILIBILI-WEB-E2E
Freshness policy: dependency-aware / exact Candidate
```

> #67 owns only real-site compatibility Evidence for the frozen public Bilibili sample. It does not implement fixes, weaken security, add DASH/remux, enable production generic-ytdlp, or start #68.

## Trigger / why Attempt 5

Attempt 4 reached the accepted Ubuntu ARM64 Target with the exact #79 offline runtime and accepted ARM64 sandbox, then exposed a generic runtime portability blocker before broker traffic:

```text
runtime_cache: offline-hit
formal Bilibili direct/no-proxy: 2xx
ARM64 ytdlp-sandbox: built
process_error: SPAWN_FAILED
broker_request_count: 0
close_range_syscall: ENOSYS
kernel: 4.19.113-964403 / aarch64
```

The blocker was not Bilibili compatibility. #85 has now been Final Accepted and merged. Its exact real-target proof established on Linux `4.19.113-964403` / aarch64:

```text
modern Linux -> close_range fast path preserved
close_range == ENOSYS -> bounded fail-closed legacy close path
retain stdio + broker fd 3
close ambient fd >=4
non-ENOSYS close_range errors remain fail-closed
BrokerProcessRunner target proof: PASS
fd-isolation focused tests: 2/2 PASS
runtime integration tests: 12/12 PASS
```

Accepted #85 identities:

```text
Task Candidate: 9b874c3b4404a776da35fd37d37abe040fb06a2b
Merged main / #67 Attempt-5 Execution Candidate: 76b2032410b19ee18cfb14f00317b97f84e3b691
PR: #86
workflow: 33045463590
J1 x86_64: 98428559107 PASS
J2 AArch64: 98428559073 PASS
J3 forced ENOSYS: 98428559066 PASS
J4 Linux 4.19 ARM64: 98428782004 PASS
```

Attempt 5 resumes the same verification-only real-site Goal on a re-frozen Candidate containing the accepted #85 legacy-kernel fix. No #67 product/security implementation is authorized.

## Accepted offline runtime identities

#79 remains the immutable runtime authority:

```text
Accepted Candidate: 3a3de8ee2f9ac8b0e1e312735a9305db7569baef
Artifact: yt_dlp-2026.8.19-py3-none-any.whl
Wheel SHA256: 86a521c89017200d7cc20173b9f1d04c6588dda4eabad324b5c76d5269ee1bf9
Trust anchor: scripts/generic-ytdlp-offline-runtime.lock.json
Manifest schema: 1
yt-dlp version: 2026.08.19
Source commit: 3a08beaf031ab68f966401ead017ac81fe8486cf
CI transport: run 32956386626 / artifact 9602124791
```

Artifact transport is not a trust root. Target trust remains the exact repository lock + wheel SHA/provenance verification.

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
→ Target verify/offline cache hit-or-prepare
→ direct/no-proxy Bilibili reachability
→ scripts/generic-ytdlp-real-smoke.sh
→ accepted ARM64 ytdlp-sandbox
→ BrokerProcessRunner with accepted #85 ENOSYS fallback
→ R008Broker
→ yt_dlp.extract_info(download=False)
→ GenericYtdlpAdapter
→ current ResolvedMedia
→ evidence-safe summary only
```

Decisive Attempt-5 signal:

```text
runtime_cache: offline-hit | offline-prepared
process_error != SANDBOX_UNAVAILABLE
process_error != SPAWN_FAILED
broker_request_count > 0
```

Only after broker traffic occurs may #67 classify actual Bilibili compatibility.

## Exact Candidate

Execute runtime/product code exactly at:

```text
76b2032410b19ee18cfb14f00317b97f84e3b691
```

This Candidate contains the accepted #66 extraction path, #73 real-site harness, #79 offline runtime/trust anchor, #83 ARM64 sandbox and #85 legacy-kernel fd-isolation support.

Task/prompt documentation may be newer than the runtime Candidate by design. Do not substitute moving `main`. If an accepted semantic change touches `plugins/generic-ytdlp/**`, `scripts/generic-ytdlp-*`, `gateway-egress/**`/R008, the fd-isolation implementation, or material SiteAdapter output authority before claim, STOP for Coordinator freshness review.

## Host / environment authority

Use the Final Accepted #63 Ubuntu ARM64 phone environment:

- Linux ARM64/aarch64;
- low-privilege `gateway-runner` uid999, non-root/no-sudo/no-admin;
- Python 3.12, pip, git, curl and user Rust toolchain available;
- direct/no-proxy public HTTPS and frozen Bilibili page previously HTTP 200;
- Linux kernel is expected to report `close_range=ENOSYS` and must use the accepted #85 fallback without security weakening.

No root/sudo/system package installation is permitted.

## J0 — Exact identity + bundle provisioning

Record bounded safe Evidence:

```text
UTC time
uname -m
kernel
uid / privilege class
exact checkout SHA
python3 version
cargo/rustc bounded versions
bundle transfer class
```

Requirements:

1. checkout equals `76b2032410b19ee18cfb14f00317b97f84e3b691`;
2. runtime user matches accepted low-privilege Target class;
3. obtain the exact #79 offline bundle without rebuilding/resolving it on Target;
4. permitted transfer shapes are authenticated download of accepted #79 artifact or Coordinator/operator-provided exact local copy;
5. transfer credentials are transport-only and removed before extraction;
6. no Target-side source/package-index resolution may create/replace yt-dlp.

If direct target Git checkout is unreliable, a trusted exact-Candidate source-bundle transport already accepted by #90/#85 may be used, provided exact SHA/tree identity is verified locally before execution. Transport choice is not site Evidence.

## J1 — Trust anchor + offline runtime/cache verification

Before real-site extraction:

```text
python3 scripts/generic-ytdlp-offline-runtime.py verify "$YTDLP_OFFLINE_BUNDLE"
```

Required Evidence:

```text
trust anchor present: yes
expected wheel SHA256: 86a521c89017200d7cc20173b9f1d04c6588dda4eabad324b5c76d5269ee1bf9
bundle verification: PASS
runtime provenance: yt-dlp 2026.08.19 / accepted source commit
runtime cache: offline-hit | offline-prepared
```

No package index/source network, global/system yt-dlp fallback, same-version replacement wheel, or provenance mismatch is allowed.

## J2 — Direct/no-bypass site reachability

Independently re-confirm formal site Evidence:

- clear upper/lowercase proxy variables for reachability checks;
- use `curl --noproxy '*'` with bounded timeouts;
- record only public HTTPS status class and frozen Bilibili page status/error class.

No Cookie/Auth/login, proxy rotation, fingerprint spoofing, CAPTCHA automation or access-control bypass. Artifact/source-bundle transfer routing is not site Evidence.

## J3 — Accepted real-site smoke on ARM64 sandbox + legacy kernel

Run only:

```text
YTDLP_OFFLINE_BUNDLE=<verified-bundle-path> \
  scripts/generic-ytdlp-real-smoke.sh \
  'https://www.bilibili.com/video/BV14V411W7r5/'
```

Do not replace the harness with ad-hoc yt-dlp/Python/Rust/CLI code.

Before extractor execution the accepted harness scrubs proxy state. Extractor HTTP(S) remains R008Broker + BrokerProcessRunner authority; the worker has no direct socket authority.

Attempt 5 must confirm the #85 downstream effect on the real path:

```text
close_range_syscall: ENOSYS (when probed/reported)
process_error != SPAWN_FAILED
broker_request_count > 0
```

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

A repeated `SANDBOX_UNAVAILABLE` or `SPAWN_FAILED` is BLOCKED and must not be bypassed.

## J4 — Post-run safety / cleanup

Verify:

- no cache staging directory remains;
- verified final user-owned cache may remain;
- no smoke/worker/sandbox/descendant process remains;
- no media payload/file was downloaded;
- checkout remains exact/unmodified apart from explicit task-owned evidence files if the transport workflow produces them;
- no production Vault/profile/Secret state was touched;
- safe-output scan contains no full resolved URL, signed query, Cookie/Auth/token/account/profile/transfer credential data.

## Result semantics

### PASS

All must hold:

- exact Candidate `76b2032410b19ee18cfb14f00317b97f84e3b691` used;
- accepted #79 bundle/trust anchor/provenance verifies;
- runtime cache is `offline-hit` or `offline-prepared`;
- direct/no-proxy sample is normally reachable;
- ARM64 sandbox starts without `SANDBOX_UNAVAILABLE`;
- accepted #85 legacy-kernel fallback clears the former `SPAWN_FAILED` blocker;
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

Use only after accepted runtime + sandbox + BrokerProcessRunner/R008 path execute correctly and site is normally reachable, but the result cannot be represented by the current first-playback contract, for example stable `UNSUPPORTED_FORMAT` / separate A/V.

Do not implement DASH/remux/FFmpeg inside #67.

### BLOCKED

Examples:

- `OFFLINE_BUNDLE_TRANSFER`;
- `FROZEN_RUNTIME_VERIFY`;
- repeated `SANDBOX_UNAVAILABLE`;
- repeated `SPAWN_FAILED` despite the accepted #85 Candidate;
- direct site no longer normally reachable;
- R008 policy/limit prevents compatibility determination;
- safe Evidence cannot be produced.

A bounded broker error such as `BROKER_RESPONSE_TOO_LARGE` must remain a bounded code/count; do not change R008 limits here.

## Claims

```text
R1 — Exact accepted runtime
#79 offline runtime + #83 ARM64 sandbox + #85 legacy fd-isolation authority are present in exact Candidate 76b203... and execute without fallback outside accepted semantics.

R2 — Target build/dependency independence
Target consumes locked/accepted inputs; no source/package-index resolution.

R3 — Normal-network public accessibility
Frozen Bilibili sample is reachable on accepted direct/no-bypass route independently of artifact/source transfer.

R4 — ARM64 sandbox + broker integrity on Linux 4.19
Accepted AArch64 seccomp sandbox starts fail-closed, close_range ENOSYS uses the accepted bounded fd fallback, direct socket creation remains denied, broker fd 3/inherited IPC remains usable, and extractor HTTP(S) is under R008Broker/BrokerProcessRunner authority.

R5 — Current ResolvedMedia compatibility
Safe result establishes whether the sample maps to current muxed HTTP/HLS first-playback contract.

R6 — Secret/evidence boundary
No Secret, signed media URL, raw page/media payload, proxy/transfer credential or profile/account state enters durable Evidence.

R7 — Cleanup / target safety
No staging/process/media payload persists; verified cache may persist; low-privilege Target boundaries remain unchanged.
```

## Success criteria

1. J0-J4 execute or a concrete bounded blocker is preserved.
2. Exact Candidate, #79 trust anchor/wheel SHA/provenance, #83 sandbox authority and #85 legacy-kernel authority are verified.
3. No Target-side source/package-index resolution occurs.
4. Direct/no-proxy Bilibili reachability is separated from artifact/source transfer.
5. `SANDBOX_UNAVAILABLE` remains cleared.
6. Former Linux 4.19 `SPAWN_FAILED` is cleared by accepted #85 behavior unless a new concrete regression is proven.
7. Brokered traffic reaches R008 (`broker_request_count > 0`) unless a new pre-broker blocker is explicitly classified.
8. Safe result classifies PASS / CONDITIONAL PASS / FAIL / BLOCKED.
9. R1-R7 are explicitly reported.
10. No implementation/security-policy modification occurs.
11. Worker reports, releases ownership and STOPs; it does not execute #68.

## Evidence contract

`[EXECUTION REPORT]` or `[BLOCKER REPORT]` must contain only bounded Evidence:

```text
Attempt / worker / environment
UTC time
Host class / arch / kernel / uid privilege class
Exact Candidate SHA
Frozen selector: BV14V411W7r5
#85 accepted Candidate / merge SHA
Offline bundle transfer class
Repository trust-anchor result
Accepted wheel SHA verification
Frozen yt-dlp version/source identity
runtime_cache: offline-hit | offline-prepared | blocked
formal site network class: direct/no-proxy | blocked
Direct public HTTPS status class
Direct Bilibili page status class
ARM64 sandbox result
close_range / legacy-fd-isolation result
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

- exact Execution Candidate `76b2032410b19ee18cfb14f00317b97f84e3b691`;
- #79 offline-runtime helper + lock;
- `scripts/generic-ytdlp-real-smoke.sh`;
- `plugins/generic-ytdlp/**`, especially sandbox and BrokerProcessRunner/fd-isolation logic;
- `gateway-egress/**` / R008;
- `site-adapter-api/**` only if an accepted change materially alters extraction output/conformance before claim.

Planning/doc-only changes and #90/#92 diagnostic infrastructure are normally `UNRELATED`. Any accepted semantic change in authorities above before claim requires Coordinator re-freeze.

## Out of scope

- repository/product/security code changes;
- sandbox or fd-isolation bypass;
- R008 policy/limit weakening;
- Cookie/login/profile/auth/access-control bypass;
- DASH/separate A/V composition/remux/FFmpeg;
- Browser/Web E2E/performance work;
- starting #68.
