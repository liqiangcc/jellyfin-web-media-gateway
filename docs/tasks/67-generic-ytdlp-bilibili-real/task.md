# Task — GENERIC-YTDLP-BILIBILI-REAL

## Metadata

```text
GitHub Issue: #67
Task ID: GENERIC-YTDLP-BILIBILI-REAL
Task kind: verification-only / real public network
Contract Revision: R11
Next Attempt: 11
Exact Execution Candidate: 1a38e403a3252239822aeb2a784a20fdfd18c0a6
Preferred worker: ubuntu-arm64
Eligible environment after publication: env:ubuntu-arm64
Accepted extraction upstream: #66 Final Accepted
Accepted harness authority: #73 R2 Final Accepted
Accepted offline runtime authority: #79 Attempt 2 Final Accepted
Accepted ARM64 sandbox authority: #83 Final Accepted
Accepted legacy-kernel fd-isolation authority: #85 Final Accepted / merge 76b2032410b19ee18cfb14f00317b97f84e3b691
Accepted anonymous response Secret containment authority: #95 Final Accepted / merge 804fd60343b081e5e055ba87f68e7939b106bb19
Accepted broker IPC/wire authority: #97 Final Accepted / merge d9c038547ed2df695571f8dd4f732bdcdd4d5c19
Accepted clean-build sandbox binding authority: #99 Final Accepted / merge cd95db5f0becb875455789f168b92c44a96a5260
Accepted bounded extractor failure authority: #101 Final Accepted / merge c2834fd046cbf29a3602e9f13ae5153217c6c886
Accepted ResolvedMedia compatibility authority: #103 Attempt 2 Coordinator ACCEPTED / PR #104 merge bec606fe0346e60fa5f05f98e27981fca8feffb2
Accepted Bilibili missing-initial-state fallback authority: #105 Attempt 3 Final Accepted / PR #106 merge 1a38e403a3252239822aeb2a784a20fdfd18c0a6
Accepted target environment: #63 Final Accepted
Accepted security/runtime authority: #60 + R008 + ADR 0007
Downstream: #68 BILIBILI-WEB-E2E
Freshness policy: dependency-aware
Publication state: non-executable until Coordinator Publication Gate passes and live Issue is status:ready
```

#67 remains verification-only. Attempt 10 preserved the accepted runtime/target/security path, reached three 2xx broker requests, and still returned bounded `EXTRACTOR_FAILURE` without a current ResolvedMedia. #105 then proved and merged the narrow repository-owned missing-initial-state continuation behind the same production-shaped `GenericYtdlpAdapter::resolve_detailed()` → `ProcessRunner::run()` → normal `extract` path used by the real smoke harness. Normal frozen yt-dlp extraction remains first; fallback admission is limited to the frozen BiliBiliIE missing-initial-state condition plus strict Bilibili video URL shape, and deterministic hosted x86_64/native ARM64 J1-J4 proved one muxed `http-file` ResolvedMedia while malformed/initial-state/redirect/Secret/non-media/separate-A/V/unexpected cases remained fail closed. R11 re-freezes the same real-site contract on exact merged Integration Candidate `1a38e403a3252239822aeb2a784a20fdfd18c0a6`; it does not claim the target result in advance.

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
→ #99 exact-Candidate clean-build sibling binding
→ #83 ARM64 sandbox
→ BrokerProcessRunner + #85 ENOSYS fd fallback
→ R008Broker + #95 response Secret containment
→ #97 bounded broker wire/framing
→ #101 bounded worker/extractor outcome envelope
→ normal frozen yt_dlp.extract_info(download=False)
→ if exact #105 admission matches: bounded missing-initial-state continuation
→ GenericYtdlpAdapter
→ current ResolvedMedia
```

Exact runtime Candidate for Attempt 11:

```text
1a38e403a3252239822aeb2a784a20fdfd18c0a6
```

Do not substitute moving main. If accepted semantic changes touch `plugins/generic-ytdlp/**`, `scripts/generic-ytdlp-*`, `gateway-egress/**`/R008/ADR0007, sandbox/fd-isolation, or current SiteAdapter/ResolvedMedia authority before claim, STOP for Coordinator freshness review. The accepted low-privilege ARM64 launch boundary remains the exact historical `setpriv --reuid=999 --regid=995 --groups=995,3003 --inh-caps=-all --ambient-caps=-all --bounding-set=-all -- env -i` shell with `HOME=/home/gateway-runner`, `USER=gateway-runner`, `LOGNAME=gateway-runner`, `PATH=/home/gateway-runner/.cargo/bin:/usr/local/bin:/usr/bin:/bin`, and the exact harness command; do not substitute root, capsh, a different identity, inherited environment, or a different shell.

## Hard boundaries

- no repository/product/security implementation changes;
- no root/sudo/system package install;
- no Target package-index/source resolution or replacement yt-dlp;
- direct/no-proxy site Evidence only;
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy rotation/access-control bypass;
- no R008 HTTP limit change, #101 taxonomy bypass, Secret declassification/store/replay, sandbox/fd bypass, alternate socket or direct worker egress;
- do not broaden, bypass, or modify the #105 fallback admission inside #67;
- no full signed media URL, query token, Secret header, raw stderr, exception text, page body or media payload in durable Evidence;
- no DASH/remux/FFmpeg/navigation/Browser/Web E2E/performance;
- production GenericYtdlpAdapter default remains DisabledRunner;
- do not execute #68.

## J0-J4

J0 — verify exact Candidate, accepted low-privilege ARM64 Target, and exact #79 bundle provisioning without Target dependency resolution.

J1 — verify repository trust anchor, wheel SHA/provenance, yt-dlp identity and `runtime_cache: offline-hit | offline-prepared`.

J2 — independently re-confirm direct/no-proxy public HTTPS and frozen Bilibili page status using bounded curl checks with proxy variables cleared.

J3 — run only:

```text
YTDLP_OFFLINE_BUNDLE="$BUNDLE_PATH" \
  scripts/generic-ytdlp-real-smoke.sh \
  'https://www.bilibili.com/video/BV14V411W7r5/'
```

Required progression signals:

```text
process_error != SANDBOX_UNAVAILABLE
process_error != SPAWN_FAILED
process_error != BROKER_PROTOCOL
process_error != NONZERO_EXIT
broker_request_count > 0
broker_error_code != BROKER_RESPONSE_SECRET_REJECTED
```

Attempt 11 specifically tests whether the accepted #105 path converts the formerly stable bounded `EXTRACTOR_FAILURE` into a valid current ResolvedMedia when the frozen real Bilibili sample reaches the same missing-initial-state condition. Do not inspect raw exception/stderr to prove fallback admission; the retained authority is the merged #105 behavior and bounded harness result only.

If resolution still does not succeed, `process_error` must be one of the accepted fixed #101 codes (`REQUEST_POLICY_REJECTED`, `BROKER_FAILURE`, `EXTRACTOR_FAILURE`, `UNSUPPORTED_FORMAT`, or `UNEXPECTED_WORKER_FAILURE`) rather than raw diagnostics or generic `NONZERO_EXIT`.

Capture only bounded safe fields: result, plugin, runtime_cache, broker_status_class, broker_error_code, broker_request_count, protocol, stream_count, title_length, process_error.

J4 — verify no staging/worker/sandbox/descendant/media payload leftovers; verified cache may remain; no Vault/profile/Secret state touched; safe-output leak scan PASS.

## Result semantics

PASS requires exact Candidate, accepted #79/#83/#85/#95/#97/#99/#101/#103/#105 path, direct site reachability, former SANDBOX/SPAWN/SECRET/BROKER_PROTOCOL/NONZERO blockers cleared, `broker_request_count > 0`, harness PASS, protocol `http-file | hls`, at least one current-contract muxed stream, and cleanup/security PASS.

CONDITIONAL PASS is only a bounded non-security condition with valid current ResolvedMedia that still permits explicit #68 routing; Coordinator decides.

FAIL is only after the complete accepted path executes correctly but the real source cannot be represented by current first-playback contract, e.g. stable unsupported/separate-A/V result. Do not implement DASH/remux here.

BLOCKED includes offline provenance/transfer failure, repeated SANDBOX/SPAWN/SECRET/BROKER_PROTOCOL errors, a new bounded R008/protocol condition, site unreachability, stable `EXTRACTOR_FAILURE` after the accepted #105 path, or inability to produce safe Evidence. Do not repair blockers in #67.

## Claims

```text
R1 exact #79/#83/#85/#95/#97/#99/#101/#103/#105 runtime authority
R2 Target dependency independence
R3 direct/no-bypass public accessibility
R4 ARM64 sandbox + #85 fd/broker integrity
R5 #95 response Secret containment
R6 #97 bounded Rust/Python broker wire continuity
R7 #105 bounded missing-initial-state continuation reaches current muxed HTTP/HLS ResolvedMedia when applicable
R8 Secret/evidence boundary
R9 cleanup / low-privilege Target safety
```

## Success criteria

- J0-J4 execute or preserve one concrete bounded blocker.
- Exact Candidate and accepted authorities are proven.
- SANDBOX_UNAVAILABLE, SPAWN_FAILED, BROKER_RESPONSE_SECRET_REJECTED and BROKER_PROTOCOL remain cleared unless a new concrete regression is proven.
- Broker traffic reaches R008 unless a new pre-broker blocker is classified.
- The accepted #105 continuation is exercised only through the existing normal `extract` path; no caller-selectable fallback authority is introduced.
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
#85/#95/#97/#99/#101/#103/#105 accepted merge SHAs
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

Never publish credentials, Secret header names/values, signed URLs/query parameters, Cookie/Auth/token/profile/account state, raw stderr/exception text/page body/media payload.

## Freshness / Integration Contract

Freshness policy: dependency-aware

Semantic authorities:
- #105 Attempt 3 Final Accepted and PR #106 merged as `1a38e403a3252239822aeb2a784a20fdfd18c0a6`; normal `extract` remains first and the bounded missing-initial-state continuation is reached only through the production-shaped adapter/process path.
- #103 Attempt 2 Coordinator ACCEPTED and PR #104 merged as `bec606fe0346e60fa5f05f98e27981fca8feffb2`.
- #79/#83/#85/#95/#97/#99/#101, R008, ADR 0007, #63 target, and `scripts/generic-ytdlp-real-smoke.sh` as accepted by the prior #67 contract.

Semantic freshness domains:
- `plugins/generic-ytdlp/**`, `scripts/generic-ytdlp-*`, GenericYtdlpAdapter/ResolvedMedia normalization, #105 fallback admission, broker/R008/Secret containment, sandbox/fd isolation, and the accepted ARM64 target launch boundary.

Integration surfaces:
- exact merged main Candidate, generic-ytdlp worker/runtime wiring, broker/sandbox composition, and target harness invocation.

Task-owned surfaces:
- none; this is verification-only and must not modify repository/product/security implementation.

Authority/domain → Claim mapping:
- #105 / normal-extract missing-initial-state continuation: R1, R7, R8.
- #103 / GenericYtdlpAdapter direct media normalization: R1, R7.
- #79/#83/#85/#95/#97/#99/#101/R008 and target launch boundary: R1, R4, R5, R6, R8, R9.
- exact Candidate and harness: R1, R2, R3, R7.

Integration verification:
- JI1: confirm the target checkout and `scripts/generic-ytdlp-real-smoke.sh` resolve to exact Candidate `1a38e403a3252239822aeb2a784a20fdfd18c0a6` before J0-J4.
- JI2: n/a; target proof is the declared J0-J4 evidence authority.

Unrelated-main policy:
- existing exact-Candidate semantic Evidence remains valid; no rebase/full rerun solely because main advanced.

Integration-overlap policy:
- preserve accepted #103/#105 semantic Evidence; no merge or source changes are performed by this verification Task. If the target harness cannot prove the exact Candidate, stop with bounded evidence.

Semantic-authority-change policy:
- reconcile the changed authority and rerun mapped Claims only when a Coordinator explicitly revises this contract; do not silently broaden the Attempt.

Strict-main reason:
- n/a; the real-site proof is frozen to the exact merged Integration Candidate above.

## Stop boundary

```text
normal: [EXECUTION REPORT] → status:review → release owner → STOP
blocked: [BLOCKER REPORT] → status:blocked → release owner → STOP
```

Worker must not merge, mark done/close, implement a blocker, or execute #68.
