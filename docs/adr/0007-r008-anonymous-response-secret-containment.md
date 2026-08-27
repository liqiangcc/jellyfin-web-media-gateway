# ADR 0007 — R008 Anonymous Response Secret Containment

## Status

Accepted for implementation planning.

## Context

Issue #67 Attempt 5 reached the accepted real `R008Broker` path on the Ubuntu ARM64 target and reproduced the same bounded result twice:

```text
broker_request_count: 1
broker_status_class: 4xx
broker_error_code: BROKER_RESPONSE_SECRET_REJECTED
ResolvedMedia: not reached
```

The accepted anonymous generic-ytdlp architecture already requires that raw Cookie / Set-Cookie / Authorization / token-classified material never crosses the worker/public boundary and that the anonymous runtime does not own a cookie jar.

The current broker implementation rejects the whole origin response when any response header is Secret-classified. Real public sites may legitimately emit response-only Secret material such as `Set-Cookie` even for anonymous pages. Treating the mere presence of such a header as a whole-response failure is stricter than the required Secret ownership invariant and can prevent compatibility determination without adding useful authority isolation.

## Decision

For the anonymous R008 broker, request Secret authority and response Secret containment are distinct rules.

### Request side — reject

Caller/worker/extractor-originated Secret authority remains fail-closed before prohibited network side effects. This includes URL userinfo and Cookie / Authorization / proxy-auth / API-token / Basic / Bearer-classified request material.

### Response side — contain

Origin response Secret-classified header material is not worker/plugin authority. It must be removed or otherwise contained inside the Gateway boundary before the broker response crosses IPC.

The anonymous broker:

- must never expose Secret-classified response header values to the worker, Site Plugin, `ResolvedMedia`, logs, artifacts, or diagnostics;
- must not create, persist, replay, or otherwise operate an anonymous cookie jar from contained response headers;
- may continue to return the origin status, bounded body, and non-Secret public response headers when all other R008 checks pass;
- must preserve existing response/header/body size bounds and fail closed when safe containment itself cannot be established;
- must not convert contained response Secret material into caller-visible credential or capability authority.

The presence of a containable Secret response header alone is therefore not sufficient reason to reject an otherwise valid anonymous public response.

## Preserved R008 authority

This decision does not change:

- `public_web` SSRF classification;
- DNS resolution and checked-address pinning;
- origin TLS hostname/certificate verification;
- redirect revalidation;
- no CONNECT/open proxy/raw tunnel rule;
- request Secret rejection;
- timeout/cancellation/lifecycle limits;
- bounded broker body/header/frame limits;
- SiteAdapter / `ResolvedMedia` Secret-free public output requirements;
- production `GenericYtdlpAdapter::default()` remaining disabled.

## Diagnostics

Durable diagnostics may expose only bounded non-Secret facts needed to prove the policy, for example a filtered Secret-response-header count or fixed policy result. They must never expose a rejected/filtered Secret header value. A real-site verification Task is not required to publish the concrete origin Secret header name.

## Cookie semantics

This ADR does not admit anonymous cookie state.

```text
Set-Cookie from origin
→ contained inside Gateway broker boundary
→ not delivered to worker
→ not stored
→ not replayed as Cookie
```

If a site requires stateful cookies for compatibility, that is a separate authenticated/session capability decision and must not be smuggled into anonymous generic-ytdlp.

## Verification consequences

The implementation Task must deterministically prove at least:

1. request Secret material is still rejected before prohibited side effects;
2. a public response containing `Set-Cookie` can retain status/body/non-Secret headers while the Secret header is absent across broker IPC;
3. `WWW-Authenticate` / proxy-auth-classified or Bearer/Basic-valued response headers are likewise contained without leaking values;
4. contained response material is never replayed as a later request Cookie/Auth header;
5. public response header limits and malformed/non-UTF8 failure behavior remain bounded;
6. redirects continue through explicit R008 revalidation with Secret response headers contained;
7. diagnostics, tests, errors and artifacts contain no Secret sentinel values;
8. #14/R008, #39 Secret/conformance, #60 broker/runtime and affected workspace regressions remain green.

Real Bilibili compatibility remains Issue #67 authority and must be re-verified only after the implementation is Final Accepted.
