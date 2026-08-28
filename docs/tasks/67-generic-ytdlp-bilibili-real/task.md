# Task — GENERIC-YTDLP-BILIBILI-REAL

## Metadata

```text
GitHub Issue: #67
Task ID: GENERIC-YTDLP-BILIBILI-REAL
Task kind: verification-only / real public network
Contract Revision: R14
Next Attempt: 14
Exact Execution Candidate: 942a0a1843f8f207332ac646f12ffe6ab5017306
Preferred worker: ubuntu-arm64
Eligible environment: env:ubuntu-arm64
Frozen sample: BV14V411W7r5
Harness: scripts/generic-ytdlp-real-smoke.sh
Downstream: #68 BILIBILI-WEB-E2E
Freshness policy: dependency-aware
Publication state: non-executable until Coordinator Publication Gate passes and live Issue is status:ready
```

## Accepted authority

R14 consumes the already accepted chain without redefining it:

```text
#79 offline runtime
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
```

Accepted #109 Candidate: `35f34fe5967d5e1d4a17671ab8b59c22bf2dacff`.
Accepted #109 PR: #110.
Accepted #109 merge authority: `af65b2e2fec4cd3b3303db19415890f4052aa026`.

Accepted #111 Candidate: `f402fd7f6b49a7afc2de744f4aa6371bb485385e`.
Accepted #111 PR: #112.
Accepted #111 merge/main authority and R14 Exact Execution Candidate: `942a0a1843f8f207332ac646f12ffe6ab5017306`.

#109 proved offline that the repository-owned continuation can traverse:

```text
webpage
→ nav / WBI
→ view
→ detail
→ playurl
→ current muxed http-file ResolvedMedia
```

and that unsupported outcomes can carry one fixed, stage-valid `fallback_reason`. #109 did **not** prove that the real Bilibili sample succeeds.

#111 then proved offline that fallback response bytes can be normalized through a closed `identity | gzip | deflate` content-coding contract into UTF-8 text while enforcing the accepted 96 KiB normalized bound and preserving all existing failure distinctions. #111 likewise did **not** run the real site or prove which coding/charset the real response used.

## Parent evidence

#67 Attempt 13 executed exact Candidate `af65b2e2fec4cd3b3303db19415890f4052aa026` on the accepted low-privilege Ubuntu ARM64 target. J0/J1/J2/J4 passed and J3 traversed the accepted broker path with four 2xx requests, but returned:

```text
result: UNSUPPORTED
runtime_cache: offline-hit
broker_status_class: 2xx
broker_error_code: n/a
broker_request_count: 4
process_error: UNSUPPORTED_FORMAT
unsupported_stage: FALLBACK_WEBPAGE
fallback_reason: RESPONSE_ENCODING
protocol: n/a
stream_count: 0
Overall: FAIL
```

That was a valid compatibility FAIL, not an infrastructure blocker. #111 was then Final Accepted as the bounded repository-owned response normalization repair for this exact `RESPONSE_ENCODING` seam. R14 reruns the same frozen public sample on the accepted #111 merge authority without claiming that the prior real response used any specific content-coding or charset.

## Frozen sample and runtime

```text
site: Bilibili
mode: public / no-login / non-DRM
selector: BV14V411W7r5
source: https://www.bilibili.com/video/BV14V411W7r5/
formal network: direct / no proxy / no bypass

yt-dlp: 2026.08.19
source commit: 3a08beaf031ab68f966401ead017ac81fe8486cf
wheel sha256: 86a521c89017200d7cc20173b9f1d04c6588dda4eabad324b5c76d5269ee1bf9
trust anchor: scripts/generic-ytdlp-offline-runtime.lock.json
```

Exact accepted target launch boundary remains:

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

Do not substitute root, sudo, capsh, inherited environment, a different identity, a different yt-dlp, or moving main.

## Goal

Execute one bounded real-target verification on exact Candidate `942a0a1843f8f207332ac646f12ffe6ab5017306`:

```text
frozen offline runtime
→ accepted low-privilege ARM64 target
→ direct/no-proxy frozen Bilibili sample
→ exact-Candidate clean build
→ sandbox / fd isolation
→ R008Broker / Secret containment
→ bounded broker framing
→ normal frozen yt_dlp.extract_info(download=False)
→ only if #105 admission matches: bounded Bilibili continuation
→ #111 bounded response normalization
→ #107 unsupported_stage + #109 fallback_reason if unsupported
→ GenericYtdlpAdapter
→ current ResolvedMedia OR one bounded actionable unsupported pair
```

The decisive question is:

```text
Does the accepted #111 normalization clear the real R13 RESPONSE_ENCODING seam and allow BV14V411W7r5 to produce a valid current muxed http-file | hls ResolvedMedia?
OR, if not, which exact closed unsupported_stage + fallback_reason pair now owns the rejection?
```

## Frozen stage taxonomy

`unsupported_stage` remains exactly the accepted #107 set:

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

## Frozen reason contract

A `fallback_reason` is valid only with `process_error: UNSUPPORTED_FORMAT` and only when admitted for the accompanying stage. R14 preserves the #109 closed mapping:

```text
PRE_FALLBACK
  → UNCLASSIFIED | MEDIA_NO_MUXED_STREAM

FALLBACK_WEBPAGE
  → RESPONSE_STATUS | RESPONSE_BODY_TOO_LARGE | RESPONSE_ENCODING
  | RESPONSE_JSON | RESPONSE_SECRET_FIELD | RESPONSE_READ
  | WEBPAGE_NOT_HTML | WEBPAGE_BANGUMI

FALLBACK_NAV
  → RESPONSE_STATUS | RESPONSE_BODY_TOO_LARGE | RESPONSE_ENCODING
  | RESPONSE_JSON | RESPONSE_SECRET_FIELD | RESPONSE_READ
  | NAV_API_ENVELOPE | NAV_SHAPE | NAV_WBI_SHAPE | NAV_WBI_URL

FALLBACK_VIEW
  → RESPONSE_STATUS | RESPONSE_BODY_TOO_LARGE | RESPONSE_ENCODING
  | RESPONSE_JSON | RESPONSE_SECRET_FIELD | RESPONSE_READ
  | VIEW_API_ENVELOPE | VIEW_ID_MISMATCH | VIEW_TITLE | VIEW_PAGES | VIEW_CID

FALLBACK_DETAIL
  → RESPONSE_STATUS | RESPONSE_BODY_TOO_LARGE | RESPONSE_ENCODING
  | RESPONSE_JSON | RESPONSE_SECRET_FIELD | RESPONSE_READ
  | DETAIL_API_ENVELOPE | DETAIL_SHAPE | DETAIL_ID_MISMATCH
  | DETAIL_TITLE | DETAIL_PAGES | DETAIL_CID_MISMATCH | DETAIL_TITLE_MISMATCH

FALLBACK_PLAYURL
  → RESPONSE_STATUS | RESPONSE_BODY_TOO_LARGE | RESPONSE_ENCODING
  | RESPONSE_JSON | RESPONSE_SECRET_FIELD | RESPONSE_READ
  | PLAYURL_API_ENVELOPE | PLAYURL_DURL_SHAPE | PLAYURL_DASH_PRESENT
  | PLAYURL_SEGMENT_SHAPE | PLAYURL_SEGMENT_FIELDS

MEDIA_SHAPE
  → MEDIA_URL_SHAPE | MEDIA_URL_SENSITIVE_QUERY | MEDIA_EXTENSION
  | MEDIA_HEADERS | MEDIA_TITLE | MEDIA_NO_MUXED_STREAM

UNCLASSIFIED
  → UNCLASSIFIED
```

These are repository-owned control-flow constants only. Do not infer or expose site payload, response text, exception text, HTTP reason text, URL/query data, BVID/CID, codec/format IDs, credentials, tokens, or signed media material from the reason.

## Hard boundaries

- verification-only; no repository/product/security implementation changes;
- exact Candidate only: `942a0a1843f8f207332ac646f12ffe6ab5017306`;
- no root/sudo/system install or Target package-index/source resolution;
- direct/no-proxy real-site Evidence only;
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy rotation/access-control bypass;
- no direct worker socket, alternate egress, or R008 bypass;
- preserve #95 Secret policy, #97 broker protocol, #99 clean-build binding, #83/#85 sandbox/fd authority;
- preserve #101 top-level taxonomy, #105 normal-extract-first narrow admission, #107 stage taxonomy, #109 reason taxonomy, #111 bounded response normalization and the 96 KiB normalized fallback bound;
- no arbitrary diagnostics or reason strings;
- no raw stderr/traceback/exception text, page/body content, request/response headers, source/redirect/media URLs, signed query material, credentials, Cookie/Auth/token/profile/account state, or media payload in durable Evidence;
- no DASH support, separate-A/V composition, remux, FFmpeg, transcoding, navigation, Browser/Native Panel, Web E2E, performance work;
- `PLAYURL_DASH_PRESENT` is an actionable compatibility result only, not authorization to implement DASH;
- production GenericYtdlpAdapter default remains `DisabledRunner`;
- do not execute #68 and do not create a downstream compatibility Task.

## J0–J4

### J0 — exact target and Candidate

Prove:

- target is the accepted Ubuntu ARM64 phone / `gateway-runner` class;
- low-privilege identity/launch boundary is unchanged;
- checkout/build/harness resolve to exact Candidate `942a0a1843f8f207332ac646f12ffe6ab5017306`;
- no moving-main substitution;
- no Target dependency resolution or root/sudo fallback.

### J1 — frozen runtime provenance

Verify trust anchor, wheel SHA/provenance, yt-dlp identity and:

```text
runtime_cache: offline-hit | offline-prepared
```

### J2 — bounded direct site reachability

Independently re-confirm direct/no-proxy public HTTPS and the frozen Bilibili page status using bounded checks with proxy variables cleared. Do not retain page contents.

### J3 — real resolver smoke

Run only the accepted entry:

```text
YTDLP_OFFLINE_BUNDLE="$BUNDLE_PATH" \
  scripts/generic-ytdlp-real-smoke.sh \
  'https://www.bilibili.com/video/BV14V411W7r5/'
```

Required infrastructure progression signals:

```text
process_error != SANDBOX_UNAVAILABLE
process_error != SPAWN_FAILED
process_error != BROKER_PROTOCOL
process_error != NONZERO_EXIT
broker_request_count > 0
broker_error_code != BROKER_RESPONSE_SECRET_REJECTED
```

If current media succeeds, report only bounded success fields:

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
process_error: n/a
unsupported_stage: n/a
fallback_reason: n/a
```

A PASS candidate must have `protocol: http-file | hls` and `stream_count >= 1`.

If `process_error == UNSUPPORTED_FORMAT`, report exactly one admitted `unsupported_stage` and exactly one stage-valid `fallback_reason`. Do not inspect raw diagnostics to explain it.

If unsupported lacks a reason, contains an unknown reason, or presents a stage→reason pair not admitted above, classify the Attempt as BLOCKED by invalid bounded Evidence and STOP. Do not patch #67.

If another fixed #101 outcome occurs, report only the fixed code and do not fabricate stage/reason.

### J4 — cleanup and safe Evidence

Verify:

- no staging/worker/sandbox/descendant/media payload leftovers;
- verified offline cache may remain;
- no Vault/profile/Secret state touched;
- safe-output leak scan PASS;
- no prohibited raw data in report/artifact/log evidence retained for #67.

## Result semantics

### PASS

Requires all of:

- exact Candidate and accepted target/runtime/security path;
- J0–J4 PASS;
- direct site reachability;
- broker path exercised (`broker_request_count > 0`);
- no former sandbox/spawn/secret/broker protocol regression;
- harness produces valid current `ResolvedMedia`;
- protocol is `http-file | hls`;
- at least one current-contract muxed stream;
- cleanup/security Evidence PASS.

Only this result makes #67 eligible for Final Acceptance and allows Coordinator to consider publishing #68.

### CONDITIONAL PASS

Only when a valid current ResolvedMedia exists with a bounded non-security limitation. Coordinator decides; an unsupported result is never CONDITIONAL PASS.

### FAIL

The accepted path executes correctly but the frozen source cannot be represented by the current first-playback contract. The canonical compatibility FAIL is:

```text
process_error: UNSUPPORTED_FORMAT
unsupported_stage: <valid #107 stage>
fallback_reason: <valid #109 reason for that stage>
```

This is actionable Evidence but does not authorize a repair inside #67.

### BLOCKED

Includes:

- provenance/transfer/Target failure;
- site unreachability;
- sandbox/spawn/broker/Secret regression;
- stable `EXTRACTOR_FAILURE` or other infrastructure/runtime blocker;
- exact-Candidate `UNSUPPORTED_FORMAT` without a valid stage+reason pair;
- inability to produce bounded safe Evidence.

Do not repair blockers in this verification Task.

## Claims

```text
R1 exact #79/#83/#85/#95/#97/#99/#101/#103/#105/#107/#109/#111 authority
R2 Target dependency independence / low privilege
R3 direct/no-bypass public accessibility
R4 ARM64 sandbox + fd/broker integrity
R5 response Secret containment
R6 bounded Rust/Python broker continuity
R7 #105 continuation preserves current muxed HTTP/HLS semantics
R8 #107 unsupported_stage attribution is closed and valid
R9 #109 fallback_reason attribution is closed, stage-scoped and valid
R10 safe Evidence / Secret boundary
R11 cleanup / target safety
R12 result is sufficient to decide #67 PASS or the next smallest compatibility authority
```

## Success criteria

1. J0–J4 execute on exact Candidate or preserve one concrete bounded blocker.
2. R1–R12 are reported explicitly.
3. Existing infrastructure blockers remain cleared unless a new concrete regression is proven.
4. #105 fallback is entered only through normal extract and exact accepted admission.
5. Unsupported results contain one valid #107 stage and one valid #109 stage-scoped reason.
6. No arbitrary diagnostic or prohibited payload is published.
7. Overall is `PASS | CONDITIONAL PASS | FAIL | BLOCKED` using the frozen semantics above.
8. No implementation/security change occurs.
9. Worker posts the report, transitions to `status:review | status:blocked`, releases owner and STOPs.
10. Worker never starts #68 or creates the next compatibility Task.

## Evidence contract

Durable report may include only:

```text
Attempt / worker / environment / UTC
host arch/kernel/uid privilege class
Exact Candidate SHA
BV14V411W7r5
accepted #85/#95/#97/#99/#101/#103/#105/#107/#109/#111 merge authorities
bundle transfer class + trust-anchor/wheel/provenance result
runtime_cache
direct public/Bilibili status class
sandbox + fd isolation
R008 containment
broker wire/framing result
harness result
protocol / stream_count / safe title_length
broker_status_class / broker_error_code / broker_request_count
process_error
unsupported_stage (fixed admitted value or n/a)
fallback_reason (fixed admitted value or n/a)
cleanup + safe-output scan
R1-R12
Overall
#68 readiness yes/no + reason
```

Never publish credentials, Secret headers/values, source/redirect/media URLs, signed query parameters, Cookie/Auth/token/profile/account state, raw stderr/exception/page/body/media payload.

## Freshness / Integration Contract

Semantic authority for this Attempt is exact merged main Candidate `942a0a1843f8f207332ac646f12ffe6ab5017306`.

Semantic freshness domains:

- `plugins/generic-ytdlp/**`;
- `scripts/generic-ytdlp-*`;
- SiteAdapter/ResolvedMedia normalization consumed by generic-ytdlp;
- #101/#105/#107/#109 error/stage/reason semantics and #111 response-normalization semantics;
- R008/broker/Secret containment;
- sandbox/fd isolation;
- accepted ARM64 target launch boundary.

Task-package docs committed after `942a0a18...` do not replace the runtime Candidate. Before claim, the Worker must prove the target runtime checkout is exactly `942a0a18...`.

If any accepted semantic change touches the freshness domains after this publication and before claim, STOP for Coordinator freshness review; do not silently substitute a later main.

## Stop boundary

```text
normal: [EXECUTION REPORT] → status:review → release owner → STOP
blocked: [BLOCKER REPORT] → status:blocked → release owner → STOP
```

Worker must not merge, mark done/close, implement a discovered blocker, create a downstream compatibility Task, or execute #68.