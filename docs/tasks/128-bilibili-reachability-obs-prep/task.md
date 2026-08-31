# Task — ENV-BILIBILI-REACHABILITY-OBS-PREP

## Metadata

```text
GitHub Issue: #128
Task ID: ENV-BILIBILI-REACHABILITY-OBS-PREP
Task kind: verification tooling / observability prep
Preferred worker: cloud
Eligible environment: env:cloud
Planning Base: c51a1ed4d6389beb708d23e331b9bc1583a38195
Trigger: #113 Attempt 2 + Attempt 3 repeated ordinary-path `2xx → 4xx → 2xx`
Publication state: non-executable until Coordinator Publication Gate passes
```

## Problem

#113 intentionally retains only status class. That keeps Evidence safe, but the same bounded ordinary direct/no-proxy request produced `2xx → 4xx → 2xx` in both Attempt 2 and Attempt 3. Repeating the same probe without richer **passive** observability has low information value.

The project needs a way for a later Coordinator-authorized reachability refresh to determine whether different status classes correlate with different remote endpoint/address-family observations, without changing request identity, selecting a destination, or publishing raw network addresses.

## Goal

Prepare a small repository-owned helper and protocol for privacy-safe endpoint correlation during a **future** bounded reachability check.

This Task is PREP-only. It MUST NOT execute a live Bilibili HTTP probe.

## Required helper semantics

The helper consumes normalized local observation records produced by a future caller after curl has completed. At minimum the normalized input may contain:

```text
status_code
remote_ip
http_version
connect_time_ms (optional)
tls_time_ms (optional)
```

The helper emits only bounded safe fields:

```text
status_class: 2xx|3xx|4xx|5xx|network-error
address_family: ipv4|ipv6|unknown
endpoint_id: same-run opaque salted identifier
ahttp_version: h1|h2|h3|other|unknown
timing buckets: optional coarse bounded classes only
```

`endpoint_id` requirements:

- generate a fresh high-entropy salt in memory for each helper process/run;
- never output the salt;
- hash the raw remote address with the in-memory salt and truncate the digest to a bounded opaque identifier;
- the same endpoint within one run maps to the same identifier;
- the same endpoint across different runs is not required to map to the same identifier;
- never output, log or persist the raw remote address in normal helper output.

The helper is an observability transform only. It MUST NOT perform DNS resolution, select/pin an endpoint, issue HTTP requests, change curl arguments, or retry.

## Request-shape invariant for future use

Using this helper later must not change the frozen ordinary request contract:

- direct/no-proxy only;
- unchanged URL/sample;
- no Cookie/Auth/login/profile;
- no custom User-Agent/fingerprint/Referer/header variation;
- no DNS pinning / `--resolve`;
- no forced IPv4/IPv6 selection;
- no endpoint steering;
- no retries-until-success;
- no response body/header retention;
- no CAPTCHA/challenge/access-control bypass.

The existing #113 PASS rule remains unchanged: two consecutive `2xx` within the bounded set. Endpoint correlation is diagnostic only and can never turn a non-PASS result into PASS.

## Implementation scope

Allowed:

- one small stdlib-only helper under `scripts/`;
- deterministic offline unit tests/fixtures under `scripts/tests/`;
- one narrow protocol/doc describing safe future consumption;
- Task Package docs.

Out of scope:

- any Bilibili/public-site HTTP request;
- product/media/browser/site-adapter/security runtime code;
- yt-dlp/generic-ytdlp/R008/broker/sandbox/resolver execution;
- proxy/login/fingerprint/bypass work;
- raw endpoint/DNS publication;
- distributed tracing/service infrastructure.

## Claims

- C1: status code is normalized to bounded status class without preserving response content.
- C2: IPv4/IPv6/unknown family is derived locally from an input address without emitting the address.
- C3: same raw endpoint in one helper run produces the same opaque `endpoint_id`.
- C4: per-run random salt prevents durable cross-run endpoint identity by default and is never emitted.
- C5: normal output never contains raw IPv4/IPv6 address material.
- C6: malformed/ambiguous input fails closed without echoing unsafe fields.
- C7: helper performs no network/DNS/HTTP operation and cannot steer request routing.
- C8: future #113-style use preserves the existing request-shape and PASS/BLOCKED semantics.

## Deterministic offline tests

At minimum cover:

1. IPv4 input → `ipv4`, opaque id, raw address absent.
2. IPv6 input → `ipv6`, opaque id, raw address absent.
3. same endpoint + fixed injected test salt → same id.
4. same endpoint + different fixed test salt → different id.
5. status classes for 2xx/3xx/4xx/5xx/network-error.
6. supported HTTP-version normalization.
7. malformed address/input → bounded `unknown`/error without raw echo.
8. scan serialized normal outputs to prove fixture raw addresses never appear.
9. no helper code path imports/uses networking or subprocess execution.

Tests may inject a deterministic salt through the imported pure function/API; the normal CLI/runtime path MUST generate its own random salt and MUST NOT offer a user-facing option that publishes/reuses a salt.

## Evidence

Worker report should include only:

```text
Candidate SHA / PR
helper/tests/doc paths
unit-test result
static network-operation check
raw-address non-leak tests
secret-pattern scan result
live Bilibili probes: 0
product/runtime changes: none
```

## Success criteria

PASS requires C1-C8, all deterministic offline tests passing, no live Bilibili request, narrow diff, and Coordinator verification that the helper is passive observability rather than endpoint/request steering.

## Lifecycle

```text
status:ready
→ cloud Worker claim
→ offline helper/docs/tests only
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Worker must not set done/close or execute #113/#67/#68.