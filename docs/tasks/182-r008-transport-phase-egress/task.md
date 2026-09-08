# Task — R008 broker transport phase and tx-node egress preflight

## Metadata

```text
GitHub Issue: #182
Parent Goal / Research Item: #68 / R008 / Issue #179
Task / Research ID: R008-TX-NODE-TRANSPORT-PHASE
Task kind: combined
Base commit: f1772939c2535532443379085bdd798f62775602
Candidate commit: n/a until Worker Attempt
Session bootstrap prompt: docs/tasks/182-r008-transport-phase-egress/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-test, cloud-interactive, interactive-linux-debug, authenticated SSH tx-node
Hard publication dependencies: Issue #179 Final Acceptance; accepted #176 diagnostic authority; accepted #169/#172 browser/artifact authorities; current R008/security contracts
```

> GitHub Actions is the build, test and package authority. The tx-node session is a bounded runtime observation only; it must never compile, install packages, or mutate a service.

## Session Bootstrap

The Worker starts from `docs/tasks/182-r008-transport-phase-egress/prompt.md`. The prompt only navigates to this contract and the live Issue; this file is the sole Task contract.

## Goal

Extend the experimental browser broker so a failure or close is classified at the earliest known transport stage—resolve/policy, TCP connect, TLS handshake, proxy/tunnel response, downstream close, or unknown—while retaining only bounded schema-safe data. Prove the seam with deterministic GitHub-hosted tests and use one fresh, read-only tx-node preflight through the accepted artifact to determine which coarse stage the current broker path reaches. The preflight must not navigate a page, request media, run a Bilibili selector, or act as an egress service.

A completed Task is a diagnostic result. It is not a Bilibili compatibility result, a production egress authorization, a Gateway playback result, or permission to change Issue #166.

## Why / Context

Issue #166 stopped before page observation with a broker connection failure. Issue #176 accepted a sanitized `broker_connect_failed` result from a clean tx-node browser session, but that enum intentionally discards the underlying transport subphase. Issue #179 accepted the evidence as a conditional R008 decision and selected this smallest next experiment. The runtime shape of tx-node is already admitted; the unresolved fact is the broker transport boundary.

## Task Decomposition Decision

```text
Verification mode: inline for the candidate and hosted fixtures; target transport evidence is a separately gated job in this same Task
Linked implementation task: n/a
Linked verification task: n/a; a new child Issue is created only if target evidence requires an independent lifecycle or a contract change
Decision reason: implementation and the bounded transport observation share one exact Candidate and one diagnostic schema, while the target action is independently gated and may return BLOCKED without changing the implementation result
```

Do not split work by x64/ARM64/tx-node runner. If the target capability is unavailable, report a blocker and let the Coordinator decide whether a separate verification Task is needed.

## Worker Routing Decision

```text
Worker/client: cloud-codex (gpt-5.6-luna, reasoning high; record whether Fast is exposed)
Environment: env:cloud
Implementation authority: GitHub Actions exact-Candidate workflows
Target execution: authenticated SSH to tx-node as the admitted low-privilege user
```

Cloud is the orchestrator/Worker and is not a build Runner or a phone/TV. No phone deployment, physical TV, VNC, or production Gateway is part of this Task.

## Work Role

### Implementation

The Candidate must make the experimental broker lifecycle observable at bounded transport stages without widening authority:

- Preserve the existing public-host allowlist, DNS/address policy and pinning, origin TLS verification, per-hop redirect checks, secret-header stripping, request/response/metadata/time/cancellation budgets, and disabled upgrade behavior.
- Add a finite transport-stage field or equivalent additive diagnostic record. The canonical stage vocabulary is `resolve_policy`, `tcp_connect`, `tls_handshake`, `proxy_response`, `downstream_close`, and `unknown`. Existing coarse `phase` output may remain for compatibility, but the new stage must be explicit and versioned/tested.
- Add a finite reason vocabulary for transport outcomes. It may expose only stable classes such as policy denial, DNS failure, connection refused/reset/timeout, TLS certificate/protocol/timeout, proxy response/tunnel failure, downstream close, or unclassified failure. Do not export the original error code/message.
- Record the earliest stage and an explicit success/failure/unknown outcome. A normal close must not be represented as a transport failure merely because a socket ended.
- Keep all counters bounded by the existing limits (200 requests, 32 MiB response bytes, 1 MiB retained metadata per session) and ensure malformed or over-limit inputs collapse to safe defaults.
- If the artifact layout or schema changes, update its manifest, contract tests, runbook references, and workflow checks in the same Candidate. Do not change production Gateway/Core/SiteAdapter code.

### Verification

Claims to verify:

```text
C1: each deterministic broker fixture identifies the earliest permitted transport stage and emits only the allowlisted schema, counters and outcome
C2: exact-Candidate hosted tests and artifact checks prove stage coverage, error-message non-retention, budget bounds, cleanup and unchanged security boundaries
C3: one fresh tx-node transport preflight reaches a known stage or reports unknown without overclaiming, and leaves no process/profile/staging residue
C4: no architecture/security invariant or accepted parent Issue status is weakened or mutated
```

## Task vs Job Boundary

```text
Task #182
→ C1–C4
→ cloud Worker with authenticated tx-node capability
→ hosted Actions jobs + one gated tx-node preflight
→ sanitized evidence
→ Coordinator Review / Final Acceptance
```

A Job does not claim an Issue and cannot unlock #166. Hosted generic ARM64, tx-node, and any later TV evidence are execution slices of these claims, not separate business Tasks unless lifecycle or Evidence Authority changes.

## Routing Rationale

- Repository changes and deterministic checks use Codex Cloud plus GitHub-hosted Actions.
- The tx-node preflight is allowed only because this Task explicitly requires authenticated SSH and the target is already admitted as a low-privilege Linux browser host. It is one transport request, not a general LAN or proxy capability.
- No local compile/test/package command is permitted in the Worker workspace, WSL, or tx-node. The Worker may perform read-only inspection and Git operations locally; required build/test/package evidence must be from GitHub Actions.

## Preconditions

- Read `AGENTS.md`, `docs/README.md`, `docs/requirements.md`, `docs/architecture.md`, `docs/implementation-contracts.md`, `docs/technical-feasibility-validation.md`, `docs/mvp-plan.md`, `docs/security.md`, `docs/development-environments.md`, `docs/runner-execution-architecture.md`, the R008 research record, and the Issue lifecycle/freshness/recovery protocols.
- Read the complete history of Issues #166, #169, #172, #176 and #179 and verify their accepted commits/artifacts from GitHub.
- Base main is `f1772939c2535532443379085bdd798f62775602`; do not silently substitute another base if the contract or accepted dependency changes.
- The admitted target is `VM-0-11-ubuntu`, Ubuntu 26.04, x86_64, user `gateway-verify` UID/GID 1001, Node 22.22.1, npm 9.2.0, and `/usr/bin/google-chrome-stable` Google Chrome 152.0.7977.82. Repeat these checks read-only before any target request and record only coarse facts.
- The target environment must have no HTTP(S)/ALL/NO proxy variables, no existing `gateway-verify` probe/browser process, and a fresh mode-700 staging directory. Never reuse the source-runtime profile or any Gateway/Chrome profile.
- The target artifact must be built, manifest-verified, and archived by GitHub Actions from the exact Candidate SHA. No npm install, package manager cache, workspace `node_modules` link, compiler, or package build may run on tx-node.

## In Scope

1. Extend only the experimental broker/probe modules, their tests, manifest/artifact inputs, runbook, and relevant GitHub Actions workflow steps.
2. Add deterministic fake resolver/socket/TLS/proxy/close fixtures for every stage and for unknown fallback. Test that sensitive error messages containing URLs, addresses, certificate text, header names, query markers, or token-like values never appear in JSON, logs, artifacts, or Issue reports.
3. Preserve and test host/scheme/port policy, public-address checks, SNI/certificate verification, redirect/upgrade denial, secret stripping, request/byte/metadata/time/cancellation limits, candidate non-export and final cleanup.
4. Package the exact Candidate runtime in Actions and verify its manifest/digest. The artifact may contain the new transport preflight entry point and its tests, but no browser binary or installer tree.
5. After hosted checks are green and the Worker has a concrete exact artifact, perform at most one target preflight via authenticated SSH. The preflight may check Chrome version and launch no browser page; its single network action must use a plugin-owned public authority (`bilibili`/`www.bilibili.com:443`) through the experimental broker with no caller URL, selector, proxy, credentials, profile, VNC, CDP, or arbitrary headers.
6. Report implementation outcome, hosted verification, target observation, and Coordinator decision as separate fields in the Issue history.

## Out of Scope

- Bilibili page navigation, selector execution, play/click/full preload, media body/candidate retrieval, independent consumer, login, DRM, cookies, Authorization, account data, or private redirects.
- Any Gateway/Core/Playback/Display/Control production behavior, SiteAdapter registry change, yt-dlp fallback, relay/proxy implementation, TLS MITM or certificate verification bypass.
- Installing/configuring a proxy, changing DNS/routes/firewall, changing Tailscale/VNC/CDP state, reusing profiles, deploying or modifying a phone, physical TV, Jellyfin, or production service.
- Local compilation, local test binaries, npm install, package installation, or browser downloads outside GitHub Actions.
- Retaining raw endpoint URLs, IP addresses, TLS transcripts, certificates, headers, bodies, DOM/HAR, error messages, cookies, tokens, or large artifacts.
- Changing Issue #166 state, rerunning its live page probe, or interpreting a target transport result as Gateway playback proof.

## Architecture Invariants

- Gateway remains the `PlaybackSession` authority; this experiment is not a production playback path.
- Site Browser Worker remains a generic Chromium runtime; plugin-owned authority is opaque and no caller supplies URL/host/selector for the preflight.
- The broker owns DNS/address validation, origin TLS verification, redirect/upgrade policy, secret containment, and budgets; no open proxy or caller-selected egress is introduced.
- No Site Plugin reads Vault or bypasses `EgressPolicy`; no Secret, Cookie, Authorization, signed URL, or private address crosses the observation boundary.
- A diagnostic failure cannot stop or mutate a production playback session, and a target process cannot inherit Vault, production secrets, root/ADB permissions, or a source-runtime profile.

## Files Expected to Change

- `experiments/bilibili-browser-probe/diagnostic.mjs` and/or a narrowly scoped transport helper.
- `experiments/bilibili-browser-probe/live.mjs` and/or a no-page transport-preflight entry point.
- Deterministic probe/broker tests and leakage/static-boundary fixtures.
- `experiments/bilibili-browser-probe/artifact.mjs`, manifest inputs, and package metadata only if required by the new entry point.
- `.github/workflows/bilibili-browser-probe.yml` and the bounded runbook/research documentation as needed.

Do not modify production Rust/Core/Playback/Display code for this Task.

## Implementation Requirements

1. Use argv/structured APIs and bounded timers; do not concatenate shell commands or accept arbitrary URL/host/proxy/header input.
2. Preserve the current `phase`/reason schema for existing accepted diagnostic consumers or make any schema version change explicit, additive where possible, and covered by migration/contract tests.
3. Keep raw errors in memory only long enough to map an allowlisted marker. The returned object and every durable log/artifact must be reconstructable from finite enums, status classes, counters, cleanup flags and boolean policy results.
4. Ensure transport stage transitions are idempotent and stale/late socket callbacks cannot overwrite an already finalized outcome.
5. Keep target preflight code independent from browser page navigation. It must fail closed if authority, port, proxy environment, budget, artifact identity, user, or cleanup admission is not exact.
6. No implementation is accepted from local execution. Push a candidate branch/PR and rely on the exact-Candidate GitHub Actions jobs below.

## Verification Plan

### Claims

```text
C1: deterministic stage classification is complete and schema-safe.
C2: hosted artifact, security, leakage, cleanup and regression checks pass.
C3: one target transport preflight records a coarse result and cleans up without page/media/secret activity.
C4: architecture, R008, SSRF and Secret boundaries remain intact; #166 is untouched.
```

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate `probe-contract` plus diagnostic/transport tests | run + logs |
| J2 | C1,C2 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate broker containment/cleanup fixtures and static leakage scan | run + logs + sanitized artifact |
| J3 | C2 | github-actions | github-hosted-x64 | runner-self | yes | manifest-addressed artifact build/verify and downloaded consumer/entry-point check | run + artifact digest |
| J4 | C2,C4 | github-actions | github-hosted-x64 | runner-self | yes | live admission/static boundary and workspace integration regressions | run + logs |
| JI1 | C2,C4 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate integration checks required by the workflow | run + logs |
| JT1 | C3 | external-codex | authenticated SSH to tx-node | `gateway-verify` / VM-0-11-ubuntu | yes after hosted PASS | one no-page broker transport preflight using the exact Actions artifact; no selector/media/consumer | sanitized target record + cleanup proof |

If the repository workflow names or job selectors change, the Worker must record the actual run/job IDs and explain the mapping. Do not claim a skipped or unrelated job as required evidence.

### Execution Plane

```text
Implementation and required build/test/package: github-actions
Target read-only transport observation: external-codex over authenticated SSH to tx-node
```

### Runner Selection

Portable tests and artifact work run on GitHub-hosted x64. Generic ARM64 jobs may be added only if a real claim requires them; they do not substitute for tx-node. The target preflight is not a phone proof and cannot be moved to a self-hosted phone Runner.

### Interactive debugging

```text
WSL / Windows / Ubuntu ARM64 external Codex required: no
Reason: the single target action is an explicitly bounded SSH command; no phone/ADB/TV interaction is in scope.
```

### Target verification

```text
Target proof required: yes
Target: tx-node / VM-0-11-ubuntu / x86_64 / gateway-verify
Why target evidence is required: the unresolved claim is the actual broker transport boundary from the admitted browser host; hosted synthetic sockets cannot establish that route's stage.
```

The Worker must not launch a page or run the Bilibili selector. A read-only Chrome version check is admission evidence only; a browser process is unnecessary for JT1.

### Runner Security Constraints

```text
Trusted candidate only: yes
Dedicated low-privilege runner user: gateway-verify UID/GID 1001
Vault/profile access: forbidden
Production service mutation: forbidden
Cleanup / timeout requirements: one preflight, 120 seconds maximum, 32 MiB/200-request/1 MiB metadata caps, explicit process/staging cleanup, no residual Chrome/Node probe process
```

## Success Criteria

### Task success

1. A pushed exact Candidate changes only the experimental diagnostic/probe/test/workflow/document surfaces and has a reviewable PR.
2. Required hosted jobs J1–JI1 pass against that exact Candidate, and the artifact manifest/digest is recorded.
3. JT1 produces a sanitized target result with a known transport stage or an explicit `unknown`/`BLOCKED` classification, plus cleanup evidence, without page/media/consumer activity or secret/policy violations.
4. The Issue contains separate Worker execution, Verification Claim and Coordinator decision records. #166 remains `status:blocked` and unmodified.

### Verification claim success

```text
C1 PASS when every permitted deterministic stage and unknown fallback is covered, stage ordering is stable, counters are bounded, and no raw error/URL/address/header/body/credential is returned.
C2 PASS when exact-Candidate hosted J1–JI1 and manifest/leakage/static/security checks pass, with no local build/test/package substituted for Actions evidence.
C3 PASS when JT1 completes once under the admitted user/route, returns a bounded known stage; CONDITIONAL PASS when it completes but remains `unknown`; BLOCKED when the approved artifact/SSH/target admission is unavailable; FAIL on policy bypass, leak, budget breach, residual process/profile/staging, or page/media activity.
C4 PASS when no production boundary, R008 policy, parent Issue state, secret boundary or target security invariant changes.
```

A known target stage does not authorize another live page request or unlock #166. A conditional/unknown result is still useful diagnostic evidence but must not be promoted to an egress capability.

## Evidence Contract

Every report must record:

```text
Task / Claim / Attempt: #182 / C1,C2,C3,C4 / N
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high (+ actual Fast availability)
Execution plane: github-actions or external-codex SSH
Runner / Target: exact hosted runner or VM-0-11-ubuntu / gateway-verify UID 1001
OS / architecture / versions: coarse target admission only
Network path: target → experimental broker → plugin-owned public authority; no proxy
Base / Candidate commit: exact SHA
Workflow / run / job / artifact: exact IDs and digests from GitHub
Commands / selector / budgets: exact entry point; no Bilibili selector for JT1
Duration / repetitions: one target preflight; hosted test counts from logs
Metrics / artifact / evidence: stage/reason/status class/counters/cleanup only
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

Do not retain raw network output or signed URLs. Sanitize Issue comments, artifacts, logs and filenames before publication.

## Failure / Blocked Handling

- `FAIL` means a reproducible code/test regression, leaked sensitive field, policy/SSRF/secret bypass, over-budget operation, stale callback corruption, or cleanup failure.
- `BLOCKED` means the exact artifact, authenticated SSH, admitted target user, required Actions job, or approved route is unavailable, or a security/architecture change is required. Do not turn an unknown stage into FAIL or PASS.
- A completed target run with `unknown` stage is `CONDITIONAL PASS` for diagnostics and leaves #166 blocked.
- On any failure, stop the target session, run the bounded finalizer, post `[BLOCKER REPORT]` or `[EXECUTION REPORT]`, release ownership, and stop. Do not rotate proxies, expand budgets, reuse profiles, add credentials, or retry the Bilibili page.
- If implementation changes the accepted browser/broker/security contract, return the Issue to `status:draft`, revise canonical/task docs, and repeat the Publication Gate. Do not silently widen this Task.

## Deliverables

- Experimental diagnostic/transport code, deterministic tests, and workflow/runbook updates in one focused Candidate PR.
- Exact-Candidate hosted Actions runs/jobs and manifest-addressed artifact digest.
- One sanitized tx-node preflight record, or an explicit BLOCKED report with the missing capability.
- Issue execution report, Coordinator Review and (if all criteria are met) Final Acceptance. No status or code mutation to #166.
- Any separate relay/architecture/security proposal as a new Task only; do not implement it here.

## Issue Feedback / Iteration Protocol

Follow `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, and `docs/tasks/freshness-integration-protocol.md`:

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ Execution Report or Blocker Report
→ status:review or status:blocked
→ Coordinator Review
→ next Attempt or Final Acceptance
```

The Worker cannot set `status:done` or close the Issue. If a Worker session stalls after pushing a Candidate/PR, the Coordinator preserves the durable result and routes the next Attempt on this same Issue.

## Completion Protocol

The Coordinator must reread Issue history, this contract, Candidate/PR, required Actions, artifact and target evidence; comment `[COORDINATOR REVIEW]` with `ACCEPT`, `REVISE`, `BLOCK`, `SPLIT`, or `NOT_PLANNED`; then only after `[FINAL ACCEPTANCE]` set `status:done` and close the Issue. Acceptance of #182 does not alter the Parent Goal or unlock #166 without a separate Coordinator gate.
