# Session Bootstrap — GENERIC-YTDLP-EXTRACT-PREP

You are executing the real extraction preparation Task for the accepted brokered generic-ytdlp runtime.

## Execution Context

```text
Repository: liqiangcc/jellyfin-web-media-gateway
GitHub Issue: #66
Task Contract: docs/tasks/66-generic-ytdlp-extract-prep/task.md
Expected worker: cloud-codex
Expected environment: env:cloud
Accepted runtime authority: #60
Downstream real-site verification: #67
Downstream Web E2E: #68
```

## Start

Actually read and obey:

1. `AGENTS.md`
2. Issue #66 and relevant comments
3. `docs/tasks/66-generic-ytdlp-extract-prep/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/freshness-integration-protocol.md`
6. #60 Final Acceptance and current generic-ytdlp runtime code
7. #39 Final Acceptance/current SiteAdapter authority
8. #14/R008 accepted security authority
9. #65 latest Coordinator Review explaining why the stale Bilibili-specific integration path is not the first-playback path

Claim only if live #66 remains:

```text
status:ready
env:cloud
no active owner
```

## Critical boundaries

- implement actual frozen yt-dlp extraction API flow, not another static probe;
- all extraction HTTP(S) must remain behind #60 broker/R008;
- target one muxed playable HTTP/HLS stream for the first playback milestone;
- do not revive #23-only NavigationContext/ResolveContext/DASH/expiry/site-specific error APIs;
- separate/non-muxed DASH is explicit unsupported in this Task;
- add only an explicit feature-gated/non-default runtime-enabled adapter construction seam;
- production `GenericYtdlpAdapter::default()` remains DisabledRunner;
- no real Bilibili request in #66;
- no login/Cookie/profile/proxy/fingerprint/CAPTCHA/access-control bypass;
- do not modify/merge #23/#37/#65;
- do not execute #67/#68;
- normal finish: `[EXECUTION REPORT] -> status:review -> release owner -> STOP`;
- blocker: `[BLOCKER REPORT] -> status:blocked -> release owner -> STOP`.
