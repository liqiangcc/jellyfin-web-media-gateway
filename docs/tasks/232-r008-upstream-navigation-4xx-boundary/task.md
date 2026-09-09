# Task — Identify the remaining upstream/navigation 4xx status boundary

## Metadata

```text
GitHub Issue: #232
Parent Goal / Research Item: Parent Goal #68; R008 browser navigation and egress evidence
Task / Research ID: R008-UPSTREAM-NAVIGATION-4XX-BOUNDARY
Task kind: combined
Planning base: 7930e1a1e1e898e75f1fc9d6991a595ba01b658a
Evidence base: 7930e1a1e1e898e75f1fc9d6991a595ba01b658a
Candidate commit: n/a until implementation Attempt
Session bootstrap prompt: docs/tasks/232-r008-upstream-navigation-4xx-boundary/prompt.md
Preferred worker: cloud
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, cloud-interactive
Hard publication dependencies: Coordinator Publication Gate; accepted #229 Attempt 2 evidence
```

## Session Bootstrap

`prompt.md` is only a navigation entry. The live Issue, this contract, canonical architecture/security documents, and lifecycle/recovery/freshness protocols are authoritative.

## Goal

Safely identify the remaining Bilibili main-navigation `4xx` status boundary after #229 established a finite `navigation_status` observation. Determine whether the status is broker-generated policy, a permitted upstream HTTP response, redirect/status handling, or unknown, without inferring anonymous-access policy or adding bypass authority.

## Frozen Evidence

#229 Attempt 2 was accepted and closed:

- Candidate `884035230ee504ff384b9780fb9eb6c59a245a30`, PR #231, merged main `7930e1a1e1e898e75f1fc9d6991a595ba01b658a`.
- Hosted workflow `bilibili-browser-probe`, run `34315789980`; required J1/J2/J3a/J3/J4/JI1 jobs passed.
- Exact artifact `10090029479`, size `4,221,960` bytes, digest `sha256:61ef5356938f740b0c2f5f4de6e05859112c7967e1f3f5ac4e8bfa94337ae41b`.
- One fresh tx-node navigation-only session observed upstream lifecycle activity, then `navigation_status=4xx` with fulfilled navigation and bounded `downstream_close`; 18 requests and 94,439 response bytes, no media; late callbacks reached 19 requests/99,545 bytes; cleanup passed.
- The accepted evidence deliberately did not infer a site-access or anonymous-policy cause.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: The bounded status/origin metadata seam, deterministic tests, hosted artifact gate and later single target observation share one narrow Claim set. Coordinator retains target dispatch authority.
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

Inspect the existing experimental browser probe and add the smallest evidence-supported status/origin seam. Record only closed response-origin classes, finite status classes and bounded metadata. Distinguish broker policy, permitted upstream HTTP, navigation/status handling and unknown only when the local observation establishes that class. A 4xx alone remains insufficient to claim anonymous or site policy.

### Verification

Verification must identify the exact Candidate, run deterministic offline fixtures in GitHub-hosted Actions and read back a matching artifact manifest/digest. Later target evidence, if dispatched by the Coordinator, remains a separate navigation-only result.

## Freshness / Integration Contract

```text
Freshness policy: dependency-aware
Semantic authorities: R008 EgressPolicy, plugin-owned selector/authority, browser-owned TLS transport, finite diagnostic/finalizer schema
Semantic freshness domains: experiments/bilibili-browser-probe/**, plugins/bilibili/**, docs/tasks/232-r008-upstream-navigation-4xx-boundary/**
Integration surfaces: package manifests/lockfiles, artifact workflow, shared diagnostic/finalizer/marker modules
Task-owned surfaces: experiments/bilibili-browser-probe/** and its deterministic tests
Authority/domain → Claim mapping:
  response/status schema and marker retention → C1,C2
  broker raw TCP/TLS and egress/plugin authority → C1,C5
  artifact workflow and manifest → C3
  target navigation ordering → C4
Unrelated-main policy: unrelated documentation or isolated plugin changes do not invalidate Task-specific evidence; Coordinator performs composition review.
Strict-main reason: none
```

If a semantic authority or integration surface changes before review, Coordinator must classify freshness and request only affected verification.

## Preconditions

- Read `AGENTS.md`, all canonical documents named there, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md` and `docs/tasks/freshness-integration-protocol.md`.
- Read the complete live #232 history and accepted #229 Attempt 2 evidence above. Do not rely on old chat or unverified local state.
- Issue must be `status:ready`, `env:cloud`, owner-free and Publication Gate read-back must pass before claim. Package preparation does not claim an Attempt.
- Existing plugin-owned authority and R008 EgressPolicy remain the only navigation/egress authorities. No site secret or authentication context is available or permitted.
- Hosted Actions must verify an exact Candidate and produce a matching manifest/digest artifact. If Actions or artifact delivery fails, report BLOCKED and do not use local fallback.
- Later target admission, if explicitly dispatched, uses fixed selector `bilibili:BV14V411W7r5:part-2`, fresh temporary profile/runtime, gateway-verify UID/GID 1001 when practical, bounded navigation-only execution and Attempt-owned cleanup.

## In Scope

- Define or refine a finite allowlisted response-origin/status vocabulary for the existing diagnostic schema.
- Record only coarse response metadata needed to distinguish broker-generated policy, permitted upstream HTTP, redirect/status handling and unknown.
- Make ordering explicit for broker request result, upstream response/body/socket lifecycle, navigation response/status, navigation promise settlement, navigation end, finalizer and cleanup.
- Preserve failure precedence and ensure late callbacks cannot replace finalized status/origin evidence.
- Add deterministic offline tests for broker policy 4xx, permitted upstream 2xx/3xx/4xx/5xx, redirect/status handling, unknown fallback, redaction, metadata/request/response limits, timeout and cleanup.
- Produce exact Candidate, hosted J1/J2/J3a/J3/J4/JI1 evidence and manifest/digest for Coordinator review.
- Permit one later fresh tx-node navigation-only session after the exact artifact gate; no second session is part of this contract.

## Out of Scope

- Playback, page click/play, full preload, media extraction, independent consumer or production Gateway changes.
- Raw URL, response body, header, error text, cookie, Authorization, token, account, certificate, profile or redirect target in evidence.
- Anonymous-access bypass, status-based access-control bypass, credential injection, proxy rotation, open proxy, arbitrary URL/host/port authority or weakening TLS/SSRF/egress policy.
- Changes to #191, #195, #188, #223, #226 or #229; phone/TV deployment, VNC/CDP observation or source-runtime stop/reconfigure/reuse.
- Real external-site access from Actions or any target action before Coordinator dispatch.
- Canonical architecture/security changes without Coordinator design review.

## Architecture Invariants

- Site Plugin owns the opaque Bilibili locator and allowed authority; Core and diagnostics do not gain site URL/DOM/cookie logic.
- EgressPolicy remains the authority for DNS/address/port restrictions and redirects are revalidated per hop.
- Browser-owned TLS remains end-to-end through policy-pinned raw TCP CONNECT; the broker does not terminate or inspect browser TLS.
- No Vault secret, Cookie, Authorization, token or sensitive response material crosses the probe boundary.
- The Gateway remains PlaybackSession authority; this diagnostic does not create playback or consumer authority.
- Diagnostic markers and metadata are finite, allowlisted, redacted and finalized once; late callbacks cannot overwrite terminal evidence.
- A response status is evidence of an HTTP/status boundary, not proof of an anonymous-access or site-policy decision.

## Files Expected to Change

- Existing experimental probe/broker diagnostic implementation where status/origin and ordering belong.
- Existing experimental deterministic tests for status, ordering, redaction, limits and cleanup.
- No canonical architecture/security file unless Coordinator explicitly approves the required design change; such a change stops this Task for review.

## Implementation Requirements

1. Use a closed response-origin vocabulary with safe `unknown` fallback, including broker policy, upstream HTTP and navigation/status observation classes without exposing authority.
2. Capture only finite `1xx`–`5xx` status classes plus bounded request, response-byte and metadata-byte counters. Never export status text, URL, headers, body, redirect target or error text.
3. Mark broker-generated policy responses separately from permitted upstream HTTP observations. For a raw CONNECT tunnel where the broker cannot observe HTTP status, use `unknown` or navigation/status observation rather than guessing upstream provenance.
4. Preserve event ordering: request start precedes response observation; upstream response/body/socket markers remain distinct from navigation status/promise/end; finalizer and cleanup seal the stream.
5. Preserve raw TCP CONNECT and browser TLS ownership from #226. Do not reintroduce a broker TLS wrapper.
6. Add deterministic tests without external network access for policy denial, permitted upstream status classes, redirects, navigation ordering, unknown fallback, redaction, budgets, timeout, precedence and cleanup.
7. Do not infer or publish anonymous/site policy from a 4xx. If evidence cannot distinguish the source safely, retain `unknown` and document the limitation.
8. Keep target-specific behavior behind the existing plugin-owned selector and live admission; do not add caller-controlled URLs, proxy, credentials or browser endpoints.

## Verification Plan

### Claims

```text
C1: Response origin/status metadata is finite, sanitized and ordered correctly with upstream and navigation lifecycle markers.
C2: Deterministic offline tests cover broker policy, permitted upstream statuses, redirects, unknown fallback, redaction, bounds, timeout, precedence and cleanup.
C3: One exact Candidate passes hosted J1/J2/J3a/J3/J4/JI1 and its artifact manifest/digest matches the accepted identity.
C4: After the artifact gate, one fresh bounded tx-node navigation-only session records new sanitized boundary evidence; no second session is required.
C5: R008 SSRF/egress/TLS/plugin-authority/secret/finalizer/source-runtime/no-playback/no-device boundaries remain intact.
```

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1,C2 | github-actions | github-hosted-x64 | runner-self | yes | probe contract, diagnostic, transport, marker and lifecycle tests | run/job logs |
| J2 | C1,C2,C5 | github-actions | github-hosted-x64 | runner-self | yes | hosted Chromium containment and deterministic broker harness | run/job/artifact |
| J3a | C3 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate manifest-addressed artifact build/verify | run/job/artifact |
| J3 | C3,C5 | github-actions | github-hosted-x64 | runner-self | yes | downloaded exact artifact verification and independent hosted contract | run/job |
| J4 | C1,C2,C5 | github-actions | github-hosted-x64 | runner-self | yes | static response schema/authority and no-secret/no-playback boundary checks | run/job logs |
| JI1 | C3,C5 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate repository integration regressions | run/job logs |
| JT1 | C4 | external-codex | authenticated Tailscale SSH | tx-node/gateway-verify | later Coordinator-controlled | fixed selector, one fresh navigation-only session | sanitized target report |

### Execution Plane and Runner Selection

```text
Implementation / required verification: github-actions on GitHub-hosted x64
Later target proof: external-codex via authenticated Tailscale SSH control plane
Later target runtime: gateway-verify UID/GID 1001 with fresh disposable profile/runtime
```

All hosted tests and target navigation are finite and bounded. Target proof is one session only; no soak, playback, consumer or repeated target request.

### Target Verification

```text
Target proof required: yes, later and Coordinator-controlled
Target: tx-node, gateway-verify low-privilege runtime
Why: #229 established the 4xx navigation boundary but could not safely distinguish broker policy from permitted upstream HTTP/status handling.
```

## Success Criteria

1. The probe records a finite response-origin/status observation that is more precise where evidence supports it and uses `unknown` otherwise.
2. Broker/upstream/navigation ordering is deterministic, bounded and finalizer-safe.
3. Deterministic tests cover policy/status/redirect/unknown cases, redaction, limits, timeout, precedence and cleanup.
4. Exact hosted verification and artifact evidence are available for Coordinator Review; target evidence remains separate.
5. No anonymous-access claim or access-control change is made solely from a 4xx.

```text
C1 PASS: only allowlisted response-origin/status classes and bounded counters are emitted with ordering markers.
C2 PASS: listed offline fixtures and negative/redaction/limit/cleanup tests pass without external network.
C3 PASS: J1/J2/J3a/J3/J4/JI1 are green for one exact Candidate and artifact identity matches.
C4 PASS: one later bounded target session returns finite sanitized evidence and cleanup; no guessed site diagnosis.
C5 PASS: static/security checks show no bypass, open proxy, secret, source-runtime, playback, device or TLS ownership regression.
```

## Evidence Contract

Every Attempt report must distinguish Implementation Result, hosted Verification Result and Target Result and include:

```text
Role: implementation | verification | target
Task / Claim: R008-UPSTREAM-NAVIGATION-4XX-BOUNDARY / C1..C5
Attempt: actual Attempt number
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high
Fast: enabled
Job ID: actual hosted job or JT1
Execution plane: github-actions or external-codex
Runner / target: actual hosted class and target facts
Base / Candidate: exact SHA
Workflow / run / job: exact IDs
Commands / selector: bounded sanitized command classes and fixed selector only when target-authorized
Duration / repetition: actual values
Metrics / artifact / evidence location: actual sanitized output and artifact details
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

Never submit secrets, cookies, tokens, raw sensitive URLs, raw headers, raw bodies or unnecessary large files.

## Failure / Blocked Handling

- A concrete broker policy response, permitted upstream HTTP status, redirect/status failure or unknown origin is valid evidence and must retain its limitation.
- Failed required hosted jobs, unavailable Actions or mismatched/missing artifact are BLOCKED; do not use local fallback or target execution.
- A required canonical architecture/security change is BLOCKED pending Coordinator design review.
- Target startup failure after the artifact gate is BLOCKED; a bounded target 4xx is evidence and is not a blocker by itself.
- Any status-based bypass, raw sensitive output, open proxy, TLS/SSRF/secret regression, unbounded callback or source-runtime/device mutation fails the Task and requires Coordinator review.

## Deliverables

- Implementation/tests: finite status/origin and navigation-ordering correction with deterministic regressions.
- Candidate/PR: exact Candidate SHA and focused PR.
- Session bootstrap: `docs/tasks/232-r008-upstream-navigation-4xx-boundary/prompt.md`.
- Verification: exact J1/J2/J3a/J3/J4/JI1 runs and artifact manifest/digest.
- Target evidence: one later Coordinator-controlled JT1 report.
- Design evidence: Issue comments documenting unknown limitations or any Coordinator-review blocker.

## Issue Feedback / Iteration Protocol

Follow `docs/tasks/issue-lifecycle-protocol.md`. Package remains `status:draft` until independent Publication Gate read-back. Execution Workers claim only `status:ready`, post `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, transition to review/blocked, release ownership and stop. Coordinator alone publishes, reviews, accepts, closes or dispatches target work.

Do not process, claim, edit or wait on #191 or #195. Do not modify #229 or earlier parent Issues from this Task.

## Completion Protocol

Package preparation creates the Issue and exactly two package files, performs GitHub read-back, reports real identifiers and leaves the Issue draft/owner-free. Later execution follows the standard Attempt report/status/release protocol. No Worker sets `status:done` or closes the Issue.

