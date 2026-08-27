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

## Read first

1. `AGENTS.md`
2. Issue #67 and all relevant comments, especially Attempt 6 authoritative `[BLOCKER REPORT]` and the #97 dependency update
3. `docs/tasks/67-generic-ytdlp-bilibili-real/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/freshness-integration-protocol.md`
6. #97 Final Acceptance / PR #98 merge authority
7. #95 Final Acceptance / ADR 0007 response Secret containment
8. #85 and #83 accepted fd-isolation/sandbox authority
9. #79 offline runtime identity and trust anchor
10. #63 accepted Ubuntu ARM64 target authority
11. #73/#66/#60 accepted harness/extraction/broker runtime authority

Do not infer live state from this prompt; verify GitHub live state before claim.

## Why Attempt 7

Attempt 6 proved the real Target path reached accepted R008 traffic and #95 containment but then failed 2/2 with:

```text
broker_status_class: 2xx
broker_error_code: n/a
broker_request_count: 1
process_error: BROKER_PROTOCOL
```

#97 proved and fixed the protocol-local root cause: decimal JSON encoding of binary response bodies could overflow the old 128 KiB broker frame. The accepted fix is now merged as:

```text
d9c038547ed2df695571f8dd4f732bdcdd4d5c19
```

Attempt 7 must verify that the former `BROKER_PROTOCOL` edge is cleared on the real frozen Bilibili path and continue to current `ResolvedMedia` classification.

## Required path

```text
exact #79 offline bundle
→ Target verify/offline cache hit-or-prepare
→ direct/no-proxy Bilibili reachability
→ scripts/generic-ytdlp-real-smoke.sh
→ accepted ARM64 sandbox
→ BrokerProcessRunner + #85 ENOSYS fd fallback
→ R008Broker + #95 response Secret containment
→ #97 bounded broker wire/framing
→ yt_dlp.extract_info(download=False)
→ GenericYtdlpAdapter
→ current ResolvedMedia
```

## Hard boundaries

- verification-only: do not modify repository/product/security code;
- execute exact Candidate `d9c038547ed2df695571f8dd4f732bdcdd4d5c19`, not moving main;
- no root/sudo/system package installation;
- no Target-side source/package-index resolution or replacement yt-dlp wheel;
- use only accepted #79 runtime/provenance;
- formal Bilibili Evidence is direct/no-proxy only;
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy rotation/access-control bypass;
- do not weaken R008 HTTP limits, #95 Secret containment, #97 wire bounds, #83 sandbox or #85 fd isolation;
- do not give the worker direct network authority or alternate socket/tunnel;
- do not expose response Secret header names/values, full resolved/signed URLs, query tokens, raw stderr, page body or media payload;
- no DASH/remux/FFmpeg/navigation/Browser/Web E2E/performance work;
- production `GenericYtdlpAdapter::default()` remains `DisabledRunner`;
- do not execute #68.

## J0-J4

Execute the exact verification matrix from `task.md`:

```text
J0 exact Candidate / Target identity / accepted bundle provisioning
J1 #79 trust anchor + offline runtime/cache verification
J2 direct/no-bypass public + frozen Bilibili reachability
J3 accepted real-site smoke through #83 + #85 + R008 + #95 + #97
J4 cleanup + safe-output leak scan
```

Use only:

```text
YTDLP_OFFLINE_BUNDLE=<verified-bundle-path> \
  scripts/generic-ytdlp-real-smoke.sh \
  'https://www.bilibili.com/video/BV14V411W7r5/'
```

No ad-hoc yt-dlp/Python/Rust substitute is allowed.

## Decisive Attempt-7 signals

```text
runtime_cache: offline-hit | offline-prepared
process_error != SANDBOX_UNAVAILABLE
process_error != SPAWN_FAILED
process_error != BROKER_PROTOCOL
broker_request_count > 0
broker_error_code != BROKER_RESPONSE_SECRET_REJECTED
```

If a bounded blocker repeats, preserve it and STOP. Do not fix it inside #67.

If the accepted path reaches media classification, report whether the current first-playback contract receives:

```text
protocol: http-file | hls | n/a
stream_count: <bounded integer>
Overall: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

Do not publish actual media URLs.

## Required report

Report Claims R1-R9 from `task.md`, including:

- exact Candidate and upstream authority verification;
- direct/no-proxy reachability;
- sandbox/fd isolation integrity;
- #95 response containment;
- #97 broker wire continuity;
- current `ResolvedMedia` compatibility;
- Secret/evidence boundary;
- cleanup/target safety;
- downstream #68 readiness yes/no + reason.

## Stop boundary

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

Worker must not merge, set `status:done`, close #67, implement a discovered blocker, re-plan the Task, or start #68.
