# Privacy-safe reachability observation consumption

This helper is **PREP-only**. Issue #128 does not run a Bilibili or other real-site request.

A future Coordinator-authorized #113-style bounded reachability refresh may pass already-observed curl connection metadata directly through a pipe/in-memory adapter to:

```text
scripts/reachability_observation_sanitizer.py
```

The helper accepts JSON Lines on **stdin only**. The raw endpoint must never be placed in process argv, a durable temp file, an Issue comment, an artifact, or a retained log. The output contains only:

```text
family = ipv4 | ipv6 | unknown
endpoint_alias = opaque per-process/run alias | unknown
http_version_class = h1 | h2 | h3 | unknown
status_class = 2xx | 3xx | 4xx | 5xx | network-error | unknown
timing_bucket = bounded bucket | unknown
```

The endpoint alias is HMAC-derived from an ephemeral process-local random key that is never emitted. It is useful for same-run correlation only and is intentionally not a stable cross-run endpoint identifier. Synthetic tests inject deterministic keys; production use does not publish or accept the key through argv.

## Future #113 integration boundary

If a later Task is explicitly authorized to run the frozen reachability probe, it may extend the existing passive curl write-out path so `%{remote_ip}`, `%{http_version}`, `%{http_code}`, and timing data flow directly to an in-memory/stdin adapter and are immediately sanitized. The raw curl values themselves are not durable Evidence.

This observability layer MUST NOT alter the existing request path. In particular, it does not add or authorize proxy use, Cookie/Auth/login/profile state, custom headers or fingerprints, DNS pinning, `--resolve`, address-family forcing, endpoint selection, retry-until-success, CAPTCHA/challenge handling, or access-control bypass.

The #113 acceptance rule is unchanged:

```text
two consecutive unchanged-path 2xx observations
```

Endpoint aliases and address-family observations are diagnostic context only. They cannot convert a `4xx`, network error, or mixed sequence into PASS and cannot by themselves authorize #67/#68 execution.

## Failure behavior

Malformed or oversized input yields only bounded `unknown` fields. Invalid endpoint text and ignored fields are never reflected in stdout/stderr. If safe piping cannot be maintained in a future probe, that probe must remain BLOCKED rather than persisting raw endpoint data.

`Live Bilibili probe: NOT RUN` for Issue #128.
