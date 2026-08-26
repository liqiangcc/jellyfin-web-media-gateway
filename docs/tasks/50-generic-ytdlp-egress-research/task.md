# Task — GENERIC-YTDLP-EGRESS-RESEARCH

## Metadata

```text
GitHub Issue: #50
Parent Goal / Research Item: optional production generic-ytdlp runtime decision
Task / Research ID: GENERIC-YTDLP-EGRESS-RESEARCH
Task kind: research
Planning Base: 18dd2b60f21d98436341e056376b26730c392ab7
Session bootstrap prompt: docs/tasks/50-generic-ytdlp-egress-research/prompt.md
Preferred worker: web
Eligible worker environments: env:web-gpt
Required capabilities: github-read-write, web-research, primary-source-reading, repository-static-analysis, security-architecture-analysis
Hard publication dependencies: #46 Final Accepted; #39 Final Accepted; #14/R008 Final Accepted
```

Realtime status/owner/Attempt/Evidence lives in Issue #50. This file is the stable Task Contract.

## Goal

Decide whether production **real-network generic-ytdlp** can be supported without weakening accepted R008 Egress/Secret authority, and record one explicit product/architecture decision:

```text
SUPPORTED
CONDITIONALLY SUPPORTED
DEFER
DROP
```

If supported, freeze one bounded network-execution architecture and the boundary of a later implementation Task. If not supported now, document the exact security/value reason and the product fallback (`generic-direct` + explicit Site Plugins).

This Task is research only. It must not enable a production network-capable yt-dlp runner.

## Why / Context

Issue #46 Final Accepted the deterministic plugin/process/parser PREP but deliberately left production real-network execution disabled. The current accepted seam is:

```text
SiteAdapterRegistry
→ generic-ytdlp lower-priority plugin
→ ProcessRequest { structured http/https source URL }
→ ProcessRunner
→ bounded machine output parser
→ ResolvedMedia
```

Current production registration uses `GenericYtdlpAdapter::default()` with `DisabledRunner`; the fixed-argv `CommandProcessRunner` is test-only.

Accepted R008 public-web authority is materially stronger than “URL scheme is http/https”:

```text
Gateway EgressPolicy
→ hostname/scheme/port validation
→ DNS resolution
→ public-IP classification
→ validated address set
→ pinned connection addresses
→ redirect disabled in client
→ each redirect hop revalidated by Gateway-owned logic
```

A normal yt-dlp subprocess owns its own DNS/HTTP/redirect behavior, so merely invoking it with a validated URL would create an R008 bypass. #50 decides whether that gap can be closed safely and with enough product value.

## Task Decomposition Decision

```text
Verification mode: inline research evidence
Linked implementation task: #46 Final Accepted PREP
Linked verification task: n/a
Future implementation task: forbidden until this research is Final Accepted
Decision reason: this Task owns one architecture/product decision, not production implementation or target/device proof
```

No device, TV, phone, real account, or real-site acceptance Task is part of this research.

## Worker Routing Decision

```text
Preferred worker: web
Environment: env:web-gpt
```

Reason: the decisive work requires both live GitHub read/write and current external/primary-source research into yt-dlp request/extractor/network execution behavior. This is capability-driven routing; it is not a reason to move production runtime authority into Web or bypass repository contracts.

## Preconditions

Before claim/execution, read and treat as authority/evidence:

- Issue #50 and relevant comments;
- Issue #46 Final Acceptance and accepted `plugins/generic-ytdlp/src/lib.rs` seam;
- Issue #39 Final Acceptance / SiteAdapter conformance boundary;
- Issue #14 Final Acceptance / R008;
- `AGENTS.md`;
- `docs/security.md`;
- `docs/research/r008-security-boundary.md`;
- `gateway-core/src/security.rs`;
- `docs/architecture.md` and `docs/implementation-contracts.md` where they define Site Plugin/Core boundaries;
- lifecycle/freshness protocols.

External research must prefer primary/current sources: official yt-dlp documentation, current yt-dlp source/code paths, and authoritative dependency/runtime documentation where needed. Record source URL plus version/tag/commit/date when available so later reviewers can distinguish current facts from assumptions.

## Research Questions / Claims

### C1 — Network authority

Determine whether **every** yt-dlp-originated HTTP(S) request, including extractor-discovered nested requests, manifests, metadata, player/config/API calls and media requests relevant to extraction, can be forced through a Gateway-owned policy boundary equivalent to R008.

### C2 — DNS pinning / rebinding

Determine how DNS is resolved and how the actual connection destination is constrained after validation. A design that validates a hostname in Gateway but lets the subprocess independently re-resolve/connect is not equivalent to R008.

### C3 — Redirect revalidation

Determine whether redirects can be observed and revalidated centrally per hop. Invisible subprocess redirect following is not accepted.

### C4 — Broker / proxy boundary

Evaluate a deployment-owned outbound broker/proxy only if it can be capability-scoped, deny arbitrary open-proxy use, deny private/loopback/link-local/metadata targets from untrusted source input, and prevent direct subprocess egress around the broker.

### C5 — HTTPS semantics

Determine how TLS hostname/certificate verification remains correct under mediation. Unsafe certificate disabling, broad MITM trust, or hostname verification weakening is rejected.

### C6 — Secret / auth ownership

Preserve current Secret authority: no caller-provided Cookie files, browser profile paths, arbitrary Authorization/Cookie headers, arbitrary proxy credentials or free-form yt-dlp flags. Any future authenticated use must consume a separately accepted narrowly scoped Core/Vault capability; this Task does not design real login.

### C7 — Process escape / alternate transports

Prove the proposed model cannot escape through direct egress, `file:`, localhost/configured services, custom protocols, shell invocation, external downloaders, arbitrary `--proxy`, config files, environment variables or caller-controlled executable/argv.

### C8 — Observability / cancellation

Determine whether timeout, cancellation, child cleanup, bounded stdout/stderr and network failure classification can remain deterministic and Secret-safe enough for the existing plugin/Gateway error boundary.

### C9 — Product value / fallback decision

Compare security/implementation complexity, compatibility risk, maintenance burden and user value against the accepted alternatives:

```text
generic-direct
+ explicit Site Plugins
+ optional future Browser Worker/site-specific runtime
```

The existence of #46 PREP is not evidence that real-network generic-ytdlp should ship.

## Candidate Solution Families to Evaluate

At minimum compare:

1. Gateway-owned HTTP(S) outbound broker/proxy + subprocess network sandbox that permits only the broker;
2. process/network namespace or equivalent sandbox forcing all yt-dlp traffic to a policy mediator;
3. yt-dlp Python/API integration or request-handler hooks that expose all requests sufficiently for Gateway mediation;
4. plugin-owned alternative extraction mechanism with a controllable HTTP client;
5. `DEFER` or `DROP` real-network generic-ytdlp and rely on generic-direct + explicit Site Plugins.

Do not assume any family is acceptable before evidence.

## In Scope

- current yt-dlp network/request architecture research;
- current #46 runner/parser seam analysis;
- R008 equivalence threat/data-flow model;
- DNS/rebinding/redirect/TLS/proxy/sandbox/Secret/process-escape analysis;
- bounded proof/spike only if necessary to resolve a material uncertainty;
- alternatives matrix;
- explicit `SUPPORTED | CONDITIONALLY SUPPORTED | DEFER | DROP` decision;
- frozen future implementation boundary if supported/conditional;
- documented fallback and canonical-doc implications if defer/drop.

## Out of Scope

- production network-capable yt-dlp implementation or registration;
- changing `GenericYtdlpAdapter::default()` to enable networking;
- weakening R008 or adding a generic private-network bypass;
- real Bilibili/site acceptance (#23/#36);
- CAPTCHA/fingerprint/access-control/DRM bypass;
- real login/account/Cookie/profile proof;
- Browser Worker runtime implementation;
- phone/TV/Jellyfin proof;
- changing Playback authority;
- treating a local/open proxy as accepted merely because it makes yt-dlp work.

## Architecture Invariants

1. Core remains site-neutral; yt-dlp extractor/site behavior stays outside Stable Core.
2. Site Plugin cannot bypass `EgressPolicy`.
3. R008 remains authority for public/private target classification, DNS/pinning, redirect revalidation and Secret boundaries.
4. `generic-ytdlp` remains lower priority than `generic-direct` for direct MP4/M4V/HLS.
5. Caller input cannot supply arbitrary executable, flags, proxy, environment, config, Cookie/profile path or Secret header authority.
6. No production open proxy or generic private-network capability may be introduced.
7. TLS verification must remain correct for the intended origin.
8. A research PASS cannot be manufactured by disabling SSRF/TLS/Secret checks.

## Research Method / Requirements

1. Read the live accepted repository implementation before external research; do not reason from a generic imagined yt-dlp wrapper.
2. Identify the actual yt-dlp request/extractor paths relevant to network access and whether there is a complete interception/control point.
3. For each candidate solution, draw the trust/data-flow boundary and list bypass/escape paths.
4. Distinguish **can observe** from **can enforce**. Logging requests after they happen does not satisfy R008.
5. Distinguish initial-source validation from nested-request authority. All relevant nested requests must remain covered.
6. Treat subprocess network isolation as part of the proof when a broker/proxy architecture is proposed; proxy configuration alone is insufficient if direct egress remains possible.
7. Treat TLS CONNECT/MITM implications explicitly rather than assuming proxying preserves TLS semantics.
8. If a small prototype is required, keep it research-only and incapable of enabling production runtime; record exact commit/run and remove/contain any unsafe experimental path.
9. Do not modify canonical product/security docs merely to make the research decision easier. The research document must list canonical changes required by the decision; Coordinator decides final canonical adoption during Review/next Task.

## Verification Plan

### Evidence Jobs

| Job | Claims | Execution Plane | Required | Evidence |
| --- | --- | --- | --- | --- |
| J1 Repository authority read-back | C1-C8 | GitHub/repository research | yes | exact live #46/R008/#39 files + accepted issue evidence, summarized in research doc |
| J2 External primary-source research + threat model | C1-C8 | Web research | yes | cited current yt-dlp/source/dependency evidence + threat/data-flow diagram + bypass analysis |
| J3 Alternatives/value decision | C1-C9 | Research synthesis | yes | alternatives matrix + explicit decision + future boundary/fallback |
| J4 Bounded prototype/spike | affected claims only | GitHub Actions or other auditable plane | no, unless a material claim cannot be resolved without runtime proof | exact candidate/run/artifact; no production enablement |

No physical target or self-hosted runner is required.

### Claim Success Rules

- **C1 PASS** only if the selected supported architecture has an enforceable choke point for all relevant yt-dlp HTTP(S) requests; otherwise the architecture cannot be `SUPPORTED`.
- **C2 PASS** only if checked DNS/address policy controls the actual connection destination or an equivalent broker enforces it; independent subprocess re-resolution is insufficient.
- **C3 PASS** only if redirects are centrally visible and revalidated per hop.
- **C4 PASS** only if mediation cannot become an open proxy and direct egress bypass is denied.
- **C5 PASS** only if TLS hostname/certificate verification remains sound.
- **C6 PASS** only if Secret/account material remains Core/Vault-owned and arbitrary credential injection is denied.
- **C7 PASS** only if alternate transports/config/flags/external downloader/direct egress escape paths are bounded.
- **C8 PASS** only if process/network lifecycle and diagnostics remain bounded and Secret-safe.
- **C9 PASS** when the final product decision explicitly weighs value/complexity and names the fallback.

A final decision of `DEFER` or `DROP` may be a **successful Task result** even when C1-C8 show that a candidate runtime architecture is not presently acceptable. Research Task success means the evidence and decision are complete, not that generic-ytdlp must be supported.

## Success Criteria

Task succeeds when all are true:

1. `docs/research/generic-ytdlp-egress-research.md` exists and is grounded in live repository state plus cited primary/current external evidence.
2. The document contains an explicit trust/data-flow/threat model covering initial and nested requests, DNS/rebinding, redirects, TLS, proxy/broker, Secret and process escape paths.
3. At least the five candidate solution families above are compared in a security/complexity/compatibility/value matrix.
4. One explicit decision is recorded: `SUPPORTED | CONDITIONALLY SUPPORTED | DEFER | DROP`.
5. If `SUPPORTED`, one bounded architecture is frozen well enough to materialize a later implementation Task without reopening the fundamental network-authority question.
6. If `CONDITIONALLY SUPPORTED`, the exact missing condition/evidence and what remains disabled are explicit.
7. If `DEFER` or `DROP`, the fallback product path and canonical-doc implications are explicit.
8. No production network runtime is enabled and no R008 invariant is weakened.
9. Issue receives a standard `[EXECUTION REPORT]` with candidate/research doc/source selectors and limitations, then returns to Coordinator Review.

## Evidence Contract

Record at minimum:

```text
Attempt / Worker / environment
Planning Base / Candidate commit
Repository files and accepted Issue/PR selectors used
External source URL + source type + version/tag/commit/date where available
Claims C1-C9 result/analysis
Threat/data-flow diagram location
Alternatives matrix location
Prototype run/job/artifact if J4 is used
Explicit final decision
Known uncertainty / stale-source risk
Canonical docs affected by the decision
Future implementation Task boundary or fallback path
```

Do not persist Secret, Cookie, Token, credentials, private URLs or unredacted sensitive logs.

## Failure / Blocked Handling

- If current external primary sources cannot be accessed sufficiently to determine a material network-control fact, report `BLOCKED`; do not fill the gap with assumption.
- If evidence proves no acceptable architecture under R008, do **not** mark the Task failed merely because generic-ytdlp is infeasible; complete the research with `DEFER` or `DROP`.
- If the only workable approach requires weakening TLS, SSRF, Secret ownership, or creates an open proxy, classify that approach rejected.
- If research discovers the Task would require changing accepted R008/SiteAdapter architecture itself, stop production-design expansion and report the required semantic reclassification to Coordinator.

## Deliverables

- `docs/research/generic-ytdlp-egress-research.md`;
- optional research-only bounded spike/evidence if needed;
- explicit architecture/product decision;
- future implementation boundary or fallback implications;
- standard Issue `[EXECUTION REPORT]`.

## Freshness / Integration Contract

```text
Freshness policy: dependency-aware
Planning Base: 18dd2b60f21d98436341e056376b26730c392ab7
```

Semantic authorities:

- #46 accepted `plugins/generic-ytdlp/**` process/runner/parser/registration seam;
- #14/R008 `gateway-core/src/security.rs`, `docs/security.md`, `docs/research/r008-security-boundary.md`;
- #39 / `site-adapter-api/**` SiteAdapter/ResolvedMedia/Secret boundary;
- `AGENTS.md` Core/site-neutrality and no-Egress-bypass invariants.

Semantic freshness domains:

- `plugins/generic-ytdlp/**`;
- `gateway-core/src/security.rs` and security-related Core HTTP/network capability surfaces;
- `site-adapter-api/**` public SiteAdapter/ResolvedMedia/security schema;
- canonical architecture/security text governing Egress, Site Plugin, Vault/Secret and process execution.

Task-owned surfaces:

- `docs/research/generic-ytdlp-egress-research.md`;
- bounded research-only spike files if explicitly necessary.

Integration surfaces:

- repository docs index/navigation only if the research doc is linked there;
- no production runtime surface is owned by this Task.

Authority/domain → Claim mapping:

- #46 runner/request/parser seam → C1,C4,C7,C8;
- R008 Egress/DNS/redirect/TLS boundary → C1-C5,C7,C8;
- R008/#39 Secret/public media boundary → C6,C7,C8;
- canonical product/plugin strategy → C9.

Unrelated-main policy:

- changes outside these semantic/integration domains do not invalidate completed research evidence;
- external-source freshness must still be reported with version/date selectors;
- semantic changes to #46/R008/#39 require affected-claim reread/reconciliation before Coordinator acceptance;
- a change that redefines Scope/Claims/R008 authority is `CONTRACT_INVALIDATING` and requires republish.

## Issue Feedback / Completion

Follow `docs/tasks/issue-lifecycle-protocol.md`:

```text
status:ready
→ Worker claim / Attempt N
→ status:in-progress
→ research + durable doc/candidate
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Worker must not set `status:done`, close #50, enable production yt-dlp networking, start a future implementation Task, or automatically execute another Task.