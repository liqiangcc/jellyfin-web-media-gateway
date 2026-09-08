# R008 tx-node egress decision (Issue #179)

Date: 2026-09-08

This is Attempt 1 of the research Task. It is a sanitized decision record,
not a Bilibili compatibility result and not authorization for another live
request. Issue #166 remains `status:blocked` until an approved capability and a
revised Publication Gate exist.

## Evidence basis

The accepted #176 evidence was read back from GitHub:

- Candidate `c19af1d91eea0fa29279d4a62470d324e6a934a1` is Final Accepted and
  merged at `50e86e666687b270b02288c444cecf35307a8c1b`.
- Hosted run `34228687357` and its exact Candidate jobs passed. The accepted
  target diagnostic was run from the target-runnable artifact and retained
  only the following bounded result: `phase=broker_connect`,
  `reason=broker_connect_failed`, `status_class=unknown`,
  `request_count=10`, `response_bytes=7612`, `metadata_bytes=0`.
- No page/source observation, candidate export or independent consumer run
  was produced. The accepted cleanup found no remaining probe/browser process
  or staging directory.

The #166 history and contract were also read back. Its earlier clean browser
attempt stopped with `ERR_SSL_PROTOCOL_ERROR` before page observation. The
accepted #169/#172 provenance, selector, target package and budgets remain the
frozen authorities; no replacement live attempt was started in this Task.

## Fresh coarse target topology

Read-only commands executed through authenticated SSH, with the capability
checks themselves run as the admitted `gateway-verify` user, observed:

| Fact | Sanitized observation |
| --- | --- |
| Target platform | Linux, x86_64; the accepted admission identifies Ubuntu 26.04 |
| Execution identity | `gateway-verify`, UID/GID 1001 |
| Runtime | Node 22.22.1; npm 9.2.0 |
| External browser | Google Chrome 152.0.7977.82 at the accepted system path |
| HTTP(S)/ALL/NO proxy variables | absent in the low-privilege environment |
| Route family | IPv4 default route present; IPv6 default route absent |
| Resolver | nameserver configuration present |
| Existing probe/browser ownership | no `gateway-verify` Chrome or Node probe process observed |

These facts show that the admitted user has a usable runtime shape and a
configured IPv4 route. They do not establish that the broker can complete an
origin connection, that the route reaches the intended public service, or that
the route is suitable for a production egress capability. No endpoint, address,
process detail, profile, credential, TLS transcript or raw command output is
retained here.

## What `broker_connect_failed` proves

The current experimental broker has two relevant paths. An HTTPS request is
resolved through the public-host policy and checked address set before
`https.request`; a browser HTTPS tunnel is admitted only for an allowed host
and port, resolved through the same policy, then opened with `tls.connect`
using the origin hostname for SNI/certificate verification. Errors from these
upstream connection objects are recorded through `recordFailure(...,
'broker_connect')`. DNS policy failures are recorded separately as
`dns_address_policy`, and an allowlisted TLS error marker such as
`ERR_SSL_PROTOCOL_ERROR` would classify as `tls_handshake`.

Therefore the accepted result proves only that the bounded failure was observed
at the broker's upstream connection boundary before an HTTP/page observation.
The diagnostic object is schema-safe: it emits only an allowlisted phase and
reason, a coarse status class, and bounded counters; the original error message
is never copied.

The result leaves these subphases unknown:

- TCP connect completion or refusal;
- TLS handshake completion, certificate failure, protocol failure or reset;
- receipt of a proxy/tunnel response;
- downstream browser connection closure after a successful tunnel.

Because the diagnostic intentionally discarded the underlying code/message,
`broker_connect_failed` cannot select among those cases. It also does not prove
that DNS policy denied the target, that the target's public route is absent,
that the origin rejected TLS, that Bilibili is incompatible, or that a media
candidate exists. The counters do not turn into page or media evidence.

The experimental browser path strips Secret-classified request headers and
keeps candidate URLs out of exported observations. Its browser CONNECT tunnel
is still an experiment, not a substitute for the production R008 response
containment and per-hop redirect authority. No canonical R008 rule is changed
by this decision.

## Option decision matrix

Statuses use the Task vocabulary. A status below is an option decision for the
next capability, not a claim that the option was exercised.

| Option | Status | Security/design conditions | Evidence authority and owner | Rollback / use by #166 |
| --- | --- | --- | --- | --- |
| Current direct broker-owned public TLS path | **BLOCKED** | Keep public-host allowlist, DNS/address checks and pinning, origin TLS verification, bounded budgets, no caller proxy/credentials, and fail-closed CONNECT/upgrade handling. Add phase telemetry before interpreting the current failure. | A follow-up hosted static check plus one separately authorized target transport diagnostic; owner: R008/Gateway maintainer with tx-node operator. | Revert to the accepted diagnostic Candidate if the change is rejected. Cannot unlock #166 while the connection subphase is unknown. |
| Gateway-owned explicitly configured relay | **BLOCKED** pending separate review | It must be Gateway-owned and operator-configured, with no caller-selected endpoint or open proxy. The relay must preserve origin host/SNI/certificate verification, checked public-address pinning, per-hop redirect revalidation, Secret containment, size/time/cancellation limits and no raw CONNECT tunnel. TLS MITM and verification bypass are excluded. | Requires a new security/architecture review and implementation Candidate; owner: Gateway/R008 maintainer and Coordinator. | Disable the relay configuration and keep the direct path blocked. It may be used by #166 only after the review, implementation Evidence and a revised #166 Publication Gate. |
| Approved public-egress runner | **CONDITIONAL PASS** for diagnostic evidence only | Use an exact Candidate on a GitHub-hosted or separately approved runner, with the same fail-closed broker, no secrets, no profile reuse and bounded requests. Treat its result as runner-path evidence; it cannot stand in for tx-node source portability. | Hosted Actions run/artifact, reviewed by Coordinator; owner: Actions/verification maintainer. | Delete/expire the ephemeral job/artifact. It can diagnose a portable broker path, but cannot unlock #166 without target-equivalent approved evidence. |
| Route unavailable / terminate this route | **BLOCKED** as a present conclusion | Do not label the route unavailable from `broker_connect_failed`; first distinguish route, TCP and TLS stages. If the operator cannot provide an approved route after that diagnostic, record an explicit unavailable capability and keep the parent blocked. | Coordinator records the external condition; owner: tx-node/network operator. | No runtime change. #166 stays blocked; the product may route through an explicitly supported alternate Site/Display path after Coordinator decision. |

The selected next step is the smallest bounded transport-phase diagnostic,
followed by an approved public-egress runner check if the Coordinator needs a
portable comparison. A relay is not selected by this Task, and no proxy is
installed or configured.

## Child Task proposal (Coordinator materialization required)

Proposed title: **R008 broker transport phase and egress preflight**

Task kind: `combined` (implementation plus verification); parent: #179; it
must not modify #166 status or authorize a live page/media probe.

Scope:

1. Add a small experimental broker seam that records only bounded stage enums
   for resolve/policy, TCP connect, TLS handshake, tunnel/proxy response and
   downstream close. Preserve the existing public-host, TLS, redirect,
   Secret, request/byte/time and cancellation boundaries.
2. Add hosted deterministic fixtures proving stage classification, counter
   bounds, error-message non-retention and no sensitive output.
3. After Coordinator approval, run at most one target transport preflight using
   a plugin-derived public authority, no page navigation, no media body,
   no credentials, no profile and no proxy rotation. The preflight must not
   become a worker egress path.
4. If a relay is later proposed, stop this Task and route a separate
   architecture/security ADR before implementation.

Claims:

- C1: the exact broker stage is classified without exporting endpoint,
  address, certificate, TLS transcript, headers, body or credentials;
- C2: the target preflight distinguishes missing route, TCP failure, TLS
  failure and broker response/close, or returns `unknown` without overclaiming;
- C3: all existing R008 limits and no-secret/no-open-proxy invariants remain
  intact.

Success criteria:

- exact Candidate SHA and hosted x64 Actions evidence are recorded;
- deterministic tests pass for every stage and all sensitive-field scans pass;
- the target report contains only coarse enums/counters and cleanup state;
- no browser page, Bilibili selector, media candidate or independent consumer
  is run;
- Coordinator decides whether the result justifies a revised #166 contract;
  the child Worker cannot unlock or mutate #166.

Required Actions: hosted x64 static/fixture tests, hosted integration scan,
and (only after a new target Publication Gate) the single bounded target
transport preflight. Required reviewers are the Coordinator and the R008
security/Gateway owner; the tx-node operator owns target admission and route
facts.

## Claim results for this Attempt

- C0: **PASS** — #166/#176 history, accepted #176 Candidate/run/artifact
  authority and current main were read back; no duplicate live attempt ran.
- C1: **CONDITIONAL PASS** — the broker lifecycle and safe diagnostic schema
  reconcile statically, but the accepted enum cannot distinguish TCP/TLS/
  post-connect subphases and the experimental browser tunnel is not a
  production redirect/response-secret authority.
- C2: **CONDITIONAL PASS** — every allowed option is evaluated and the child
  diagnostic/owner/condition is concrete. No option is promoted to a usable
  #166 egress capability.
- C3: **PASS** — coarse target facts were collected read-only as
  `gateway-verify`; no browser/profile/proxy/credential/process mutation,
  live site request, package install or build occurred.

Decision: **CONDITIONAL PASS for the research result; #166 remains blocked**.
The next authorized decision point is Coordinator review of this document and,
if accepted, materialization/publication of the child diagnostic Task.
