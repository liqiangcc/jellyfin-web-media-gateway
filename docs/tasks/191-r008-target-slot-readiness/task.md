# Task — [R008-TX-NODE-CLEAN-SLOT] Prove clean browser slot readiness for #188

## Metadata

```text
GitHub Issue: #191
Parent Goal / Research Item: #188 R008 browser-navigation diagnostic (unblock prerequisite only)
Task / Research ID: R008-TX-NODE-CLEAN-SLOT
Task kind: verification
Base commit: e414f049c3119287f39d1d42726d15646b8bac17
Candidate commit: n/a (target-state evidence; #188 owns its accepted Candidate)
Session bootstrap prompt: docs/tasks/191-r008-target-slot-readiness/prompt.md
Preferred worker: cloud
Eligible worker environments: env:cloud
Required capabilities: github-read-write, cloud-interactive, lan-access
Hard publication dependencies: none; #188 remains blocked until this independent readiness result is reviewed
Requested model: gpt-5.6-luna, reasoning high
Fast availability: unavailable in the originating runtime; worker records actual availability
```

> GitHub Actions / Runner is not required for this target-state observation. The execution plane is external-codex over the authenticated Tailscale SSH control path, and the target is tx-node `VM-0-11-ubuntu`. This task does not turn that control path into a general target shell or Runner authority.
>
> Live status, owner, Attempt, result and review history belong to Issue #191. #188, #166, #182 and #185 keep their existing statuses and histories.

## Session Bootstrap

The bootstrap entry is [prompt.md](prompt.md). It only navigates a future worker to this contract, the live Issue and the lifecycle/recovery protocols.

## Goal

Using only bounded, authenticated Tailscale SSH read-only admission, prove whether tx-node has a clean low-privilege browser slot for a future #188 navigation-only Attempt. A clean slot means the external source-runtime owner has already released `source-chrome`, `source-mcp-gateway`, `source-xvfb`, the persistent browser profile, remote-debugging state and proxy state, with no pre-existing Chrome/profile/remote-debugging/proxy/production Gateway residue visible to the admission checks.

This Task returns `PASS` only when the complete clean-slot contract is observed. If any pre-existing source-runtime/session state remains, it returns `BLOCKED` and records the minimum owner action. The worker must never stop, attach to, reconfigure, inspect contents of, or reuse that state.

## Why / Context

Issue #188 Attempt 2 reached its target admission boundary and found an already-running source-runtime session under a separate owner, including `source-chrome.service`, `source-mcp-gateway.service`, `source-xvfb.service`, a persistent profile, remote debugging and an upstream proxy. #188 correctly left that session untouched and remains blocked. #182 transport evidence and #185 artifact delivery evidence are accepted but do not prove a clean browser slot.

This independent Task has a separate target-state Evidence Authority and lifecycle. It only establishes whether the owner has made a clean slot available; it does not rerun #188 navigation or decide the R008/browser result.

## Task Decomposition Decision

```text
Verification mode: separate-task
Linked implementation task: #188 / docs/tasks/188-r008-browser-navigation-diagnostic/task.md
Linked verification task: n/a
Decision reason: target admission is an independent external state, owner and BLOCKED/PASS result; #188 implementation/artifact evidence remains reusable and must not be reopened by this readiness check
```

## Worker Routing Decision

```text
Worker/client: cloud-codex
Environment: env:cloud
Execution plane: external-codex
Target access: authenticated Tailscale SSH to tx-node, read-only admission only
```

The requested Worker model is `gpt-5.6-luna` with high reasoning. Fast availability is unavailable in the originating runtime and must be recorded by the executing Worker. The target is not a build/test host and no local or target build/test/package/install is authorized.

## Work Role

### Implementation

N/A. This is a verification-only Task. It produces no application, workflow, runtime, artifact or target configuration change.

### Verification

Claims to verify:

- C1: Read-only target admission identifies the target OS/architecture, Chrome and Node versions, and the dedicated `gateway-verify` identity without exposing credentials or sensitive command output.
- C2: A bounded read-only check proves whether zero pre-existing Chrome/source-runtime processes, persistent profiles, remote-debugging markers, proxy state and production Gateway activity remain in the candidate slot. The check must include `source-chrome`, `source-mcp-gateway` and `source-xvfb` service state.
- C3: Read-only cleanup/residue checks show no worker-created temporary state and provide bounded evidence that the slot is clean; if any pre-existing residue remains, the result is BLOCKED and the residue is left untouched.
- C4: The admission preserves all task boundaries: no artifact download or execution, no Node/Chrome launch, no page/navigation/media/selector/consumer request, no Bilibili access, no VNC/CDP/phone/TV/production action, no proxy setup, no profile/credential access, no service stop/restart/reconfiguration, and no #188/#166/#182/#185 mutation.

## Task vs Job Boundary

```text
Task
→ clean-slot readiness Claims C1–C4
→ cloud-codex / env:cloud
→ one bounded read-only SSH admission Job
→ tx-node VM-0-11-ubuntu
→ sanitized Issue Evidence
```

The single Job is one admission set, not a retry loop. A Job does not claim an Issue or own target state.

## Routing Rationale

The Claims depend on tx-node's live source-runtime ownership and process/profile/service state. GitHub-hosted x64/ARM64 cannot substitute for that target. Tailscale SSH is only the explicitly scoped management path; it is not a media or browser execution path.

## Preconditions

- Parent Issue: #188 is open and `status:blocked`; do not change it.
- Accepted parent implementation/artifact evidence: read references from #188 only; do not download, stage or execute the artifact.
- Target: tx-node `VM-0-11-ubuntu`.
- Access: an already authorized Tailscale SSH route; no new key, token, auth key or privilege escalation.
- Identity: read target facts as permitted; no root/sudo fallback and no production service account use.
- Owner action before this Task can PASS: source-runtime owner independently releases `source-chrome`, `source-mcp-gateway`, `source-xvfb`, persistent profile, remote-debugging state and proxy state.
- Start with one bounded admission set. Do not poll, retry-until-clean, or consume a second session.

## In Scope

- One bounded read-only SSH admission set against the fixed tx-node target.
- OS, architecture, Node, Chrome and `gateway-verify` uid/gid/runtime identity read-back.
- Sanitized checks for the source-runtime services, existing Chrome/browser processes, persistent profile presence, remote-debugging markers, proxy environment/process/service state, and production Gateway activity, using only fixed allowlisted checks.
- Read-only residue/cleanup evidence for the admission attempt and confirmation that no worker-created state exists.
- A final sanitized `PASS` or `BLOCKED` classification and the exact safe condition required for a later #188 Attempt.

## Out of Scope

- Running or downloading any Actions artifact, Node script, probe, package, build, test, install or executable.
- Starting, connecting to, attaching to, observing or terminating Chrome, Chromium, Node runtime, source MCP, Xvfb or any browser session.
- Opening Bilibili or any website; DNS, TLS, CONNECT, proxy, media, page, selector or consumer activity.
- Reading profile contents, Cookies, Authorization, environment secrets, credentials, URLs, headers, page source, DOM, screenshots, VNC or CDP.
- Stopping, killing, restarting, reconfiguring or cleaning any existing source-runtime/session/process/profile/service.
- Starting or mutating Gateway, Vault, production services, runner registration, Tailscale, SSH, phone, TV or Jellyfin.
- Modifying #188, #166, #182 or #185 status, contract, candidate, artifact or history.
- Repeating #188 navigation, selecting a selector, making a media request, or claiming source portability/playback.

## Architecture Invariants

- Existing user/source-runtime state is not worker-owned; a pre-existing session must be left untouched.
- Target Runner/control access is least privilege and separate from Gateway Vault, production runtime and browser profiles.
- Browser Worker is not started by this Task and no Site Plugin or Core state is changed.
- No Secret, profile content, proxy credential, URL, Cookie, Authorization or raw process/service output is retained in Issue Evidence.
- `BLOCKED` is the correct result when clean admission is absent; the worker must not lower criteria or force cleanup to obtain `PASS`.
- This Task's result is separate from #188's browser navigation result and from the Parent Goal/R008 Coordinator Gate.

## Files Expected to Change

- `docs/tasks/191-r008-target-slot-readiness/task.md`
- `docs/tasks/191-r008-target-slot-readiness/prompt.md`

No other repository file is in scope for the package PR.

## Implementation Requirements

N/A.

## Verification Plan

### Claims

```text
C1: target identity and required runtime/version admission is read-only and sanitized.
C2: clean-slot absence/presence of source-runtime, browser, profile, remote-debugging, proxy and production Gateway state is determined without mutation.
C3: bounded cleanup/residue evidence is complete; pre-existing residue is never removed by the worker.
C4: all no-artifact/no-browser/no-site/no-production/no-parent-mutation boundaries are preserved.
```

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1,C2,C3,C4 | external-codex | cloud-codex over authenticated Tailscale SSH | tx-node `VM-0-11-ubuntu` | yes | One bounded fixed allowlist admission: OS/arch, `id gateway-verify`, Node/Chrome versions; service-active and process/profile/remote-debugging/proxy/Gateway residue checks; sanitized cleanup read-back. No launch, attach, network request, staging or mutation. | Issue report with sanitized booleans/counts, target/version facts, command classes, cleanup evidence and `PASS`/`BLOCKED` |

### Execution Plane

```text
Execution plane: external-codex
Execution host: cloud-codex
Target: tx-node VM-0-11-ubuntu
Access path: authenticated Tailscale SSH, read-only admission
```

This target check is not GitHub Actions Evidence and must not be reported as a hosted or phone proof. No build/test/package/install command is allowed on any host.

### Runner Selection

```text
Target-specific source-runtime/process/profile/service state
→ external-codex over authenticated Tailscale SSH
→ tx-node VM-0-11-ubuntu
```

### Long-running / repeated verification

No long-running or repeated verification is allowed. Run one bounded admission set and stop. Do not poll, wait for the owner, or run a second check set in the same Attempt.

### Interactive debugging

```text
WSL / Windows / Ubuntu ARM64 external Codex required: no
Reason: only the authorized cloud Tailscale SSH read-only admission is in scope
```

### Target verification

```text
Target proof required: yes
Target: tx-node VM-0-11-ubuntu
Why target evidence is required: clean-slot readiness is a live property of the external source-runtime/session owner and target process/profile/service state
```

### Runner Security Constraints

```text
Trusted candidate only: yes (read-only Task contract; no artifact or code execution)
Dedicated low-privilege runner user: gateway-verify facts must be read back; no privilege escalation
Vault/profile access: forbidden
Production service mutation: forbidden
Cleanup / timeout requirements: one bounded SSH admission; sanitized output only; leave all pre-existing state untouched; stop on any unsafe ambiguity
```

### Freshness / Integration Contract

```text
Freshness policy: dependency-aware
Semantic authorities: #188 target admission boundary; docs/security.md; docs/runner-execution-architecture.md; source-runtime ownership boundary recorded in #188 history
Semantic freshness domains: target identity/version, source-runtime service/process/profile/remote-debugging/proxy state, production Gateway absence, read-only cleanup boundary
Integration surfaces: package files only; no code, workflow, dependency or runtime surface
Task-owned surfaces: this task's admission classification and sanitized target-state evidence
Authority/domain → Claim mapping: target admission → C1; clean slot → C2; residue/cleanup → C3; safety boundary → C4
Integration verification jobs: none; package-only documentation has no runtime integration job
Unrelated-main policy: exact target-state evidence remains valid only for the observed target/time; unrelated documentation changes do not require a new admission, while any authority/routing/target-contract change requires Coordinator review
Strict-main reason: n/a
```

The target state is inherently time-sensitive. A `PASS` is evidence for the bounded observation only; Coordinator decides whether it is sufficiently fresh for a future #188 Attempt. A new #188 Candidate or artifact does not automatically invalidate this independent slot observation, but a change to the admission/security boundary does.

## Success Criteria

Success criteria are frozen before any future Attempt.

### Task success

1. One exact target admission set produces a complete sanitized report with C1–C4 individually classified.
2. `PASS` is issued only if the source-runtime owner has already released the named services/session/profile/remote-debugging/proxy and all required zero-residue checks pass; otherwise the result is `BLOCKED` with no target mutation.
3. The report contains no artifact bytes, Secret, credential, profile content, raw URL, raw header, raw process command line, raw service output, page/media evidence or unsupported claim.
4. The result explicitly says it only supplies clean target-slot readiness to #188 and does not rerun, unlock or change #188, #166, #182 or #185.

### Verification claim success

```text
C1 PASS when target OS/architecture, Chrome, Node and gateway-verify identity are read-only and sanitized; otherwise BLOCKED.
C2 PASS when all required source-runtime/browser/profile/remote-debugging/proxy/production-Gateway absence checks are clean; any pre-existing item means BLOCKED and must be left untouched.
C3 PASS when bounded cleanup/residue checks show no worker-created state and no unreviewed residue; inability to distinguish worker residue from pre-existing state is BLOCKED.
C4 PASS when the entire admission obeys the no-artifact/no-browser/no-site/no-production/no-parent-mutation boundary; any violation is FAIL and must be reported immediately.
```

## Evidence Contract

```text
Role: verification
Task / Claim: #191 / C1–C4
Attempt: assigned by Issue lifecycle after publication
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high; Fast availability recorded as unavailable if unchanged
Job ID: J1
Execution plane: external-codex
Runner class / image / labels: cloud-codex execution host; no self-hosted Runner claim
Execution host: cloud-codex
Target host/device: tx-node VM-0-11-ubuntu
OS / architecture: target facts read back by J1
Relevant versions: Node, Chrome and gateway-verify identity from J1
Network path: authenticated Tailscale SSH control path only; no site/media path
Base / Candidate commit: base e414f049c3119287f39d1d42726d15646b8bac17 / Candidate n/a; parent #188 Candidate may be referenced from live Issue only
Workflow / run / job: n/a
Commands / steps: fixed read-only admission allowlist, sanitized classification and cleanup read-back
Duration / repetitions / shards: one bounded set / no retry / no shard
Metrics / artifact / raw evidence location: sanitized Issue evidence only; no artifact and no raw output
Result: PASS | BLOCKED | FAIL
```

No evidence may include Cookie, Authorization, token, credential, profile data, proxy URL/credential, complete command line, complete service output, complete URL, page/DOM/body/header, screenshot, VNC/CDP data or artifact bytes.

## Failure / Blocked Handling

- `FAIL` means the worker violated the read-only/safety contract, launched or connected to a prohibited runtime, performed prohibited network or production activity, leaked sensitive output, or otherwise produced unsafe side effects. Stop, preserve only sanitized evidence, and report immediately.
- `BLOCKED` means the target is unreachable, required read-only admission is unavailable, any named source-runtime/profile/remote-debugging/proxy/production residue remains, or state cannot be distinguished safely. Do not stop or reconfigure the existing session; release ownership and report the minimum owner action.
- `PASS` means only clean target-slot readiness for a bounded future #188 Attempt. It does not authorize that Attempt, artifact staging, browser launch, navigation, Bilibili, transport, source portability or playback.
- If the Tailscale/SSH route is unavailable, do not substitute root, another host, VNC, CDP, phone, TV, local shell or an unreviewed route.
- A future #188 Attempt must independently re-read this result and its own exact Candidate/artifact gate. This Task does not change #188 status.

## Deliverables

- Implementation / docs: this two-file Task package only.
- Candidate commit / PR: package commit and PR, supplied in Issue/PR read-back by the Coordinator.
- Session bootstrap prompt: `docs/tasks/191-r008-target-slot-readiness/prompt.md`.
- Linked verification task: n/a.
- Verification Jobs / runs: J1 external-codex target admission, future only.
- Target Evidence: future sanitized tx-node admission report on Issue #191.
- Research evidence doc: n/a; readiness is retained in Issue history unless Coordinator later requests a research record.

## Issue Feedback / Iteration Protocol

Follow `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md` and `docs/tasks/freshness-integration-protocol.md`. This package is intentionally left in `status:draft` until the Coordinator completes the full Publication Gate. A worker must not claim or execute it while draft.

## Completion Protocol

Any future Worker must post `[EXECUTION REPORT]` or `[BLOCKER REPORT]` before changing state to `status:review` or `status:blocked`, release ownership and stop. Only the Coordinator may review, accept, revise, unblock, set `status:done` or close the Issue. This Task's acceptance cannot itself alter #188, #166, #182 or #185.
