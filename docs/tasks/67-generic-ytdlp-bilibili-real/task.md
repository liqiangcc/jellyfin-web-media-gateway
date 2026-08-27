# Task — GENERIC-YTDLP-BILIBILI-REAL

## Metadata

```text
GitHub Issue: #67
Task ID: GENERIC-YTDLP-BILIBILI-REAL
Task kind: verification-only / real public network
Contract Revision: R6
Next Attempt: 6
Exact Execution Candidate: 804fd60343b081e5e055ba87f68e7939b106bb19
Preferred worker: ubuntu-arm64
Eligible environment after publication: env:ubuntu-arm64
Accepted extraction upstream: #66 Final Accepted
Accepted harness authority: #73 R2 Final Accepted
Accepted offline runtime authority: #79 Attempt 2 Final Accepted
Accepted ARM64 sandbox authority: #83 Final Accepted
Accepted legacy-kernel fd isolation authority: #85 Final Accepted / merge 76b2032410b19ee18cfb14f00317b97f84e3b691
Accepted anonymous response Secret containment authority: #95 Final Accepted / merge 804fd60343b081e5e055ba87f68e7939b106bb19
Accepted target environment: #63 Final Accepted
Accepted security/runtime authority: #60 + R008 + ADR 0007
Downstream: #68 BILIBILI-WEB-E2E
Freshness policy: dependency-aware / exact Candidate
```

> #67 owns only real-site compatibility Evidence for the frozen public Bilibili sample. It does not implement fixes, weaken security, add DASH/remux, enable production generic-ytdlp, or start #68.

## Trigger / why Attempt 6

Attempt 5 cleared every previous Target/runtime blocker and reached accepted real R008 broker traffic on the Ubuntu ARM64 target:

```text
runtime_cache: offline-hit
formal Bilibili direct/no-proxy: 2xx
ARM64 sandbox: PASS
close_range syscall: ENOSYS
#85 bounded legacy fd fallback: PASS
BrokerProcessRunner: PASS
broker_request_count: 1 per run
R008 result: 4xx / BROKER_RESPONSE_SECRET_REJECTED
reproduction: 2/2
ResolvedMedia: not reached
```

#95 was split to own that independent response-boundary security/compatibility blocker. #95 is now Final Accepted. Its exact Candidate `0738e1826b17400a92aff483cba4bd37f683e673` passed exact-Candidate J1-J3 in workflow `33061040363`; PR #96 was squash-merged as `804fd60343b081e5e055ba87f68e7939b106bb19`.

Accepted #95 semantics remain frozen:

```text
request Secret material
→ REJECT before prohibited side effects

origin response
→ existing Secret classifier remains authoritative
→ Secret response headers remain Secret
→ consume existing bounded response-header budget
→ CONTAIN before BrokerResponse / broker IPC
→ no cookie/auth store or replay
→ safe status/body/non-Secret headers continue only when all other R008 checks pass
```

`Set-Cookie` and other accepted Secret classes were not declassified. R008 DNS/public-IP/address-pinning/TLS/per-hop redirect/body/frame/time/cancellation authority remains unchanged. Production `GenericYtdlpAdapter::default()` remains disabled.

Attempt 6 resumes the same verification-only real-site Goal on the first exact Candidate containing both #85 and #95. No #67 product/security implementation is authorized.

## Accepted offline runtime identities

#79 remains immutable runtime authority:

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

Artifact transport is not a trust root. Target trust remains exact repository lock + wheel SHA/provenance verification.

## Frozen sample

```text
site: Bilibili
mode: public / no-login / non-DRM
selector: BV14V411W7r5
source: https://www.bilibili.com/video/BV14V411W7r5/
formal site network class: normal direct / no bypass proxy
```

Durable Evidence must not publish full resolved/signed media URLs, query tokens, Cookie, Authorization, profile/account state, artifact-transfer credentials, raw worker stderr, page body, response Secret header names/values, or media payload.

## Goal

Determine whether accepted generic-ytdlp can resolve the frozen public Bilibili sample on the accepted Ubuntu ARM64 phone/network to the current first-playback muxed HTTP/HLS `ResolvedMedia` contract.

Required path:

```text
exact #79 offline bundle
→ Target verify/offline cache hit-or-prepare
→ direct/no-proxy Bilibili reachability
→ scripts/generic-ytdlp-real-smoke.sh
→ accepted ARM64 ytdlp-sandbox
→ BrokerProcessRunner with #85 ENOSYS fallback
→ R008Broker with #95 response Secret containment
→ yt_dlp.extract_info(download=False)
→ GenericYtdlpAdapter
→ current ResolvedMedia
→ evidence-safe summary only
```

Decisive Attempt-6 signal:

```text
runtime_cache: offline-hit | offline-prepared
process_error != SANDBOX_UNAVAILABLE
process_error != SPAWN_FAILED
broker_request_count > 0
broker_error_code != BROKER_RESPONSE_SECRET_REJECTED
```

Only after broker traffic occurs may #67 classify actual Bilibili compatibility.

## Exact Candidate

Execute runtime/product code exactly at:

```text
804fd60343b081e5e055ba87f68e7939b106bb19
```

This Candidate contains accepted #66 extraction, #73 real-site harness, #79 offline runtime/trust anchor, #83 ARM64 sandbox, #85 legacy-kernel fd isolation and #95 anonymous response Secret containment.

Task/prompt documentation may be newer than runtime Candidate by design. Do not substitute moving `main`. If accepted semantic changes touch `plugins/generic-ytdlp/**`, `scripts/generic-ytdlp-*`, `gateway-egress/**` / R008 / ADR 0007, fd-isolation implementation, or material SiteAdapter output authority before claim, STOP for Coordinator freshness review.

## Host / environment authority

Use Final Accepted #63 Ubuntu ARM64 phone environment:

- Linux ARM64/aarch64;
- low-privilege `gateway-runner` uid999, non-root/no-sudo/no-admin;
- Python 3.12, pip, git, curl and user Rust toolchain available;
- direct/no-proxy public HTTPS and frozen Bilibili page previously HTTP 200;
- Linux kernel expected to report `close_range=ENOSYS` and must use accepted #85 fallback without security weakening.

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

1. checkout equals `804fd60343b081e5e055ba87f68e7939b106bb19`;
2. runtime user matches accepted low-privilege Target class;
3. obtain exact #79 offline bundle without rebuilding/resolving it on Target;
4. permitted transfer is authenticated download of accepted #79 artifact or Coordinator/operator-provided exact local copy;
5. transfer credentials are transport-only and removed before extraction;
6. no Target-side source/package-index resolution may create/replace yt-dlp.

If direct Target Git checkout is unreliable, accepted trusted exact-source transport from #90/#85 may be used, provided exact SHA/tree identity is verified locally before execution. Transport choice is not site Evidence.

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

## J3 — Accepted real-site smoke with #95 response containment

Run only:

```text
YTDLP_OFFLINE_BUNDLE=<verified-bundle-path> \
  scripts/generic-ytdlp-real-smoke.sh \
  'https://www.bilibili.com/video/BV14V411W7r5/'
```

Do not replace harness with ad-hoc yt-dlp/Python/Rust/CLI code.

Before extractor execution, accepted harness scrubs proxy state. Extractor HTTP(S) remains R008Broker + BrokerProcessRunner authority; worker has no direct socket authority.

Attempt 6 must confirm:

```text
close_range_syscall: ENOSYS (when probed/reported)
process_error != SPAWN_FAILED
broker_request_count > 0
broker_error_code != BROKER_RESPONSE_SECRET_REJECTED
```

Do not publish or infer concrete real response Secret header names/values. #95 deterministic fixtures own policy proof; #67 only verifies accepted containment on the real flow.

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

Repeated `SANDBOX_UNAVAILABLE`, `SPAWN_FAILED`, or `BROKER_RESPONSE_SECRET_REJECTED` on exact R6 Candidate is BLOCKED and must not be bypassed or fixed inside #67.

## J4 — Post-run safety / cleanup

Verify:

- no cache staging directory remains;
- verified final user-owned cache may remain;
- no smoke/worker/sandbox/descendant process remains;
- no media payload/file was downloaded;
- checkout remains exact/unmodified apart from explicit task-owned evidence files if transport workflow produces them;
- no production Vault/profile/Secret state was touched;
- safe-output scan contains no full resolved URL, signed query, Cookie/Auth/token/account/profile/transfer credential or response Secret material.

## Result semantics

### PASS

All must hold:

- exact Candidate `804fd60343b081e5e055ba87f68e7939b106bb19` used;
- #79 bundle/trust anchor/provenance verifies;
- runtime cache is `offline-hit` or `offline-prepared`;
- direct/no-proxy sample normally reachable;
- ARM64 sandbox starts without `SANDBOX_UNAVAILABLE`;
- #85 fallback clears former `SPAWN_FAILED`;
- #95 containment clears former whole-response `BROKER_RESPONSE_SECRET_REJECTED` without Secret leakage or cookie/auth replay;
- harness reaches brokered extraction and returns `result: PASS`;
- `broker_request_count > 0`;
- protocol is `http-file` or `hls`;
- at least one current-contract accepted muxed stream is represented;
- no security/Secret/policy violation;
- J4 PASS.

PASS means frozen source is compatible with current generic-ytdlp first-playback resolution contract. It does not prove #68 browser playback.

### CONDITIONAL PASS

Only if brokered extraction produces valid current `ResolvedMedia` with a bounded non-security condition that still permits explicit #68 route. Coordinator decides.

### FAIL

Use only after accepted runtime + sandbox + BrokerProcessRunner/R008 path execute correctly and site is normally reachable, but result cannot be represented by current first-playback contract, e.g. stable `UNSUPPORTED_FORMAT` / separate A/V.

Do not implement DASH/remux/FFmpeg inside #67.

### BLOCKED

Examples:

- `OFFLINE_BUNDLE_TRANSFER`;
- `FROZEN_RUNTIME_VERIFY`;
- repeated `SANDBOX_UNAVAILABLE`;
- repeated `SPAWN_FAILED` despite #85;
- repeated `BROKER_RESPONSE_SECRET_REJECTED` or another concrete R008 policy/limit condition after #95;
- direct site no longer normally reachable;
- safe Evidence cannot be produced.

A bounded broker error such as `BROKER_RESPONSE_TOO_LARGE` remains a bounded code/count; do not change R008 limits here.

## Claims

```text
R1 — Exact accepted runtime
#79 offline runtime + #83 ARM64 sandbox + #85 fd-isolation + #95 response Secret containment authorities are present in exact Candidate 804fd603... and execute without fallback outside accepted semantics.

R2 — Target build/dependency independence
Target consumes locked/accepted inputs; no source/package-index resolution.

R3 — Normal-network public accessibility
Frozen Bilibili sample is reachable on accepted direct/no-bypass route independently of artifact/source transfer.

R4 — ARM64 sandbox + broker integrity on Linux 4.19
Accepted AArch64 seccomp sandbox starts fail-closed, close_range ENOSYS uses accepted bounded fd fallback, direct socket creation remains denied, broker fd 3/inherited IPC remains usable, and extractor HTTP(S) is under R008Broker/BrokerProcessRunner authority.

R5 — R008 response Secret containment on real flow
Real origin response Secret material remains contained before broker IPC with no disclosure/store/replay, and otherwise safe public response material can continue according to accepted #95 semantics.

R6 — Current ResolvedMedia compatibility
Safe result establishes whether sample maps to current muxed HTTP/HLS first-playback contract.

R7 — Secret/evidence boundary
No Secret, signed media URL, raw page/media payload, proxy/transfer credential, response Secret material or profile/account state enters durable Evidence.

R8 — Cleanup / target safety
No staging/process/media payload persists; verified cache may persist; low-privilege Target boundaries remain unchanged.
```

## Success criteria

1. J0-J4 execute or a concrete bounded blocker is preserved.
2. Exact Candidate, #79 trust anchor/wheel SHA/provenance, #83 sandbox, #85 fd isolation and #95 containment authorities are verified.
3. No Target-side source/package-index resolution occurs.
4. Direct/no-proxy Bilibili reachability is separated from artifact/source transfer.
5. `SANDBOX_UNAVAILABLE` remains cleared.
6. Former Linux 4.19 `SPAWN_FAILED` remains cleared unless a new concrete regression is proven.
7. Brokered traffic reaches R008 (`broker_request_count > 0`) unless a new pre-broker blocker is explicitly classified.
8. Former whole-response `BROKER_RESPONSE_SECRET_REJECTED` is cleared or a new concrete bounded R008 blocker is reported without security weakening.
9. Safe result classifies PASS / CONDITIONAL PASS / FAIL / BLOCKED.
10. R1-R8 are explicitly reported.
11. No implementation/security-policy modification occurs.
12. Worker reports, releases ownership and STOPs; it does not execute #68.

## Evidence contract

`[EXECUTION REPORT]` or `[BLOCKER REPORT]` must contain only bounded Evidence:

```text
Attempt / worker / environment
UTC time
Host class / arch / kernel / uid privilege class
Exact Candidate SHA
Frozen selector: BV14V411W7r5
#85 accepted Candidate / merge SHA
#95 accepted Candidate / merge SHA
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
R008 response-containment result: accepted-path | bounded-blocker
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
Claims R1-R8
Overall: PASS | CONDITIONAL PASS | FAIL | BLOCKED
Downstream #68 readiness: yes/no + reason
```

Never publish transfer credentials, response Secret header names/values, full resolved/signed media URLs, signed query parameters, Cookie, Authorization, tokens, profile/account state, setup logs, raw worker stderr, page body, or media payload.

## Freshness

Semantic authorities:

- exact Execution Candidate `804fd60343b081e5e055ba87f68e7939b106bb19`;
- #79 offline-runtime helper + lock;
- `scripts/generic-ytdlp-real-smoke.sh`;
- `plugins/generic-ytdlp/**`, especially sandbox and BrokerProcessRunner/fd-isolation logic;
- `gateway-egress/**` / R008 / ADR 0007 response Secret containment;
- `site-adapter-api/**` only if accepted changes materially alter extraction output/conformance before claim.

Planning/doc-only changes and #90/#92 diagnostic infrastructure are normally `UNRELATED`. Any accepted semantic change in authorities above before claim requires Coordinator re-freeze.

## Out of scope

- repository/product/security code changes;
- sandbox or fd-isolation bypass;
- R008 policy/limit weakening;
- response Secret declassification or response-cookie/auth replay;
- Cookie/login/profile/auth/access-control bypass;
- DASH/separate A/V composition/remux/FFmpeg;
- Browser/Web E2E/performance work;
- starting #68.
