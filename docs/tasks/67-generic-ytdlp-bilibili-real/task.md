# Task — GENERIC-YTDLP-BILIBILI-REAL

## Metadata

```text
GitHub Issue: #67
Task ID: GENERIC-YTDLP-BILIBILI-REAL
Task kind: verification-only / real public network
Contract Revision: R16
Next Attempt: 16
Exact Execution Candidate: 942a0a1843f8f207332ac646f12ffe6ab5017306
Preferred worker: ubuntu-arm64
Eligible environment: env:ubuntu-arm64
Frozen sample: BV14V411W7r5
Harness: scripts/generic-ytdlp-real-smoke.sh
Downstream: #68 BILIBILI-WEB-E2E
Freshness policy: dependency-aware
Publication state: non-executable until Coordinator Publication Gate passes
```

## Accepted authority

R16 consumes the accepted chain without redefining it:

```text
#79 offline runtime
→ #90 trusted exact-source transport recovery
→ #83 ARM64 sandbox
→ #85 legacy-kernel fd isolation
→ #95 anonymous response Secret containment
→ #97 broker framing
→ #99 exact-Candidate clean-build binding
→ #101 bounded worker/extractor outcome taxonomy
→ #103 current ResolvedMedia normalization
→ #105 narrow Bilibili missing-initial-state continuation
→ #107 closed unsupported_stage attribution
→ #109 closed full fallback stage→reason attribution
→ #111 bounded fallback response normalization
→ #113 frozen-sample reachability refresh
```

Accepted #90 merge: `b7774f216e723d6b5eab90f712c2b746ad132f76`.
Accepted #109 merge: `af65b2e2fec4cd3b3303db19415890f4052aa026`.
Accepted #111 Candidate: `f402fd7f6b49a7afc2de744f4aa6371bb485385e`, PR #112, accepted merge/runtime authority: `942a0a1843f8f207332ac646f12ffe6ab5017306`.
Accepted #113 verification package: `1c83159d1a5d4d93ec3f682d259c7e2d01d48556`; reachability result `4xx → 2xx → 2xx`, `BILIBILI_HOST_ELIGIBLE_FOR_#67_REFRESH=yes`.

#109 proves offline full fallback traversal to current muxed `http-file` ResolvedMedia and closed stage→reason attribution. #111 adds only bounded response normalization: `identity | gzip | deflate → UTF-8`, normalized output <= 96 KiB, with malformed/unknown/ambiguous/nested/trailing/truncated inputs fail-closed. Neither #109 nor #111 claims real-site success.

## Parent evidence

Attempt 13 on exact Candidate `af65b2e2fec4cd3b3303db19415890f4052aa026` executed the accepted low-privilege ARM64/R008/broker path and returned:

```text
process_error: UNSUPPORTED_FORMAT
unsupported_stage: FALLBACK_WEBPAGE
fallback_reason: RESPONSE_ENCODING
broker_status_class: 2xx
broker_request_count: 4
protocol: n/a
stream_count: 0
Overall: FAIL
```

This was a valid compatibility FAIL and led to #111.

Attempt 14 froze exact Candidate `942a0a1843f8f207332ac646f12ffe6ab5017306` but blocked at J0 because direct Target Git did not transfer the Candidate; J1–J4 were not run. #90 already provides the accepted exact-source recovery path.

Attempt 15 restored exact Candidate provenance and passed J0/J1, but J2 returned public HTTPS `2xx` and the unchanged frozen Bilibili sample `4xx`; J3 was correctly NOT RUN. This was site-reachability/environment BLOCKED, not a compatibility result.

#113 then rechecked the same unchanged URL on the same accepted phone with the same ordinary direct/no-proxy request shape and returned `4xx → 2xx → 2xx`, satisfying its two-consecutive-2xx PASS rule. #113 did not run yt-dlp/resolver/R008/broker/sandbox and did not vary request identity or use bypass.

R16 therefore preserves runtime Candidate `942a0a1843f8f207332ac646f12ffe6ab5017306`, preserves #90 transport recovery, consumes #113 only as publication-level reachability Evidence, and must independently re-run J2 before J3.

## Frozen sample/runtime

```text
site: Bilibili
mode: public / no-login / non-DRM
selector: BV14V411W7r5
source: https://www.bilibili.com/video/BV14V411W7r5/
network: direct / no proxy / no bypass

yt-dlp: 2026.08.19
source commit: 3a08beaf031ab68f966401ead017ac81fe8486cf
wheel sha256: 86a521c89017200d7cc20173b9f1d04c6588dda4eabad324b5c76d5269ee1bf9
trust anchor: scripts/generic-ytdlp-offline-runtime.lock.json
```

Exact target boundary:

```text
setpriv --reuid=999 --regid=995 --groups=995,3003 \
  --inh-caps=-all --ambient-caps=-all --bounding-set=-all -- env -i
```

with `HOME=/home/gateway-runner`, `USER=LOGNAME=gateway-runner`, and `PATH=/home/gateway-runner/.cargo/bin:/usr/local/bin:/usr/bin:/bin`.

## Goal

Execute one bounded real-target Attempt 16:

```text
exact Candidate source
→ direct exact Git OR accepted #90 trusted source-bundle
→ exact Candidate/tree/archive/file integrity on Target
→ #79 frozen offline runtime
→ accepted low-privilege ARM64 target
→ live direct/no-proxy frozen Bilibili J2
→ exact-Candidate clean build
→ #83/#85 sandbox/fd isolation
→ R008/#95 Secret containment
→ #97 bounded broker framing
→ normal frozen yt_dlp.extract_info(download=False)
→ only if #105 admission matches: bounded Bilibili continuation
→ #111 response normalization
→ #107 unsupported_stage + #109 fallback_reason when unsupported
→ current ResolvedMedia OR one bounded actionable result
```

Decisive question:

```text
Does accepted #111 normalization clear the real R13 RESPONSE_ENCODING seam and allow BV14V411W7r5 to produce a current muxed http-file | hls ResolvedMedia?
OR, if not, which exact closed unsupported_stage + fallback_reason now owns the rejection?
```

## Frozen unsupported taxonomy

`unsupported_stage` remains exactly:

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

The #109 stage→reason mapping remains frozen. In particular:

```text
PRE_FALLBACK → UNCLASSIFIED | MEDIA_NO_MUXED_STREAM
FALLBACK_WEBPAGE → RESPONSE_STATUS | RESPONSE_BODY_TOO_LARGE | RESPONSE_ENCODING | RESPONSE_JSON | RESPONSE_SECRET_FIELD | RESPONSE_READ | WEBPAGE_NOT_HTML | WEBPAGE_BANGUMI
FALLBACK_NAV → RESPONSE_STATUS | RESPONSE_BODY_TOO_LARGE | RESPONSE_ENCODING | RESPONSE_JSON | RESPONSE_SECRET_FIELD | RESPONSE_READ | NAV_API_ENVELOPE | NAV_SHAPE | NAV_WBI_SHAPE | NAV_WBI_URL
FALLBACK_VIEW → RESPONSE_STATUS | RESPONSE_BODY_TOO_LARGE | RESPONSE_ENCODING | RESPONSE_JSON | RESPONSE_SECRET_FIELD | RESPONSE_READ | VIEW_API_ENVELOPE | VIEW_ID_MISMATCH | VIEW_TITLE | VIEW_PAGES | VIEW_CID
FALLBACK_DETAIL → RESPONSE_STATUS | RESPONSE_BODY_TOO_LARGE | RESPONSE_ENCODING | RESPONSE_JSON | RESPONSE_SECRET_FIELD | RESPONSE_READ | DETAIL_API_ENVELOPE | DETAIL_SHAPE | DETAIL_ID_MISMATCH | DETAIL_TITLE | DETAIL_PAGES | DETAIL_CID_MISMATCH | DETAIL_TITLE_MISMATCH
FALLBACK_PLAYURL → RESPONSE_STATUS | RESPONSE_BODY_TOO_LARGE | RESPONSE_ENCODING | RESPONSE_JSON | RESPONSE_SECRET_FIELD | RESPONSE_READ | PLAYURL_API_ENVELOPE | PLAYURL_DURL_SHAPE | PLAYURL_DASH_PRESENT | PLAYURL_SEGMENT_SHAPE | PLAYURL_SEGMENT_FIELDS
MEDIA_SHAPE → MEDIA_URL_SHAPE | MEDIA_URL_SENSITIVE_QUERY | MEDIA_EXTENSION | MEDIA_HEADERS | MEDIA_TITLE | MEDIA_NO_MUXED_STREAM
UNCLASSIFIED → UNCLASSIFIED
```

These values are repository-owned control-flow constants only. They must not expose or imply raw site payload, exception text, headers, URLs/query material, credentials or media metadata beyond the fixed enum.

## Hard boundaries

- verification-only; no repository/product/security implementation changes;
- exact runtime Candidate only: `942a0a1843f8f207332ac646f12ffe6ab5017306`;
- no moving-main/package-head or alternate-source substitution;
- Candidate transport may use only direct exact-Candidate Git or accepted #90 trusted source-bundle for the same Candidate;
- #90 source-bundle path must verify Candidate/repository/schema/tree/archive SHA256/per-file manifest/safe extraction and carry no `.git` or credential state into runtime;
- no root/sudo/system install or Target package-index/source dependency resolution;
- direct/no-proxy site Evidence only;
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy rotation/access-control bypass;
- no alternate worker egress / R008 bypass;
- preserve #83/#85/#90/#95/#97/#99/#101/#103/#105/#107/#109/#111 and `DisabledRunner`;
- preserve #111 96 KiB normalized fallback bound;
- no raw stderr/traceback/exception text, page/body, request/response headers, source/redirect/media URLs, signed query material, credentials, Secret, Cookie/Auth/token/profile/account state or media payload in durable Evidence;
- no DASH/separate-A/V/remux/FFmpeg/transcoding/navigation/Browser/Web-E2E/performance scope;
- `PLAYURL_DASH_PRESENT` is compatibility Evidence only, not implementation authority;
- no #68 and no downstream compatibility Task from Worker.

## J0–J4

### J0 — exact target and Candidate

Prove accepted ARM64 `gateway-runner` low-privilege identity and exact Candidate `942a0a1843f8f207332ac646f12ffe6ab5017306`. Use direct exact Git if it succeeds; otherwise only accepted #90 trusted source-bundle. If #90 is used, verify Candidate/repository/schema/tree/archive/per-file integrity and safe extraction. No moving main, alternate source, root/sudo or Target dependency resolution.

### J1 — frozen runtime provenance

Verify trust anchor, exact wheel SHA/provenance, yt-dlp identity and `runtime_cache: offline-hit | offline-prepared`.

### J2 — live direct site reachability

Independently re-confirm direct/no-proxy public HTTPS and the unchanged frozen Bilibili page status with proxy variables cleared. #113 is publication eligibility only and does not substitute for live J2. Do not retain page contents. If the frozen sample is not normally reachable in Attempt 16, classify BLOCKED and STOP before J3; do not vary UA/fingerprint/headers, use Cookie/login/proxy, or attempt bypass.

### J3 — real resolver smoke

Only after J2 PASS, run exactly:

```text
YTDLP_OFFLINE_BUNDLE="$BUNDLE_PATH" scripts/generic-ytdlp-real-smoke.sh 'https://www.bilibili.com/video/BV14V411W7r5/'
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

If `process_error: UNSUPPORTED_FORMAT`, require exactly one valid #107 `unsupported_stage` and one #109 reason valid for that stage. Missing/unknown/forged/wrong-stage evidence is BLOCKED, not permission to inspect raw diagnostics.

### J4 — cleanup / safe-output

Verify zero staging/worker/sandbox/descendant/media-payload residue, verified cache only as allowed, no Vault/profile/Secret mutation, clean exact source, and safe-output leak scan PASS.

## Result semantics

PASS requires exact Candidate, J0–J4 PASS, direct site reachability, broker path exercised, valid current muxed `http-file | hls` ResolvedMedia, `stream_count >= 1`, and cleanup/safe-output PASS.

CONDITIONAL PASS requires a valid current ResolvedMedia plus only a bounded non-security limitation; unsupported is never CONDITIONAL PASS.

FAIL means the complete accepted path executes correctly but the frozen source cannot be represented by the current first-playback contract, canonically:

```text
process_error: UNSUPPORTED_FORMAT
unsupported_stage: <valid stage>
fallback_reason: <valid reason for stage>
protocol: n/a
stream_count: 0
```

BLOCKED covers provenance/Target/site reachability/sandbox/spawn/broker/Secret/runtime/evidence failures, stable `EXTRACTOR_FAILURE`, or invalid unsupported stage+reason evidence.

## Claims / report

Report J0–J4 and R1–R12 explicitly:

1. exact Candidate and accepted authority chain;
2. low-privilege target + frozen runtime provenance;
3. direct/no-proxy site reachability;
4. ARM64 sandbox/fd/broker path;
5. response Secret containment;
6. bounded broker framing/continuity;
7. #105 normal-extract-first semantics;
8. #107 closed unsupported stage;
9. #109 stage-scoped reason;
10. no prohibited raw diagnostics;
11. cleanup/target safety;
12. bounded result sufficient for Coordinator decision.

Durable report may include only bounded identities/status classes/result enums and authority SHAs, including #113 reachability refresh authority. Never publish credentials, response contents, headers, raw stderr, signed/resolved URLs or media payload.

## Freshness

Semantic runtime authority is exact Candidate `942a0a1843f8f207332ac646f12ffe6ab5017306`. Later task-package docs do not replace it. If accepted semantic changes touch generic-ytdlp/runtime/R008/broker/Secret/sandbox/fd domains after publication and before claim, STOP for Coordinator freshness review.

## Stop boundary

Normal: `[EXECUTION REPORT] → status:review → release owner → STOP`.

Blocked: `[BLOCKER REPORT] → status:blocked → release owner → STOP`.

Worker must not merge, done/close #67, implement a blocker, create a compatibility Task, or start #68.
