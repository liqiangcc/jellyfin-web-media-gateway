# Session Bootstrap — GENERIC-YTDLP-BILIBILI-REAL

Execute Issue #67 using the repository Worker protocol.

## Claim gate

Claim only if live #67 is `status:ready + env:ubuntu-arm64 + no active owner`.

## Frozen execution

```text
Contract Revision: R6
Attempt: 6
Exact Candidate: 804fd60343b081e5e055ba87f68e7939b106bb19
Target: accepted Ubuntu ARM64 phone / gateway-runner
Sample: BV14V411W7r5
Harness: scripts/generic-ytdlp-real-smoke.sh
```

Read `AGENTS.md`, Issue #67 comments, `task.md`, lifecycle/freshness protocols, and accepted #95/#85/#83/#79/#63/#73/#66/#60/#90 authorities before claim.

Attempt 5 cleared offline runtime, ARM64 sandbox, #85 ENOSYS fd isolation and BrokerProcessRunner, reached real R008 broker traffic, and reproduced `BROKER_RESPONSE_SECRET_REJECTED` 2/2. #95 was split for that independent response-boundary blocker and is now Final Accepted/merged as exact Candidate `804fd60343b081e5e055ba87f68e7939b106bb19`.

Accepted #95 semantics remain frozen: request Secret material is rejected before prohibited side effects; origin response Secret headers remain Secret-classified, count against existing bounded header budget, are contained before BrokerResponse/IPC, create no cookie/auth state or replay, and safe status/body/non-Secret headers continue only when all other R008 checks pass.

## Required path

```text
exact #79 offline bundle
→ direct/no-proxy Bilibili reachability
→ accepted harness
→ ARM64 sandbox
→ BrokerProcessRunner + #85 ENOSYS fallback
→ R008Broker + #95 response Secret containment
→ yt_dlp.extract_info(download=False)
→ current ResolvedMedia
```

Decisive checks:

```text
runtime_cache: offline-hit | offline-prepared
process_error != SANDBOX_UNAVAILABLE
process_error != SPAWN_FAILED
broker_request_count > 0
broker_error_code != BROKER_RESPONSE_SECRET_REJECTED
```

Do not alter repository/product/security code, weaken R008/sandbox/fd-isolation, declassify response Secrets, create/replay cookie/auth state, use login/Cookie/profile/proxy bypass, substitute another Candidate/runtime/harness, or enter DASH/remux/FFmpeg/navigation/Browser/Web-E2E/performance work.

Durable Evidence must not expose full signed/resolved media URLs, Cookie/Auth/token, response Secret header names/values, transfer credentials, raw worker stderr, page body or media payload.

Normal completion: `[EXECUTION REPORT] -> status:review -> release owner -> STOP`.
Blocker: `[BLOCKER REPORT] -> status:blocked -> release owner -> STOP`.
Never start #68, mark done or close #67.
