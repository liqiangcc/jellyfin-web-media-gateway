# Session Bootstrap — GENERIC-YTDLP-BILIBILI-REAL

Execute Issue #67 using the repository Worker protocol.

## Read first

1. `AGENTS.md`
2. Issue #67 and relevant comments
3. `docs/tasks/67-generic-ytdlp-bilibili-real/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. #63 Final Acceptance
6. #73 Final Acceptance and accepted PR #74 merge
7. #66 Final Acceptance
8. #60 / R008 accepted runtime-security authority
9. `docs/product-roadmap.md`

Claim only if live #67 remains:

```text
status:ready
env:ubuntu-arm64
no active owner
```

## Frozen execution

```text
Exact Candidate: 826d02c22105ee1877ae79706d2cb03112f995a9
Target: accepted Ubuntu ARM64 phone / gateway-runner
Sample: BV14V411W7r5
Harness: scripts/generic-ytdlp-real-smoke.sh
```

## Critical boundaries

- verification-only; do not modify repository/product/security policy;
- exact checkout `826d02c...`; do not silently use moving main;
- preflight user-owned Python/pip + accepted target identity; no root/sudo/package install;
- direct/no-proxy route only for formal real-site Evidence;
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy rotation/access-control bypass;
- run only the accepted #73 harness for real extraction; no ad-hoc yt-dlp CLI/Python/Rust substitute;
- all extractor HTTP(S) remains R008Broker + BrokerProcessRunner authority;
- preserve only harness safe summary; never durable-log full resolved/signed media URL, Cookie/Auth/token/page/media payload/raw worker stderr;
- if `UNSUPPORTED_FORMAT`, broker policy/limit error, pip/setup failure or site challenge occurs, classify and report; do not fix it in this Task;
- no DASH/remux/FFmpeg/navigation/Browser/Web E2E/production enablement/performance scope;
- normal completion: `[EXECUTION REPORT] -> status:review -> release owner -> STOP`;
- blocker: `[BLOCKER REPORT] -> status:blocked -> release owner -> STOP`;
- never auto-start #68, set done, close Issue or change security limits.