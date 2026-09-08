# Task — R008 exact Actions artifact delivery to tx-node staging

## Metadata

```text
GitHub Issue: #185
Parent Goal / Research Item: #68 / R008 / #182 blocker
Task / Research ID: R008-ARTIFACT-DELIVERY
Task kind: combined
Base commit: 39c09481d17eac9a5eb8d718be0adb3d29b04ca4
Candidate commit: n/a until Worker Attempt
Session bootstrap prompt: docs/tasks/185-r008-artifact-delivery/prompt.md
Preferred worker: cloud
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, cloud-interactive, authenticated SSH tx-node
Hard publication dependencies: #182 Attempt 1 blocker; exact Candidate 7de63b231fc4582fc29ceb5a359050f4ffe22fcb and Actions artifact 10060036115
```

> GitHub Actions is the build, test and package authority. This Task does not authorize local compilation, package installation, target runtime execution, or a Bilibili request.

## Session Bootstrap

The Worker starts from `docs/tasks/185-r008-artifact-delivery/prompt.md`. This file is the sole Task contract; the Prompt only navigates to it.

## Goal

Establish one reliable, integrity-verified way to move the exact target-runnable artifact produced by Issue #182 from GitHub Actions to a fresh tx-node staging directory, or produce a bounded `BLOCKED` result with evidence and a reviewed delivery design. The Task must never execute the artifact or contact Bilibili.

A successful delivery result unblocks only the artifact prerequisite for #182 Attempt 2. It is not transport, browser, media, playback, or egress evidence.

## Why / Context

Issue #182 Attempt 1 produced Candidate `7de63b231fc4582fc29ceb5a359050f4ffe22fcb` and a fully passing exact-Candidate Actions run `34236219201`. Artifact `10060036115` is 4,201,531 bytes and its recorded Actions upload digest is `53c57ce5c22219753ff8ad7f3fbea6c112958d88f84daa799bc03aa8e3f02259`. Cloud artifact downloads were slow/TLS-timeout prone and a reconstructed archive failed integrity validation, so JT1 never started. The artifact is currently retained until `2026-09-22T14:09:21Z`; expiry is a blocker, not permission to substitute another Candidate.

## Task Decomposition Decision

```text
Verification mode: separate-task
Linked implementation task: #182 / docs/tasks/182-r008-transport-phase-egress/task.md
Linked verification task: n/a
Decision reason: artifact delivery has an independent failure mode, owner/lifecycle, security boundary and repeatable evidence authority; #182 must remain blocked until this prerequisite is accepted.
```

Do not split by runner or environment. The separate boundary is the artifact transport and integrity claim itself.

## Worker Routing Decision

```text
Worker/client: cloud-codex (gpt-5.6-luna, reasoning high; record Fast availability)
Environment: env:cloud
Implementation authority: GitHub Actions for any workflow/helper changes
Optional target staging check: authenticated SSH to tx-node as gateway-verify, staging only
```

Cloud is the Worker, not a Runner. No phone, TV, VNC, production Gateway or browser page is part of this Task.

## Work Role

### Implementation

If the existing bounded download path is insufficient, the Candidate may add a narrowly scoped workflow/helper/runbook change that makes the artifact delivery method resumable and digest-checked. Any change must preserve the exact Candidate/artifact identity and must not add a proxy, public bucket, long-lived target credential, arbitrary URL authority, or production coupling. A research-only result with no code change is valid when an existing method is proven or the blocker is external.

### Verification

Claims to verify:

```text
C1: a bounded Cloud-side method can retrieve the exact Actions artifact or the failure is reproducibly characterized without exposing signed URLs or secrets
C2: verified bytes match the recorded artifact identity/digest and manifest; corruption or truncation is rejected
C3: if target staging is attempted, gateway-verify receives only the verified archive in a fresh mode-700 directory, then it is removed; no target process or application request runs
C4: no proxy/open relay/credential/profile/phone/TV/production boundary is widened, and #182/#166 state is changed only by the Coordinator
```

## Task vs Job Boundary

```text
Task #185
→ C1–C4
→ cloud Worker
→ Actions checks plus optional SSH staging verification
→ sanitized delivery evidence
→ Coordinator Review / Unblock of #182
```

A Job does not claim an Issue or authorize the #182 target preflight.

## Routing Rationale

- Repository/workflow changes and required build/test/package evidence use Codex Cloud plus GitHub Actions.
- Cloud-side download diagnostics may use GitHub API/CLI with explicit timeouts and bounded output.
- Target SSH is optional and limited to copying/checking/removing an already verified archive; it may not execute Node, Chrome, the probe, or any network request to Bilibili.
- No local compile, test binary, npm install, package installation or browser download is allowed.

## Preconditions

- Read `AGENTS.md`, all required canonical documents, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, the full #182 history and the accepted R008/#176/#169/#172 evidence.
- Exact source identity is fixed: Candidate `7de63b231fc4582fc29ceb5a359050f4ffe22fcb`, workflow `34236219201`, required jobs J1 `102094492167`, J2 `102094492067`, J3a `102094491740`, J3 `102095295856`, J4 `102094492237`, JI1 `102094492032`.
- Exact artifact identity is fixed: artifact `10060036115`, 4,201,531 bytes, Actions upload digest `53c57ce5c22219753ff8ad7f3fbea6c112958d88f84daa799bc03aa8e3f02259`, expiry `2026-09-22T14:09:21Z`.
- If SSH staging is used, recheck only coarse target admission: `VM-0-11-ubuntu`, Ubuntu 26.04/x86_64, `gateway-verify` UID/GID 1001, and a fresh mode-700 directory. Do not reuse any existing runtime profile or staging tree.
- Do not print or persist redirect URLs, signed query strings, headers, cookies, tokens, archive contents or raw network errors.

## In Scope

1. Diagnose bounded GitHub artifact retrieval alternatives (CLI/API, retry/resume or another already authorized GitHub-hosted path) while preserving exact identity.
2. Verify archive size, SHA/digest and manifest/Candidate identity before any target transfer.
3. Optionally transfer the verified archive over the existing authenticated SSH session to a fresh tx-node staging directory, verify again as `gateway-verify`, and remove it with cleanup evidence.
4. If a repository/workflow/helper change is required, keep it limited to artifact packaging/delivery metadata, integrity checks, runbook and security guards; produce a Candidate PR and Actions evidence.
5. Append a sanitized execution report or blocker report that gives #182 a concrete Attempt 2 prerequisite and does not claim transport/playback.

## Out of Scope

- Executing the artifact, Node, Chromium, probe, selector, consumer or any page/media request.
- DNS/TLS/CONNECT to Bilibili, egress/proxy/relay setup, cookies, Authorization, credentials, source-runtime profiles, VNC/CDP, phone/TV/Jellyfin or production Gateway mutation.
- Substituting another Candidate, rebuilding on tx-node, npm install, local compilation/test/package, browser download, public object storage or an unreviewed long-lived SSH key.
- Changing #182 or #166 status except through the Coordinator lifecycle protocol.

## Architecture Invariants

- GitHub Actions remains the build/test/package authority; tx-node is only an optional staging evidence target.
- No Site Plugin, Core, PlaybackSession, DisplayAdapter or Vault boundary is changed.
- Target Runner remains low privilege and separate from Vault/production secrets; no secret or signed URL crosses the evidence boundary.
- Transfer tooling is not an open proxy and accepts no caller-selected egress authority.
- Every failure is fail-closed; partial/corrupt bytes are never executed or forwarded.

## Files Expected to Change

- A narrowly scoped artifact/delivery helper, workflow, manifest metadata or runbook only if the Worker proves a repository change is required.
- No production Rust/Core/Playback/Display/SiteAdapter code.

## Implementation Requirements

1. Use argv/structured APIs, explicit connect/read/write timeouts, bounded retries and digest verification; never shell-concatenate untrusted redirect or archive input.
2. Treat archive length, digest and Candidate/manifest identity as admission conditions. Reject mismatch and clean partial files.
3. Keep all logs/evidence finite and sanitized; do not emit signed URLs, headers, credentials, archive contents or raw errors.
4. Required build/test/package checks run only in GitHub Actions. Target staging performs no compilation, installation or runtime execution.
5. Preserve exact artifact identity. If it expires or cannot be retrieved, report `BLOCKED`; do not silently create a replacement artifact.

## Verification Plan

### Claims

```text
C1: retrieval behavior is bounded and accurately classified
C2: exact bytes/digest/manifest identity is verified and corruption is fail-closed
C3: optional target staging is low-privilege, clean and execution-free
C4: architecture, secret, SSRF, parent-Issue and no-local-build boundaries remain intact
```

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1,C2 | github-actions | github-hosted-x64 | runner-self | yes if helper/workflow changes | exact-Candidate retrieval/manifest/digest fixtures with bounded failure cases | run/log/artifact |
| J2 | C2,C4 | github-actions | github-hosted-x64 | runner-self | yes if helper/workflow changes | static secret/authority/partial-file/leakage checks | run/log |
| J3 | C3 | external-codex SSH | tx-node / gateway-verify | staging only | yes when retrieval succeeds | copy verified archive, verify digest/manifest, remove; no runtime invocation | sanitized command summary + cleanup |
| J4 | C4 | github-actions | github-hosted-x64 | runner-self | yes | workspace/security regression checks for any Candidate | run/log |

If no code change is needed, the Worker may cite the existing exact artifact metadata and bounded Cloud retrieval attempts as research evidence; it must still provide a reproducible `PASS` or `BLOCKED` result and never substitute a different artifact.

### Execution Plane

```text
Cloud retrieval / repository changes: github-actions or external-codex cloud
Optional staging proof: external-codex authenticated SSH to tx-node
```

### Target verification

```text
Target proof required: conditional
Target: VM-0-11-ubuntu / gateway-verify UID 1001
Why: #182 requires the exact archive to reach the target before its single transport preflight, but this Task itself must never execute that preflight.
```

### Runner Security Constraints

```text
Trusted candidate only: yes
Dedicated low-privilege target user: gateway-verify UID/GID 1001
Vault/profile access: forbidden
Production service mutation: forbidden
Cleanup / timeout: explicit transfer timeout, fresh mode-700 staging, remove archive and partials, no target process
```

## Success Criteria

1. The Issue contains a reproducible bounded result for exact artifact `10060036115`: either a verified delivery method or an explicit `BLOCKED` classification with tested failure evidence and a concrete reviewed next method.
2. Any implementation Candidate changes only delivery/manifest/runbook/security surfaces and passes its exact Actions checks; no local build/test/package substitutes Actions evidence.
3. If delivery succeeds, both Cloud and target staging digest/manifest checks match the recorded identity and all temporary target files are removed; no artifact execution or application request occurs.
4. The Issue has separate Worker, Verification and Coordinator records. #182 remains blocked until the Coordinator explicitly unblocks Attempt 2; #166 remains untouched.

## Evidence Contract

Every report records:

```text
Task / Claim / Attempt: #185 / C1,C2,C3,C4 / N
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high (+ actual Fast availability)
Execution plane: github-actions or external-codex SSH
Runner / Target: hosted runner or VM-0-11-ubuntu / gateway-verify UID 1001
Base / Candidate: exact SHA or n/a
Artifact: exact Actions run/id/size/digest; no redirect URL
Commands / budgets: bounded retrieval/staging commands; no runtime invocation
Metrics / result: status class, bytes, digest match, manifest match, cleanup only
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

## Failure / Blocked Handling

- `BLOCKED` covers artifact expiry, GitHub endpoint unavailability, missing authenticated SSH, or any integrity mismatch; it is not permission to retry with another artifact or bypass TLS.
- On corruption, stop and remove partial files. Do not execute, upload, proxy or forward them.
- If the blocker needs a contract/security change, return this Task to `status:draft` and revise canonical/task docs before publishing a new Candidate.
- If delivery succeeds, the Coordinator writes `[COORDINATOR UNBLOCK]` to #182, sets it `status:ready`, verifies the queue and gives the Attempt 2 handoff. The Worker for #185 must not run #182.

## Deliverables

- A focused Candidate PR only when a delivery helper/workflow/document change is needed.
- Sanitized exact-artifact retrieval/integrity evidence, optional target staging evidence, and a concrete handoff or blocker reason for #182.
- Issue execution/blocker report and Coordinator Review. No Bilibili, browser, media, phone, TV or production claim.

## Issue Feedback / Iteration Protocol

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ Execution Report or Blocker Report
→ status:review or status:blocked
→ Coordinator Review
→ ACCEPT / REVISE / BLOCK
```

The Worker cannot set `status:done` or close the Issue. The Coordinator must reread the contract, artifact evidence and any Candidate before deciding.
