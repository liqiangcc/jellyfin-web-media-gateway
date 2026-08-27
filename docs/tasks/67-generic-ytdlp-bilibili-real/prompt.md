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
Contract Revision: R7
Attempt: 7
Exact Candidate: d9c038547ed2df695571f8dd4f732bdcdd4d5c19
Target: accepted Ubuntu ARM64 phone / gateway-runner
Sample: BV14V411W7r5
Harness: scripts/generic-ytdlp-real-smoke.sh
```

Read `AGENTS.md`, Issue #67/comments, `task.md`, lifecycle/freshness protocols, #97 Final Acceptance, and accepted #95/#85/#83/#79/#63/#73/#66/#60 authorities before claim.

Attempt 6 reached R008 2xx with `broker_request_count: 1` but reproduced `process_error: BROKER_PROTOCOL` 2/2. #97 proved the root cause and was Final Accepted/merged as `d9c038547ed2df695571f8dd4f732bdcdd4d5c19`.

Required path:

```text
#79 offline runtime
→ direct/no-proxy frozen Bilibili sample
→ accepted ARM64 sandbox
→ #85 fd fallback
→ R008 + #95 response containment
→ #97 bounded broker framing
→ yt_dlp.extract_info(download=False)
→ current ResolvedMedia
```

Hard boundaries:
- verification-only; no implementation changes;
- exact Candidate only, not moving main;
- no root/sudo/system install or Target package-index resolution;
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy/bypass;
- no R008/#95/#97/#83/#85 weakening;
- no direct worker network/alternate socket;
- no Secret/full signed URL/raw stderr/page/media payload in Evidence;
- no DASH/remux/FFmpeg/navigation/Browser/Web E2E/performance;
- production default remains DisabledRunner;
- do not execute #68.

Run J0-J4 exactly from `task.md`. The only real-site harness is:

```text
YTDLP_OFFLINE_BUNDLE=<verified-bundle-path> \
  scripts/generic-ytdlp-real-smoke.sh \
  'https://www.bilibili.com/video/BV14V411W7r5/'
```

Decisive Attempt-7 signals:

```text
runtime_cache: offline-hit | offline-prepared
process_error != SANDBOX_UNAVAILABLE
process_error != SPAWN_FAILED
process_error != BROKER_PROTOCOL
broker_request_count > 0
broker_error_code != BROKER_RESPONSE_SECRET_REJECTED
```

If a bounded blocker repeats, report it and STOP; do not fix it in #67. If media classification is reached, report only bounded protocol/stream/title fields and Overall `PASS | CONDITIONAL PASS | FAIL | BLOCKED`.

Report Claims R1-R9 from `task.md` and downstream #68 readiness.

This prompt is execution authority only after Coordinator Publication Gate records PUBLISH and live #67 is `status:ready + env:ubuntu-arm64 + no active owner`.

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
