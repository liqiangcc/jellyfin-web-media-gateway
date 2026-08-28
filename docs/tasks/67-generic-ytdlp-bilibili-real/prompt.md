# Session Bootstrap — GENERIC-YTDLP-BILIBILI-REAL

Execute Issue #67 using the repository Worker protocol.

## Claim gate

Claim only if live #67 is exactly:

```text
status:ready
env:ubuntu-arm64
no active owner
```

If Issue state is `status:draft`, `status:blocked`, `status:review`, or already owned, STOP.

## Frozen execution

```text
Contract Revision: R12
Attempt: 12
Exact Candidate: 234c616f128deaee55156675d480d03ac5e8670d
Target: accepted Ubuntu ARM64 phone / gateway-runner
Sample: BV14V411W7r5
Harness: scripts/generic-ytdlp-real-smoke.sh
```

This R12 bootstrap is non-executable until Coordinator Publication Gate publishes the Issue as a fresh ready Attempt.

Read `AGENTS.md`, Issue #67/comments, `task.md`, lifecycle/freshness protocols, #107 Final Acceptance, #105 Final Acceptance, #103/#101/#99/#97 Final Acceptance, and accepted #95/#85/#83/#79/#63/#73/#66/#60 authorities before claim.

Attempt 11 executed exact Candidate `1a38e403a3252239822aeb2a784a20fdfd18c0a6` on the accepted low-privilege ARM64 target. The runtime/security/broker path remained healthy and reached four 2xx broker requests, but the bounded result was `process_error: UNSUPPORTED_FORMAT` with no current ResolvedMedia. That result did not prove DASH, separate A/V, or any other concrete media-format cause.

#107 is now Final Accepted and PR #108 is merged as exact Candidate `234c616f128deaee55156675d480d03ac5e8670d`. It preserves top-level `UNSUPPORTED_FORMAT` and adds only one closed repository-owned secondary stage:

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

No arbitrary diagnostic string is authorized. Do not infer a concrete media-format cause merely from one of these stages.

#105 authority still preserves the production-shaped path:

```text
GenericYtdlpAdapter::resolve_detailed()
→ ProcessRunner::run()
→ normal extract first
→ only on frozen BiliBiliIE missing-initial-state admission
→ bounded webpage/nav/view/detail/playurl continuation
→ #107 bounded unsupported-stage attribution when current contract rejects
→ current ResolvedMedia OR bounded unsupported result
```

There is no caller-selectable fallback action. Do not broaden or bypass #105 admission inside #67.

Required path:

```text
#79 offline runtime
→ direct/no-proxy frozen Bilibili sample
→ #99 exact-Candidate clean-build sibling binding
→ accepted ARM64 sandbox
→ #85 fd fallback
→ R008 + #95 response containment
→ #97 bounded broker framing
→ #101 bounded worker/extractor outcome envelope
→ normal frozen yt_dlp.extract_info(download=False)
→ #105 bounded continuation when exact admission matches
→ #107 closed unsupported-stage attribution if unsupported
→ current ResolvedMedia OR bounded actionable result
```

Hard boundaries:
- verification-only; no implementation changes;
- exact Candidate only, not moving main/package head;
- no root/sudo/system install or Target package-index resolution;
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy/bypass;
- no R008/#101/#99/#95/#97/#83/#85/#105/#107 weakening;
- no direct worker network/alternate socket;
- no Secret/full signed URL/raw stderr/exception text/page/media payload in Evidence;
- no DASH/remux/FFmpeg/navigation/Browser/Web E2E/performance;
- no interpretation of `MEDIA_SHAPE` as DASH/separate-A/V without separate bounded evidence;
- production default remains DisabledRunner;
- do not execute #68 or create a downstream compatibility Task.

Run J0-J4 exactly from `task.md`. The only real-site harness is:

```text
YTDLP_OFFLINE_BUNDLE="$BUNDLE_PATH" \
  scripts/generic-ytdlp-real-smoke.sh \
  'https://www.bilibili.com/video/BV14V411W7r5/'
```

Decisive Attempt-12 signals:

```text
runtime_cache: offline-hit | offline-prepared
process_error != SANDBOX_UNAVAILABLE
process_error != SPAWN_FAILED
process_error != BROKER_PROTOCOL
process_error != NONZERO_EXIT
broker_request_count > 0
broker_error_code != BROKER_RESPONSE_SECRET_REJECTED
```

The decisive question is now two-part:

```text
Can the frozen sample produce a valid current muxed http-file | hls ResolvedMedia?
OR, if not, which exact closed #107 unsupported_stage owns the bounded rejection?
```

If `process_error == UNSUPPORTED_FORMAT`, report exactly one admitted stage from the list above. Do not inspect or publish raw stderr/exception text, page data, URLs, headers, tokens, or media payloads to explain it. If the exact Candidate returns unsupported without an admitted stage, report that as a bounded evidence/instrumentation blocker and STOP; do not patch #67.

If a different bounded #101 blocker occurs, report only the fixed process code and STOP; do not repair it in #67.

If media classification succeeds, report only bounded protocol/stream/title fields and Overall `PASS | CONDITIONAL PASS | FAIL | BLOCKED`.

Report Claims R1-R10 from `task.md` and downstream #68 readiness.

This prompt is execution authority only after Coordinator Publication Gate records PUBLISH and live #67 is `status:ready + env:ubuntu-arm64 + no active owner`. Use the exact accepted low-privilege ARM64 `setpriv --reuid=999 --regid=995 --groups=995,3003 --inh-caps=-all --ambient-caps=-all --bounding-set=-all -- env -i` shell with `HOME=/home/gateway-runner USER=gateway-runner LOGNAME=gateway-runner PATH=/home/gateway-runner/.cargo/bin:/usr/local/bin:/usr/bin:/bin`; do not replace it with root/capsh/inherited environment.

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

Worker must not merge, set `status:done`, close #67, implement a discovered blocker, create a downstream compatibility Task, or start #68.
