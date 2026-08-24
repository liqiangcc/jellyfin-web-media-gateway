# R008 P0 Security Boundary Evidence

Status: implementation candidate; final verification is bound to the exact
Candidate SHA and J1/J2/J3 Actions jobs recorded in Issue #14 Attempt 1's
`[EXECUTION REPORT]`. This document deliberately does not copy secrets or
large logs; the Issue report is the durable selector for the run and jobs.

## Scope and current surface inventory

The current repository contains the R001 proof `GatewayService`, the generic
direct Site Plugin, and the accepted R007 `PlaybackSession` implementation.
It does not yet contain a production Control API, Session Vault, Browser
Worker, or Native Site Panel runtime. The R008 implementation therefore adds
reusable boundary primitives and deterministic tests without manufacturing
those absent components.

| Surface | R008 disposition |
| --- | --- |
| Public-web egress and redirect checks | Implemented in `security::EgressPolicy`; deterministic IPv4/IPv6 matrix and media redirect regression |
| Configured local service | Implemented as a named Core policy entry bound to configured scheme/host/port; unknown names and origin changes reject |
| Site access | Implemented as metadata-only `SiteAccessCapability`; site, host, and expiry checks; no Vault bytes in the capability |
| Media capability / ResolvedMedia | Existing session/item/revision/resource binding retained; public Secret headers and bearer-like values reject |
| Diagnostics | `redact_text` and `redact_url` remove known Secret and signed-query material; tests use fake sentinels only |
| External process invocation | `StructuredCommand` preserves argv boundaries; static test rejects shell construction patterns in runtime Rust sources |
| Target workflow | Static test covers all repository workflows containing `self-hosted`; target workflow remains manual-only and contents-read |
| Playback freshness | Accepted R007 suite rerun by J1; R008 does not add another authority |
| Browser Worker / Native Site Panel | `NOT IMPLEMENTED / DEFERRED TO R006`; no runtime PASS claimed |
| Production Control/API HTTP boundary | `NOT YET INSTANTIATED`; the R001 proof routes are not called a production API |

## Claim mapping

| Claim | Evidence mapping and boundary |
| --- | --- |
| C1 | J1/J2 on the exact candidate: `EgressPolicy` rejects loopback, private, link-local, metadata, multicast, unspecified, documentation, mapped IPv4, and reserved classes; the media fixture rechecks every redirect hop. |
| C2 | J1/J2: local access requires the named Core-configured entry and exact origin. A user/plugin URL cannot create the entry or change its host/port; the `not-configured` and metadata redirect tests reject. |
| C3 | J1: metadata-only SiteAccess fixture rejects wrong site, host, and expiry. `GatewayService` owns policy/configuration; no plugin API exposes raw Vault access. |
| C4 | J1/J2: header schema rejects Cookie, Authorization, proxy/API-key names, and bearer/basic values. Existing R001 media tests pass invalid, expired, cross-session, cross-item, revision, and resource bindings without upstream hits. |
| C5 | J1/J2: fake sentinel redaction and signed-query removal tests pass; Actions logs are scanned for sentinels and no sensitive artifact is uploaded. Accepted R001 browser proof remains the prior non-leakage authority for the integrated media path (Issue #3, Attempt 2 candidate `42c92db2a380895ec3909cdc9afa847478150eb0`, run `32735124301`). |
| C6 | J1/J3: structured argv helper and executable runtime-source scan pass. No FFmpeg, yt-dlp, or Chromium production launcher exists in this candidate, so no absent launcher is falsely claimed. |
| C7 | J1 reruns accepted R007 Playback tests, including stale item/media/display generation and handoff ABA cases. Accepted authority: Issue #2 Attempt 2 candidate `0cad62b08c190400def900e9b142edd1a0afd900`, run `32729228923`. |
| C8 | J3 statically checks final repository target workflows. Accepted low-privilege target facts remain Issue #1 final accepted candidate `6e9027a5a28c04f5aee1a713e5a7d9363f13222e`, target run `32727443950`; R008 does not rerun phone jobs. |
| C9 | J3 records the deferred classification only. Canonical Browser Worker requirements remain in `docs/security.md`; no runtime implementation or security PASS is asserted. |
| C10 | J1/J2 verify the currently instantiated proof surface and token-bound media routes. A production Control/API Host/Origin/CSRF/content-type/size contract is explicitly not yet instantiated and remains future verification scope. |

## Architecture impact

The implementation centralizes public-web and configured-local-service
decisions in Core policy, keeps private integration targets deployment-owned,
and preserves `ResolvedMedia.public_headers` as a public-only surface. The
new capabilities do not read Vault, expose credentials to Display, or alter
Playback authority. No canonical product/security scope is changed.

## Accepted upstream evidence used

- Issue #1 / INFRA-001: accepted low-privilege Target Runner and trusted manual
  workflow boundary.
- Issue #2 / R007: accepted request/revision/item/media/display-generation and
  handoff freshness behavior.
- Issue #3 / R001: accepted scoped media capability, redirect revalidation,
  protected upstream injection, replay rejection, and browser non-leakage.

The exact R008 Candidate SHA and required J1/J2/J3 run/job selectors are kept
in the Issue #14 Attempt 1 report so a later candidate cannot accidentally
inherit an earlier run's security claim.
