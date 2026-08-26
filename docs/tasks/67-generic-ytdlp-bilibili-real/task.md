# Task — GENERIC-YTDLP-BILIBILI-REAL

## Metadata

```text
GitHub Issue: #67
Task ID: GENERIC-YTDLP-BILIBILI-REAL
Task kind: verification-only / real public network
Contract Revision: R2
Trigger: #67 Attempt 1 FROZEN_RUNTIME_SETUP before broker traffic
Exact Execution Candidate: f2c8736ea705ebf942da833550fe96182b377813
Preferred worker: ubuntu-arm64
Eligible environment: env:ubuntu-arm64
Accepted extraction upstream: #66 Final Accepted
Accepted harness/runtime-cache upstream: #73 R2 Final Accepted / PR #77
Accepted target environment: #63 Final Accepted
Accepted security/runtime authority: #60 + R008
Downstream: #68 BILIBILI-WEB-E2E
Freshness policy: dependency-aware / exact Candidate
```

> #67 owns only real-site compatibility Evidence for the frozen public Bilibili sample. It does not implement fixes, weaken security, add DASH/remux, enable production generic-ytdlp, or start Web E2E.

## Frozen sample

```text
site: Bilibili
mode: public / no-login / non-DRM
selector: BV14V411W7r5
source: https://www.bilibili.com/video/BV14V411W7r5/
formal site network class: normal direct / no bypass proxy
```

The source URL is Task input. Durable Evidence must not publish the full source URL again, any resolved/signed media URL, query token, Cookie, Authorization, profile/account state, raw worker stderr, page body, or media payload.

## Why Attempt 2

Attempt 1 established:

```text
direct public HTTPS: HTTP 200
direct frozen Bilibili page: HTTP 200
accepted harness invoked
result: BLOCKED
process_error: FROZEN_RUNTIME_SETUP
broker_request_count: 0
```

Coordinator classified that result as a harness/runtime-preparation blocker rather than Bilibili incompatibility.

#73 R2 is now Final Accepted and merged as:

```text
f2c8736ea705ebf942da833550fe96182b377813
```

R2 separates fixed dependency acquisition from formal extractor network authority:

```text
fixed user-owned yt-dlp cache
→ exact version/commit provenance verification
→ warm reuse or atomic cold prepare
→ scrub setup proxy/Python import state
→ R008Broker
→ BrokerProcessRunner
→ frozen yt-dlp extract_info(download=False)
→ current ResolvedMedia
→ safe summary
```

Exact frozen upstream identity remains:

```text
yt-dlp 2026.08.19
commit 3a08beaf031ab68f966401ead017ac81fe8486cf
```

## Goal

Determine whether the accepted current generic-ytdlp path can resolve the frozen public Bilibili sample on the accepted Ubuntu ARM64 phone/network to the current first-playback muxed HTTP/HLS `ResolvedMedia` contract.

Required path:

```text
frozen public Bilibili selector
→ scripts/generic-ytdlp-real-smoke.sh
→ verified user-owned frozen runtime cache
→ R008Broker
→ BrokerProcessRunner sandbox
→ yt_dlp.extract_info(download=False)
→ GenericYtdlpAdapter
→ current ResolvedMedia
→ evidence-safe summary only
```

## Exact Candidate

Execute exactly:

```text
f2c8736ea705ebf942da833550fe96182b377813
```

This merge contains the accepted #66 extraction runtime, #73 R1 safe harness, and #73 R2 durable frozen-runtime cache/setup separation.

Do not silently substitute moving main. Later Task/Prompt/planning-only commits are `UNRELATED`; accepted semantic changes in `plugins/generic-ytdlp/**`, `gateway-egress/**` / R008, or relevant SiteAdapter output authority require Coordinator freshness reclassification.

## Host / environment authority

Use the Final Accepted #63 Ubuntu ARM64 phone environment unless Coordinator explicitly revises the Task before claim.

Re-read and verify, do not blindly assume:

- Linux ARM64/aarch64;
- low-privilege `gateway-runner` uid999, non-root/no-sudo/no-admin;
- user Rust toolchain available;
- Python 3.12, pip, git, curl available;
- direct/no-proxy public HTTPS and frozen Bilibili page were previously HTTP 200;
- FFmpeg/Chromium/Node are absent but not required for this extraction-only Task.

No root/sudo/system package install is permitted.

## J0 — Preflight / exact identity

Record only bounded safe Evidence:

```text
date -u
uname -m
uid / privilege class
exact checkout SHA
python3 --version
python3 -m pip --version availability
git --version
cargo/rustc bounded versions
```

Requirements:

- checkout equals exact Candidate `f2c8736ea705ebf942da833550fe96182b377813`;
- execution user matches accepted low-privilege target class;
- Python/pip capability needed for a cold fixed-cache prepare is available;
- no package installation or privilege escalation is performed outside the repository-owned harness/cache helper;
- record proxy presence/class only in sanitized form; never print credentials or full proxy URLs with secrets.

## J1 — Direct/no-bypass site reachability

Before extraction, independently re-confirm the formal site network class:

- clear `HTTP_PROXY/HTTPS_PROXY/ALL_PROXY` and lowercase equivalents for these reachability checks;
- use `curl --noproxy '*'` with strict bounded timeouts;
- record only public HTTPS status class and frozen Bilibili page status/error class.

Rules:

- no Cookie/Authorization/login;
- no proxy rotation/residential proxy;
- no fingerprint spoofing;
- no CAPTCHA/challenge automation;
- no access-control bypass;
- proxy-mediated page reachability cannot be reported as formal direct site Evidence.

If the frozen sample is no longer normally reachable on the accepted direct route, report BLOCKED.

## J2 — Accepted real-site smoke

Run only the repository-owned accepted harness from the exact Candidate:

```text
scripts/generic-ytdlp-real-smoke.sh 'https://www.bilibili.com/video/BV14V411W7r5/'
```

Do not replace it with ad-hoc Python/Rust/yt-dlp CLI code.

### Setup network versus extractor network

Do **not** manually scrub ordinary setup proxy variables before invoking the harness merely to force dependency acquisition direct. The accepted #73 R2 harness owns this separation:

1. a warm valid fixed cache is reused without setup network;
2. if cold preparation is required, only the repository-fixed yt-dlp dependency may use the setup process's ordinary network/proxy route;
3. setup output is suppressed and proxy values/credentials are not persisted or published;
4. before the smoke binary/extractor starts, the harness removes setup proxy variables;
5. `BrokerProcessRunner` uses `env_clear()` and the yt-dlp worker receives only the accepted broker capability; HTTP(S) extractor traffic remains under R008Broker authority.

A setup proxy therefore is **not** Bilibili/site Evidence and must never be used to justify site accessibility. Formal Bilibili reachability remains J1 direct/no-proxy Evidence; extractor network authority remains R008.

The harness/cache helper owns:

- fixed cache path under user-owned cache storage;
- exact version + VCS commit + cache-local import verification;
- atomic staging/promotion and corrupt/partial-cache fail-closed behavior;
- warm reuse;
- R008Broker + BrokerProcessRunner runtime;
- fixed muxed HTTP/HLS selection;
- bounded safe output.

Capture only fields intentionally emitted by the safe harness, including when present:

```text
result
plugin
runtime_cache: hit | prepared
broker_status_class
broker_error_code
broker_request_count
protocol
stream_count
title_length
process_error
```

Never preserve setup logs/raw stderr or full source/resolved URLs.

## J3 — Post-run safety / cleanup

Verify:

- no cache staging directory remains;
- the final verified user-owned frozen cache **may remain** by design for warm reuse;
- no task-owned smoke/worker/sandbox/descendant process remains;
- no media payload/file was downloaded;
- repository checkout remains exact/unmodified;
- no production Vault/profile/Secret state was touched;
- safe-output leak scan contains no full resolved URL, signed query, Cookie/Authorization/token/account/profile data.

Do not delete a verified final cache merely to satisfy cleanup; `invalidate` is an explicit bounded maintenance operation, not the normal success path.

## Result semantics

### PASS

All must hold:

- J1 direct/no-proxy frozen sample normally reachable;
- exact frozen runtime provenance verifies;
- harness reaches brokered extraction and returns `result: PASS`;
- `broker_request_count > 0`;
- protocol is `http-file` or `hls`;
- at least one current-contract accepted muxed stream is represented;
- no security/Secret/policy violation;
- J3 cleanup/leak boundary PASS.

PASS means the frozen public Bilibili source is compatible with the accepted generic-ytdlp first-playback resolution contract. It does **not** prove #68 Web Display playback.

### CONDITIONAL PASS

Only when brokered extraction reaches a valid current `ResolvedMedia` but a bounded non-security condition still permits an explicit #68 path. State the condition; Coordinator decides acceptance.

### FAIL

Use when the site is normally reachable and the accepted brokered runtime executes correctly, but the sample cannot be represented by the current first-playback contract, for example stable `UNSUPPORTED_FORMAT` because only separate audio/video is usable.

Do not add DASH/remux/FFmpeg inside #67.

### BLOCKED

Use for environment/security/runtime conditions that prevent compatibility determination, such as:

- direct site no longer normally reachable;
- fixed runtime cache cannot be safely prepared/verified;
- R008 policy/limit blocks before compatibility can be determined;
- safe Evidence cannot be produced.

If a bounded broker code such as `BROKER_RESPONSE_TOO_LARGE` is emitted, preserve only the safe code and request/status counts. Do not change R008 limits in this Task.

## Claims

```text
R1 — Exact accepted runtime
The exact #73 R2 accepted merge and frozen yt-dlp provenance execute on the target.

R2 — Normal-network public accessibility
The frozen Bilibili sample is reachable on the accepted direct/no-bypass route.

R3 — Brokered extraction integrity
Real extractor HTTP(S) remains under R008Broker/BrokerProcessRunner authority; setup dependency routing is not extractor/site authority.

R4 — Current ResolvedMedia compatibility
Safe result establishes whether the sample maps to the current muxed HTTP/HLS first-playback contract.

R5 — Secret/evidence boundary
No Secret, signed media URL, raw page/media payload, setup proxy credential or profile/account state enters durable Evidence.

R6 — Cleanup / target safety
No staging/process/media payload persists; verified final cache may persist; low-privilege target boundaries remain unchanged.
```

## Success Criteria

1. J0-J3 execute or a concrete safe blocker is preserved.
2. Exact Candidate and frozen yt-dlp version/commit provenance are verified.
3. Direct/no-proxy Bilibili reachability is classified separately from setup dependency routing.
4. Brokered extractor traffic demonstrably reaches R008 (`broker_request_count > 0`) unless a pre-broker blocker is explicitly classified.
5. Safe result is sufficient to classify PASS / CONDITIONAL PASS / FAIL / BLOCKED.
6. R1-R6 are explicitly reported.
7. No implementation/security-policy modification occurs.
8. Worker reports, releases ownership, and STOPs; it does not execute #68.

## Evidence Contract

`[EXECUTION REPORT]` or `[BLOCKER REPORT]` must include only bounded Evidence:

```text
Attempt / worker / environment
UTC time
Host class / arch / runtime uid privilege class
Exact Candidate SHA
Frozen selector: BV14V411W7r5
Frozen yt-dlp version/commit verification: pass/fail
runtime_cache: hit | prepared | blocked
setup network class: ordinary setup route available | warm-cache no setup network | blocked
formal site network class: direct/no-proxy | blocked
Direct public HTTPS status class
Direct Bilibili page status class
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
Claims R1-R6
Overall: PASS | CONDITIONAL PASS | FAIL | BLOCKED
Downstream #68 readiness: yes/no + reason
```

Never publish full resolved media URLs, signed query parameters, Cookie, Authorization, tokens, proxy credentials, profile/account state, setup logs, raw worker stderr, page body, or media payload.

## Freshness

Semantic authorities:

- exact accepted #73 R2 merge `f2c8736ea705ebf942da833550fe96182b377813`;
- `plugins/generic-ytdlp/**`;
- `scripts/generic-ytdlp-real-smoke.sh` and runtime-cache helper;
- `gateway-egress/**` / R008;
- `site-adapter-api/**` only if an accepted change materially alters extraction output/conformance before claim.

#71/#75 may proceed independently. If a later accepted semantic change materially affects #67 before claim, Coordinator must explicitly re-freeze the Candidate rather than silently use moving main.

## Out of Scope

- code changes/fixes;
- R008 policy/limit weakening;
- Cookie/login/profile/auth/access-control bypass;
- DASH/separate A/V composition/remux/FFmpeg;
- Bilibili navigation/multipart (#72);
- Browser Worker/Native Panel;
- Web Display/control E2E (#68);
- production generic-ytdlp enablement;
- performance/capacity/thermal/soak (#9).

## Completion Protocol

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ J0-J3
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ status:review or status:blocked
→ release owner
→ STOP
```

Worker cannot set status:done, close #67, execute #68, or modify product/security policy.