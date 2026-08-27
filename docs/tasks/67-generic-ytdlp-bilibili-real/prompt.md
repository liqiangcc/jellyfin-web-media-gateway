# Session Bootstrap — GENERIC-YTDLP-BILIBILI-REAL

Execute Issue #67 using the repository Worker protocol.

## Read first

1. `AGENTS.md`
2. Issue #67 and all relevant comments, especially Attempt 5 authoritative `[BLOCKER REPORT]` and Coordinator `[SPLIT]` / dependency update
3. `docs/tasks/67-generic-ytdlp-bilibili-real/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/freshness-integration-protocol.md`
6. #95 Final Acceptance / PR #96 / ADR 0007
7. #85 Final Acceptance / PR #86
8. #83 Final Acceptance / PR #84
9. #79 Final Acceptance / PR #82
10. #63 Final Acceptance
11. #73 R2 Final Acceptance
12. #66 Final Acceptance
13. #60 / accepted R008 runtime-security authority
14. #90 accepted trusted exact-source transport Evidence when direct Target Git is unreliable

Claim only if live #67 is:

```text
status:ready
env:ubuntu-arm64
no active owner
```

## Frozen execution

```text
Contract Revision: R6
Attempt: 6
Exact Candidate: 804fd60343b081e5e055ba87f68e7939b106bb19
Target: accepted Ubuntu ARM64 phone / gateway-runner
Sample: BV14V411W7r5
Harness: scripts/generic-ytdlp-real-smoke.sh
```

## Why Attempt 6

Attempt 5 cleared offline runtime, ARM64 sandbox, #85 ENOSYS fd isolation and BrokerProcessRunner, reached real R008 broker traffic, and reproduced `BROKER_RESPONSE_SECRET_REJECTED` 2/2. #95 was split for that response-boundary blocker and is now Final Accepted/merged as `804fd60343b081e5e055ba87f68e7939b106bb19`.

Accepted #95 semantics remain frozen: request Secret material is rejected before prohibited side effects; origin response Secret headers remain Secret-classified, count against existing bounded header budget, are contained before BrokerResponse/IPC, create no cookie/auth state or replay, and safe status/body/non-Secret headers continue only when all other R008 checks pass.

## Goal

```text
exact Candidate 804fd603...
→ exact #79 bundle / offline cache hit-or-prepare
→ direct/no-proxy Bilibili reachability
→ scripts/generic-ytdlp-real-smoke.sh
→ accepted ARM64 sandbox
→ BrokerProcessRunner + #85 ENOSYS fallback
→ R008Broker + #95 response Secret containment
→ yt_dlp.extract_info(download=False)
→ safe PASS / CONDITIONAL PASS / FAIL / BLOCKED
```

## Critical boundaries

- verification-only; no repository/product/security policy changes;
- exact Candidate only: `804fd60343b081e5e055ba87f68e7939b106bb19`;
- accepted low-privilege Ubuntu ARM64 target only; no root/sudo/system package install;
- exact #79 offline runtime only; no Target package-index/source resolution or replacement wheel;
- formal Bilibili reachability is direct/no-proxy; artifact/source transfer is not site Evidence;
- extractor network remains R008Broker + BrokerProcessRunner; ARM64 sandbox/#85 fd isolation stay enabled;
- #95 containment stays exactly accepted: no Secret declassification, cookie/auth store/replay, R008 weakening, or response Secret exposure;
- run only accepted `scripts/generic-ytdlp-real-smoke.sh` harness;
- durable Evidence must not expose full signed/resolved media URLs, Cookie/Auth/token, response Secret header names/values, transfer credentials, raw worker stderr, page body or media payload;
- no DASH/remux/FFmpeg/navigation/Browser/Web E2E/production enablement/performance work;
- normal completion: `[EXECUTION REPORT] -> status:review -> release owner -> STOP`;
- blocker: `[BLOCKER REPORT] -> status:blocked -> release owner -> STOP`;
- never auto-start #68, set done, close Issue or change security limits.

## Result routing

```text
PASS + broker_request_count > 0 + protocol http-file|hls
→ Coordinator review
→ possible #67 Final Acceptance
→ then #68 publication

UNSUPPORTED_FORMAT / separate A/V after real brokered extraction
→ FAIL
→ Coordinator plans only smallest generic media-format capability required by Evidence

R008 / response containment / sandbox / SPAWN_FAILED / site / transfer / trust blocker
→ BLOCKED
→ preserve bounded Evidence
```
