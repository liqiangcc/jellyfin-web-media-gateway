# ENV-BILIBILI-REACHABILITY-OBS-PREP

## Identity

- Issue: #128
- Kind: verification tooling / observability prep
- Preferred Worker: cloud
- Eligible environment: `env:cloud`
- Planning base: `5e423d170623411428eab008365151ffa67d0030`
- Trigger authority: #113 repeated bounded `2xx -> 4xx -> 2xx` observations on the accepted phone
- Scope: passive reachability-observation helper/protocol, deterministic offline tests, lifecycle documentation only
- Product/runtime/security semantics: unchanged

## Problem

#113 deliberately retains only bounded status-class evidence. Repeating the same unchanged direct/no-proxy request while only recording `2xx|4xx|...` cannot tell whether alternating results correlate with a different address family or endpoint. More repetitions without better passive observability have low information value.

## Goal

Prepare the smallest repository-owned, privacy-safe transform that a future Coordinator-authorized #113-style bounded probe may use to correlate status class with passive connection metadata without changing HTTP request identity, destination selection, retry behavior, or the existing PASS rule.

This Task is PREP only. It MUST NOT perform a live Bilibili request.

## Hard boundaries

The implementation MUST NOT:

- run Bilibili, generic-ytdlp, resolver, #67, #68, #113, or other real-site network probes;
- use proxy, Cookie/Auth/login/profile state, custom UA/fingerprint/Referer/header variation;
- use DNS pinning, `--resolve`, destination forcing, IPv4/IPv6 forcing, retries-until-success, CAPTCHA/challenge/access-control bypass;
- retain or publish response body, response headers, request headers, raw URL query material, Cookie/Auth/token/profile state;
- emit raw remote IP/address, raw DNS answer, or another reversible endpoint identifier to stdout/stderr, Issue comments, logs, artifacts, fixtures, process argv, or durable files;
- change product/media/browser/site/security/deployment runtime behavior;
- change #113's frozen eligibility/PASS rule: two consecutive unchanged-path `2xx` observations remain required.

## Required behavior

Implement one small pure helper/library/script plus deterministic offline tests. The helper consumes already-normalized local observation fields and emits only a bounded durable record such as:

```text
family: ipv4 | ipv6 | unknown
endpoint_alias: <opaque run-local alias | unknown>
http_version_class: h1 | h2 | h3 | unknown
status_class: 2xx | 3xx | 4xx | 5xx | network-error | unknown
timing_bucket: optional bounded bucket | unknown
```

Requirements:

1. `endpoint_alias` MUST be privacy-safe and useful only for correlation within one bounded run. A run-local opaque alias table, an HMAC/truncated digest using an ephemeral per-run key that is never durably emitted, or an equivalently safe design is acceptable. A plain/unsalted hash of an IP is NOT sufficient.
2. Raw endpoint input MUST be passed through an in-memory/stdin/API boundary, not command-line argv, and MUST never appear in normal output or exception/error messages.
3. Unknown/malformed input MUST fail closed to bounded `unknown`/error classification without echoing the raw value.
4. Output schema and field lengths MUST be bounded and deterministic for the same normalized run-local context.
5. Observation is passive only. The helper MUST NOT resolve DNS, open sockets, steer curl, choose address family, retry, or alter HTTP options.
6. Documentation MUST show how a future #113-style Task may consume the helper while preserving the unchanged direct/no-proxy request shape and two-consecutive-`2xx` PASS gate.

## Claims

- C1: helper is pure/offline and cannot perform network I/O.
- C2: authorized normalized observations produce bounded family/status/http-version metadata plus a run-local opaque endpoint alias.
- C3: raw IPv4 and IPv6 endpoint values never appear in normal output, stderr/error output, durable fixture output, or documented invocation argv.
- C4: endpoint correlation is run-local; durable output cannot be used as a stable cross-run raw endpoint identifier.
- C5: malformed/unknown endpoint data fails closed without raw-value reflection.
- C6: helper does not select/resolve/force/retry a destination and therefore does not alter request identity.
- C7: #113 PASS rule and all existing reachability restrictions remain unchanged.
- C8: no live Bilibili/real-site request is executed by this Task.

## Deterministic evidence

Offline tests MUST cover at least:

- IPv4 input -> `family=ipv4` + opaque alias, with raw address absent from all captured output;
- IPv6 input -> `family=ipv6` + opaque alias, with raw address absent;
- same endpoint in one run -> same alias;
- different endpoint in one run -> different alias with bounded output;
- same endpoint under a new run-local context -> different alias or otherwise no stable cross-run identifier;
- malformed endpoint -> bounded unknown/rejection without reflection;
- bounded status/http-version/timing normalization;
- stdout/stderr/exception leakage checks using sentinel endpoint strings;
- static/runtime proof that the helper imports/uses no network/DNS client path or that any standard-library surface is constrained to pure parsing/hashing only;
- documentation/fixture scan for raw IPs, auth/cookie/header/body retention patterns, or destination-steering flags.

## Evidence / verification

Required before Worker report:

- exact Candidate SHA and PR;
- deterministic offline test results;
- syntax/lint/compile checks appropriate to the helper language;
- targeted secret/leak scan of changed files;
- diff-scope proof showing only Task Package / observability helper/tests/docs are changed;
- explicit statement: `Live Bilibili probe: NOT RUN`.

## Success criteria

PASS requires C1-C8, deterministic offline tests, bounded privacy-safe output, no raw endpoint leakage, no live real-site probe, no product/runtime/security change, and a narrow reviewable Candidate.

## Failure / blocked rule

If privacy-safe correlation cannot be implemented without durable raw endpoint exposure or request-path mutation, report BLOCKED. Do not weaken privacy restrictions and do not run a live probe to compensate.
