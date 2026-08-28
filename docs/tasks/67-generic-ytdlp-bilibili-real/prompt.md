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
Contract Revision: R17
Attempt: 17
Exact Candidate: 80fb081b129f8f664124b84ddcc9698039e2cfd1
Target: accepted Ubuntu ARM64 phone / gateway-runner
Sample: BV14V411W7r5
Harness: scripts/generic-ytdlp-real-smoke.sh
```

Task-package docs may be newer than runtime Candidate. Never execute moving main. J0–J4 source/build/harness must resolve exactly to `80fb081b129f8f664124b84ddcc9698039e2cfd1`. Prefer direct exact-Candidate Git; if it cannot establish the Candidate, only accepted #90 trusted source-bundle for the same Candidate may be used.

Read `AGENTS.md`, live #67 and latest R17 comments, `task.md`, lifecycle/freshness protocols, #114/#113/#111/#90/#109/#107/#105 Final Acceptance, and accepted #103/#101/#99/#97/#95/#85/#83/#79/#73/#63/#60/R008 authorities before claim.

## Why R17

R13 produced the valid real compatibility pair `UNSUPPORTED_FORMAT + FALLBACK_WEBPAGE + RESPONSE_ENCODING` with four 2xx broker requests. #111 repaired only the bounded response-normalization seam.

After R14 provenance and R15 reachability blockers, #113 restored publication eligibility using unchanged direct/no-proxy probes. R16 then independently passed J0/J1/J2/J4 and completed the accepted real J3 path with:

```text
UNSUPPORTED_FORMAT
FALLBACK_WEBPAGE
RESPONSE_BODY_TOO_LARGE
broker_status_class: 2xx
broker_request_count: 4
protocol: n/a
stream_count: 0
```

Source-first review showed raw broker/R008 96 KiB and JSON fallback 96 KiB must remain unchanged. #114 was Final Accepted and merged as exact R17 Candidate `80fb081b129f8f664124b84ddcc9698039e2cfd1`; it adds only a FALLBACK_WEBPAGE normalized marker scan bounded at 512 KiB, with strict incremental UTF-8, admitted identity/gzip/deflate, cross-chunk marker correctness and fail-closed malformed/truncated/trailing/concatenated coding.

Only fresh Attempt 17 Target Evidence can prove whether #114 clears the observed R16 seam. #113 does not replace live J2.

## Required path

```text
exact Candidate 80fb081b...
→ direct exact Git OR #90 same-Candidate trusted source-bundle
→ #79 frozen runtime
→ accepted low-privilege ARM64 target
→ live direct/no-proxy J2
→ #99 clean build
→ #83/#85 sandbox/fd
→ R008/#95
→ #97 broker framing
→ #101 bounded outcome
→ #111 response normalization
→ #114 webpage-only bounded marker scan
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
B. #90 trusted source-bundle for candidate_sha=80fb081b129f8f664124b84ddcc9698039e2cfd1
```

If B is used, preserve #90 Candidate/repository/schema/tree/archive SHA256/per-file manifest/safe-extraction checks. Extracted source must contain no `.git` or credential state; transfer credentials must be absent from J1–J4. No alternate commit/source or moving main.

## Live reachability gate

Attempt 17 must independently re-confirm direct/no-proxy public HTTPS and the unchanged frozen Bilibili page with proxy variables cleared and retain only bounded status classes. If the sample is not normally reachable, report BLOCKED and STOP before J3. Do not vary UA/fingerprint/Referer/headers, use Cookie/login/proxy, or attempt bypass.

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
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy/bypass;
- preserve #79/#90/#83/#85/R008/#95/#97/#99/#101/#103/#105/#107/#109/#111/#114 and `DisabledRunner`;
- raw broker/R008 body remains 96 KiB; JSON fallback remains 96 KiB; only #114 webpage normalized marker scan may use fixed 512 KiB;
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

## Stop boundary

Normal: `[EXECUTION REPORT] → status:review → release owner → STOP`.
Blocked: `[BLOCKER REPORT] → status:blocked → release owner → STOP`.

Worker must not merge, done/close #67, implement a blocker, create another compatibility Task, or start #68.

This prompt becomes execution authority only after Coordinator R17 Publication Gate records PUBLISH and live #67 is `status:ready + env:ubuntu-arm64 + no active owner`.
