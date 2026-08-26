# Session Bootstrap — GENERIC-YTDLP-BILIBILI-REAL

Execute Issue #67 using the repository Worker protocol.

## Read first

1. `AGENTS.md`
2. Issue #67 and all relevant comments, especially Attempt 1 blocker and Coordinator review
3. `docs/tasks/67-generic-ytdlp-bilibili-real/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/freshness-integration-protocol.md`
6. #63 Final Acceptance
7. #73 R2 Final Acceptance and PR #77 accepted merge
8. #66 Final Acceptance
9. #60 / R008 accepted runtime-security authority
10. `docs/product-roadmap.md`

Claim only if live #67 is:

```text
status:ready
env:ubuntu-arm64
no active owner
```

## Frozen execution

```text
Attempt: 2
Exact Candidate: f2c8736ea705ebf942da833550fe96182b377813
Target: accepted Ubuntu ARM64 phone / gateway-runner
Sample: BV14V411W7r5
Harness: scripts/generic-ytdlp-real-smoke.sh
```

## Critical boundaries

- verification-only; do not modify repository/product/security policy;
- exact checkout `f2c8736...`; do not silently use moving main;
- low-privilege accepted target only; no root/sudo/system package install;
- J1 formal site reachability must be direct/no-proxy and bounded;
- do **not** pre-clear ordinary setup proxy variables before the harness solely to force fixed dependency acquisition direct;
- the accepted #73 R2 harness may use the setup process's ordinary route/proxy only to prepare the repository-fixed yt-dlp dependency, then must scrub proxy state before extractor runtime;
- setup routing is not Bilibili/site Evidence; all yt-dlp extractor HTTP(S) remains R008Broker + BrokerProcessRunner authority;
- use only the accepted harness/cache path; no ad-hoc yt-dlp CLI/Python/Rust substitute or global fallback;
- exact frozen upstream remains `yt-dlp 2026.08.19@3a08beaf031ab68f966401ead017ac81fe8486cf`;
- preserve only bounded safe summary fields, including `runtime_cache`, broker status/error/request count, protocol and stream count;
- never durable-log full source/resolved/signed media URL, Cookie/Auth/token, setup proxy credentials, setup logs, raw worker stderr, page body or media payload;
- verified final user-owned cache may persist for warm reuse; staging/process/media payload must not persist;
- if `UNSUPPORTED_FORMAT`, broker policy/limit error, cache/provenance failure or site challenge occurs, classify and report; do not fix it in this Task;
- no DASH/remux/FFmpeg/navigation/Browser/Web E2E/production enablement/performance scope;
- normal completion: `[EXECUTION REPORT] -> status:review -> release owner -> STOP`;
- blocker: `[BLOCKER REPORT] -> status:blocked -> release owner -> STOP`;
- never auto-start #68, set done, close Issue or change security limits.

## Key acceptance distinction

```text
Dependency setup network
!=
Formal Bilibili site Evidence
!=
Extractor network authority
```

Formal site Evidence must be direct/no-proxy. The fixed dependency setup may use the ordinary setup route. Extractor traffic must still be brokered by R008, and a successful compatibility determination should show `broker_request_count > 0`.