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
```

#67 remains verification-only. Attempt 6 reached real R008 2xx and #95 containment but reproduced `BROKER_PROTOCOL` 2/2. #97 proved the decimal-JSON binary-body framing overflow and was Final Accepted / merged as `d9c038547ed2df695571f8dd4f732bdcdd4d5c19`.

## Frozen sample and runtime

```text
site: Bilibili
mode: public / no-login / non-DRM
selector: BV14V411W7r5
source: https://www.bilibili.com/video/BV14V411W7r5/
formal network: direct / no bypass proxy

yt-dlp: 2026.08.19
source commit: 3a08beaf031ab68f966401ead017ac81fe8486cf
wheel sha256: 86a521c89017200d7cc20173b9f1d04c6588dda4eabad324b5c76d5269ee1bf9
trust anchor: scripts/generic-ytdlp-offline-runtime.lock.json
```

## Goal

```text
#79 offline runtime
→ direct/no-proxy frozen Bilibili sample
→ scripts/generic-ytdlp-real-smoke.sh
→ #83 ARM64 sandbox
→ BrokerProcessRunner + #85 ENOSYS fd fallback
→ R008Broker + #95 response Secret containment
→ #97 bounded broker wire/framing
→ yt_dlp.extract_info(download=False)
→ GenericYtdlpAdapter
→ current ResolvedMedia
```

Exact runtime Candidate:

```text
d9c038547ed2df695571f8dd4f732bdcdd4d5c19
```

Do not substitute moving main. If accepted semantic changes touch `plugins/generic-ytdlp/**`, `scripts/generic-ytdlp-*`, `gateway-egress/**`/R008/ADR0007, sandbox/fd-isolation, or current SiteAdapter/ResolvedMedia authority before claim, STOP for Coordinator freshness review.

## Hard boundaries

- no repository/product/security implementation changes;
- no root/sudo/system package install;
- no Target package-index/source resolution or replacement yt-dlp;
- direct/no-proxy site Evidence only;
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy rotation/access-control bypass;
- no R008 HTTP limit change, Secret declassification/store/replay, sandbox/fd bypass, alternate socket or direct worker egress;
- no full signed media URL, query token, Secret header, raw stderr, page body or media payload in durable Evidence;
- no DASH/remux/FFmpeg/navigation/Browser/Web E2E/performance;
- production GenericYtdlpAdapter default remains DisabledRunner;
- do not execute #68.

## J0-J4

J0 — verify exact Candidate, accepted low-privilege ARM64 Target, and exact #79 bundle provisioning without Target dependency resolution.

J1 — verify repository trust anchor, wheel SHA/provenance, yt-dlp identity and `runtime_cache: offline-hit | offline-prepared`.

J2 — independently re-confirm direct/no-proxy public HTTPS and frozen Bilibili page status using bounded curl checks with proxy variables cleared.

J3 — run only:

```text
YTDLP_OFFLINE_BUNDLE=<verified-bundle-path> \
  scripts/generic-ytdlp-real-smoke.sh \
  'https://www.bilibili.com/video/BV14V411W7r5/'
```

Required progression signals:

```text
process_error != SANDBOX_UNAVAILABLE
process_error != SPAWN_FAILED
process_error != BROKER_PROTOCOL
broker_request_count > 0
broker_error_code != BROKER_RESPONSE_SECRET_REJECTED
```

Capture only bounded safe fields: result, plugin, runtime_cache, broker_status_class, broker_error_code, broker_request_count, protocol, stream_count, title_length, process_error.

J4 — verify no staging/worker/sandbox/descendant/media payload leftovers; verified cache may remain; no Vault/profile/Secret state touched; safe-output leak scan PASS.

## Result semantics

PASS requires exact Candidate, accepted #79/#83/#85/#95/#97 path, direct site reachability, former SANDBOX/SPAWN/SECRET/BROKER_PROTOCOL blockers cleared, `broker_request_count > 0`, harness PASS, protocol `http-file | hls`, at least one current-contract muxed stream, and cleanup/security PASS.

CONDITIONAL PASS is only a bounded non-security condition with valid current ResolvedMedia that still permits explicit #68 routing; Coordinator decides.

FAIL is only after the complete accepted path executes correctly but the real source cannot be represented by current first-playback contract, e.g. stable unsupported/separate-A/V result. Do not implement DASH/remux here.

BLOCKED includes offline provenance/transfer failure, repeated SANDBOX/SPAWN/SECRET/BROKER_PROTOCOL errors, a new bounded R008/protocol condition, site unreachability, or inability to produce safe Evidence. Do not repair blockers in #67.

## Claims

```text
R1 exact #79/#83/#85/#95/#97 runtime authority
R2 Target dependency independence
R3 direct/no-bypass public accessibility
R4 ARM64 sandbox + #85 fd/broker integrity
R5 #95 response Secret containment
R6 #97 bounded Rust/Python broker wire continuity
R7 current muxed HTTP/HLS ResolvedMedia compatibility
R8 Secret/evidence boundary
R9 cleanup / low-privilege Target safety
```

## Success criteria

- J0-J4 execute or preserve one concrete bounded blocker.
- Exact Candidate and accepted authorities are proven.
- SANDBOX_UNAVAILABLE, SPAWN_FAILED, BROKER_RESPONSE_SECRET_REJECTED and BROKER_PROTOCOL remain cleared unless a new concrete regression is proven.
- Broker traffic reaches R008 unless a new pre-broker blocker is classified.
- Overall is PASS / CONDITIONAL PASS / FAIL / BLOCKED.
- R1-R9 reported explicitly.
- no implementation/security-policy change.
- Worker reports, releases owner and STOPs; never starts #68.

## Evidence contract

Report bounded metadata only:

```text
Attempt / worker / environment / UTC
host arch/kernel/uid privilege class
Exact Candidate SHA
BV14V411W7r5
#85/#95/#97 accepted merge SHAs
bundle transfer class + trust-anchor/wheel/provenance result
runtime_cache
direct public/Bilibili status class
sandbox + close_range/fd isolation
R008 containment
broker wire/framing result
harness result
protocol / stream_count / safe title length
broker_status_class / broker_error_code / broker_request_count
process_error
cleanup + safe-output scan
R1-R9
Overall
#68 readiness yes/no + reason
```

Never publish credentials, Secret header names/values, signed URLs/query parameters, Cookie/Auth/token/profile/account state, raw stderr/page body/media payload.

## Stop boundary

```text
normal: [EXECUTION REPORT] → status:review → release owner → STOP
blocked: [BLOCKER REPORT] → status:blocked → release owner → STOP
```

Worker must not merge, mark done/close, implement a blocker, or execute #68.
