# Session Bootstrap — GENERIC-YTDLP-BILIBILI-REAL

Execute Issue #67 using the repository Worker protocol.

## Claim gate

Claim only if live #67 is exactly:

```text
status:ready
env:ubuntu-arm64
no active owner
```

If it is draft, blocked, review, done, closed, or already owned: STOP.

## Frozen execution

```text
Contract Revision: R15
Attempt: 15
Exact Candidate: 942a0a1843f8f207332ac646f12ffe6ab5017306
Target: accepted Ubuntu ARM64 phone / gateway-runner
Sample: BV14V411W7r5
Harness: scripts/generic-ytdlp-real-smoke.sh
```

Task-package docs may be newer than the runtime Candidate. Do **not** execute moving main. The target source/build/harness used for J0–J4 must resolve exactly to `942a0a1843f8f207332ac646f12ffe6ab5017306`. Direct exact-Candidate Git is preferred when it succeeds; if it does not, only the already-accepted #90 trusted source-bundle route for this same Candidate may be used.

Read `AGENTS.md`, live #67 and its latest R15 comments, `docs/tasks/67-generic-ytdlp-bilibili-real/task.md`, lifecycle/freshness protocols, #111 Final Acceptance, #90 Final Acceptance, #109 Final Acceptance, #107/#105 Final Acceptance, and accepted #103/#101/#99/#97/#95/#85/#83/#79/#73/#63/#60/R008 authorities before claim.

## Why R15 exists

Attempt 13 reached the real frozen sample through the accepted low-privilege ARM64, sandbox, broker and R008 path, but returned:

```text
process_error: UNSUPPORTED_FORMAT
unsupported_stage: FALLBACK_WEBPAGE
fallback_reason: RESPONSE_ENCODING
broker_request_count: 4
```

#111 is now Final Accepted and merged as exact R15 Candidate `942a0a1843f8f207332ac646f12ffe6ab5017306`. It adds only a closed bounded response-normalization layer before existing UTF-8/JSON/HTML fallback admission:

```text
identity | gzip | deflate
→ UTF-8 only
→ normalized output <= 96 KiB
→ existing JSON/HTML admission
```

Malformed, unknown, ambiguous, nested, trailing or truncated coding remains fail-closed. #111 did not run the real site and does not prove what coding/charset the R13 response actually used.

Attempt 14 did not test this compatibility change. It stopped at J0 because the Target remained on prior Candidate `af65b2e2fec4cd3b3303db19415890f4052aa026`, and two bounded direct exact-Candidate fetch attempts did not transfer `942a0a18...`. J1–J4 were not run. This is an infrastructure/provenance blocker only.

#90 is already Final Accepted and merged as `b7774f216e723d6b5eab90f712c2b746ad132f76`. Its generic workflow accepts an exact `candidate_sha`, creates a hosted source-only archive with Candidate SHA, tree SHA, archive SHA256 and per-file manifest, and verifies that exact identity plus safe extraction on the accepted ARM64 Target. #90 Target Evidence proved both smart-HTTP exact fetch and trusted source-bundle verification 3/3.

R15 is therefore the next bounded real-target decision point after restoring exact-Candidate provenance through this already-accepted transport authority.

## Required path

```text
exact Candidate `942a0a18...`
→ direct exact Git OR #90 trusted source-bundle transport
→ Target Candidate/tree/archive/file integrity verification
→ #79 frozen offline runtime
→ accepted low-privilege ARM64 target
→ direct/no-proxy frozen Bilibili sample
→ #99 exact-Candidate clean-build binding
→ #83 sandbox + #85 fd fallback
→ R008 + #95 response containment
→ #97 broker framing
→ #101 bounded worker outcome
→ #111 bounded response normalization
→ normal frozen yt_dlp.extract_info(download=False)
→ only if exact #105 admission matches: bounded continuation
→ #107 unsupported_stage + #109 fallback_reason when unsupported
→ current ResolvedMedia OR bounded actionable result
```

There is no caller-selectable fallback action.

## Decisive question

```text
Does the accepted #111 normalization clear the real R13 RESPONSE_ENCODING seam and allow BV14V411W7r5 to produce a valid current muxed http-file | hls ResolvedMedia?
OR, if not, which exact closed unsupported_stage + fallback_reason pair now owns the rejection?
```

A valid current ResolvedMedia is still required for #67 PASS.

## Unsupported evidence

If `process_error == UNSUPPORTED_FORMAT`, report exactly:

```text
unsupported_stage: <one fixed #107 value>
fallback_reason: <one #109 reason valid for that stage>
```

The stage set is:

```text
PRE_FALLBACK
FALLBACK_WEBPAGE
FALLBACK_NAV
FALLBACK_VIEW
FALLBACK_DETAIL
FALLBACK_PLAYURL
MEDIA_SHAPE
UNCLASSIFIED
```

Use the exact stage→reason mapping frozen in `task.md`. Never invent, normalize, abbreviate or infer a reason from exception text, response content, URL/query data, headers, media metadata or site diagnostics.

If the exact Candidate returns `UNSUPPORTED_FORMAT` without a valid stage+reason pair, report BLOCKED and STOP. Do not patch #67.

`PLAYURL_DASH_PRESENT` is only a bounded compatibility result. It is not permission to add DASH, remux, FFmpeg or separate-A/V support.

## Exact-Candidate transport gate

J0 must establish exact Candidate `942a0a1843f8f207332ac646f12ffe6ab5017306` before any runtime/site check.

Accepted routes are exactly:

```text
A. direct exact-Candidate Git succeeds
OR
B. #90 trusted source-bundle for candidate_sha=942a0a1843f8f207332ac646f12ffe6ab5017306
```

If route B is used, preserve the accepted #90 verification contract: Candidate SHA, repository/schema, tree SHA, archive SHA256, per-file manifest/hash and safe extraction must all pass on Target. The extracted source must contain no `.git` or credential state. Any GitHub workflow/artifact transfer credential is transport-only and must be absent from the J1–J4 runtime environment.

Do not use moving main/package head, a different commit, an unverified archive, Target package-index/source dependency resolution, root/sudo, or any alternate network/source bypass. Artifact/source transfer is not real-site Evidence.

## Real-site command

Run J0–J4 exactly from `task.md`. The only accepted real resolver command is:

```text
YTDLP_OFFLINE_BUNDLE="$BUNDLE_PATH" \
  scripts/generic-ytdlp-real-smoke.sh \
  'https://www.bilibili.com/video/BV14V411W7r5/'
```

The smoke output is already bounded to safe fields and can emit fixed `unsupported_stage` / `fallback_reason` values. Do not inspect raw worker stderr or page/media payload to obtain more detail.

## Required progression signals

```text
runtime_cache: offline-hit | offline-prepared
process_error != SANDBOX_UNAVAILABLE
process_error != SPAWN_FAILED
process_error != BROKER_PROTOCOL
process_error != NONZERO_EXIT
broker_request_count > 0
broker_error_code != BROKER_RESPONSE_SECRET_REJECTED
```

## Hard boundaries

- verification-only; no implementation changes;
- exact Candidate only, not moving main/package head;
- no root/sudo/system install or Target package-index/source dependency resolution;
- exact source transport is limited to direct exact-Candidate Git or the accepted #90 trusted source-bundle for the same Candidate;
- no unverified source bundle, moving main/package-head, alternate commit/source, `.git` or transfer credential state in runtime;
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy/bypass;
- no #90/R008/#95/#97/#99/#83/#85/#101/#105/#107/#109/#111 weakening;
- preserve the accepted #111 96 KiB normalized fallback bound;
- no direct worker network or alternate socket;
- no arbitrary diagnostic strings;
- no raw stderr/traceback/exception text, source/redirect/media URL, request/response headers, page/body contents, signed query material, credentials, Secret, Cookie/Auth/token/profile/account state, or media payload in Evidence;
- no DASH/separate-A/V/remux/FFmpeg/transcoding/navigation/Browser/Web E2E/performance;
- production default remains DisabledRunner;
- do not execute #68 and do not create a downstream compatibility Task.

## Target identity

Use the accepted shell exactly:

```text
setpriv --reuid=999 --regid=995 --groups=995,3003 \
  --inh-caps=-all --ambient-caps=-all --bounding-set=-all -- env -i
```

with:

```text
HOME=/home/gateway-runner
USER=gateway-runner
LOGNAME=gateway-runner
PATH=/home/gateway-runner/.cargo/bin:/usr/local/bin:/usr/bin:/bin
```

Do not replace it with root/capsh/inherited environment.

## Report

Report J0–J4 and Claims R1–R12 from `task.md`.

Bounded report fields may include:

```text
Exact Candidate
host arch/kernel/uid privilege class
runtime_cache
direct site status class
sandbox/fd/R008/broker result
broker_status_class
broker_error_code
broker_request_count
protocol
stream_count
title_length
process_error
unsupported_stage
fallback_reason
cleanup/safe-output scan
R1-R12
Overall
#68 readiness
```

Never include prohibited raw material.

Result semantics are exactly those in `task.md`:

- `PASS`: valid current muxed `http-file | hls` ResolvedMedia, all safety/cleanup gates pass;
- `CONDITIONAL PASS`: only valid ResolvedMedia with bounded non-security limitation;
- `FAIL`: complete accepted path but current contract rejects, including valid `UNSUPPORTED_FORMAT + stage + reason`;
- `BLOCKED`: environment/provenance/security/runtime/site/evidence blocker or invalid/missing stage+reason.

## Stop boundary

Normal:

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

Worker must not merge, set `status:done`, close #67, implement a discovered blocker, create another compatibility Task, or start #68.

This prompt becomes execution authority only after the Coordinator R15 Publication Gate records PUBLISH and live #67 is `status:ready + env:ubuntu-arm64 + no active owner`.
