# Task — R008-TX-NODE-EGRESS-DECISION

## Metadata

```text
GitHub Issue: #179
Parent / blocked dependency: #166 BILIBILI-BROWSER-SOURCE-REAL
Related accepted diagnostic: #176 TX-NODE-BROWSER-EGRESS
Task kind: research
Planning base: 50e86e666687b270b02288c444cecf35307a8c1b
Task contract: docs/tasks/179-r008-tx-node-egress-decision/task.md
Bootstrap prompt: docs/tasks/179-r008-tx-node-egress-decision/prompt.md
Preferred worker: cloud-codex (gpt-5.6-luna, reasoning high; Fast only if exposed and recorded)
Eligible environment: env:cloud
Required capabilities: github-read-write, repository-static-analysis, cloud-interactive, interactive-linux-debug, authenticated SSH tx-node
Execution planes: GitHub-hosted Actions for durable document/static checks; authenticated SSH to tx-node for read-only topology facts only
```

This Task is a decision study. It does not claim Bilibili compatibility or authorize another live request.

## Problem and goal

#166 Attempt 1 failed during clean browser navigation with `ERR_SSL_PROTOCOL_ERROR`. #176 added sanitized phase telemetry and one fresh target diagnostic, which recorded:

```text
phase: broker_connect
reason: broker_connect_failed
request_count: 10
response_bytes: 7612
page/source observation: not reached
independent consumer: not run
```

The browser runtime itself is admitted on tx-node. The unresolved question is which next capability can make a broker-owned public path usable without violating R008: target network/TLS reachability, broker connection handling, or an absent approved egress route.

Produce a durable decision record that:

1. distinguishes what the accepted evidence proves from what remains unknown;
2. evaluates architecture-safe egress options and rejects unsafe shortcuts;
3. defines the smallest follow-up implementation or operator-capability Task, if one is justified;
4. leaves #166 blocked until an approved capability and a revised Publication Gate exist.

## Authority and inputs

Read back before work:

- #166 full history, Task Contract and its Coordinator Reviews;
- #176 full history, Final Acceptance and merged Candidate `c19af1d91eea0fa29279d4a62470d324e6a934a1`;
- #176 hosted run `34228687357`, target evidence and artifact `10056820405`;
- `AGENTS.md`, `docs/security.md`, `docs/architecture.md`, `docs/implementation-contracts.md`, R008 research/ADR, browser/site-plugin contracts, lifecycle/recovery/freshness protocols;
- current main `50e86e666687b270b02288c444cecf35307a8c1b`.

If any authority or evidence identity differs, record the mismatch and stop; do not run a replacement live probe.

## Scope

### Evidence and static analysis

1. Compare #166/#176 sanitized counters and phase classification with the actual `live.mjs` broker lifecycle. Identify whether `broker_connect_failed` is a TCP connect, TLS handshake, tunnel response or downstream reset classification; if current telemetry cannot distinguish it, record that as an explicit uncertainty and define the smallest diagnostic change for a child Task.
2. Inspect tx-node read-only capability facts already admitted: host identity/OS/architecture, low-privilege user, Node/npm, external Chrome, proxy-variable state, route/resolver configuration and existing process ownership. Do not attach to or inspect the source-runtime Chrome/profile, its proxy, cookies or content. Do not dump credentials, secret environment values or unrelated user data.
3. Build a decision matrix for:
   - current direct broker-owned public TLS path;
   - a Gateway-owned, explicitly configured outbound relay/proxy, only if it can preserve host/SNI validation, public-address pinning, redirect revalidation, Secret containment, bounded limits and no caller authority;
   - moving the diagnostic browser to an approved public-egress runner;
   - terminating this route as unavailable.
4. For each option record required security/design changes, evidence authority, operational owner, rollback and whether it can be used by #166. Use `PASS`, `CONDITIONAL PASS`, `FAIL` or `BLOCKED`; do not call an option feasible solely from theory.
5. If a code or canonical-design change is needed, create a focused child Task/ADR proposal with exact Scope, Claims, Success Criteria and required Actions. Do not implement a proxy, change R008, alter TLS verification, or update requirements/architecture/security silently in this Task.

### Durable output

Add one focused research/decision document under `docs/research/` (or an ADR only when the canonical design-change process requires it). It must contain no raw URL, IP address, proxy endpoint, certificate, TLS transcript, header/body, cookie, token, profile path or target credential. Link the document from Issue #179 and state that #166 remains blocked.

No live Bilibili request, media candidate capture, independent consumer, phone/TV/VNC operation or production Gateway/Vault mutation is allowed in this Task.

## Claims and decision outcomes

| Claim | PASS condition | Evidence authority |
| --- | --- | --- |
| C0 | #166/#176 provenance, sanitized target result and current main are read back exactly; no duplicate live attempt is started. | GitHub |
| C1 | Broker lifecycle and telemetry are statically reconciled; known/unknown failure subphases are explicit and no sensitive field can leave the process. | GitHub-hosted static/contract checks |
| C2 | A decision matrix evaluates every allowed option and selects either a concrete safe follow-up Task or an explicit blocked gate with owner/condition. | research document + Coordinator review |
| C3 | Target topology facts are collected read-only under the admitted low-privilege boundary, or the missing capability is recorded as BLOCKED; no process/profile/secret mutation occurs. | authenticated SSH / tx-node |

C2 may be `CONDITIONAL PASS` when no approved egress route exists but the required external condition and next Task are concrete. It must not be promoted to a live-site or playback PASS.

## Verification matrix

| Job | Claims | Plane / target | Required evidence |
| --- | --- | --- | --- |
| J1 | C0,C1 | GitHub-hosted x64 Actions | exact Candidate/document static scan, focused diagnostic tests if a child proposal includes code, no sensitive marker |
| J2 | C3 | authenticated SSH / tx-node | read-only identity/tool/network-policy facts; no direct public-site handshake, no existing profile/process access |
| J3 | C2 | Coordinator research review | decision matrix, option status, owner/condition, child Task/ADR link and parent #166 gate state |
| JI1 | C0–C2 | GitHub-hosted x64 Actions | exact Candidate integration/document checks when the research document changes repository surfaces |

No local compilation, npm install, Cargo command, browser build, proxy setup or target package installation is permitted. Any code/build/test needed by a child Task must run in GitHub-hosted Actions.

## Evidence contract

Every Attempt must record:

```text
Task / Claims / Attempt: #179 / C0,C1,C2,C3 / N
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high (+ actual Fast availability)
Execution plane: GitHub-hosted Actions and read-only authenticated SSH
Runner / target: hosted x64; tx-node VM-0-11-ubuntu / gateway-verify
Base / Candidate: exact SHAs; accepted #166/#176 provenance
Workflow / run / job: exact IDs for any Actions evidence
Topology: coarse capability enums only; no endpoint/IP/process/profile/credential data
Decision: option status and required external condition
Parent: #166 state and next gate
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

Store only coarse, sanitized facts. Never retain raw target network output, DNS answers/IPs, certificate details, TLS transcripts, URLs/queries, proxy endpoints, headers/body, cookies, authorization, account/profile data or credentials.

## Architecture and security boundaries

- Gateway remains the `PlaybackSession` authority; this research document is not a playback path.
- Browser/site semantics remain plugin-owned and the current Bilibili selector stays opaque.
- The broker remains fail-closed and retains DNS/address, TLS, redirect, transport, Secret and budget authority.
- A proxy/relay is never caller-selected, open, unauthenticated or a way to bypass public-address/TLS checks.
- No private/SSRF exception, TLS verification bypass, Cookie/Auth injection, profile reuse, VNC/CDP attach or source-runtime proxy reuse.
- No production Gateway, Vault, phone, TV or self-hosted runner mutation.

## Out of scope

- Re-running #166 or any live Bilibili/browser/media request.
- Implementing a production egress proxy, SiteAdapter, SourceLocator/ResolvedMedia, Playback/Display/Control, remux/MSE or login/DRM bypass.
- Any direct browser-to-public-site route that bypasses the experimental broker.
- Network scans, packet captures, raw TLS/HTTP diagnostics or reading another user's process/profile/secret data.

## Freshness and lifecycle

Freshness policy: dependency-aware.

Semantic authorities are the accepted #166/#176 history, #176 Candidate and run, current R008/security/browser contracts and main `50e86e666687b270b02288c444cecf35307a8c1b`. A change to broker/TLS authority, target image, selector or budgets invalidates the decision and requires Coordinator contract revision before any child Task.

Follow `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md` and `docs/tasks/freshness-integration-protocol.md`. Claim only from `status:ready + env:cloud`; use fresh terminal-write guards; post `[EXECUTION REPORT]` or `[BLOCKER REPORT]`; transition to review/blocked, release ownership and stop. The Worker cannot close this Issue.

Coordinator must review the evidence, assign C2 an allowed research result, and decide whether to create/revise a child implementation Task. This Task does not change #166 status automatically.

## Success criteria

1. C0–C3 have exact, sanitized evidence on one Candidate/document revision; no duplicate live probe was run.
2. The decision record states why tx-node browser launch is usable, why the current Bilibili route is blocked, and what external capability or child Task is required next.
3. Any proposed egress option preserves R008/TLS/Secret/SSRF boundaries and has a named owner/verification plan.
4. The Coordinator accepts the research result before revising #166. No phone, TV, VNC or local build is introduced.

