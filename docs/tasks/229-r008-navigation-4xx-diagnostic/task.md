# Task — Classify the remaining main-navigation 4xx response

## Metadata

```text
GitHub Issue: #229
Parent Goal / Research Item: Parent Goal #68; R008 browser navigation and egress evidence
Task / Research ID: R008-NAV-4XX-DIAGNOSTIC
Task kind: combined
Base commit: 7a90759af6abd92a9bf8f57f77c514a3c0a61e9a
Candidate commit: n/a until implementation Attempt
Session bootstrap prompt: docs/tasks/229-r008-navigation-4xx-diagnostic/prompt.md
Preferred worker: cloud
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, cloud-interactive
Hard publication dependencies: Coordinator Publication Gate; accepted #226 evidence
```

## Session Bootstrap

`prompt.md` is only a navigation entry. The live Issue, this contract, canonical architecture/security documents, and lifecycle/recovery/freshness protocols are authoritative.

## Goal

Diagnose the remaining Bilibili main-navigation `4xx` response after the #226 CONNECT/TLS correction by recording finite status/response metadata and browser navigation ordering that can distinguish broker policy/status behavior from an upstream anonymous/site-access response, without exposing raw URLs, bodies, headers, credentials, or bypass authority.

## Why / Context

#226 established that the earlier `tls_handshake / tls_protocol_error` was caused by inconsistent CONNECT transport ownership: raw policy-pinned TCP now forwards browser TLS bytes and the browser owns origin TLS/SNI/certificate verification. Its target session progressed through upstream response start, body completion, and socket close; the navigation promise fulfilled, then the response was `4xx` and the final bounded result was `downstream_close`. The session made 16 requests and counted 117,488 response bytes with 0 metadata bytes, and had no media/playback activity. The next useful boundary is to determine whether the 4xx is produced by broker policy, an upstream HTTP response, redirect/status handling, or anonymous/site-access policy.

Frozen parent evidence:

- Candidate `148bccc6a48dde637518e98addb6160f5ad4d146`, PR #228, merged main `7a90759af6abd92a9bf8f57f77c514a3c0a61e9a`.
- Hosted run `34313510150`; required J1/J2/J3a/J3/J4/JI1 jobs passed after the documented JI1 rerun.
- Artifact `10089250396`, 4,219,957 bytes, digest `sha256:73096f790ab5621ab78613721a7165f2d219191f386ebfa7139fa1f69087708a`.
- One fresh target session had `upstream_response_start`, `upstream_response_end/body_complete`, `upstream_socket_close/closed_after_body`, `navigation_promise_result=fulfilled`, `navigation_status=4xx`, and final bounded `navigation_promise_rejected/downstream_close`; media request, click, play, preload, and consumer were false; cleanup passed.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: The status classification seam, deterministic tests, hosted artifact gate, and later bounded target observation use one narrow diagnostic Claim set. Coordinator retains target dispatch authority.
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

Inspect the experimental broker/probe and introduce the smallest evidence-supported diagnostic seam. The seam must preserve the distinction between:

- broker policy outcomes such as denied host/scheme/port, broker-generated 4xx, or CONNECT response;
- upstream HTTP response status classes after a permitted request reaches the upstream;
- redirect/status handling observed by browser navigation; and
- unknown/unclassified status origin.

Use only closed enumerations and bounded counters/metadata. Do not infer site access rules from a status alone, and do not add a bypass, credentials, cookie, Authorization header, alternate proxy, or arbitrary URL authority. If an accurate distinction requires changing a canonical architecture or security invariant, stop and return the evidence for Coordinator design review.

### Verification

Verification must identify the exact Candidate. It must prove status classification and ordering using deterministic offline fixtures, then produce exact hosted Actions and artifact evidence. Target evidence, if later dispatched, remains separate from repository verification.

## Task vs Job Boundary

```text
Task
→ finite status/response metadata and navigation ordering
→ exact Candidate
→ GitHub-hosted verification jobs
→ Coordinator-controlled target navigation evidence
```

Jobs do not claim this Issue or authorize Bilibili access.

## Routing Rationale

Implementation and deterministic verification use Codex Cloud with GitHub-hosted Actions as the only build/test/package execution plane. A later target proof uses the existing authenticated Tailscale SSH control plane and low-privilege gateway-verify runtime only after exact Candidate/artifact gate. No local compilation, test, install, or target mutation is permitted during package or implementation work.

## Preconditions

- Read `AGENTS.md`, all canonical documents named there, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, and `docs/tasks/freshness-integration-protocol.md`.
- Read the complete live #229 history and accepted #226 Attempt 2 evidence above. Do not rely on old chat or unverified local state.
- Issue must be `status:ready`, `env:cloud`, owner-free, and Publication Gate read-back must pass before claim. Draft package preparation cannot claim an Attempt.
- Existing plugin-owned authority and R008 EgressPolicy are the only navigation/egress authorities. No site secret or authentication context is available or permitted.
- Required Actions must verify an exact Candidate and produce a manifest/digest artifact. If Actions or artifact delivery fails, report BLOCKED and do not use local fallback.
- Later target admission, if Coordinator dispatches it, uses the fixed selector `bilibili:BV14V411W7r5:part-2`, a fresh temporary profile/runtime, gateway-verify UID/GID 1001 when practical, bounded navigation-only execution, and Attempt-owned cleanup.

## In Scope

- Define a finite allowlisted status origin/class vocabulary appropriate to the existing diagnostic schema.
- Record only coarse response metadata needed to distinguish broker policy, upstream HTTP, redirect/status handling, and unknown outcomes.
- Make event ordering explicit for request start, response/status, response body/socket lifecycle, navigation promise settlement, navigation status/end, finalizer, and cleanup.
- Preserve status precedence and prevent late callbacks from replacing a finalized result.
- Add deterministic offline tests for 4xx/3xx/2xx status classes, policy denial, redirect handling, upstream response/body/socket ordering, bounded metadata, redaction, limits, timeout, and cleanup.
- Produce exact Candidate, hosted J1/J2/J3a/J3/J4/JI1 evidence and manifest/digest for Coordinator review.
- Permit one later fresh tx-node navigation-only session first after the exact artifact gate; a second requires a stated ambiguity.

## Out of Scope

- Playback, page click/play, full preload, media extraction, independent consumer, or production Gateway changes.
- Raw URL, response body, header, cookie, Authorization, token, account, certificate, or profile data in evidence.
- Anonymous-access bypass, status-based policy bypass, credential injection, proxy rotation, open proxy, arbitrary URL/host/port authority, or weakening TLS/SSRF/egress policy.
- Changes to #191, #195, #188, #223, or #226; phone/TV deployment; VNC/CDP observation; source-runtime stop/reconfigure/reuse; local build/test/package/install.
- Real external-site access from Actions or target actions before Coordinator dispatch.
- Canonical architecture/security changes without Coordinator design review.

## Architecture Invariants

- Site Plugin owns the opaque Bilibili locator and allowed authority; Core and diagnostics do not gain site URL/DOM/cookie logic.
- EgressPolicy remains the authority for DNS/address/port restrictions and redirects are revalidated per hop.
- Browser-owned TLS remains end-to-end through policy-pinned raw TCP CONNECT; the broker does not terminate or inspect browser TLS.
- No Vault secret, Cookie, Authorization, token, or sensitive response material crosses the probe boundary.
- The Gateway remains PlaybackSession authority; the diagnostic does not create a playback or consumer authority.
- Diagnostic markers and metadata are finite, allowlisted, redacted, and finalized once; late callbacks cannot overwrite the terminal result.

## Files Expected to Change

- Existing experimental probe/broker diagnostic implementation where the status/ordering seam belongs.
- Existing experimental deterministic tests for status, lifecycle, redaction, limits, and cleanup.
- No canonical architecture/security file unless Coordinator explicitly approves the required design change; such a change stops this Task for review.

## Implementation Requirements

1. Use a closed `response_origin`/status classification (or an equivalent existing schema field) with safe `unknown` fallback. The vocabulary must distinguish broker policy, upstream HTTP, navigation/status handling, and unknown without carrying raw authority.
2. Capture status class only as finite `1xx`–`5xx`/`unknown`; bound response and metadata counters by the existing budgets. Do not export raw status text, URLs, headers, body, redirect targets, or errors.
3. Preserve event ordering and precedence: request start precedes response observation; response/body/socket markers remain distinguishable from navigation promise/status/end; finalizer and cleanup seal the stream; late status/errors cannot replace a committed failure or success.
4. Keep the current raw TCP CONNECT semantics and browser TLS ownership intact. Do not reintroduce `tls.connect()` for a browser byte tunnel.
5. Add meaningful deterministic tests without external network access, including broker denial versus permitted upstream 4xx, redirect/status classification, body/socket closure before and after navigation settlement, redaction, request/response/metadata ceilings, timeout, and cleanup.
6. Do not manufacture a site-access diagnosis from a 4xx. If evidence cannot distinguish origin safely, report `unknown` and preserve that limitation.

## Verification Plan

### Claims

```text
C1: Main-navigation status/response origin is represented by a finite, sanitized vocabulary and is ordered correctly with response/body/socket and navigation lifecycle markers.
C2: Deterministic offline tests cover broker policy denial, upstream status classes, redirect/status handling, unknown fallback, redaction, limits, timeout, precedence, and cleanup.
C3: Exact Candidate passes GitHub-hosted J1/J2/J3a/J3/J4/JI1 and produces a matching manifest/digest artifact.
C4: After the artifact gate, one fresh bounded tx-node navigation-only session provides sanitized evidence about the remaining 4xx boundary; a second requires a stated ambiguity.
C5: R008 SSRF/egress/TLS/plugin-authority/secret/finalizer/source-runtime/no-playback/no-device boundaries remain intact.
```

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1,C2 | github-actions | github-hosted-x64 | runner-self | yes | probe contract, diagnostic, transport and lifecycle tests | run/job logs |
| J2 | C1,C2,C5 | github-actions | github-hosted-x64 | runner-self | yes | hosted Chromium containment and deterministic broker harness | run/job/artifact |
| J3a | C3 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate manifest-addressed artifact build/verify | run/job/artifact |
| J3 | C3,C5 | github-actions | github-hosted-x64 | runner-self | yes | downloaded exact artifact verification and independent hosted contract | run/job |
| J4 | C1,C2,C5 | github-actions | github-hosted-x64 | runner-self | yes | static authority, status schema, and no-secret/no-playback boundary checks | run/job logs |
| JI1 | C3,C5 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate repository integration regressions | run/job logs |
| JT1 | C4 | external-codex | authenticated Tailscale SSH | tx-node/gateway-verify | later Coordinator-controlled | fixed selector, navigation-only, one fresh session first | sanitized target report |

### Execution Plane

```text
Implementation / required verification: github-actions
Later target proof: external-codex via authenticated Tailscale SSH control plane
```

### Runner Selection

```text
Portable implementation/tests/artifact → GitHub-hosted x64
Later tx-node navigation proof → gateway-verify UID/GID 1001, fresh disposable runtime
```

### Long-running / repeated verification

All hosted tests and target navigation are finite and bounded. Target session count is one first; a second is allowed only to resolve an explicitly recorded ambiguity. No soak or repeated target requests are allowed.

### Interactive debugging

```text
WSL / Windows / Ubuntu ARM64 external Codex required: no for implementation; external control plane only for later Coordinator-dispatched JT1
Reason: deterministic fixtures should establish the classification; tx-node is needed only to observe the target-specific 4xx boundary.
```

### Target verification

```text
Target proof required: yes, later and Coordinator-controlled
Target: tx-node, gateway-verify low-privilege runtime
Why target evidence is required: the remaining 4xx ordering was observed only in tx-node Chromium after the transport correction.
```

### Runner Security Constraints

```text
Trusted candidate only: yes
Dedicated low-privilege runner user: gateway-verify UID/GID 1001 for later target proof
Vault/profile access: forbidden
Production service mutation: forbidden
Cleanup / timeout requirements: fresh Attempt-owned profile/runtime; bounded session and counters; cleanup/read-back of only Attempt-owned files/processes; source-runtime untouched
```

## Success Criteria

### Task success

1. A finite status/response-origin diagnostic seam makes the 4xx boundary more precise without exposing sensitive material or adding authority.
2. Navigation and upstream lifecycle ordering is deterministic, bounded, and finalizer-safe.
3. Deterministic tests cover policy/status/redirect/unknown cases, redaction, limits, timeout, precedence, and cleanup.
4. Exact hosted verification and artifact evidence are available for Coordinator Review; target evidence remains separate.

### Verification claim success

```text
C1 PASS when only allowlisted status/origin classes and bounded counters are emitted, with ordering markers proving the observed boundary.
C2 PASS when all listed offline fixtures and negative/redaction/limit/cleanup tests pass without external network.
C3 PASS when J1/J2/J3a/J3/J4/JI1 are green for one exact Candidate and artifact Candidate/entry/platform/manifest/digest read back matches.
C4 PASS when later JT1 returns finite sanitized target evidence and cleanup; a concrete 4xx remains evidence, not a guessed site diagnosis.
C5 PASS when security/static checks show no bypass, open proxy, secret, source-runtime, playback, device, or TLS ownership regression.
```

## Evidence Contract

Reports must distinguish Implementation Result, hosted Verification Result, and Target Result and include:

```text
Role: implementation | verification | target
Task / Claim: R008-NAV-4XX-DIAGNOSTIC / C1..C5
Attempt: actual Attempt number
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high
Fast: enabled
Job ID: actual job or JT1
Execution plane: github-actions or external-codex
Runner / target: actual hosted class and target facts
Base / Candidate: exact SHA
Workflow / run / job: exact IDs
Commands / selector: bounded, sanitized command classes and fixed selector when target-authorized
Duration / repetition: actual values
Metrics / artifact / evidence location: actual sanitized output and artifact details
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

Never submit secrets, cookies, tokens, raw sensitive URLs, raw headers, raw bodies, or unnecessary large files.

## Failure / Blocked Handling

- A concrete broker denial, upstream 4xx, redirect/status failure, or unknown origin is valid evidence and must retain the precise limitation.
- Failed required hosted jobs, unavailable Actions, or mismatched/missing artifact are BLOCKED; do not use local fallback or target execution.
- A required canonical architecture/security change is BLOCKED pending Coordinator design review.
- Target startup failure after the artifact gate is BLOCKED; a bounded target 4xx is evidence and is not a blocker by itself.
- Any status-based bypass, raw sensitive output, open proxy, TLS/SSRF/secret regression, unbounded callback, or source-runtime/device mutation fails the Task and requires Coordinator review.

## Deliverables

- Implementation/tests: finite status/response-origin and navigation-ordering correction with deterministic regressions.
- Candidate/PR: exact Candidate SHA and focused PR.
- Session bootstrap: `docs/tasks/229-r008-navigation-4xx-diagnostic/prompt.md`.
- Verification: exact J1/J2/J3a/J3/J4/JI1 runs and artifact manifest/digest.
- Target evidence: later Coordinator-controlled JT1 report, if dispatched.
- Design evidence: Issue comments documenting unknown limitations or Coordinator-review blocker.

## Issue Feedback / Iteration Protocol

Follow `docs/tasks/issue-lifecycle-protocol.md`. Package remains `status:draft` until independent Publication Gate read-back. Execution Workers claim only `status:ready`, post `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, transition to review/blocked, release owner, and stop. Coordinator alone reviews, reopens, accepts, closes, or dispatches target work.

Do not process, claim, edit, or wait on #191 or #195. Do not modify #226 or earlier parent Issues from this Task.

## Completion Protocol

Package preparation creates the Issue and exactly two package files, performs GitHub read-back, reports real identifiers, and leaves the Issue draft/owner-free. Execution later follows the standard Attempt report/status/release protocol. No Worker sets `status:done` or closes the Issue.
