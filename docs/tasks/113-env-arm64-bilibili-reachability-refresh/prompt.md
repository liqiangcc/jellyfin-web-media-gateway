# Session Bootstrap — ENV-ARM64-BILIBILI-REACHABILITY-REFRESH R3

Execute Issue #113 Contract Revision R3 / Attempt 4 using the repository Worker protocol.

## Claim gate

Claim only if live #113 is exactly:

```text
OPEN
status:ready
env:ubuntu-arm64
no active owner
```

Otherwise STOP.

Before claim read:

- `AGENTS.md`;
- live #113 and relevant comments, especially Attempt 3 BLOCK + R3 Contract Revision;
- `docs/tasks/113-env-arm64-bilibili-reachability-refresh/task.md`;
- `.agents/skills/task-worker/SKILL.md` and lifecycle protocols;
- #128 Final Acceptance and `docs/tasks/128-env-bilibili-reachability-obs-prep/usage.md`;
- accepted `scripts/reachability_observation_sanitizer.py`;
- #67 R17 blocker authority and frozen runtime Candidate `80fb081b129f8f664124b84ddcc9698039e2cfd1`.

## Goal

Run one bounded ordinary-path reachability refresh of frozen selector `BV14V411W7r5` with privacy-safe same-run endpoint correlation.

This is reachability verification only. Do not execute yt-dlp, generic-ytdlp, resolver, #67 J3 or #68.

## Exact execution boundary

Use the accepted phone / low-privilege `gateway-runner` target and accepted `setpriv ... env -i` boundary.

Before probe 1, start exactly one #128 sanitizer process and keep it alive for the whole Attempt. All three probe observations, if all are needed, must use that same sanitizer process so `endpoint_alias` values are comparable only inside this Attempt.

Raw endpoint values must flow only through a direct pipe/FD/in-memory path into sanitizer stdin. Do not put them in argv, shell trace, temp files, retained stdout/stderr, Issue comments or artifacts. Do not emit/persist the sanitizer run key.

For each probe:

- clear all proxy variables and use `curl --noproxy '*'`;
- same frozen URL and same ordinary request shape only;
- no Cookie/Auth/login/profile;
- no custom UA/fingerprint/Referer/header variation;
- no `--resolve`, DNS pinning, endpoint/family/interface forcing or resolver steering;
- no alternate URL/sample;
- discard response body and response headers;
- pass only passive remote-endpoint / HTTP-version / status / optional timing observation fields directly to the sanitizer;
- durably retain only the sanitizer's bounded record plus probe index.

Run at most 3 probes in one bounded set. Stop early only after two consecutive `2xx`. Use a fixed bounded delay and never adapt the request to status/alias/family observations.

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

Alias/family/http-version/timing observations are diagnostic only and cannot change eligibility.

## Evidence boundary

Per probe report only:

```text
probe_index
status_class
family
endpoint_alias
http_version_class
timing_bucket
```

Plus target class, frozen selector, direct/no-proxy network class, eligibility yes/no, cleanup/safe-output and Overall.

Never publish raw endpoint/DNS values, sanitizer key, raw pre-sanitized curl write-out, response body/header content, request headers, Cookie/Auth/token/profile state or challenge details.

## Stop boundary

Before every terminal Issue mutation, follow the current fresh terminal-write authority guard.

PASS:
`[EXECUTION REPORT] -> status:review -> release owner -> STOP`.

BLOCKED:
`[BLOCKER REPORT] -> status:blocked -> release owner -> STOP`.

Do not run a second bounded set. Do not merge/close/done, execute #67/#68, vary request identity/path, or create another Task.
