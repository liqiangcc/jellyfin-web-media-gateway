# Task — Resolve CONNECT TLS transport semantics after navigation rejection

## Metadata

```text
GitHub Issue: #226
Parent Goal / Research Item: R008 egress/secret baseline; browser Gateway→Bilibili navigation
Task / Research ID: R008-TRANSPORT-DIAGNOSTIC
Task kind: combined
Base commit: 103fb2a1119a4bb4a2ad45ac4026b9ce7d645651
Candidate commit: n/a until implementation Attempt
Session bootstrap prompt: docs/tasks/226-r008-connect-tls-transport-semantics/prompt.md
Preferred worker: cloud
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, cloud-interactive
Hard publication dependencies: Coordinator Publication Gate; accepted #223 evidence
```

## Session Bootstrap

Use `prompt.md` only to navigate into this contract. The live Issue, this file, the lifecycle/recovery/freshness protocols, and canonical security/architecture documents are authoritative.

## Goal

Determine whether the experimental browser broker's CONNECT path mixes broker TLS termination with browser TLS byte forwarding, and implement the smallest evidence-supported transport correction or diagnostic seam that makes the selected mode internally consistent while preserving all R008 authority, TLS, SSRF/egress, secret, bounds, finalizer, and cleanup guarantees.

## Why / Context

#223 Attempt 1 established a repeatable navigation rejection after upstream response/body completion and socket close. Its sanitized target result was `tls_handshake / tls_protocol_error`, with no media request or playback activity. Current source inspection shows the CONNECT path calls `tls.connect()` and then pipes browser bytes into that TLS-wrapped socket. That is a concrete, reviewable hypothesis for a protocol mismatch, not an accepted root cause. This Task isolates the transport decision so later target evidence can distinguish a corrected end-to-end tunnel from a broker-terminated request path.

Parent evidence anchor:

- #223 Attempt 1 Candidate `774c2a30d155d20b3368274a8af649f3a8d7e849`, PR #225, merged main `103fb2a1119a4bb4a2ad45ac4026b9ce7d645651`.
- Hosted probe run `34311451068`; required jobs were green and artifact `10088522113` was 4,219,341 bytes with digest `sha256:97f43e9f2f451723037724939123c164908e1dd23cc46ffd005ce50b6bc78afd`.
- Target evidence retained upstream response/body/socket-close markers and ended with sanitized `navigation_promise_rejected / tls_handshake / tls_protocol_error`; request count was 13, response bytes 15,224, metadata bytes 0; no media/playback/click/preload/consumer activity; Attempt-owned cleanup passed.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: The transport correction, deterministic regressions, hosted artifact gate, and later bounded target proof share one narrow Claim set. Coordinator retains target dispatch authority.
```

## Worker Routing Decision

```text
Preferred Worker: cloud-codex
Eligible environment: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, cloud-interactive
Model: gpt-5.6-luna
Reasoning: high
Fast: enabled for this Task
```

## Work Role

### Implementation

Inspect the current experimental broker and keep the transport mode explicit. The implementation may either:

1. use policy-pinned raw TCP CONNECT forwarding so the browser owns the origin TLS handshake, hostname, and certificate verification; or
2. use a broker-terminated HTTP/TLS request path in which the broker owns the upstream TLS request and forwards only the permitted HTTP result.

Choose only the mode supported by source evidence and the canonical contracts. Do not feed browser TLS bytes into a broker TLS wrapper. Do not turn either mode into a generic proxy, arbitrary authority, or credential-bearing path. If the safe choice requires changing a canonical architecture/security invariant, stop with evidence for Coordinator design review instead of changing that invariant in this Task.

### Verification

Verification must identify the exact Candidate. It must prove deterministic transport behavior and preserve the existing bounded diagnostic/cleanup contract. A later target run is separate runtime evidence and cannot be replaced by local commands or hosted tests.

## Task vs Job Boundary

```text
Task
→ transport semantics and bounded regressions
→ exact Candidate
→ GitHub-hosted verification jobs
→ Coordinator-controlled target navigation evidence
```

Actions jobs verify the repository Candidate; they do not claim this Issue or authorize target execution.

## Routing Rationale

Repository implementation and deterministic verification use Codex Cloud with GitHub-hosted Actions as the execution backend. No local build/test/package/install is permitted. Target proof, if dispatched after the exact artifact gate, uses the existing authenticated Tailscale SSH control plane only; that control plane is not a media proxy.

## Preconditions

- Read `AGENTS.md`, `docs/README.md`, `docs/requirements.md`, `docs/architecture.md`, `docs/implementation-contracts.md`, `docs/technical-feasibility-validation.md`, `docs/mvp-plan.md`, `docs/security.md`, `docs/development-environments.md`, `docs/runner-execution-architecture.md`, `docs/adr/0007-r008-anonymous-response-secret-containment.md`, this package, and `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, and `docs/tasks/freshness-integration-protocol.md`.
- Read all relevant #223 history and retain the exact evidence anchor above.
- Issue must be `status:ready`, `env:cloud`, owner-free, and have Coordinator Publication Gate read-back before claim. While this package is draft, do not claim or execute an Attempt.
- Required implementation authority is the existing plugin-owned broker path; no new site authority, proxy authority, credential, cookie, token, or production secret is available or permitted.
- Verification workflows must be GitHub-hosted and able to publish an exact Candidate artifact. If Actions or artifact delivery is unavailable, report BLOCKED; do not fall back to local execution.
- Target admission, if later authorized, requires a fresh temporary profile/display/debug endpoint as needed, gateway-verify UID/GID 1001 when practical, fixed selector `bilibili:BV14V411W7r5:part-2`, and bounded navigation-only execution.

## In Scope

- Compare raw TCP end-to-end browser TLS forwarding with a broker-terminated HTTP/TLS request path using the current source and canonical security contracts.
- Correct the selected CONNECT/request semantics or add a narrowly bounded diagnostic seam that proves the selected mode without weakening policy.
- Keep plugin-owned host/port authority, policy-pinned address selection, hostname/certificate verification, SSRF/egress rules, no-secret handling, byte/request/metadata limits, failure precedence, redaction, timeout, and cleanup.
- Add deterministic tests for CONNECT handshake and byte forwarding, both directions, failure and late-callback precedence, redacted bounded output, request/response/metadata limits, timeout, and cleanup.
- Publish exact Candidate, hosted J1/J2/J3a/J3/J4/JI1 evidence and artifact manifest/digest for Coordinator review.
- If Coordinator later dispatches target proof, perform at most one fresh navigation-only session first and a second only to resolve a stated ambiguity.

## Out of Scope

- Playback, click/play, full preload, media extraction, independent consumer, or production Gateway behavior.
- Bilibili selectors/pages/media beyond the fixed navigation diagnostic selector; no cookies, Authorization, credentials, proxy rotation, or open proxy.
- Changes to #191, #195, #188, #223, phone/TV deployment, VNC, CDP observation, or source-runtime stop/reconfigure/reuse.
- Real external-site access from Actions, local or tx-node compilation/build/test/package/install, or target resource provisioning.
- Canonical architecture/security changes without an explicit Coordinator design review.

## Architecture Invariants

- Gateway remains the PlaybackSession authority; the probe does not create a second business-state or consumer authority.
- Site/plugin authority owns the opaque Bilibili locator and allowed authority; Core does not gain site URL/DOM/cookie logic.
- EgressPolicy remains the only authority for DNS/address/port restrictions; no arbitrary CONNECT or open proxy is introduced.
- TLS hostname and certificate verification remain enforced by the component that owns the selected upstream TLS handshake.
- No Vault secret, Cookie, Authorization, token, or production credential crosses the broker or diagnostic output.
- Results are bounded, redacted, finalizer-safe, and late callbacks cannot overwrite a finalized outcome.

## Files Expected to Change

- Existing experimental broker/probe implementation only where the evidence-supported transport correction belongs.
- Existing broker deterministic test file(s) for the required regressions.
- No canonical architecture/security file unless Coordinator first approves a design change; such a change ends this Task for review.

## Implementation Requirements

1. Make transport mode explicit and internally consistent. If using end-to-end browser TLS, use a policy-pinned TCP connection and preserve the browser's TLS handshake bytes; if using broker-terminated HTTP/TLS, do not forward browser TLS bytes into that socket.
2. Keep `servername`/hostname and certificate verification aligned with the selected mode, and keep plugin-owned authority and EgressPolicy checks before connection.
3. Preserve one terminal failure/result precedence rule across handshake, transport, navigation, timeout, abort, and cleanup; late errors must be recorded only as bounded diagnostics and never overwrite a committed result.
4. Keep all output finite and sanitized: no raw error/URL/header/cookie/token/profile/media payloads. Preserve request/response/metadata ceilings and cleanup on every exit.
5. Add deterministic fake transport tests that prove browser-to-upstream and upstream-to-browser byte identity for the raw tunnel mode, or prove the broker HTTP request/response boundary for the terminated mode, plus failure precedence, redaction, limits, timeout, and cleanup.
6. Do not make a speculative bypass solely to make the target succeed. If evidence is insufficient to select a safe mode, leave the implementation unchanged and report the design decision as BLOCKED for Coordinator review.

## Verification Plan

### Claims

```text
C1: The selected CONNECT/request transport mode is explicit and protocol-correct; browser TLS bytes are never piped into a broker TLS wrapper, and the selected TLS owner preserves hostname/certificate verification.
C2: Deterministic tests prove handshake/byte forwarding, failure precedence, redaction, request/response/metadata limits, timeout, and cleanup without external network access.
C3: The exact Candidate passes all required GitHub-hosted verification jobs and its artifact manifest/digest matches that Candidate.
C4: After the hosted artifact gate, one fresh bounded tx-node navigation-only session provides sanitized target evidence about whether the protocol error boundary changed; a second session requires a stated ambiguity.
C5: SSRF/egress, plugin authority, secret, bounded output, finalizer, cleanup, source-runtime, no-playback, and no-device boundaries remain intact.
```

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1,C2 | github-actions | github-hosted-x64 | runner-self | yes | repository static/security checks and deterministic broker tests | run/job logs |
| J2 | C1,C2 | github-actions | github-hosted-x64 | runner-self | yes | portable Node/probe test matrix | run/job logs |
| J3a | C1,C2,C5 | github-actions | github-hosted-x64 | runner-self | yes | R008 boundary/regression checks | run/job logs |
| J3 | C1,C2,C5 | github-actions | github-hosted-x64 | runner-self | yes | workspace and repository regression suite | run/job logs |
| J4 | C1,C2,C5 | github-actions | github-hosted-x64 | runner-self | yes | `bilibili-browser-probe` hosted artifact/contract verification | run/job/artifact |
| JI1 | C3 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate integration/artifact manifest and digest gate | run/job/artifact |
| JT1 | C4 | external-codex | authenticated Tailscale SSH control plane | tx-node, gateway-verify | later Coordinator-controlled | fixed navigation-only selector after exact artifact gate | sanitized target report |

### Execution Plane

```text
Implementation / verification execution plane: github-actions
Later target evidence execution plane: external-codex over authenticated Tailscale SSH control plane
```

### Runner Selection

```text
Portable implementation and deterministic tests → GitHub-hosted x64
Later target navigation proof → tx-node with gateway-verify UID/GID 1001, only after Coordinator dispatch
```

### Long-running / repeated verification

No unbounded soak is required. Hosted deterministic tests must be finite. Target navigation is one fresh session first; a second is permitted only to distinguish a stated ambiguity, with bounded timeout, request, response, metadata, and output limits.

### Interactive debugging

```text
WSL / Windows / Ubuntu ARM64 external Codex required: no for implementation; external control plane only if Coordinator dispatches JT1
Reason: the suspected defect is testable through deterministic fake transports, while target proof is a separate authorized runtime action.
```

### Target verification

```text
Target proof required: yes, later and Coordinator-controlled
Target: tx-node, gateway-verify low-privilege runtime
Why target evidence is required: the accepted symptom occurred on tx-node Chromium and the transport correction must be checked in that bounded runtime.
```

### Runner Security Constraints

```text
Trusted candidate only: yes
Dedicated low-privilege runner user: gateway-verify UID/GID 1001 for later target proof
Vault/profile access: forbidden
Production service mutation: forbidden
Cleanup / timeout requirements: fresh Attempt-owned profile/display/debug endpoint; bounded session; cleanup and read-back of only Attempt-owned processes/files; source-runtime untouched
```

## Success Criteria

### Task success

1. The source-supported transport decision and any correction are documented in the Candidate/Issue evidence.
2. The selected mode cannot mix broker TLS wrapping with forwarded browser TLS bytes and does not weaken R008 authority or security boundaries.
3. Deterministic regressions cover transport behavior, precedence, redaction, limits, timeout, and cleanup.
4. Exact hosted verification and artifact evidence are available for Coordinator Review. Any target proof is reported separately and does not convert a concrete failure into PASS.

### Verification claim success

```text
C1 PASS when source and tests show one explicit transport mode, no TLS-layer mixing, and correct hostname/certificate ownership.
C2 PASS when deterministic tests cover both transport directions or the broker HTTP boundary plus all listed error/limit/cleanup cases without external network access.
C3 PASS when J1/J2/J3a/J3/J4/JI1 are green for one exact Candidate and the artifact manifest/entry/platform/size/digest read back matches it.
C4 PASS when the later target run records a finite sanitized result and cleanup; a protocol failure is valid evidence and is classified as FAIL/CONDITIONAL evidence rather than guessed away.
C5 PASS when boundary checks and review show no SSRF/egress/secret/plugin-authority/playback/source-runtime/device regression.
```

## Evidence Contract

Every report must separate implementation, hosted verification, and target evidence and record:

```text
Role: implementation | verification | target
Task / Claim: R008-TRANSPORT-DIAGNOSTIC / C1..C5
Attempt: actual Attempt number
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high
Fast: enabled
Job ID: actual GitHub job or JT1 when applicable
Execution plane: github-actions or external-codex
Runner class / image: actual hosted runner or tx-node target facts
Execution host / target: actual host and low-privilege identity when authorized
Base / Candidate commit: exact SHA
Workflow / run / job: exact run and job IDs
Commands / steps: bounded, sanitized command classes
Duration / repetitions / shards: actual values
Metrics / artifact / evidence location: actual logs/artifact/Issue comment
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

Never include secrets, cookies, tokens, account data, raw sensitive URLs, raw socket errors, or media payloads.

## Failure / Blocked Handling

- A protocol mismatch, deterministic regression, or preserved target `tls_protocol_error` is evidence and must be classified precisely; do not claim success from navigation reaching a later marker.
- Missing Actions, failed required jobs, missing/mismatched artifact, or inability to read back exact Candidate evidence is BLOCKED. Do not run target and do not use local build/test/install as fallback.
- A required canonical architecture/security change is BLOCKED pending Coordinator design review. Preserve the evidence and do not modify the invariant.
- A target startup/transport impossibility after the artifact gate is BLOCKED; a concrete bounded navigation result is valid target evidence.
- Any source-runtime mutation, secret/credential exposure, open proxy, unbounded output, or boundary regression fails the Task and requires Coordinator review.

## Deliverables

- Implementation / tests: the smallest evidence-supported transport correction or diagnostic seam and deterministic regressions.
- Candidate commit / PR: exact Candidate SHA and PR from the Worker Attempt.
- Session bootstrap prompt: `docs/tasks/226-r008-connect-tls-transport-semantics/prompt.md`.
- Verification Jobs / runs: exact J1/J2/J3a/J3/J4/JI1 IDs and artifact manifest/digest.
- Target Evidence: later Coordinator-controlled JT1 report, if dispatched.
- Research/design evidence: Issue comments and Candidate diff documenting the selected mode or Coordinator-review blocker.

## Issue Feedback / Iteration Protocol

Follow `docs/tasks/issue-lifecycle-protocol.md`. This package remains `status:draft` until Coordinator completes independent read-back and Publication Gate. Workers claim only `status:ready`, post `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, transition to `status:review`/`status:blocked`, release ownership, and stop. Coordinator alone reviews, reopens, accepts, closes, or dispatches later target work.

Do not process, claim, edit, or wait on #191 or #195. Do not alter #188 or #223 from this Task.

## Completion Protocol

A package-preparation Worker creates the Issue and exactly these two package files, performs GitHub read-back, and reports the package identifiers while leaving the Issue draft and owner-free. An execution Worker follows the standard Attempt report/status/release protocol. No Worker sets `status:done` or closes the Issue.
