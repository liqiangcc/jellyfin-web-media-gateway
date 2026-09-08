# Task — [R008-ISOLATED-BROWSER-SLOT] Research an isolated browser slot alongside `source-runtime`

## Metadata

```text
GitHub Issue: #193
Parent Goal / Research Item: #188/#191 R008 browser-navigation blocker
Task / Research ID: R008-ISOLATED-BROWSER-SLOT
Task kind: research
Base commit: 1d9c228c257f211c64345fe840719d6436e386d6
Candidate commit: n/a (research result; package candidate is tracked by the Issue/PR)
Session bootstrap prompt: docs/tasks/193-r008-isolated-browser-slot-research/prompt.md
Preferred worker: cloud
Eligible worker environments: env:cloud
Required capabilities: github-read-write, cloud-interactive, lan-access, repository-static-analysis
Hard publication dependencies: none; #188 and #191 remain blocked until an independent result is reviewed
Requested model: gpt-5.6-luna, reasoning high
Fast availability: unavailable in the originating runtime; worker records actual availability
```

> GitHub Actions is not required for this docs-only research package. A future target observation, if available, uses only the authenticated Tailscale SSH control path and read-only admission checks defined below; tx-node is not a build host or a browser execution authority for this Task.
>
> Issue #188, #191, #166, #182 and #185 are external authorities/history. This Task must not change their status, contracts, candidates, artifacts or evidence.

## Session Bootstrap

The bootstrap entry is [prompt.md](prompt.md). It only navigates a future Worker to this contract, the live Issue and the lifecycle/recovery protocols.

## Goal

Determine, with architecture and security evidence plus only permitted read-only target capability/admission evidence, whether tx-node can retain the existing `source-runtime` VNC/Chrome session and safely provide a second browser slot that is fully isolated from it. “Fully isolated” means separate ownership, browser/profile state, process/cgroup lifecycle, display and remote-debugging endpoints, network namespace/egress path, proxy state, ports and cleanup authority; the second slot must not attach to, inspect contents of, mutate or terminate the existing session.

The result must recommend a concrete admissible isolation design or classify the proposal as `CONDITIONAL PASS`, `FAIL` or `BLOCKED`. It must identify any requirement for an architecture/security document change or explicit external owner/admin authorization. A future Worker must not implement the slot, rerun #188, or use the slot for Bilibili/playback evidence.

## Why / Context

Issue #188 Attempt 2 and Issue #191 Attempt 1 both reached the same safe boundary: tx-node already has an owner-controlled `source-runtime` service/session under a separate UID, with persistent Chrome profile, remote debugging and proxy state. The workers correctly left that state untouched, so #188/#191 cannot prove clean-slot readiness by waiting for it to disappear. Accepted #182 transport and #185 artifact-delivery evidence does not authorize reusing that session or establish browser portability.

This is a new research lifecycle because the question is whether coexistence can be designed safely while preserving an external user session. It does not reinterpret the negative #166 result, change the Gateway/Browser Worker contract, or turn a target control connection into a general shell.

## Hypotheses and Isolation Dimensions

Evaluate each candidate as a complete boundary, including failure and cleanup behavior:

1. **Identity and filesystem ownership:** a dedicated non-root UID/GID and independently provisioned runtime/profile root; no shared profile, cookies, sockets, hardlinks/symlinks, or Vault file access. Any profile materialization must be Vault-mediated and scoped, with cleanup ownership limited to the new slot.
2. **Process and resource lifecycle:** a dedicated systemd unit/scope and cgroup/resource budget whose stop, timeout and cleanup operations cannot target `source-runtime`; no broad `pkill`, service wildcard, parent-process kill, or host-wide cleanup. `systemd`/cgroup support is evidence only unless an authorized admin performs provisioning in a later Task.
3. **Display and browser control:** a separate X/Wayland/display or equivalent compositor/session and separately owned VNC/frame/input endpoint; a unique loopback-only CDP/remote-debugging socket or port with an independently scoped short-lived control capability. No connection to the existing VNC/CDP/X display is allowed.
4. **Network and egress:** an independently controlled network namespace/container or equivalent process network boundary where available, with explicit route/DNS/proxy state. Namespace separation does not bypass the Core `EgressPolicy`, public-web restrictions, Secret boundary or site allowlist; inherited host proxy variables, proxy credentials and open-relay behavior must fail closed.
5. **Ports and admission:** deterministic or broker-assigned unique ports/sockets for browser control, display and any worker endpoint; ownership and liveness checks must be scoped to the new slot. No public bind, arbitrary port forwarding, CDP exposure or connection to an existing listener.
6. **Failure, cleanup and observability:** startup, timeout, crash and cleanup evidence is attributable to the new UID/unit/cgroup/profile root only. Diagnostics are finite and sanitized, without command lines, URLs, headers, cookies, profile contents, proxy credentials, screenshots or remote frames.

Compare at least these mechanism families where target capability permits: dedicated UID/profile plus systemd/cgroup; network namespace/container boundary; separate display/VNC/CDP endpoints; and a combined slot contract. Do not treat any single mechanism as sufficient by itself.

## Task Decomposition Decision

```text
Verification mode: separate-task
Linked implementation task: n/a (no implementation is authorized)
Linked verification task: #188/#191 are related external Tasks, not children to modify
Decision reason: live target capability and external owner authorization have independent evidence authority; the research conclusion must remain separate from the blocked browser diagnostic/readiness Tasks
```

## Worker Routing Decision

```text
Worker/client: cloud-codex
Environment: env:cloud
Execution plane: external-codex for any target admission; repository reading may be local/GitHub read-only
Target access: authenticated Tailscale SSH to tx-node, read-only capability/admission evidence only
```

The requested Worker model is `gpt-5.6-luna` with high reasoning. Fast availability is unavailable in the originating runtime and must be recorded by the executing Worker. No local or target build/test/package/install is authorized.

## Work Role

### Implementation

N/A. This is a research-only Task. The package PR contains only `task.md` and bootstrap-only `prompt.md`; a future Worker may add a redacted research result only if the Issue review explicitly accepts that as part of the Task Scope. No runtime, service, profile, network, workflow or canonical document change is authorized.

### Research Claims

- **C1 — Architecture fit:** the candidate isolation design preserves Gateway/Browser Worker/Vault/Display ownership and does not make Core understand target-specific site or browser state.
- **C2 — Security boundary:** the design prevents cross-slot access or control across UID/profile/process-cgroup/display-CDP/network/proxy/port/cleanup boundaries, preserves central `EgressPolicy`, and contains Secret/profile material.
- **C3 — Target capability/admission:** permitted read-only target evidence establishes whether tx-node can provision and admit the required independent primitives without inspecting or changing existing `source-runtime` contents/state. Lack of a provisioned slot or owner/admin authorization remains a condition, not a guessed PASS.
- **C4 — Coexistence and failure safety:** retaining the existing session while starting, timing out, crashing or cleaning the proposed slot has a bounded, owner-scoped behavior and cannot stop, attach to, reconfigure or reuse the existing session.
- **C5 — Decision/change boundary:** the report gives `PASS`, `CONDITIONAL PASS`, `FAIL` or `BLOCKED`, states exact conditions and evidence limits, and flags any required canonical architecture/security change or external authorization without applying it.

## Task vs Job Boundary

```text
Task
→ isolation mechanism comparison and Claims C1–C5
→ cloud-codex / env:cloud
→ static contract review + one bounded target capability/admission read-only Job (if admitted)
→ sanitized research evidence
→ Coordinator Gate decision
```

Jobs do not claim an Issue, provision a slot or own target state.

## Routing Rationale

The isolation conclusion needs live tx-node capability facts and an external source-runtime ownership boundary, which GitHub-hosted runners cannot represent. Cloud is the requested Worker because it can perform GitHub research and, only if authorized by the future Issue state, the narrow Tailscale admission. No target runner or browser session is started for this Task.

## Preconditions

- Parent blockers #188 and #191 are open and `status:blocked`; preserve them and all related #166/#182/#185 history.
- Read the complete histories of #188, #191, #157, #166, #182 and #185 before any target action.
- Read `AGENTS.md`, all canonical documents listed there, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/README.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, and `docs/tasks/freshness-integration-protocol.md`.
- Target access, if used, is the already approved authenticated Tailscale SSH control path; no new key, token, auth key, privilege escalation or production account.
- Obtain no external owner/admin authorization implicitly. Any need to stop/release/reconfigure the existing source-runtime session, create a UID/profile/unit/namespace/port, or change Gateway/security policy is recorded as an explicit future prerequisite and is not executed.
- Start with one bounded static review and, only if the environment admits it, one bounded read-only target capability/admission set. Do not poll, retry-until-clean or consume a browser session.

## In Scope

- Compare the isolation dimensions and mechanism families above against canonical architecture, security and runner rules.
- Identify the minimum independent slot contract: UID/GID, profile/runtime ownership, process/cgroup scope, display/VNC boundary, CDP/socket/port boundary, network namespace/proxy/egress boundary, resource limits, timeout and cleanup authority.
- Define read-only target capability/admission evidence: OS/architecture/kernel and cgroup/user-namespace/network-namespace/container capability facts; availability of dedicated service/scope primitives; and a sanitized owner/admin-provided slot descriptor if one exists. Record only booleans, counts, versions, ownership IDs and bounded status classes.
- Produce a sanitized research result with per-claim classification, mechanism tradeoffs, exact conditions, failure/cleanup model, and any design-change or external-authorization request.
- Record the real Worker/model/Fast availability, execution plane, target, command classes, time bounds and evidence references.

## Out of Scope

- Starting, connecting to, attaching to, observing, inspecting contents of, reconfiguring or terminating Chrome, Chromium, Node, VNC, CDP, Xvfb, Wayland, `source-runtime` or any existing browser/session/process.
- Checking or publishing contents of existing `source-runtime` service units, profile directories, proxy configuration/credentials, command lines, environment values, browser frames, cookies, URLs, headers, DOM, screenshots or logs.
- Stopping, killing, restarting, masking or changing `source-chrome`, `source-mcp-gateway`, `source-xvfb`, their owner/profile/proxy, or any production service; no broad process/service/socket cleanup.
- Creating or modifying a UID/GID, profile, systemd unit/scope, cgroup, network namespace/container, display server, VNC/CDP endpoint, firewall/proxy/route/port mapping or target configuration. Such provisioning requires a separately authorized implementation/deployment Task.
- Downloading, staging, executing or generating any artifact; local/target build, test, package, install, `cargo`, `npm`, Node/Chrome launch, FFmpeg, browser probe or consumer.
- Opening Bilibili or any site; DNS/TLS/CONNECT/media/page/selector/consumer activity; proxy setup/relay; VNC/CDP/SSH tunnel to a browser endpoint; phone, TV, Jellyfin, Gateway or Vault runtime activity.
- Reading profile/Vault contents or secrets, accessing credentials, changing EgressPolicy, exposing an open proxy, weakening SSRF or claiming playback/source portability.
- Modifying #188, #191, #157, #166, #182 or #185 status, contract, candidate, artifact or history.
- Editing canonical architecture/security/requirements docs in this Task. If evidence requires a change, report `design change required` and stop for Coordinator review.

## Architecture Invariants

- Gateway remains the `PlaybackSession` authority; no target browser slot becomes playback authority.
- Site Browser Worker is generic runtime; no site-specific knowledge enters the isolation mechanism or Core.
- Session Vault is the sole owner of profiles/secrets; a slot receives only a scoped capability/materialization.
- Display Adapter and Browser Worker cannot read Vault; profile/cookie material is never exposed through Control, Display, VNC or CDP.
- Site Plugin/Browser Worker cannot bypass `EgressPolicy`, use an open proxy or inherit unrestricted host proxy/Secret state.
- Existing user-owned `source-runtime` state is outside this Task's ownership and remains untouched.
- Target access is least privilege; target runner/control paths do not inherit production Vault, root, SSH, Tailscale or browser authority.
- Public/CDP/VNC ports are not exposed; each control endpoint is loopback/private, uniquely scoped and short-lived.
- Failure and cleanup are owner-scoped and bounded; a slot cleanup cannot affect another UID/cgroup/profile/service.
- Native panel or browser-slot failure cannot stop an already started Display playback.

## Files Expected to Change

- `docs/tasks/193-r008-isolated-browser-slot-research/task.md`
- `docs/tasks/193-r008-isolated-browser-slot-research/prompt.md`

No application, workflow, runtime, profile, service or canonical document file is in scope for the package PR. A future redacted report path may be proposed by Coordinator review; it is not created by this publication package.

## Implementation Requirements

N/A. Do not implement or provision an isolation mechanism in this Task.

## Verification / Research Plan

### Claims

```text
C1: Candidate mechanism(s) fit the current architecture and ownership boundaries.
C2: Candidate mechanism(s) provide complete cross-slot security and failure isolation, including Secret/profile/egress/port boundaries.
C3: Allowed read-only target capability/admission evidence is sufficient, or the exact missing capability/authorization is recorded without touching source-runtime.
C4: Existing source-runtime can remain running without any operation in this Task stopping, attaching to, inspecting, reconfiguring or reusing it; proposed-slot cleanup is bounded.
C5: Final PASS/CONDITIONAL PASS/FAIL/BLOCKED decision and design-change/external-authorization boundary are explicit and evidence-backed.
```

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1,C2,C5 | external-codex / repository read-only | cloud-codex | canonical docs + Task history | yes | Read-only contract/security/runner comparison; no build/test/package | sanitized mechanism matrix, citations to canonical rules, per-claim result |
| J2 | C3,C4 | external-codex | cloud-codex over authenticated Tailscale SSH | tx-node | conditional | One bounded capability/admission set limited to OS/arch/kernel/cgroup/user-namespace/network-namespace/container primitives, generic dedicated-port/socket capability, and an owner/admin-provided slot descriptor; never inspect source-runtime contents/state or connect to browser endpoints | booleans/versions/counts/ownership IDs only; no raw service/process/profile/proxy output |
| J3 | C2,C4,C5 | none / evidence synthesis | cloud-codex | proposed slot contract | yes | Failure matrix for startup, timeout, crash, cleanup and cross-slot denial; no runtime execution | redacted research report and explicit conditions/design-change flags |

J2 is not a readiness check for #191 and cannot return `PASS` merely because generic host primitives exist. It must record `BLOCKED` or `CONDITIONAL PASS` when provisioning, owner authorization or safe slot descriptor evidence is absent. No Job may launch, connect to or terminate any browser/session.

### Allowed Target Admission Evidence

If the future Issue is `status:ready` and the Worker has the required capability, one bounded read-only admission may collect:

- target identity: OS family/version, architecture, kernel and cgroup mode/controllers;
- generic primitive availability: dedicated non-root identity capability as an externally asserted/provisionable fact, user/network namespace or container support, systemd scope/cgroup primitives, and loopback/private socket binding support;
- sanitized resource/port facts for a proposed new slot: only whether a separately assigned port/socket set is available and whether the Worker has an owner-scoped descriptor;
- owner/admin attestations describing the proposed new UID/GID, profile/runtime root, unit/cgroup, display endpoint, CDP/socket endpoint, network namespace/proxy policy and cleanup owner, with secrets and paths redacted.

The Worker must stop if evidence would require inspecting existing `source-runtime` services/processes/profiles/proxy, connecting to any existing endpoint, or changing target state. Generic availability is not proof that a slot has been provisioned.

### Forbidden Target Actions

The target admission must not:

- run `systemctl`, `ps`, `/proc` or socket/process queries against named `source-runtime` units, their UID, profile, proxy, display or browser;
- read command lines, environment blocks, profile directories, service contents, logs, screenshots, frames, cookies or network captures;
- launch or attach to Chrome/Node/VNC/CDP/X/Wayland, send a request, open a page, contact Bilibili, or make DNS/TLS/CONNECT/media traffic;
- create/delete/chown/chmod/mount a resource, start/stop/restart/reconfigure a service, alter a namespace/container/route/proxy/port, or clean any pre-existing residue;
- use root/sudo, a production account, a Vault/profile capability, an artifact or an unbounded retry/poll loop.

### Decision Criteria

- **PASS:** a concrete combined slot contract is supported by canonical mapping and sufficient bounded target capability/owner-descriptor evidence; every cross-slot, Secret, EgressPolicy, endpoint, resource and cleanup condition is explicit; no canonical change or ungranted admin action is required for the proposed design.
- **CONDITIONAL PASS:** the design satisfies the invariants in principle and the missing item is a bounded, explicit provisioning/authorization or target capability condition; the report names who/what must supply it and does not claim a ready slot or playback.
- **FAIL:** the mechanism inherently shares an existing profile/process/display/CDP/proxy/port/cleanup authority, requires weakening SSRF/Secret boundaries, exposes a public/open endpoint, or cannot prevent cleanup/control across slots.
- **BLOCKED:** required read-only target capability evidence, owner/admin descriptor, SSH capability or safe evidence authority is unavailable, or obtaining it would require a forbidden action or canonical design change that has not been reviewed.

These labels apply to research claims and the Task result separately from Coordinator `ACCEPT / REVISE / BLOCK / SPLIT / NOT_PLANNED`. Do not call generic theoretical support `PASS`.

## Evidence Contract

```text
Role: research
Task / Claim: #193 / R008-ISOLATED-BROWSER-SLOT / C1–C5
Attempt: assigned by Issue lifecycle after publication
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high; Fast availability recorded as unavailable if unchanged
Execution plane: external-codex for target admission; repository/static review otherwise
Runner class: cloud-codex execution host; no self-hosted Runner claim
Target host/device: tx-node, only if J2 is admitted
Network path: authenticated Tailscale SSH control path only for bounded admission; no site/media path
Base / Candidate commit: base 1d9c228c257f211c64345fe840719d6436e386d6 / Candidate n/a unless a research-report commit is explicitly accepted later
Workflow / run / job: n/a unless a future docs-only research check is explicitly added
Evidence: canonical-doc mapping, mechanism matrix, sanitized target capability/descriptor facts, failure/cleanup analysis
Secret policy: no Cookie, Authorization, token, key, credential, profile content, raw URL, raw header, proxy credential, command line or screenshot
Parent boundary: #188/#191 remain blocked and untouched; #157/#166/#182/#185 remain unchanged
```

## Freshness / Integration Contract

```text
Freshness policy: dependency-aware
Semantic authorities: docs/architecture.md, docs/implementation-contracts.md, docs/security.md, docs/development-environments.md, docs/runner-execution-architecture.md, #188/#191 blocker history
Semantic freshness domains: target primitive availability, owner-provided slot descriptor, source-runtime coexistence boundary, security/architecture invariants
Integration surfaces: package files only; no code, workflow, dependency or runtime surface
Task-owned surfaces: mechanism comparison, admission boundary and research classification
Authority/domain → Claim mapping: canonical mapping → C1/C2; target capability/descriptor → C3/C4; synthesis/change boundary → C5
Integration verification jobs: none; package-only docs have no runtime integration job
Unrelated-main policy: unrelated documentation changes do not invalidate the contract; any architecture/security/routing/target-boundary change requires Coordinator review and republishing
Strict-main reason: n/a
```

Target evidence is time-sensitive and cannot be reused as a future ready-slot or #188 browser proof without Coordinator review. A future implementation/provisioning Task must carry its own exact candidate, authorization and cleanup evidence.

## Success Criteria

### Task success

1. The report compares complete isolation designs across identity/profile, process/cgroup, display/VNC/CDP, network/proxy/egress, ports, resources, failure and cleanup.
2. Each C1–C5 has an evidence-backed `PASS`, `CONDITIONAL PASS`, `FAIL` or `BLOCKED` classification; unsupported theoretical claims are not `PASS`.
3. The report preserves the existing `source-runtime` session and explicitly lists forbidden inspection/mutation boundaries.
4. The report states whether a separate implementation/design-change Task and explicit source-runtime owner/admin authorization are required.
5. The result says exactly what it does and does not unblock: it may inform #188/#191 review, but it does not rerun, unlock or modify them and does not prove Bilibili playback/source portability.

### Coordinator Gate

Coordinator may accept the research only after reading the live Issue/history, this Task Contract, the sanitized report and all required evidence. `ACCEPT` does not automatically set #188/#191 ready; any downstream slot provisioning or browser diagnostic requires a separately scoped Task and publication gate.

