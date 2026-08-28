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
Contract Revision: R11
Attempt: 11
Exact Candidate: 1a38e403a3252239822aeb2a784a20fdfd18c0a6
Target: accepted Ubuntu ARM64 phone / gateway-runner
Sample: BV14V411W7r5
Harness: scripts/generic-ytdlp-real-smoke.sh
```

Formal R11 bootstrap authority is established only after old R10 execution authority has been revoked and the Coordinator Publication Gate publishes the Issue as a fresh ready Attempt. Pre-Publication-Gate materialization is non-executable history.

Read `AGENTS.md`, Issue #67/comments, `task.md`, lifecycle/freshness protocols, #105 Final Acceptance, #103/#101/#99/#97 Final Acceptance, and accepted #95/#85/#83/#79/#63/#73/#66/#60 authorities before claim.

Attempt 10 preserved the exact runtime and Target identity, reached three 2xx broker requests, but still returned `process_error: EXTRACTOR_FAILURE` with no ResolvedMedia. #105 Attempt 3 is Final Accepted and PR #106 is merged; run only against exact merged Candidate `1a38e403a3252239822aeb2a784a20fdfd18c0a6`.

#105 authority now preserves the production-shaped path:

```text
GenericYtdlpAdapter::resolve_detailed()
→ ProcessRunner::run()
→ normal extract first
→ only on frozen BiliBiliIE missing-initial-state admission
→ bounded webpage/nav/view/detail/playurl continuation
→ current ResolvedMedia
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
→ current ResolvedMedia
```

Hard boundaries:
- verification-only; no implementation changes;
- exact Candidate only, not moving main;
- no root/sudo/system install or Target package-index resolution;
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy/bypass;
- no R008/#101/#99/#95/#97/#83/#85/#105 weakening;
- no direct worker network/alternate socket;
- no Secret/full signed URL/raw stderr/exception text/page/media payload in Evidence;
- no DASH/remux/FFmpeg/navigation/Browser/Web E2E/performance;
- production default remains DisabledRunner;
- do not execute #68.

Run J0-J4 exactly from `task.md`. The only real-site harness is:

```text
YTDLP_OFFLINE_BUNDLE="$BUNDLE_PATH" \
  scripts/generic-ytdlp-real-smoke.sh \
  'https://www.bilibili.com/video/BV14V411W7r5/'
```

Decisive Attempt-11 signals:

```text
runtime_cache: offline-hit | offline-prepared
process_error != SANDBOX_UNAVAILABLE
process_error != SPAWN_FAILED
process_error != BROKER_PROTOCOL
process_error != NONZERO_EXIT
broker_request_count > 0
broker_error_code != BROKER_RESPONSE_SECRET_REJECTED
```

The decisive new question is whether the accepted #105 normal-extract continuation turns the former stable `EXTRACTOR_FAILURE` into a valid current muxed `http-file | hls` ResolvedMedia on frozen sample `BV14V411W7r5`.

If no ResolvedMedia is produced, require one fixed #101 classification and report only that code. For a repeated `EXTRACTOR_FAILURE` or `UNSUPPORTED_FORMAT`, post a bounded report and a precise next split recommendation; never inspect or publish raw stderr/exception text, and never claim Bilibili fixed.

If a bounded blocker repeats, report it and STOP; do not fix it in #67. If media classification is reached, report only bounded protocol/stream/title fields and Overall `PASS | CONDITIONAL PASS | FAIL | BLOCKED`.

Report Claims R1-R9 from `task.md` and downstream #68 readiness.

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

Worker must not merge, set `status:done`, close #67, implement a discovered blocker, or start #68.
