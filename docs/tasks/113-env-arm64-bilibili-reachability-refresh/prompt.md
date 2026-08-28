# Session Bootstrap — ENV-ARM64-BILIBILI-REACHABILITY-REFRESH

Execute Issue #113 using the repository Worker protocol.

## Claim gate

Claim only if live #113 is exactly:

```text
OPEN
status:ready
env:ubuntu-arm64
no active owner
```

Otherwise STOP.

Read `AGENTS.md`, live #113, `docs/tasks/113-env-arm64-bilibili-reachability-refresh/task.md`, lifecycle protocols, #67 R15 blocker review, #63 Final Acceptance and #36 site-reachability boundary before claim.

## Goal

Only refresh normal-network reachability of the unchanged frozen sample:

```text
BV14V411W7r5
https://www.bilibili.com/video/BV14V411W7r5/
```

Do not execute yt-dlp, generic-ytdlp, #67 J3 or #68.

## Probe contract

Use the accepted low-privilege `gateway-runner` target and exact `setpriv ... env -i` boundary.

For every probe:
- clear all proxy variables;
- use `curl --noproxy '*'`;
- identical ordinary request shape only;
- no Cookie/Authorization/login/profile;
- no custom User-Agent/fingerprint/Referer/header rotation;
- no alternate URL/sample;
- discard response body and do not retain response headers;
- record only the status class.

Run at most 3 probes. Stop early after two consecutive `2xx` results. Use a small fixed bounded delay between repeats; do not adapt the request to a 4xx.

## Result

PASS only if two consecutive probes are `2xx`:

```text
BILIBILI_HOST_ELIGIBLE_FOR_#67_REFRESH=yes
```

Otherwise:

```text
BILIBILI_HOST_ELIGIBLE_FOR_#67_REFRESH=no
Overall: BLOCKED
```

A BLOCKED result must not trigger proxy/fingerprint/Cookie/challenge/bypass behavior.

## Evidence boundary

Report only target class, frozen selector, direct/no-proxy network class, bounded probe status classes, eligibility yes/no, cleanup and Overall.

Never publish body/header content, Cookie/Auth/token/profile state, URL query material, challenge details or media payload.

## Stop boundary

PASS:
`[EXECUTION REPORT] → status:review → release owner → STOP`.

BLOCKED:
`[BLOCKER REPORT] → status:blocked → release owner → STOP`.

Do not merge/close/done, do not execute #67/#68, and do not create another Task.
