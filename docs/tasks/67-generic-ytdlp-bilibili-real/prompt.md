# Session Bootstrap — GENERIC-YTDLP-BILIBILI-REAL

Execute Issue #67 using the repository Worker protocol.

## Claim gate

Claim only if live #67 is exactly:

```text
status:ready
env:ubuntu-arm64
no active owner
```

Otherwise STOP.

## Frozen execution

```text
Contract Revision: R16
Attempt: 16
Exact Candidate: 942a0a1843f8f207332ac646f12ffe6ab5017306
Target: accepted Ubuntu ARM64 phone / gateway-runner
Sample: BV14V411W7r5
Harness: scripts/generic-ytdlp-real-smoke.sh
```

Task-package docs may be newer than runtime Candidate. Never execute moving main. Target source/build/harness used for J0–J4 must resolve exactly to `942a0a1843f8f207332ac646f12ffe6ab5017306`. Prefer direct exact-Candidate Git; if it cannot establish the Candidate, only accepted #90 trusted source-bundle for the same Candidate may be used.

Read `AGENTS.md`, live #67 and latest R16 comments, `task.md`, lifecycle/freshness protocols, #113 Final Acceptance, #111/#90/#109/#107/#105 Final Acceptance, and accepted #103/#101/#99/#97/#95/#85/#83/#79/#73/#63/#60/R008 authorities before claim.

## Why R16

Attempt 13 reached the accepted real ARM64/R008/broker path but returned:

```text
UNSUPPORTED_FORMAT
FALLBACK_WEBPAGE
RESPONSE_ENCODING
broker_request_count: 4
```

#111 was Final Accepted and merged as exact runtime Candidate `942a0a1843f8f207332ac646f12ffe6ab5017306`, adding bounded `identity | gzip | deflate → UTF-8` response normalization with normalized output <= 96 KiB and fail-closed malformed/unknown encoding.

Attempt 14 did not test #111 because exact Candidate transport blocked at J0. #90 already provides accepted exact-source recovery.

Attempt 15 restored exact Candidate and frozen runtime, but J2 returned direct public `2xx` and frozen Bilibili `4xx`; J3 was correctly NOT RUN.

#113 then performed only identical direct/no-proxy reachability probes on the unchanged URL and returned:

```text
4xx → 2xx → 2xx
BILIBILI_HOST_ELIGIBLE_FOR_#67_REFRESH=yes
```

#113 did not run resolver/yt-dlp/R008/broker/sandbox and did not vary request identity or use bypass. It resolves publication eligibility only; Attempt 16 must independently re-confirm J2.

## Required path

```text
exact Candidate 942a0a18...
→ direct exact Git OR #90 trusted source-bundle
→ Target exact identity/integrity
→ #79 frozen runtime
→ low-privilege ARM64 target
→ live direct/no-proxy frozen-sample J2
→ #99 clean build
→ #83/#85 sandbox/fd
→ R008/#95
→ #97 broker framing
→ #101 bounded outcome
→ #111 response normalization
→ normal frozen extract_info(download=False)
→ exact #105 continuation only if admitted
→ #107 stage + #109 reason if unsupported
→ current ResolvedMedia OR bounded result
```

## Exact-Candidate transport gate

Accepted routes are exactly:

```text
A. direct exact-Candidate Git
OR
B. #90 trusted source-bundle for candidate_sha=942a0a1843f8f207332ac646f12ffe6ab5017306
```

If B is used, preserve #90 Candidate/repository/schema/tree/archive SHA256/per-file manifest/safe-extraction checks. Extracted source must contain no `.git` or credential state; transfer credentials must be absent from J1–J4 runtime. No alternate commit/source or moving main.

## Live reachability gate

#113 makes a fresh Attempt eligible but does not replace J2. Attempt 16 must independently re-confirm direct/no-proxy public HTTPS and the unchanged frozen Bilibili page with proxy variables cleared and no retained page content. If the sample is not normally reachable, report BLOCKED and STOP before J3. Do not vary UA/fingerprint/Referer/headers, use Cookie/login/proxy, or attempt bypass.

## Real resolver

Only after J2 PASS run:

```text
YTDLP_OFFLINE_BUNDLE="$BUNDLE_PATH" \
  scripts/generic-ytdlp-real-smoke.sh \
  'https://www.bilibili.com/video/BV14V411W7r5/'
```

Do not inspect raw worker stderr or page/media payload.

Required progression:

```text
runtime_cache: offline-hit | offline-prepared
process_error != SANDBOX_UNAVAILABLE
process_error != SPAWN_FAILED
process_error != BROKER_PROTOCOL
process_error != NONZERO_EXIT
broker_request_count > 0
broker_error_code != BROKER_RESPONSE_SECRET_REJECTED
```

If `process_error == UNSUPPORTED_FORMAT`, report exactly one admitted #107 `unsupported_stage` and one #109 `fallback_reason` valid for that stage. Invalid/missing pairs are BLOCKED. `PLAYURL_DASH_PRESENT` is compatibility Evidence only, never repair authority.

## Hard boundaries

- verification-only; no implementation changes;
- exact Candidate only, not moving main/package head;
- no root/sudo/system install or Target dependency resolution;
- source transport only direct exact Git or accepted #90 same-Candidate bundle;
- no unverified source, alternate Candidate, `.git` or transfer credentials in runtime;
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy/bypass;
- no weakening #90/#83/#85/R008/#95/#97/#99/#101/#103/#105/#107/#109/#111;
- preserve #111 96 KiB normalized bound and `DisabledRunner`;
- no arbitrary diagnostics or raw stderr/exception/page/body/header/source/redirect/media URL/signed-query/Secret/Cookie/Auth/token/profile/media payload Evidence;
- no DASH/separate-A/V/remux/FFmpeg/transcoding/navigation/Browser/Web-E2E/performance;
- do not execute #68 or create downstream compatibility Task.

## Target identity

Use exactly:

```text
setpriv --reuid=999 --regid=995 --groups=995,3003 \
  --inh-caps=-all --ambient-caps=-all --bounding-set=-all -- env -i
```

with the accepted `gateway-runner` HOME/USER/LOGNAME/PATH. Do not substitute root/capsh/inherited environment.

## Report / result

Report J0–J4 and R1–R12 from `task.md` using bounded fields only.

- PASS: valid current muxed `http-file | hls` ResolvedMedia + `stream_count >= 1`, J0–J4 and safety/cleanup PASS.
- CONDITIONAL PASS: valid ResolvedMedia plus only bounded non-security limitation.
- FAIL: complete accepted path but current contract rejects, including valid `UNSUPPORTED_FORMAT + stage + reason`.
- BLOCKED: provenance/environment/site/security/runtime/evidence blocker or invalid/missing stage+reason.

Never publish prohibited raw material.

## Stop boundary

Normal: `[EXECUTION REPORT] → status:review → release owner → STOP`.

Blocked: `[BLOCKER REPORT] → status:blocked → release owner → STOP`.

Worker must not merge, done/close #67, implement a blocker, create another compatibility Task, or start #68.

This prompt becomes execution authority only after Coordinator R16 Publication Gate records PUBLISH and live #67 is `status:ready + env:ubuntu-arm64 + no active owner`.
