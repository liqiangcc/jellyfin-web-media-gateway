# Task — GENERIC-YTDLP-BILIBILI-REAL

## Metadata

```text
GitHub Issue: #67
Task ID: GENERIC-YTDLP-BILIBILI-REAL
Task kind: verification-only / real public network
Planning Base: d206ee3e1fc8f0e605d25dea0694201b1826924e
Exact Execution Candidate: 826d02c22105ee1877ae79706d2cb03112f995a9
Preferred worker: ubuntu-arm64
Eligible environment: env:ubuntu-arm64
Accepted extraction upstream: #66 Final Accepted
Accepted harness upstream: #73 Final Accepted
Accepted target environment: #63 Final Accepted
Accepted security/runtime authority: #60 + R008
Downstream: #68 BILIBILI-WEB-E2E
Freshness policy: dependency-aware / exact candidate
```

> #67 owns only real-site compatibility Evidence for the frozen public Bilibili sample. It does not implement fixes, weaken security, add DASH/remux, enable production generic-ytdlp or start Web E2E.

## Frozen sample

```text
site: Bilibili
mode: public / no-login / non-DRM
selector: BV14V411W7r5
source: https://www.bilibili.com/video/BV14V411W7r5/
network class: normal direct / no bypass proxy
```

The public source URL is Task input, but durable execution output must not echo it or any resolved/signed media URL.

## Goal

Determine whether the accepted current generic-ytdlp path can resolve the frozen Bilibili sample on the accepted Ubuntu ARM64 phone/network to one current-contract muxed HTTP/HLS stream suitable for the first Web playback milestone.

Required path:

```text
frozen public Bilibili URL
→ accepted #73 scripts/generic-ytdlp-real-smoke.sh
→ isolated frozen yt-dlp 2026.08.19 @ 3a08beaf...
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
826d02c22105ee1877ae79706d2cb03112f995a9
```

This merge contains the Final Accepted #66 extraction implementation and Final Accepted #73 real-site harness.

Subsequent main movement that only adds Task/Prompt/planning documents for #71/#75/#67 does not replace the frozen Candidate. Accepted runtime/security changes in generic-ytdlp/R008/SiteAdapter domains require Coordinator reclassification before execution.

## Host / environment authority

Use the Final Accepted #63 Ubuntu ARM64 phone environment unless Coordinator explicitly revises the Task before claim.

Accepted facts to re-read, not blindly assume:

- Linux ARM64/aarch64;
- runtime user `gateway-runner` uid999, non-root/no-sudo/no-admin;
- user Rust toolchain available;
- Python 3.12 and git/curl available;
- direct/no-proxy public HTTPS works;
- frozen Bilibili page returned HTTP 200 direct/no-proxy;
- FFmpeg/Chromium/Node are absent but are not required for this extraction-only Task.

#67 must not install system packages or elevate privilege.

## J0 — Preflight / exact identity

Before real-site execution, record bounded safe Evidence:

```text
date -u
uname -m
id / uid
exact checkout SHA
python3 --version
git --version
cargo/rustc bounded versions
python3 -m pip --version (availability only)
```

Requirements:

- exact checkout equals `826d02c22105ee1877ae79706d2cb03112f995a9`;
- user is non-root `gateway-runner` or the accepted equivalent low-privilege target user;
- `python3 -m pip` must be available to the user because the accepted #73 harness prepares frozen yt-dlp with isolated user-writable `pip --target`;
- do not install pip/system packages/root dependencies inside this Task;
- if required harness setup capability is absent, report BLOCKED.

Record sanitized proxy metadata only. Do not print proxy credentials or unrelated environment.

## J1 — Direct/no-bypass network recheck

Before running the extractor, re-confirm the network class with bounded checks:

- clear `HTTP_PROXY/HTTPS_PROXY/ALL_PROXY` and lowercase equivalents for the direct check;
- use `curl --noproxy '*'` with strict timeouts;
- public HTTPS reachability bounded status;
- frozen Bilibili page bounded HTTP/error class.

Rules:

- no Cookie/Authorization;
- no login;
- no proxy rotation;
- no fingerprint spoofing;
- no CAPTCHA/challenge automation;
- no residential/proxy/access-control bypass;
- a proxy-mediated result cannot be used as direct Evidence.

If the frozen sample is no longer normally reachable on the accepted direct route, report BLOCKED; do not try to evade site behavior.

## J2 — Accepted real-site smoke

Run only the repository-owned accepted harness from exact Candidate:

```text
scripts/generic-ytdlp-real-smoke.sh 'https://www.bilibili.com/video/BV14V411W7r5/'
```

Do not replace it with ad-hoc Python/Rust/yt-dlp CLI code.

The harness itself owns:

- exact isolated yt-dlp source/version verification;
- transient setup cleanup;
- R008Broker + BrokerProcessRunner runtime;
- direct worker socket denial/sandbox;
- fixed first-playback muxed HTTP/HLS selection;
- safe result summary.

Capture only the safe summary fields intentionally emitted by the harness, e.g.:

```text
result
plugin
broker_status_class
broker_error_code
broker_request_count
protocol
stream_count
title_length
process_error (when present)
```

Do not capture setup logs/raw stderr except a bounded Coordinator-approved diagnostic class when the harness emits no safe result. Never publish full source/resolved URL, signed query, response body, Cookie/Auth/token or media payload.

## J3 — Post-run safety / cleanup

Verify:

- no Task-owned frozen yt-dlp temp directory remains;
- no `generic-ytdlp-real-smoke`, worker, sandbox or descendant process remains;
- no task-owned media payload/file was downloaded;
- repository checkout remains exact/unmodified;
- no production Vault/profile/Secret state was touched;
- safe output leak scan finds no full resolved URL, signed query, Cookie/Authorization/token/account/profile data.

## Result semantics

### PASS

All must hold:

- direct/no-proxy frozen sample normally reachable;
- harness `result: PASS`;
- protocol is `http-file` or `hls`;
- `stream_count >= 1` and first-playback policy is represented by one accepted muxed playable stream;
- no security/policy/Secret violation;
- cleanup/leak scan PASS.

This means the frozen public Bilibili source is compatible with the accepted generic-ytdlp first-playback resolution contract. It still does not prove #68 Web Display playback.

### CONDITIONAL PASS

Use only when the accepted harness reaches a valid current `ResolvedMedia` but exposes a bounded non-security condition that still permits an explicit #68 path. State the exact condition; Coordinator decides acceptance.

### FAIL

Use when:

- the site is normally reachable;
- accepted harness/runtime executes correctly;
- but the frozen sample cannot be represented by current first-playback contract (for example only separate audio/video formats produce `UNSUPPORTED_FORMAT`), without an environment/security blocker.

Do not implement DASH/remux inside #67. A FAIL may trigger a new evidence-driven generic media-format Task.

### BLOCKED

Examples:

- direct sample no longer normally reachable without bypass;
- pip/frozen-runtime setup unavailable on target;
- R008 policy/limit blocks extraction before compatibility can be determined;
- resolver/network/tooling condition invalidates the attempt;
- safe Evidence cannot be produced.

If broker output shows a bounded code such as `BROKER_RESPONSE_TOO_LARGE`, preserve only that safe code. Do not raise the limit in this Task.

## Claims

```text
R1 — Exact accepted runtime
Real-site verification executes the exact #73 accepted harness Candidate and exact frozen yt-dlp provenance.

R2 — Normal-network public accessibility
The frozen Bilibili sample is reached on the accepted direct/no-bypass network class.

R3 — Brokered extraction integrity
Real extractor HTTP(S) remains under R008Broker/BrokerProcessRunner authority with no alternate/proxy/credential path.

R4 — Current ResolvedMedia compatibility
The safe result establishes whether the sample maps to the current first-playback muxed HTTP/HLS contract.

R5 — Secret/evidence boundary
No source-site Secret, signed media URL, raw page/media payload or profile/account data enters durable Evidence.

R6 — Cleanup / target safety
No Task-owned process/temp runtime/media payload persists and low-privilege target boundaries remain unchanged.
```

## Success Criteria

1. J0-J3 execute or a concrete safe blocker is preserved.
2. Exact Candidate and frozen yt-dlp provenance are verified.
3. Direct/no-proxy route is classified without bypass.
4. Harness safe result is preserved exactly enough to classify PASS/CONDITIONAL PASS/FAIL/BLOCKED.
5. R1-R6 are explicitly reported.
6. No implementation/security-policy modification occurs.
7. Worker reports to #67, releases ownership and STOPs; it does not execute #68.

## Evidence Contract

`[EXECUTION REPORT]` or `[BLOCKER REPORT]` must include only bounded Evidence:

```text
Attempt / worker / environment
UTC time
Host class / arch / runtime uid privilege class
Exact Candidate SHA
Frozen selector: BV14V411W7r5
Frozen yt-dlp version/commit verification: pass/fail
pip isolated setup capability: available/blocked
Network class: direct/no-proxy | blocked
Direct public HTTPS status class
Direct Bilibili page status class
Harness result
protocol: http-file | hls | n/a
stream_count
safe title length/hash if emitted
broker_status_class
broker_error_code
process_error
cleanup result
safe-output leak scan
Claims R1-R6
Overall: PASS | CONDITIONAL PASS | FAIL | BLOCKED
Downstream #68 readiness: yes/no + reason
```

Never publish full resolved media URLs, signed query parameters, Cookie, Authorization, token, profile/account state or media payload.

## Freshness

Semantic authorities:

- exact #73 merge Candidate `826d02c...`;
- `plugins/generic-ytdlp/**`;
- `gateway-egress/**` / R008;
- `site-adapter-api/**` if accepted changes affect extraction output/conformance.

#71 and #75 may execute in parallel, but accepted semantic changes from those Tasks do not retroactively alter #67 Candidate. If Coordinator accepts a change that materially affects #67 before #67 claim, re-freeze Candidate explicitly instead of silently using moving main.

## Out of Scope

- code changes/fixes;
- R008 limit/policy weakening;
- proxy/Cookie/login/profile/auth;
- DASH/separate audio-video composition/remux/FFmpeg;
- Bilibili navigation/multipart (#72);
- Browser Worker/Native Panel;
- Web Display playback/control E2E (#68);
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

Worker cannot set status:done, close #67, execute #68, or modify code/security policy.