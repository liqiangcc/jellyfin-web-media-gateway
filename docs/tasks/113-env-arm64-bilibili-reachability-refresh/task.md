# Task — ENV-ARM64-BILIBILI-REACHABILITY-REFRESH

## Metadata

```text
GitHub Issue: #113
Task ID: ENV-ARM64-BILIBILI-REACHABILITY-REFRESH
Task kind: verification-only / environment + public-site reachability
Contract Revision: R3
Attempt: 4
Parent: #67 GENERIC-YTDLP-BILIBILI-REAL / R17 Attempt 17
Planning Base: 1f9b186b93aa2c680f74cc7a524dfa348c7f007c
Accepted observability authority: #128 Final Acceptance / merge 1f9b186b93aa2c680f74cc7a524dfa348c7f007c
Preferred worker: ubuntu-arm64
Eligible environment: env:ubuntu-arm64
Target: accepted Ubuntu ARM64 phone / gateway-runner
Frozen sample: BV14V411W7r5
Publication state: non-executable until Coordinator R3 Publication Gate passes
```

## Trigger

#113 R2 Attempts 2 and 3 both used the accepted phone, identical direct/no-proxy ordinary request shape and one bounded three-probe set. Both returned:

```text
2xx -> 4xx -> 2xx
BILIBILI_HOST_ELIGIBLE_FOR_#67_REFRESH=no
```

The latest Attempt 3 Coordinator Review classified this as environment/site-reachability BLOCKED Evidence only, prohibited immediate status-only repetition/request variation, and directed planning to diagnose ordinary-path instability without bypass.

#128 is now Final Accepted. Its accepted helper `scripts/reachability_observation_sanitizer.py` is pure/offline and can transform already-observed local connection metadata into bounded same-run diagnostic Evidence without exposing raw endpoint identities or changing HTTP behavior.

The #67 runtime Candidate remains frozen at:

```text
80fb081b129f8f664124b84ddcc9698039e2cfd1
```

R3 still does not execute or evaluate generic-ytdlp compatibility.

## Goal

Perform one later Coordinator-authorized bounded refresh of the unchanged frozen public Bilibili page on the accepted phone, while passively correlating each status result with privacy-safe same-run endpoint/address-family metadata.

The diagnostic metadata is explanatory only. Eligibility semantics are unchanged.

## Exact probe contract

Use the accepted low-privilege `gateway-runner` target identity and the accepted `setpriv ... env -i` boundary with the target's normal curl.

Frozen URL:

```text
https://www.bilibili.com/video/BV14V411W7r5/
```

For the entire Attempt:

1. Start exactly one accepted #128 sanitizer process/run context before probe 1 and keep that same process alive through the final probe so aliases are comparable only within this bounded Attempt.
2. The sanitizer runtime key is ephemeral process-local state and must never be emitted, persisted or passed through argv.
3. Raw curl observation fields, including the remote endpoint value, must flow directly through a pipe/FD/in-memory boundary into sanitizer stdin. They MUST NOT be tee'd, echoed, shell-traced, placed in argv, written to temp files, retained logs, artifacts or Issue comments.
4. Curl stderr must not be durably retained. Use a bounded generic transport result/classification rather than publishing raw curl diagnostics that could contain endpoint or request details.

For every probe:

- clear `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, `http_proxy`, `https_proxy`, `all_proxy`;
- use `curl --noproxy '*'`;
- use the unchanged frozen URL and the same ordinary request shape;
- do not add Cookie, Authorization, Referer, custom User-Agent, fingerprint/device headers or any other request-identity variation;
- do not use `--resolve`, DNS pinning, endpoint forcing, interface/address-family forcing or alternate resolver behavior;
- do not follow an alternate URL/sample;
- discard response body and do not retain response headers;
- passively observe only fields needed by the accepted sanitizer: remote endpoint, HTTP version, HTTP status and optional total-time value;
- immediately sanitize those fields through the same already-running sanitizer process.

Perform at most 3 identical bounded probes. Use the same small fixed bounded delay between probes. Do not adapt the request in response to status/family/alias observations. Stop early only after two consecutive `2xx` results.

## Durable Evidence boundary

Per probe, durable Evidence may contain only:

```text
probe_index
status_class: 2xx | 3xx | 4xx | 5xx | network-error | unknown
family: ipv4 | ipv6 | unknown
endpoint_alias: ep-<bounded opaque run-local token> | unknown
http_version_class: h1 | h2 | h3 | unknown
timing_bucket: bounded sanitizer bucket | unknown
```

Never publish or retain:

- raw remote IP/address;
- raw DNS answer;
- sanitizer key/context;
- raw curl write-out line before sanitization;
- response body or response headers;
- request headers, Cookie/Auth/token/profile state;
- URL query material;
- challenge/CAPTCHA/access-control details;
- media payload.

Endpoint aliases are diagnostic only and are intentionally meaningless across Attempts/runs.

## Result semantics

### PASS

Requires two consecutive unchanged-path `2xx` status classes within the maximum 3 probes.

PASS proves only:

```text
BILIBILI_HOST_ELIGIBLE_FOR_#67_REFRESH=yes
```

It does not matter whether those two `2xx` observations use the same or different endpoint aliases/address families. Diagnostic correlation cannot strengthen or weaken the PASS rule.

PASS does not prove extraction, ResolvedMedia, #114 behavior, playback or #68 readiness.

### BLOCKED

If the bounded set does not produce two consecutive `2xx` results, report BLOCKED and STOP.

The report may describe the bounded alias/family/status pattern, but must not infer access-control causes that were not proven. Do not attempt a workaround or a second bounded set.

## Hard boundaries

- verification-only; no repository/product/security implementation changes during Attempt 4;
- no root/sudo/system changes beyond entering the already accepted low-privilege execution boundary;
- no proxy or proxy rotation;
- no Cookie/login/profile/session import;
- no CAPTCHA/challenge automation or access-control bypass;
- no browser/fingerprint/User-Agent/header variation;
- no alternate sample or mirror;
- no DNS pinning, `--resolve`, endpoint/address-family forcing or resolver steering;
- no retry-until-success or extra bounded set;
- no response body/header retention or publication;
- no raw endpoint/DNS publication or durable retention;
- no yt-dlp / generic-ytdlp / R008 / broker / sandbox / media resolver execution;
- no #67 J3 and no #68 execution;
- no media payload.

## Success criteria

1. Accepted target/low-privilege identity is used.
2. One accepted #128 sanitizer process covers the whole Attempt.
3. Raw endpoint observation data reaches only the sanitizer stdin/in-memory boundary and is not durably emitted.
4. Probe path remains direct/no-proxy with identical request shape.
5. At most 3 probes are run in exactly one bounded set.
6. Durable Evidence is bounded to sanitizer output plus probe index/eligibility/cleanup.
7. PASS only on two consecutive `2xx` status classes.
8. Persistent/intermittent non-2xx remains BLOCKED without bypass or request variation.
9. No resolver/generic-ytdlp/#67 J3/#68 execution occurs.
10. Worker uses the fresh terminal-write authority guard for report/status/owner mutations, releases owner and STOPs.

## Lifecycle

PASS:

```text
status:ready
-> Worker claim
-> one sanitizer process + one bounded probe set
-> [EXECUTION REPORT]
-> status:review
-> release owner
-> STOP
```

BLOCKED:

```text
status:ready
-> Worker claim
-> one sanitizer process + one bounded probe set
-> [BLOCKER REPORT]
-> status:blocked
-> release owner
-> STOP
```

Worker must not merge, close, set `status:done`, execute #67/#68 or create another Task.
