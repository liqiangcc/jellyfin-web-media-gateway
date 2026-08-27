# Task — GENERIC-YTDLP-RUNTIME-PREP

## Metadata

```text
GitHub Issue: #60
Parent decision: #50 GENERIC-YTDLP-EGRESS-RESEARCH / CONDITIONALLY SUPPORTED
Task / Research ID: GENERIC-YTDLP-RUNTIME-PREP
Task kind: combined / implementation + deterministic security verification
Planning Base: c43c54bcf4f1a96ab06b21c6ad4569df76c40613
Session bootstrap prompt: docs/tasks/60-generic-ytdlp-runtime-prep/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, rust, python, linux-process-sandboxing, unix-ipc, security-testing, github-actions-orchestration
Hard publication dependencies: #50 Final Accepted; #46 Final Accepted; #39 Final Accepted; #14/R008 Final Accepted
Frozen upstream selector: yt-dlp 2026.08.19 @ 3a08beaf031ab68f966401ead017ac81fe8486cf
```

Realtime status/owner/Attempt/Candidate/Evidence lives in Issue #60.

## Goal

Implement and deterministically prove the architecture frozen by #50 without enabling production real-network `generic-ytdlp`:

```text
exact-version yt-dlp Python worker
→ inherited per-attempt structured IPC capability
→ Gateway-owned HTTP(S) broker
→ accepted R008 DNS/public-IP/address-pin/TLS/redirect authority
→ worker + descendants denied direct AF_INET/AF_INET6
→ bounded machine output
→ existing #46 parser
```

This Task must leave production `GenericYtdlpAdapter::default()` fail-closed on `DisabledRunner`.

## Decomposition Decision

```text
Verification mode: inline for deterministic architecture/security proof
Real-network/public-site acceptance: separate later Verification Task
```

A later real-network compatibility/acceptance Task may be materialized only after #60 Final Acceptance. #60 success does not mean production generic-ytdlp is enabled or supported on real sites.

## Frozen Authority

Read and preserve:

- #50 Final Acceptance and `docs/research/generic-ytdlp-egress-research.md`;
- #46 accepted generic-ytdlp process/parser/registration seam;
- #39 SiteAdapter/ResolvedMedia/Secret authority;
- #14/R008 `EgressPolicy`, validated-address pinning, redirect and Secret authority;
- `AGENTS.md`, `docs/security.md`, architecture/implementation contracts, lifecycle/freshness protocols.

Do not silently bump yt-dlp. The implementation and Evidence are bound to:

```text
tag/version: 2026.08.19
upstream commit: 3a08beaf031ab68f966401ead017ac81fe8486cf
```

A version change is security-semantic work and requires Coordinator reclassification before acceptance.

## Claims

### C1 — Pinned dedicated Python worker

Use the exact frozen yt-dlp version through a repository-owned Python worker/API path. Caller input cannot choose executable, argv, config, plugin path, proxy, environment, downloader, postprocessor, JS runtime, Cookie/profile/netrc or TLS options. Normal CLI config/plugin discovery is not runtime authority.

### C2 — Structured IPC capability

Worker receives only a per-attempt inherited IPC capability (Unix-domain/socketpair/pipe-equivalent). There is no public broker listener, CONNECT, SOCKS, generic URL tunnel or caller-selected local-service scope. IPC framing, request/response size, method, URL, headers and body are bounded.

### C3 — Gateway-owned R008 broker

The broker is site-neutral and Gateway-owned. For HTTP(S) requests it must reuse accepted R008 public-web authority:

```text
validate URL/scheme/host
→ resolve
→ public-IP classification
→ checked SocketAddr set
→ pinned connection
→ normal origin TLS hostname/certificate verification
→ automatic redirect disabled
→ each Location fully revalidated before next hop
```

No yt-dlp-specific extractor logic belongs in Core.

### C4 — No direct worker/descendant Internet sockets

On required Linux Evidence, the Python worker and descendants must fail closed when attempting direct AF_INET/AF_INET6 networking while broker IPC remains usable. At minimum prove denial for:

- ordinary Python socket;
- a test/custom request handler path;
- a spawned child process.

Use an OS enforcement mechanism that descendants cannot simply loosen (for example inherited seccomp/no-new-privs, network namespace, or an equivalent proven boundary). Application monkey-patching alone is insufficient.

### C5 — Redirect/DNS/TLS equivalence

Deterministically prove the broker rejects private/loopback/link-local/metadata/reserved redirect targets, does not permit DNS TOCTOU to change the actual connected address, and does not use MITM or certificate-verification bypass. Ordinary proxy-only semantics are not accepted.

### C6 — Anonymous Secret authority

Initial profile is anonymous only. Reject before network side effects:

- URL userinfo;
- Cookie / Authorization / proxy-auth / API-token-classified headers;
- caller or worker/extractor Basic/Bearer credentials;
- browser profile / Cookie file / netrc inputs;
- arbitrary proxy credentials/config.

Any future authenticated generic-ytdlp requires a separate accepted Core/Vault capability Task.

### C7 — Alternate transport/config/runtime escape

The admitted runtime must disable or fail closed for direct WS/WSS, FTP, file/data escape, direct `curl_cffi`/impersonation transport, external downloader, remote components, default/custom plugin loading and arbitrary runtime executable discovery. If a compute-only child is needed by tests, it must inherit the same no-direct-network boundary.

### C8 — Lifecycle / cancellation / diagnostics

Preserve #46 bounded stdout/stderr, timeout, kill/reap and redaction behavior and extend it to the worker process group/descendants and broker I/O. Timeout/cancel/crash/overflow must not leave a descendant or broker request alive and must not leak source URLs, response bodies or Secret-classified material.

### C9 — Existing parser/registry authority remains intact

Brokered worker output continues through the existing bounded #46 parser and #39 conformance. `generic-direct` priority remains unchanged. No Core direct yt-dlp fallback is introduced.

### C10 — Production remains disabled

The brokered runtime candidate may be reachable only through explicit verification/test/non-default construction required for this Task. The default production registry must still register the fail-closed PREP adapter with `DisabledRunner`; no public real-network generic-ytdlp route is enabled.

## In Scope

- exact-version yt-dlp dependency/pin for the worker;
- repository-owned Python worker using yt-dlp API/request layer;
- generic structured broker IPC protocol;
- Gateway-owned site-neutral HTTP(S) broker that consumes R008;
- Linux no-direct-network sandbox for worker/descendants;
- deterministic synthetic broker/extractor fixtures sufficient to exercise actual pinned yt-dlp request flow without a real site;
- Secret/escape/lifecycle negatives;
- minimal generic capability plumbing if required, provided no yt-dlp type/behavior enters Stable Core;
- exact-Candidate GitHub Actions verification.

## Out of Scope

- enabling production `generic-ytdlp` networking;
- replacing `DisabledRunner` as default production registration;
- real Bilibili or other real-site acceptance;
- login/account/Cookie/profile authentication;
- CAPTCHA/fingerprint/access-control/DRM bypass;
- TV/phone/Jellyfin proof;
- caller-controlled CLI/config/plugin/runtime flags;
- weakening R008, TLS, SSRF or Secret boundaries;
- a public/open proxy.

## Implementation Requirements

1. Pin and verify exact yt-dlp `2026.08.19@3a08beaf...`; do not consume moving master.
2. Use a repository-owned Python worker/API path, not the normal user CLI entrypoint.
3. Remove/avoid ordinary network handlers as authority; worker logical HTTP(S) requests must be served through the broker request layer.
4. Broker protocol must be structured and bounded; raw tunnels/CONNECT are forbidden.
5. Broker production implementation must call accepted R008 validation/pinning primitives rather than reimplementing a weaker SSRF filter.
6. Broker transport must not auto-follow redirects; redirect handling is an explicit per-hop policy operation.
7. Install OS-level no-direct-AF_INET/AF_INET6 enforcement before untrusted yt-dlp/extractor work; descendants inherit it.
8. Keep IPC usable under the sandbox and prove a child cannot escape by opening its own network socket.
9. Enforce anonymous Secret policy at the broker boundary before side effects.
10. Fail closed for unsupported alternate transports/config/plugins/external runtime paths.
11. Retain #46 parser, output caps and stable error taxonomy where compatible; add bounded classifications only when required.
12. Keep default production registry on `DisabledRunner` and add an explicit regression proving it.
13. If a shared public contract is added, it must be site-neutral and corresponding #39/R008 architecture/conformance regressions must pass.
14. Do not add a real-site smoke to manufacture production support; real compatibility is a later Task.

## Verification Job Matrix

| Job | Claims | Plane / Runner | Required | Evidence |
| --- | --- | --- | --- | --- |
| J1 — pinned worker + broker flow | C1,C2,C3,C9,C10 | GitHub Actions / ubuntu-latest | yes | actual pinned yt-dlp Python worker uses structured IPC/broker fixture and existing parser; default registration remains disabled |
| J2 — sandbox / escape matrix | C4,C7 | GitHub Actions / ubuntu-latest | yes | Python socket, custom-handler/direct path and child AF_INET/AF_INET6 attempts denied; IPC remains functional |
| J3 — R008 / Secret / redirect / TLS negatives | C3,C5,C6 | GitHub Actions / ubuntu-latest | yes | private/loopback/metadata redirect denial, pinning path, no CONNECT/MITM, userinfo/Secret/header/config rejection |
| J4 — lifecycle + regressions | C8,C9,C10 | GitHub Actions / ubuntu-latest | yes | timeout/cancel/crash/overflow descendant cleanup + workspace/#46/#39/R008/architecture regressions |

All required jobs must assert the exact Task Candidate SHA.

No phone/self-hosted/TV target is required.

## Success Criteria

Task succeeds only when:

1. C1-C10 are proven on one exact Candidate.
2. Actual frozen yt-dlp code participates in J1; a fake process alone is insufficient.
3. Required Linux Evidence proves OS-level direct-network denial for worker and descendants while structured IPC continues to work.
4. Broker code consumes R008 validation/address-pinning authority and manual redirect handling; no weaker duplicate policy becomes authoritative.
5. Anonymous Secret/credential and alternate transport/config/runtime escape matrices fail closed before prohibited side effects.
6. Crash/timeout/cancel/overflow leaves no live descendant/broker activity and diagnostics remain bounded/redacted.
7. Existing #46 parser/#39 conformance and `generic-direct` priority remain intact.
8. Production `GenericYtdlpAdapter::default()` remains `DisabledRunner`; no production network path is enabled.
9. J1-J4 and affected regressions pass on exact Candidate in a reviewable PR.
10. Limitations explicitly state that real-network/site compatibility and production enablement remain unproved and require a separate Task.

## Evidence Contract

`[EXECUTION REPORT]` must include:

```text
Attempt / Worker / Environment
Planning Base
Candidate SHA / PR
Pinned yt-dlp version + upstream commit + packaging/hash selector
Worker entrypoint
Broker/IPC implementation locations
OS sandbox mechanism and no-new-privilege/inheritance facts
Direct-network denial matrix (Python/custom handler/child)
Broker R008/pinning/redirect result
TLS result
Anonymous Secret/header/userinfo result
Alternate transport/config/plugin/runtime result
Lifecycle/descendant cleanup result
Default DisabledRunner proof
Claims C1-C10
J1-J4 run/job IDs + exact-Candidate assertions
Affected #46/#39/R008/workspace regressions
Problems / limitations
Production enablement status: MUST remain disabled
```

Never persist Secret, Cookie, token, credential, complete sensitive URL or unnecessary response/media content.

## Failure / Blocked Handling

- If GitHub-hosted Linux cannot provide an OS mechanism capable of proving descendant direct-network denial, report BLOCKED; do not replace it with Python monkey-patching or theory.
- If current pinned yt-dlp request internals cannot be forced through the structured broker without leaving an escape path, report the exact defect and return to Coordinator; do not weaken R008.
- If implementation requires changing #50/R008 security invariants or admitting a new transport/Secret authority, stop and request Contract/architecture revision.
- Test defects/evidence gaps with the same frozen contract stay in this Issue as the next Attempt.

## Deliverables

- brokered worker/runtime PREP implementation;
- deterministic worker/broker/sandbox fixtures;
- exact-Candidate J1-J4 Actions Evidence;
- reviewable PR;
- no production enablement.

## Freshness / Integration Contract

```text
Freshness policy: dependency-aware
Planning Base: c43c54bcf4f1a96ab06b21c6ad4569df76c40613
```

Semantic authorities:

- #50 research decision and `docs/research/generic-ytdlp-egress-research.md`;
- #46 `plugins/generic-ytdlp/**` process/parser/registration seam;
- #14/R008 `gateway-core/src/security.rs`, `docs/security.md`, R008 evidence;
- #39 `site-adapter-api/**` / conformance / Secret boundary;
- frozen upstream yt-dlp `2026.08.19@3a08beaf...`.

Semantic freshness domains:

- `plugins/generic-ytdlp/**`;
- R008 Egress/Secret/pinning surfaces;
- `site-adapter-api/**` if capability plumbing changes;
- canonical text changing Core/Site Plugin/Egress authority;
- frozen yt-dlp version/RequestHandler/network/config/plugin/runtime assumptions.

Task-owned surfaces:

- generic-ytdlp worker/runtime/sandbox code and tests;
- generic site-neutral broker capability/service code required by this architecture;
- Task-specific workflow/fixtures.

Integration surfaces:

- Cargo workspace/lockfile;
- gateway-core service composition if a generic broker is added;
- site-adapter-api only if unavoidable generic capability plumbing is introduced.

Authority/domain → Claims:

- #50/upstream worker model → C1,C2,C4,C7,C8;
- R008 → C3,C5,C6,C8;
- #46/#39 → C9,C10;
- production registry boundary → C10.

Unrelated-main changes do not invalidate Task-specific Evidence. Semantic changes to the domains above require affected-claim reconciliation. A yt-dlp version bump or change that redefines the frozen runtime architecture is security-semantic and cannot be treated as ordinary integration drift.

## Completion Protocol

Follow `docs/tasks/issue-lifecycle-protocol.md`:

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ implementation + exact-SHA J1-J4
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Worker must not merge its own PR, set `status:done`, close #60, enable production generic-ytdlp networking, publish the later real-network Verification Task, or automatically execute another Task.