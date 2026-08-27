# Task — GENERIC-YTDLP-BILIBILI-REAL

## Metadata

```text
GitHub Issue: #67
Task ID: GENERIC-YTDLP-BILIBILI-REAL
Task kind: verification-only / real public network
Contract Revision: R7
Next Attempt: 7
Exact Execution Candidate: d9c038547ed2df695571f8dd4f732bdcdd4d5c19
Preferred worker: ubuntu-arm64
Eligible environment after publication: env:ubuntu-arm64
Accepted extraction upstream: #66 Final Accepted
Accepted harness authority: #73 R2 Final Accepted
Accepted offline runtime authority: #79 Attempt 2 Final Accepted
Accepted ARM64 sandbox authority: #83 Final Accepted
Accepted legacy-kernel fd-isolation authority: #85 Final Accepted / merge 76b2032410b19ee18cfb14f00317b97f84e3b691
Accepted anonymous response Secret containment authority: #95 Final Accepted / merge 804fd60343b081e5e055ba87f68e7939b106bb19
Accepted broker IPC/wire authority: #97 Final Accepted / merge d9c038547ed2df695571f8dd4f732bdcdd4d5c19
Accepted target environment: #63 Final Accepted
Accepted security/runtime authority: #60 + R008 + ADR 0007
Downstream: #68 BILIBILI-WEB-E2E
Freshness policy: dependency-aware / exact Candidate
Publication state: status:draft until Coordinator Publication Gate passes
Formal draft freeze: issue must be status:draft before this revision is eligible for publication
```

> #67 owns only real-site compatibility Evidence for the frozen public Bilibili sample. It does not implement fixes, weaken security, add DASH/remux, enable production generic-ytdlp, or start #68.

## Trigger / why Attempt 7

Attempt 6 executed the accepted real path on the Ubuntu ARM64 target and cleared all earlier blockers through the R008 response boundary:

```text
runtime_cache: offline-hit
formal Bilibili direct/no-proxy: 2xx
ARM64 sandbox: PASS
close_range syscall: ENOSYS
#85 bounded legacy fd fallback: PASS
#95 response containment: accepted-path
broker_status_class: 2xx
broker_error_code: n/a
broker_request_count: 1
process_error: BROKER_PROTOCOL
reproduction: 2/2
ResolvedMedia: not reached
```

#97 was split to own that independent post-R008 broker wire/framing blocker. #97 is now Final Accepted. Its reviewed Candidate `21bff2d30ffbd4ba1436e1fcf9df5a70a6d7acb4` passed exact-Candidate J1-J4 in workflow `33081472601`; PR #98 was squash-merged as `d9c038547ed2df695571f8dd4f732bdcdd4d5c19`.

Accepted #97 semantics remain frozen:

```text
R008-accepted bounded BrokerResponse
→ bounded fixed-width hex body wire representation
→ fixed derived IPC payload envelope
→ inherited fd 3
→ Python worker bounded decode
→ extractor continuation
```

The old decimal `Vec<u8>` JSON-array framing overflow is fixed. R008 HTTP body/header/count/value limits are unchanged; malformed/zero/truncated/oversize framing remains fail-closed; #95 Secret containment, request Secret rejection, #83 sandbox, #85 fd isolation, no-direct-egress and production `DisabledRunner` remain intact.

Attempt 7 resumes the same verification-only real-site Goal on the first exact Candidate containing #97. No #67 product/security implementation is authorized.

## Frozen runtime identity

#79 remains immutable runtime authority:

```text
yt-dlp version: 2026.08.19
Source commit: 3a08beaf031ab68f966401ead017ac81fe8486cf
Wheel SHA256: 86a521c89017200d7cc20173b9f1d04c6588dda4eabad324b5c76d5269ee1bf9
Trust anchor: scripts/generic-ytdlp-offline-runtime.lock.json
Accepted artifact workflow: 32956386626
Accepted artifact: 9602124791
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
→ BrokerProcessRunner + #85 ENOSYS fd fallback
→ R008Broker + #95 response Secret containment
→ #97 bounded broker wire/framing
→ yt_dlp.extract_info(download=False)
→ GenericYtdlpAdapter
→ current ResolvedMedia
→ evidence-safe summary only
```

Decisive Attempt-7 signals:

```text
runtime_cache: offline-hit | offline-prepared
process_error != SANDBOX_UNAVAILABLE
process_error != SPAWN_FAILED
process_error != BROKER_PROTOCOL
broker_request_count > 0
broker_error_code != BROKER_RESPONSE_SECRET_REJECTED
```

Only after the accepted broker path continues beyond the former `BROKER_PROTOCOL` edge may #67 classify actual Bilibili media compatibility.

## Exact Candidate

Execute runtime/product code exactly at:

```text
d9c038547ed2df695571f8dd4f732bdcdd4d5c19
```

This Candidate contains accepted #66 extraction, #73 harness, #79 offline runtime/trust anchor, #83 ARM64 sandbox, #85 legacy-kernel fd isolation, #95 anonymous response Secret containment and #97 bounded broker IPC/wire framing.

Task/prompt documentation may be newer than runtime Candidate by design. Do not substitute moving `main`.

If accepted semantic changes touch any of these before claim, STOP for Coordinator freshness review:

- `plugins/generic-ytdlp/**`;
- `scripts/generic-ytdlp-*`;
- `gateway-egress/**` / R008 / ADR 0007;
- sandbox or fd-isolation implementation;
- SiteAdapter / `ResolvedMedia` output authority.

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

1. checkout equals `d9c038547ed2df695571f8dd4f732bdcdd4d5c19`;
2. runtime user matches accepted low-privilege Target class;
3. obtain exact #79 offline bundle without rebuilding/resolving it on Target;
4. permitted transfer is authenticated download of accepted #79 artifact or Coordinator/operator-provided exact local copy;
5. transport credentials are removed before extraction;
6. no Target-side source/package-index resolution may create/replace yt-dlp.

If direct Target Git checkout is unreliable, accepted trusted exact-source transport may be used, provided exact SHA/tree identity is verified locally before execution. Transport choice is not site Evidence.

## J1 — Trust anchor + offline runtime verification

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

No package-index/source network, global/system yt-dlp fallback, same-version replacement wheel, or provenance mismatch is allowed.

## J2 — Direct/no-bypass site reachability

Independently re-confirm formal site Evidence:

- clear upper/lowercase proxy variables for reachability checks;
- use `curl --noproxy '*'` with bounded timeouts;
- record only public HTTPS status class and frozen Bilibili page status/error class.

No Cookie/Auth/login, proxy rotation, fingerprint spoofing, CAPTCHA automation or access-control bypass. Artifact/source-bundle transfer routing is not site Evidence.

## J3 — Accepted real-site smoke with #97 framing

Run only:

```text
YTDLP_OFFLINE_BUNDLE=<verified-bundle-path> \
  scripts/generic-ytdlp-real-smoke.sh \
  'https://www.bilibili.com/video/BV14V411W7r5/'
```

Do not replace the harness with ad-hoc yt-dlp/Python/Rust/CLI code.

Before extractor execution, the accepted harness scrubs proxy state. Extractor HTTP(S) remains under `R008Broker + BrokerProcessRunner`; the worker has no direct socket authority.

Attempt 7 must confirm, when the corresponding fields are available:

```text
close_range_syscall: ENOSYS
process_error != SPAWN_FAILED
process_error != BROKER_PROTOCOL
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

If `SANDBOX_UNAVAILABLE`, `SPAWN_FAILED`, `BROKER_PROTOCOL`, `BROKER_RESPONSE_SECRET_REJECTED`, or another bounded broker/security error repeats on exact R7 Candidate, report BLOCKED and stop. Do not repair it inside #67.

## J4 — Post-run safety / cleanup

Verify:

- no cache staging directory remains;
- verified final user-owned cache may remain;
- no smoke/worker/sandbox/descendant process remains;
- no media payload/file was downloaded;
- checkout remains exact/unmodified apart from task-owned evidence files if any;
- no production Vault/profile/Secret state was touched;
- safe-output scan contains no full resolved URL, signed query, Cookie/Auth/token/account/profile/transfer credential or response Secret material.

## Result semantics

### PASS

All must hold:

- exact Candidate `d9c038547ed2df695571f8dd4f732bdcdd4d5c19` used;
- #79 bundle/trust anchor/provenance verifies;
- runtime cache is `offline-hit` or `offline-prepared`;
- direct/no-proxy sample normally reachable;
- ARM64 sandbox starts without `SANDBOX_UNAVAILABLE`;
- #85 fallback keeps former `SPAWN_FAILED` cleared;
- #95 keeps former whole-response `BROKER_RESPONSE_SECRET_REJECTED` cleared without Secret leakage/store/replay;
- #97 keeps former `BROKER_PROTOCOL` cleared and bounded broker IPC remains functional;
- harness reaches brokered extraction and returns `result: PASS`;
- `broker_request_count > 0`;
- protocol is `http-file` or `hls`;
- at least one current-contract accepted muxed stream is represented;
- no security/Secret/policy violation;
- J4 PASS.

PASS means the frozen source is compatible with the current generic-ytdlp first-playback resolution contract. It does not prove #68 browser playback.

### CONDITIONAL PASS

Only if brokered extraction produces valid current `ResolvedMedia` with a bounded non-security condition that still permits an explicit #68 route. Coordinator decides.

### FAIL

Use only after accepted runtime + sandbox + broker/R008/#95/#97 path execute correctly and site is normally reachable, but the result cannot be represented by the current first-playback contract, for example stable `UNSUPPORTED_FORMAT` / separate A/V.

Do not implement DASH/remux/FFmpeg inside #67.

### BLOCKED

Examples:

- offline bundle transfer/provenance failure;
- repeated `SANDBOX_UNAVAILABLE`;
- repeated `SPAWN_FAILED` despite #85;
- repeated `BROKER_RESPONSE_SECRET_REJECTED` or another concrete R008 policy/limit blocker;
- repeated `BROKER_PROTOCOL` despite #97;
- direct site no longer normally reachable;
- safe Evidence cannot be produced.

A bounded broker error such as `BROKER_RESPONSE_TOO_LARGE` remains a bounded result; do not change R008 limits here.

## Claims

```text
R1 — Exact accepted runtime
#79 + #83 + #85 + #95 + #97 authorities are present in exact Candidate d9c038... and execute without fallback outside accepted semantics.

R2 — Target dependency independence
Target consumes locked/accepted inputs; no source/package-index resolution.

R3 — Normal-network public accessibility
Frozen Bilibili sample is reachable on accepted direct/no-bypass route independently of artifact/source transfer.

R4 — ARM64 sandbox + broker integrity on Linux 4.19
Accepted AArch64 sandbox starts fail-closed, close_range ENOSYS uses accepted bounded fd fallback, direct socket creation remains denied, inherited fd 3 remains the sole worker HTTP capability, and extractor HTTP(S) stays under R008Broker/BrokerProcessRunner authority.

R5 — R008 response Secret containment
Real origin response Secret material remains contained before broker IPC with no disclosure/store/replay, while otherwise safe public response material may continue according to accepted #95 semantics.

R6 — Bounded broker wire continuity
Accepted #97 fixed-width wire framing carries the accepted response across Rust/Python IPC without the former BROKER_PROTOCOL overflow and without increasing R008 HTTP authority.

R7 — Current ResolvedMedia compatibility
Safe result establishes whether the sample maps to the current muxed HTTP/HLS first-playback contract.

R8 — Secret/evidence boundary
No Secret, signed media URL, raw page/media payload, proxy/transfer credential, response Secret material or profile/account state enters durable Evidence.

R9 — Cleanup / target safety
No staging/process/media payload persists; verified cache may persist; low-privilege Target boundaries remain unchanged.
```

## Success criteria

1. J0-J4 execute or a concrete bounded blocker is preserved.
2. Exact Candidate and #79/#83/#85/#95/#97 authorities are verified.
3. No Target-side source/package-index resolution occurs.
4. Direct/no-proxy Bilibili reachability is separated from artifact/source transfer.
5. `SANDBOX_UNAVAILABLE` remains cleared.
6. Former Linux 4.19 `SPAWN_FAILED` remains cleared unless a new concrete regression is proven.
7. Former #95 `BROKER_RESPONSE_SECRET_REJECTED` remains cleared unless a new concrete bounded R008 blocker is proven.
8. Former #97 `BROKER_PROTOCOL` remains cleared unless a new concrete protocol regression is proven.
9. Broker traffic reaches R008 (`broker_request_count > 0`) unless a new pre-broker blocker is explicitly classified.
10. Safe result classifies PASS / CONDITIONAL PASS / FAIL / BLOCKED.
11. R1-R9 are explicitly reported.
12. No implementation/security-policy modification occurs.
13. Worker reports, releases ownership and STOPs; it does not execute #68.

## Evidence contract

`[EXECUTION REPORT]` or `[BLOCKER REPORT]` must contain only bounded Evidence:

```text
Attempt / worker / environment
UTC time
Host class / arch / kernel / uid privilege class
Exact Candidate SHA
Frozen selector: BV14V411W7r5
#85 accepted merge SHA
#95 accepted merge SHA
#97 accepted merge SHA
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
broker wire/framing result: accepted-path | BROKER_PROTOCOL | bounded-blocker
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
Claims R1-R9
Overall: PASS | CONDITIONAL PASS | FAIL | BLOCKED
Downstream #68 readiness: yes/no + reason
```

Never publish transfer credentials, response Secret header names/values, full resolved/signed media URLs, signed query parameters, Cookie, Authorization, tokens, profile/account state, setup logs, raw worker stderr, page body, or media payload.

## Freshness

Semantic authorities:

- exact Execution Candidate `d9c038547ed2df695571f8dd4f732bdcdd4d5c19`;
- #79 offline-runtime helper + lock;
- `scripts/generic-ytdlp-real-smoke.sh`;
- `plugins/generic-ytdlp/**`, including #97 wire/framing;
- `gateway-egress/**` / R008 / ADR 0007 response containment;
- sandbox and fd-isolation implementation;
- current SiteAdapter / `ResolvedMedia` output authority.

Before claim, if any accepted change touches those semantics after `d9c038...`, Worker must STOP for Coordinator freshness review rather than silently use moving main.

## Stop boundary

Normal completion:

```text
[EXECUTION REPORT]
→ status:review
→ release active owner
→ STOP
```

Blocked:

```text
[BLOCKER REPORT]
→ status:blocked
→ release active owner
→ STOP
```

Worker must not merge, set `status:done`, close #67, execute #68, or implement a newly discovered blocker inside this verification-only Task.
