# Task — Diagnose tx-node Chromium navigation failure and premature termination

## Metadata

```text
GitHub Issue: #188
Parent Goal / Research Item: #68 / R008; follow-up to #166
Task / Research ID: R008-BROWSER-NAV-TERMINATION
Task kind: combined
Base commit: a8d1707acdc7228382fd4d54dbf9c7b2b2bcddde
Candidate commit: n/a until Worker Attempt
Session bootstrap prompt: docs/tasks/188-r008-browser-navigation-diagnostic/prompt.md
Preferred worker: cloud-codex (gpt-5.6-luna, reasoning high; record Fast availability)
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-test, cloud-interactive, authenticated Tailscale SSH tx-node
Hard publication dependencies: #166 Final Acceptance; #182 Final Acceptance; #185 Final Acceptance; #187 merged contract refresh; current canonical R008/security contracts
Freshness policy: dependency-aware; exact Candidate and artifact identity are mandatory for target execution
```

GitHub Actions is the only build/test/package authority. The tx-node connection is an explicitly bounded external-Codex SSH control plane for target evidence; it is not a build host, runner, proxy, relay, or production environment.

## Session Bootstrap

The Worker starts from `docs/tasks/188-r008-browser-navigation-diagnostic/prompt.md`. That prompt is navigation only; this file is the sole Task Contract.

## Goal

Diagnose the accepted #166 tx-node browser navigation failure after #182 transport admission by making known Chromium navigation/connection failure classes and premature process termination observable through a finite sanitized schema. Ensure every observable termination/finalizer path yields a bounded sanitized result, or an explicit `unknown`/`BLOCKED` classification with cleanup evidence. Prove the change with exact-Candidate hosted Actions checks and at most two fresh tx-node browser sessions using the current artifact/selector.

This Task produces diagnostic evidence only. It does not implement Gateway playback, establish Bilibili compatibility, authorize egress, add a proxy, or change the status/history of #166, #182, or #185.

## Why / Context

#166 Final Acceptance accepted a bounded negative research result: two fresh sessions with `bilibili:BV14V411W7r5:part-2` produced no source candidate; one returned sanitized `chromium_navigation / downstream_close / failure`, while the other terminated before emitting a sanitized result. #182 then added and accepted a finite transport diagnostic, and its one no-page preflight reached `proxy_response / success / 2xx` with no page/media activity. #185 accepted delivery of that exact artifact.

The transport result is admission evidence only. It does not explain page navigation or process termination, and it does not authorize a relay or playback. This follow-up isolates that remaining diagnostic seam before any later #166 attempt.

## Task Decomposition Decision

```text
Verification mode: inline for implementation and hosted deterministic checks; target browser evidence is a separately gated job in this same Task
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: the diagnostic schema, finalizer, artifact, and target observation must share one exact Candidate, while target evidence may independently be CONDITIONAL PASS or BLOCKED and must never be inferred from hosted tests
```

Do not split this Task by runner, environment, or browser code. A separate child Task is warranted only if the diagnostic change requires a new egress/architecture/security authority or target evidence gets an independent lifecycle.

## Worker Routing Decision

```text
Worker/client: cloud-codex (gpt-5.6-luna, reasoning high; record actual Fast availability)
Environment: env:cloud
Implementation/build/test authority: GitHub Actions
Target execution: external-codex over the existing authenticated Tailscale SSH control plane to tx-node
Target identity: gateway-verify UID/GID 1001 on VM-0-11-ubuntu
```

Cloud is the repository Worker/orchestrator and is not a Runner. No phone, TV, VNC, CDP, Jellyfin, or production Gateway capability is needed.

## Work Role

### Implementation

The Candidate must:

1. Extend the experimental browser diagnostic only. Use a finite, versioned, allowlisted navigation classification that can distinguish known Chromium classes, including HTTP/2 protocol/stream failures, connection closed/reset/refused/timeout, proxy connection failure, tunnel failure, TLS protocol/certificate/timeout failure, navigation timeout/abort, downstream close, and `unknown`. Map only fixed error-code markers; never copy an Error message or accept caller-provided URL, host, proxy, headers, body, address, certificate, cookie, token, or credential data.
2. Preserve the existing coarse phase and transport fields where compatible, or make an additive/versioned migration explicit. Output only finite enums/status classes, bounded counters, policy booleans, activity booleans, and cleanup state. Malformed, contradictory, overlong, or unrecognized values must collapse to safe `unknown`/bounded defaults.
3. Make the lifecycle idempotent and finalizer-owned. Normal success, navigation error, timeout, abort, uncaught error/rejection, observable signal termination, and browser/broker close must converge on one sanitized result path. A premature path must emit a bounded result with termination class and cleanup fields, or explicitly classify `unknown`/`BLOCKED` when emission is impossible; it must never leave an unsanitized process error as the only evidence. Late callbacks cannot replace a finalized result.
4. Prove cleanup with bounded evidence: browser/context, broker/server, child process, disposable profile, ephemeral candidate map, DNS pins and staging state must be closed/cleared or explicitly marked `unknown`/`BLOCKED`. Cleanup output must contain no raw paths, URLs, headers, process command lines, or error text.
5. Keep the existing public-host allowlist, DNS/address policy and pinning, origin TLS verification, redirect revalidation, disabled upgrade behavior, secret-header stripping, request/response/metadata/time/cancellation budgets, and server-only candidate handling unchanged.
6. Update only experimental probe/diagnostic tests, manifest/artifact inputs, workflow checks, and a narrowly scoped runbook/research reference if needed. Do not modify production Core/Playback/Display/Control behavior.

### Verification

Claims to verify:

```text
C1: deterministic fixtures map every admitted Chromium navigation/connection class and unknown fallback to the finite sanitized schema without raw leakage.
C2: finalizer and premature-termination paths are bounded, idempotent, stale-callback safe, and leave cleanup evidence or an explicit BLOCKED/unknown classification.
C3: required hosted Actions checks and the manifest-addressed artifact pass against the exact Candidate; no local build/test/package substitutes for them.
C4: at most two fresh tx-node sessions using the exact artifact and selector produce sanitized navigation evidence or a bounded BLOCKED/unknown result, with no click/play/full preload/consumer and cleanup proof.
C5: broker, SSRF, TLS, secret, no-proxy, no-phone, no-TV, no-local-build and parent-Issue status invariants remain intact.
```

## Preconditions

- Read `AGENTS.md`, all required canonical documents, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, and this Task's referenced probe/runbook files.
- Read complete GitHub history and Final Acceptance for #166, #182 and #185, plus the accepted #187 contract-refresh PR. Treat the accepted #166 result as a bounded negative research result and the accepted #182 `proxy_response / success / 2xx` as transport admission only.
- The accepted baseline artifact is #182/#185 artifact `10060036115`, produced by Candidate `7de63b231fc4582fc29ceb5a359050f4ffe22fcb`, Actions run `34236219201`, size `4,201,531` bytes, digest `sha256:53c57ce5c22219753ff8ad7f3fbea6c112958d88f84daa799bc03aa8e3f02259`. It is reference/provenance context only when the code changes; a changed Candidate must receive its own exact Actions artifact and digest before target execution.
- The fixed target selector is the plugin-owned opaque `bilibili:BV14V411W7r5:part-2`. The caller supplies no page URL, host, proxy, profile, headers, or credentials.
- The target is `VM-0-11-ubuntu`, Ubuntu 26.04, x86_64, `gateway-verify` UID/GID 1001, external `/usr/bin/google-chrome-stable` Google Chrome 152.0.7977.82, Node 22.22.1, npm 9.2.0. These facts must be rechecked read-only through the existing Tailscale SSH control route immediately before a target session.
- Before target execution, hosted required checks must be green for the exact Candidate; the artifact must be retrieved by its immutable artifact ID, verified for manifest/Candidate/size/digest/ZIP integrity, and staged in a fresh mode-700 directory. If any identity, route, user, browser, proxy, or cleanup admission differs, report `BLOCKED`.
- Do not run local build/test/package/install, npm install, browser download, proxy setup, route/DNS/firewall change, or target runtime under root/sudo. No existing profile, Gateway process, VNC/CDP session, or credentials may be used.

## In Scope

1. Experimental diagnostic classification/finalizer code, deterministic tests/fixtures, artifact manifest/package inventory, workflow checks, and a narrow diagnostic runbook/reference update when required by the new schema.
2. Finite mapping tests for HTTP/2, connection, proxy/tunnel, TLS, timeout/abort, downstream, and unknown classes; malformed input and raw URL/error/body/header/address/certificate/credential leakage tests.
3. Finalizer tests for success, browser/broker errors, timeout, abort, observable process signal/error termination, duplicate/late callbacks, partial cleanup, and explicit `BLOCKED`/unknown reporting.
4. Exact-Candidate hosted Actions checks and artifact provenance read-back.
5. One or at most two fresh tx-node sessions, each with a new disposable profile, using the exact artifact and selector only after hosted/artifact admission. Permit bounded page navigation observation; do not click play, trigger full preload, request media independently, or run any consumer.

## Out of Scope

- Gateway playback implementation, `PlaybackSession`, `PlaybackItem`, Display Adapter, Jellyfin, Control UI, or production Core changes.
- Bilibili playback/source-portability success, media candidate extraction as a product feature, independent consumer reads, media body capture, click/play/full preload, login, cookies, Authorization, DRM, or private redirects.
- Egress authorization, relay/proxy/CONNECT service, proxy rotation, TLS verification bypass/MITM, arbitrary URL/host/port/header input, SSRF relaxation, DNS/route/firewall changes, or secret boundary changes.
- Phone deployment, ARM64 target proof, physical TV, VNC/CDP, existing profile/Gateway process, root/sudo runtime, or production mutation.
- Local compilation, local test execution that builds dependencies, npm install, browser/package download, or any build/test/package action outside GitHub-hosted Actions.
- Publishing raw error strings, URLs, query strings, IPs, addresses, certificates, headers, bodies, DOM/HAR, command lines, paths, cookies, tokens, or credentials in logs, artifacts, Issue comments, or research docs.
- Changing any existing Issue status, especially #166, #182, or #185, or interpreting this diagnostic result as permission to rerun #166.

## Architecture Invariants

- Gateway remains the `PlaybackSession` authority; this is diagnostic evidence, not playback.
- Site Browser Worker remains generic; Bilibili owns only the opaque selector/authority interpretation.
- The broker remains fail-closed and owns host/scheme/port policy, DNS/address validation, TLS verification, redirects, upgrade denial, budgets, and secret stripping.
- No Site Plugin reads Vault or bypasses `EgressPolicy`; no Cookie, Authorization, signed URL, private address, or credential crosses the diagnostic boundary.
- Target code runs as `gateway-verify` with no production Secret/Vault/root/ADB permission and uses no proxy variables.
- Diagnostic finalization is monotonic: a stale or late callback cannot overwrite a finalized result or cleanup state.
- A diagnostic failure cannot stop or mutate an existing production playback session.
- #166/#182/#185 states and accepted evidence remain unchanged.

## Files Expected to Change

- `experiments/bilibili-browser-probe/diagnostic.mjs`
- `experiments/bilibili-browser-probe/live.mjs`
- `experiments/bilibili-browser-probe/probe.mjs` and/or a narrowly scoped finalizer/entry-point helper
- deterministic diagnostic/finalizer tests and leakage/static-boundary fixtures
- `experiments/bilibili-browser-probe/artifact.mjs`, manifest inputs, package metadata only if necessary
- `.github/workflows/bilibili-browser-probe.yml` only for required exact-Candidate checks/artifact inventory
- `docs/research/bilibili-browser-probe-runbook.md` only for the narrow schema/finalizer/result handling reference, if required

Do not modify production Rust/Core/Playback/Display/Control code.

## Implementation Requirements

1. Use argv/structured APIs and finite timers; never concatenate shell commands or expose process/transport text.
2. Keep the allowlist explicit and bounded. For each known Chromium marker, output a stable enum/class and coarse phase/stage/reason only. Never use substring matching to export arbitrary input; unknown marker collisions must resolve safely.
3. Preserve diagnostic schema compatibility where possible. Any version change must be additive, documented, and covered by migration/contract tests. Every durable output must be JSON-safe and bounded by the existing request/response/metadata budgets.
4. Implement one re-entrant finalizer that owns result publication and cleanup. It must handle normal return, thrown/rejected error, timeout, abort, browser/context close, broker close, and observable signal/error termination without duplicate output. If the OS prevents any output, the report must be `BLOCKED` or `unknown` based on external cleanup read-back; never claim an unobserved PASS.
5. Ensure the finalizer records termination class, diagnostic class, bounded counters/activity booleans, cleanup statuses and a stable result status. It must not record raw error, URL, body, header, address, certificate, process, profile, or candidate details.
6. Keep candidate URLs and response bodies process-local and clear ephemeral maps on every exit. Do not add an independent consumer to this Task.
7. All implementation/build/test/package validation must run on GitHub-hosted Actions against the pushed exact Candidate. The Worker may use local read-only inspection and Git operations only.
8. Target commands must use the exact current artifact/manifest and selector through the existing Tailscale SSH control plane, under `gateway-verify`, with all proxy environment variants explicitly removed. Use a fresh mode-700 profile, one session at a time, a 120-second navigation deadline, 200-request/32-MiB/1-MiB metadata caps, and at most two sessions total. Stop on limit touch or any policy mismatch.

## Task vs Job Boundary

```text
Task #188
→ C1–C5
→ cloud Worker
→ exact-Candidate hosted Actions jobs
→ gated tx-node browser navigation sessions
→ sanitized evidence + cleanup
→ Coordinator Review / Final Acceptance
```

A Job does not claim an Issue or acquire egress authority. The target job is a verification slice of C4 and may return `BLOCKED` without changing implementation acceptance or #166.

## Verification Plan

### Claims

```text
C1: finite Chromium navigation failure classification is complete and leakage-safe.
C2: premature termination/finalizer behavior emits bounded result or explicit BLOCKED/unknown and proves cleanup.
C3: exact-Candidate hosted checks and artifact provenance pass.
C4: one/two fresh tx-node browser sessions yield only sanitized navigation evidence and cleanup.
C5: security/architecture/parent status boundaries remain intact.
```

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate diagnostic schema, Chromium-code mapping, malformed/unknown and leakage tests | run/job logs, sanitized JSON |
| J2 | C2 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate finalizer/signal/timeout/late-callback/cleanup fixtures and static boundary scan | run/job logs, cleanup assertions |
| J3 | C3 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate manifest build/verify, dependency inventory and downloaded artifact consumer/entry check | run/jobs, artifact ID/digest |
| J4 | C3,C5 | github-actions | github-hosted-x64 | runner-self | yes | required probe workflow regression, SSRF/TLS/secret/no-proxy/static checks | run/jobs/logs |
| JI1 | C3,C5 | github-actions | github-hosted-x64 | runner-self | yes | exact-Candidate integration jobs required by the workflow | run/jobs/logs |
| JT1 | C4,C5 | external-codex | authenticated Tailscale SSH to tx-node | VM-0-11-ubuntu / gateway-verify | yes after J1–JI1 and artifact admission | one fresh `probe.mjs --mode live --selector bilibili:BV14V411W7r5:part-2` per session, max two; no click/play/full preload/consumer | sanitized result, target admission, cleanup read-back |

If workflow names/selectors change, record the actual run/job IDs and explain their mapping. A skipped, unrelated, or prior-Candidate job is not required evidence.

### Exact Candidate / Artifact Gate

The target job is forbidden until all of the following are independently read back:

1. Candidate is a full 40-hex SHA pushed from the Worker branch and is the exact source for the hosted run.
2. Required J1–JI1 checks are successful for that SHA.
3. The artifact is identified by immutable artifact ID from that run, downloaded without publishing redirect URLs, and verified for exact manifest Candidate SHA, expected entry point, ZIP integrity, byte count, and recorded digest.
4. Any changed artifact is treated as a new artifact; the accepted #182/#185 artifact is baseline provenance and cannot be reused as evidence for #188's changed Candidate.
5. Target staging is fresh, mode 700, owned/verified by `gateway-verify` UID/GID 1001; no partial bytes are executed.
6. The target admission and proxy-variable checks pass immediately before each fresh session.

Failure of any gate is `BLOCKED`; do not use another Candidate, artifact, proxy, profile, route, or credential.

### Target Session Contract

- Use the existing Tailscale SSH alias only for bounded control-plane admission, artifact staging, invocation and cleanup.
- Use one fresh mode-700 profile per session and no profile reuse. The only admitted selector is `bilibili:BV14V411W7r5:part-2`; do not pass caller URL/host/proxy/profile/header/credential inputs.
- Permit only bounded page navigation observation. Do not click, play, trigger full preload, call the independent consumer, or claim media/source portability.
- Stop after the first useful sanitized result or at the deadline; no indefinite retry or proxy rotation. A second session is allowed only if the first result is missing/unknown and Coordinator scope still permits it.
- After each session run the re-entrant finalizer and read-only cleanup check. Record only coarse target facts, finite diagnostic/result fields, activity booleans, counters, and cleanup statuses.

### Execution Plane

```text
Implementation and build/test/package: github-actions
Target browser observation: external-codex over authenticated Tailscale SSH control plane
```

### Target Proof

```text
Target proof required: yes
Target: tx-node / VM-0-11-ubuntu / x86_64 / gateway-verify UID/GID 1001
Why: the unresolved question is the actual Chromium navigation/termination boundary on the admitted host after the accepted transport admission; hosted fixtures cannot establish that host-specific behavior.
```

### Runner Security Constraints

```text
Trusted candidate only: yes
Dedicated low-privilege target user: gateway-verify UID/GID 1001
Vault/profile/credential access: forbidden
Proxy variables: explicitly unset; no proxy permitted
Production mutation: forbidden
Timeout/repetition: max two fresh sessions, 120 seconds each; bounded request/byte/metadata budgets
Cleanup: browser/context/broker/child process/profile/candidate/pin/staging residue must be removed or explicitly classified BLOCKED/unknown
```

## Success Criteria

### Task success

1. A focused Candidate PR changes only experimental diagnostic/probe/test/workflow/runbook surfaces and has a reviewable exact SHA.
2. Required hosted Actions jobs J1–JI1 pass against that SHA and produce a manifest-addressed artifact with recorded identity/digest.
3. JT1 completes zero, one, or two fresh sessions only after the exact artifact gate. Each observable termination yields a sanitized result or explicit `unknown`/`BLOCKED` classification and cleanup evidence; no prohibited page/media/consumer activity occurs.
4. Issue #188 records Worker execution, Verification Claim results, and Coordinator review separately. Existing #166/#182/#185 statuses and history remain untouched.

### Verification claim success

```text
C1 PASS when every admitted Chromium class and unknown fallback is covered by deterministic tests, ordering/precedence is stable, counters remain bounded, and no raw input can appear in durable output.
C2 PASS when normal/error/timeout/abort/signal/late-callback paths publish at most one bounded result, cleanup is re-entrant, and any impossible emission is explicitly reported as BLOCKED/unknown with external cleanup evidence.
C3 PASS when all required hosted checks and the exact Candidate artifact manifest/digest pass on GitHub Actions; local build/test/package is never substituted.
C4 PASS when one or two fresh tx-node sessions produce bounded navigation evidence or an explicit BLOCKED/unknown outcome under the fixed selector/artifact/no-proxy/no-click boundary, with zero target residue. A target unknown result is CONDITIONAL PASS for diagnosis; missing admission, unobserved termination, policy violation, leak, or cleanup failure is BLOCKED/FAIL as applicable.
C5 PASS when broker/SSRF/TLS/secret/no-proxy/no-phone/no-TV/no-local-build invariants and all existing parent Issue statuses remain unchanged.
```

No C4 result authorizes egress, a proxy, playback, media consumer, or another #166 attempt. A diagnostic `unknown` is evidence of uncertainty, not success.

## Evidence Contract

Every report must record:

```text
Task / Claim / Attempt: #188 / C1–C5 / N
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high (+ actual Fast availability)
Execution plane: github-actions or external-codex SSH
Runner / Target: exact hosted runner or VM-0-11-ubuntu / gateway-verify UID 1001
Base / Candidate commit: exact full SHA
Workflow / run / job / artifact: exact IDs and digest for this Candidate
Target admission: coarse OS/architecture/Node/npm/Chrome/proxy state only
Selector/command shape: fixed opaque selector; no raw URL or secrets
Session count/budgets: actual count and bounded limits
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
Diagnostic: finite phase/stage/class/reason/outcome/status only
Termination: finite normal/error/timeout/abort/signal/unknown only
Cleanup: finite statuses/booleans and residue read-back only
Activity: page navigation/media/click/play/preload/consumer booleans
Problems/limitations: sanitized and bounded
```

Do not retain raw error text, URL, query, endpoint/address, certificate, headers, body, DOM/HAR, process command line, profile path, candidate URL, cookie, token, or credential.

## Failure / Blocked Handling

- `FAIL`: reproducible regression, schema/leakage violation, policy bypass, over-budget action, stale callback corruption, unsanitized termination, or cleanup failure.
- `BLOCKED`: exact Candidate/artifact/manifest/target/SSH admission unavailable, hard process termination prevents an evidence record, required hosted job missing, or an architecture/security change is required. Use external cleanup read-back before claiming residue state.
- `CONDITIONAL PASS`: hosted implementation is sound and a bounded target session yields only `unknown`/insufficient navigation evidence without leakage or policy violation.
- On any target failure, stop, finalize/clean up within bounded limits, post the appropriate report, release ownership, and stop. Do not rotate proxy, expand budgets, reuse profile, add credentials, or start playback/consumer.
- If implementation changes the accepted broker/security authority, return the Issue to `status:draft`, revise the canonical/task contract, and repeat the Publication Gate. Do not silently widen this Task.

## Deliverables

- One focused implementation/diagnostic Candidate PR, if code changes are required.
- Deterministic hosted test evidence and exact Candidate artifact identity/digest.
- At most two sanitized tx-node navigation/termination records plus cleanup read-back, or an explicit BLOCKED report.
- Narrow runbook/research reference update only if needed to describe the accepted diagnostic schema.
- Issue execution report, Coordinator Review and (if all criteria are met) Final Acceptance. No mutation to #166/#182/#185.

## Issue Feedback / Iteration Protocol

Follow `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, and `docs/tasks/freshness-integration-protocol.md`:

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ exact Candidate / hosted evidence / gated target observation
→ [EXECUTION REPORT] → status:review
or [BLOCKER REPORT] → status:blocked
→ Coordinator Review
→ next Attempt or Final Acceptance
```

The Worker must release active ownership and stop after its report. The Worker cannot set `status:done` or close the Issue. The Coordinator must keep the existing parent statuses unchanged.

## Completion Protocol

The Coordinator must reread Issue history, this contract, Candidate/PR, required Actions jobs, artifact, target evidence and cleanup read-back; comment `[COORDINATOR REVIEW]` with `ACCEPT`, `REVISE`, `BLOCK`, `SPLIT`, or `NOT_PLANNED`. Only after `[FINAL ACCEPTANCE]` may the Coordinator set #188 to `status:done` and close it. Publication of #188 does not close #166 or authorize Gateway playback.
