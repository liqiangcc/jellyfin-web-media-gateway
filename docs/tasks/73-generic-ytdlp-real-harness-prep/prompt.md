# Session Bootstrap — GENERIC-YTDLP-REAL-HARNESS-PREP R2

Execute reopened Issue #73 Attempt 2 using the repository Worker protocol.

## Read first

1. `AGENTS.md`
2. Issue #73 and all relevant comments, especially `[COORDINATOR REOPEN]`
3. #67 Attempt 1 `[BLOCKER REPORT]`
4. `docs/tasks/73-generic-ytdlp-real-harness-prep/task.md`
5. `docs/tasks/issue-lifecycle-protocol.md`
6. `docs/tasks/freshness-integration-protocol.md`
7. #66 Final Acceptance / accepted extraction implementation
8. #60 Final Acceptance
9. current R008 / `gateway-egress` authority
10. parallel #71/#75 ownership boundaries

Claim only if live #73 is still:

```text
status:ready
env:cloud
no active owner
```

Start **Attempt 2**.

## R2 target problem

#67 proved:

```text
direct public HTTPS: HTTP 200
direct Bilibili page: HTTP 200
accepted harness: BLOCKED / FROZEN_RUNTIME_SETUP
broker_request_count: 0
```

Do not treat this as a Bilibili/extractor failure. Fix only the frozen runtime preparation/reuse layer owned by #73.

## Critical boundaries

- no real Bilibili/site request in #73;
- add durable user-owned exact-version+commit cache with atomic prepare/verify/reuse;
- supply-chain acquisition may use the setup process's ordinary outbound/proxy environment only for acquiring the fixed frozen yt-dlp source;
- never persist/log setup proxy URL or credentials;
- before extractor runtime, scrub HTTP_PROXY/HTTPS_PROXY/ALL_PROXY and lowercase equivalents;
- actual extractor HTTP(S) remains only `R008Broker`; do not add a second/direct/open-proxy client;
- no caller-controlled yt-dlp source/version/commit/executable/argv/config/plugin/profile/Cookie/Auth/format selector;
- no global yt-dlp fallback;
- corrupt/partial/mismatched cache fails closed;
- preserve safe summary/raw-stderr suppression;
- do not increase R008 96 KiB limit or weaken DNS/public-IP/pinning/TLS/redirect/Secret policy;
- production `GenericYtdlpAdapter::default()` / normal registry remains disabled;
- do not modify #71 navigation surfaces or #75 Browser/Chromium surfaces;
- do not execute #67/#68;
- normal finish: `[EXECUTION REPORT] -> status:review -> release owner -> STOP`;
- blocker: `[BLOCKER REPORT] -> status:blocked -> release owner -> STOP`.

Required exact-Candidate J1-J4 must prove cold prepare, warm/offline cache reuse, setup-vs-runtime proxy separation, provenance validation, corrupt/partial cache rejection, safe output, cleanup and existing #60/#66/R008 regressions.
