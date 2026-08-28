# Task — ENV-ARM64-BILIBILI-REACHABILITY-REFRESH

## Metadata

```text
GitHub Issue: #113
Task ID: ENV-ARM64-BILIBILI-REACHABILITY-REFRESH
Task kind: verification-only / environment + public-site reachability
Contract Revision: R2
Attempt: 2
Parent: #67 GENERIC-YTDLP-BILIBILI-REAL / R17 Attempt 17
Planning Base: 9a6fc52a70a83ab49b5b07c426c74985734b664e
Preferred worker: ubuntu-arm64
Eligible environment: env:ubuntu-arm64
Target: accepted Ubuntu ARM64 phone / gateway-runner
Frozen sample: BV14V411W7r5
Publication state: non-executable until Coordinator Publication Gate passes
```

## Trigger

#67 R17 established exact Candidate `80fb081b129f8f664124b84ddcc9698039e2cfd1` and frozen runtime provenance on the accepted phone, but stopped at J2 before resolver execution:

```text
J0 exact Candidate: PASS
J1 frozen runtime: PASS
direct public HTTPS: 2xx
direct frozen Bilibili sample: 4xx
J3: NOT RUN
Overall: BLOCKED
```

This is site-reachability/environment Evidence only. It is not a generic-ytdlp compatibility result and does not invalidate #114. The accepted #114 webpage-normalization repair remains unexercised against a normally reachable frozen sample in R17.

Historical accepted Evidence:
- #113 Attempt 1 previously refreshed the same frozen sample on the same accepted phone with identical direct/no-proxy probes and returned `4xx → 2xx → 2xx`, satisfying two consecutive `2xx`.
- #63 Final Acceptance also proved the same frozen sample reachable with direct/no-proxy HTTP 200 on the accepted phone and recorded a transient direct-path failure before a bounded identical-path retry recovered.
- #36 freezes the policy that if the unchanged public sample is unavailable from a permitted normal-network host, the result is BLOCKED and no proxy/fingerprint/Cookie/login/CAPTCHA/access-control bypass may be attempted.

## Goal

Determine only whether the unchanged frozen public Bilibili page has returned to stable normal direct/no-proxy reachability on the accepted phone.

Do not run yt-dlp, the generic-ytdlp resolver, sandbox, broker, R008 or media resolution.

## Exact probe contract

Run as the accepted low-privilege target identity using the accepted `setpriv ... env -i` boundary and the normal target `curl`.

Frozen URL:

```text
https://www.bilibili.com/video/BV14V411W7r5/
```

Before each probe:
- clear `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, `http_proxy`, `https_proxy`, `all_proxy`;
- use `curl --noproxy '*'`;
- do not add Cookie, Authorization, Referer, custom User-Agent, fingerprint/device headers, or any other request-shape variation;
- do not follow an alternate URL/sample;
- do not retain response body or headers.

Perform at most 3 identical bounded probes. Preserve only:

```text
probe_index
status_class: 2xx | 3xx | 4xx | 5xx | network-error
```

The Worker may stop early after two consecutive `2xx` results. Between repeated probes use a small fixed bounded delay; do not increase frequency or vary request identity in response to a 4xx.

## Result semantics

### PASS

Requires two consecutive identical-path `2xx` results within the maximum 3 probes.

PASS proves only:

```text
BILIBILI_HOST_ELIGIBLE_FOR_#67_REFRESH=yes
```

It does not prove extraction, ResolvedMedia, #114 behavior, playback or #68 readiness.

### BLOCKED

If the maximum bounded probe set does not produce two consecutive `2xx` results, report BLOCKED and STOP.

Examples include persistent/intermittent `4xx`, `5xx`, redirects that do not satisfy the normal page contract, or transport/DNS failure.

Do not attempt a workaround.

## Hard boundaries

- verification-only; no repository/product/security changes;
- no root/sudo/system changes;
- no proxy or proxy rotation;
- no Cookie/login/profile/session import;
- no CAPTCHA/challenge automation;
- no browser/fingerprint/User-Agent/header variation to evade a status result;
- no alternate sample or mirror;
- no response body/header retention or publication;
- no yt-dlp / generic-ytdlp / R008 / broker / sandbox / media resolver execution;
- no #67 J3 and no #68 execution;
- no media payload.

## Evidence

Durable report may include only:

```text
Attempt / Worker / Environment / UTC
Target: aarch64 Linux 4.19.113-964403 / gateway-runner low privilege
Frozen selector: BV14V411W7r5
Network: direct/no-proxy
probe 1 status_class
probe 2 status_class
probe 3 status_class (if run)
BILIBILI_HOST_ELIGIBLE_FOR_#67_REFRESH=yes|no
cleanup / safe-output
Overall: PASS | BLOCKED
```

## Success criteria

1. Accepted target/low-privilege identity is used.
2. Probe path is direct/no-proxy with identical request shape.
3. At most 3 probes are run.
4. No response content or sensitive material is retained.
5. PASS only on two consecutive `2xx` status classes.
6. Persistent/intermittent non-2xx is BLOCKED without bypass.
7. Worker reports, releases owner and STOPs.

## Lifecycle

Normal PASS:

```text
status:ready
→ Worker claim
→ bounded probes
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Blocked:

```text
status:ready
→ Worker claim
→ bounded probes
→ [BLOCKER REPORT]
→ status:blocked
→ release owner
→ STOP
```

Worker must not merge, close, set `status:done`, execute #67/#68, or create another Task.
