# Session Bootstrap — GENERIC-YTDLP-REAL-HARNESS-PREP

Execute Issue #73 using the repository Worker protocol.

## Read first

1. `AGENTS.md`
2. Issue #73 and relevant comments
3. `docs/tasks/73-generic-ytdlp-real-harness-prep/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/freshness-integration-protocol.md`
6. #66 Final Acceptance and PR #70 accepted implementation
7. #60 Final Acceptance
8. current R008 / `gateway-egress` authority
9. `docs/product-roadmap.md`

Claim only if live #73 is still:

```text
status:ready
env:cloud
no active owner
```

## Critical boundaries

- harness/prep only; **no real Bilibili request**;
- use real `R008Broker` for the verification runtime path;
- no second/direct/open-proxy network client for yt-dlp extractor traffic;
- no caller executable/argv/config/plugin/profile/Cookie/Auth/proxy/format-selector authority;
- do not increase R008 96 KiB body limit or weaken DNS/public-IP/pinning/TLS/redirect/Secret policy;
- safe output only: never durable-log full source/resolved URL, signed query, Cookie/Auth/token or media payload;
- raw worker stderr remains suppressed;
- production `GenericYtdlpAdapter::default()` / normal registry remains disabled;
- do not execute #67/#68;
- normal finish: `[EXECUTION REPORT] -> status:review -> release owner -> STOP`;
- blocker: `[BLOCKER REPORT] -> status:blocked -> release owner -> STOP`.

The output of #73 is a durable, safe command/harness for later #67 real-site verification, not real-site Evidence itself.
