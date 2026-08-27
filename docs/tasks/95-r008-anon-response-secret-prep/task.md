# Task — R008-ANON-RESPONSE-SECRET-PREP

## Metadata

```text
GitHub Issue: #95
Task ID: R008-ANON-RESPONSE-SECRET-PREP
Task kind: implementation + deterministic security verification
Planning Base / canonical decision: bc56fb33f31bf8461ce1c0fe28c121c1c5028c89
Parent blocker: #67 Attempt 5
Preferred worker: cloud-codex
Eligible environment after publication: env:cloud
Downstream: #67 next Attempt after Final Acceptance
```

Canonical authority added by this Task's planning phase:

- `docs/adr/0007-r008-anonymous-response-secret-containment.md`

Accepted authority that must remain intact:

- #14 / R008;
- #39 shared Secret classifier / SiteAdapter conformance;
- #50 anonymous broker architecture decision;
- #60 brokered worker / no-direct-egress runtime;
- #66 extraction path;
- #67 Attempt 5 bounded real-target Evidence.

## Trigger

#67 Attempt 5 cleared the previous runtime blockers and reached real broker traffic on the accepted Ubuntu ARM64 target:

```text
runtime_cache: offline-hit
direct/no-proxy Bilibili: 2xx
ARM64 sandbox: PASS
#85 ENOSYS fallback: PASS
broker_request_count: 1 per run
R008: 4xx / BROKER_RESPONSE_SECRET_REJECTED
reproduction: 2/2
ResolvedMedia: not reached
```

The real-site report intentionally did not publish the triggering Secret header name/value. This Task must not infer that the specific header was `Set-Cookie`; deterministic fixtures own policy proof.

## Goal

Implement ADR 0007 so anonymous response Secret material stays inside the Gateway boundary without making the mere presence of a containable Secret response header a whole-response failure.

Required shape:

```text
worker/extractor request
→ request Secret check: REJECT
→ R008 public-web DNS/pinning/TLS/redirect authority
→ origin response
→ classify response headers
→ Secret response headers: CONTAIN / never cross IPC
→ no cookie jar / no replay
→ bounded status + body + non-Secret headers continue when safe
→ worker/extractor
```

## Frozen security boundary

The Worker MUST NOT solve this by weakening Secret classification.

In particular:

- `Set-Cookie` remains Secret-classified;
- `Cookie`, `Authorization`, `Proxy-Authorization`, API/token headers and Basic/Bearer material remain Secret-classified;
- request-side Secret material remains rejected before prohibited network side effects;
- response Secret values never cross broker IPC;
- no anonymous cookie jar, cookie persistence, cookie replay, auth replay or new credential capability;
- no change to R008 SSRF/public-IP/address pinning/TLS/redirect authority;
- no CONNECT/open proxy/raw tunnel;
- no body/frame/time/cancellation weakening;
- no caller/extractor proxy/profile/netrc/Cookie/Auth authority;
- production `GenericYtdlpAdapter::default()` remains disabled;
- no real Bilibili/site request in this Task;
- no #67 implementation, DASH/remux/Browser/Web E2E/performance work.

If safe containment cannot be proven under these invariants, report BLOCKED. Do not revert to whole-response compatibility by declassifying Secret material.

## In scope

- `gateway-egress/**` response-boundary handling;
- narrowly required shared helpers/tests under `site-adapter-api/**` only if they preserve existing Secret classification;
- narrowly required `plugins/generic-ytdlp/**` diagnostics/regressions without adding authority;
- deterministic synthetic response fixtures;
- bounded diagnostics needed to prove containment, such as a fixed policy result/count with no Secret value;
- syncing `docs/security.md` with accepted ADR 0007 semantics;
- exact-Candidate GitHub-hosted verification.

## Out of scope

- real Bilibili or any public-site verification;
- changing frozen yt-dlp version;
- authenticated generic-ytdlp;
- Cookie jar/session state;
- Vault/profile/login work;
- request Secret relaxation;
- R008 DNS/pinning/TLS/redirect relaxation;
- response body size expansion;
- SiteAdapter public Secret headers;
- production enablement.

## Claims

### C1 — Request Secret rejection is unchanged

URL userinfo and caller/worker/extractor Cookie/Auth/proxy-auth/API-token/Basic/Bearer request material remain rejected before prohibited network side effects.

### C2 — Response Secret containment

Secret-classified origin response headers remain Secret but are removed/contained before `BrokerResponse` crosses IPC. Their values are never observable by the worker/plugin/public output.

### C3 — Safe public response continuity

When all other R008 checks pass, origin status, bounded body, and admitted non-Secret response headers can continue even when one or more containable Secret response headers were present.

### C4 — No cookie/auth state creation or replay

Contained response headers do not populate any cookie/auth store and cannot cause a later request to carry Cookie/Auth material. Repeated requests remain anonymous unless a separately accepted capability exists.

### C5 — Redirect / R008 authority preserved

A response containing both a redirect location and Secret response material still exposes only the non-Secret redirect metadata needed for the existing explicit per-hop R008 revalidation. Private/loopback/etc redirect denial remains unchanged.

### C6 — Bounds and malformed handling remain fail closed

Public header count/name/value bounds, body/frame bounds and malformed/non-UTF8 behavior remain deterministic and bounded. Secret containment must not create an unbounded header sink or diagnostics channel.

### C7 — Diagnostics are Secret-safe

Tests/logs/errors/artifacts may report only fixed/bounded policy facts (for example filtered count/result). No Secret sentinel value or raw Secret header enters durable Evidence.

### C8 — Generic-ytdlp runtime authority preserved

#60/#66 brokered worker still has no direct network authority, receives no Secret response header, uses no cookie jar, and production default remains `DisabledRunner`.

### C9 — Canonical security semantics are synchronized

`docs/security.md` reflects ADR 0007: request Secret authority is rejected; anonymous origin response Secret headers are contained, not replayed, and may be filtered while otherwise safe public response material continues.

## Verification matrix

| Job | Claims | Runner | Required Evidence |
| --- | --- | --- | --- |
| J1 — R008 response policy | C1-C3,C6 | GitHub-hosted Linux | deterministic fixtures for ordinary headers, `Set-Cookie`, auth/challenge-classified headers and Basic/Bearer-valued response headers; Secret values absent from admitted response |
| J2 — state/redirect/security negatives | C1,C4-C7 | GitHub-hosted Linux | no cookie/auth replay across repeated requests; request Secret still rejected; redirect revalidation unaffected; bounded/malformed and Secret-sentinel negatives |
| J3 — brokered generic-ytdlp + regressions | C7-C9 | GitHub-hosted Linux | #60/#66 worker/broker tests, #39/R008/security/workspace regressions, default DisabledRunner proof, exact Candidate assertion |

A real phone/Bilibili run is explicitly not required here; #67 owns that Evidence after this Task is accepted.

## Success criteria

Task succeeds only when one exact Candidate proves all of the following:

1. C1-C9 PASS.
2. Secret classification itself is not weakened to obtain compatibility.
3. `Set-Cookie` and other Secret-classified response material cannot cross broker IPC or enter worker/public output.
4. Safe synthetic public responses with contained Secret headers can continue with status/body/non-Secret headers intact.
5. No anonymous cookie/auth state is stored or replayed.
6. Request Secret rejection and R008 SSRF/DNS/pinning/TLS/redirect rules remain green.
7. Existing body/frame/header/time/cancel bounds remain fail closed.
8. `docs/security.md` is synchronized with ADR 0007.
9. Required exact-Candidate J1-J3 and affected regressions pass.
10. Production generic-ytdlp remains disabled.
11. Worker posts `[EXECUTION REPORT]`, moves to `status:review`, releases owner and STOPs.

## Evidence contract

The report must include only bounded data:

```text
Attempt / worker / environment
Base SHA
Candidate SHA / PR
Implementation paths
Response containment policy result
Request Secret negative result
Set-Cookie containment fixture result
Auth/challenge/Bearer/Basic response containment fixture result
No-cookie/auth-replay result
Redirect/R008 regression result
Bounds/malformed result
Secret-sentinel leak scan
Generic-ytdlp broker/runtime regression result
Default DisabledRunner result
Canonical docs sync result
Claims C1-C9
J1-J3 run/job IDs + exact Candidate assertions
Problems / limitations
Downstream #67 readiness: yes/no + reason
```

Never publish Secret fixture values beyond fixed synthetic sentinels used only to assert absence; do not publish real-site headers, Cookies, tokens, Authorization values, full sensitive URLs or response bodies.

## Freshness / integration contract

```text
Planning Base: bc56fb33f31bf8461ce1c0fe28c121c1c5028c89
Freshness policy: dependency-aware
```

Semantic authorities:

- ADR 0007;
- `gateway-egress/**`;
- `site-adapter-api/src/security.rs` and Secret/conformance semantics;
- `docs/security.md`;
- #14/R008;
- #50/#60 anonymous generic-ytdlp broker architecture.

Any accepted change that adds credential/cookie authority, weakens request Secret rejection, or changes R008 DNS/pinning/TLS/redirect semantics requires Coordinator reclassification before acceptance.
