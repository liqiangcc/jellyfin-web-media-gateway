# Task — [R008-TX-NODE-ISOLATED-SLOT-PROVISION] Provision a combined isolated browser slot on tx-node

## Metadata

```text
GitHub Issue: #195
Parent Goal / Research Item: R008 isolated browser slot (#193)
Related blockers: #188 browser navigation diagnostic, #191 target-slot readiness
Task / Research ID: R008-TX-NODE-ISOLATED-SLOT-PROVISION
Task kind: implementation (target provisioning)
Planning / package base commit: 462ca6c9cc934d191ed72ebc359381397d9ee26f
Candidate commit: n/a until an authorized implementation change exists
Session bootstrap prompt: docs/tasks/195-r008-tx-node-isolated-slot-provisioning/prompt.md
Preferred Worker: cloud-codex
Eligible environment: env:cloud
Required capabilities: github-read-write, code-authoring, cloud-interactive, interactive-linux-debug, lan-access, remote-control, repository-static-analysis
Requested model: gpt-5.6-luna, reasoning high
Fast availability: unavailable in the originating runtime; future Worker records actual availability
```

> This is a future provisioning Task. It is not authorized while Issue #195 is `status:draft`, and it cannot start until the Coordinator has independently read back written owner/admin authorization, a low-privilege target access route, and a sanitized slot descriptor. The existing `source-runtime` session remains outside this Task's ownership.
>
> #191 remains the separate target readiness evidence authority. Provisioning a slot does not claim that it is ready, does not rerun #188, and does not authorize Bilibili, playback or source portability.

## Goal

Provision a second browser slot on tx-node that can coexist with the owner-controlled `source-runtime` session while preserving the architecture and security boundaries established by #193. The slot must have independent ownership, profile/runtime storage, process and resource lifecycle, display and browser-control endpoints, network/egress policy, ports, and cleanup authority. It must be possible to prove that stopping, timing out, crashing or cleaning the new slot cannot stop, attach to, inspect, reconfigure or reuse the existing session.

The final slot is a generic Site Browser Worker runtime. Core remains the `PlaybackSession` authority and does not learn tx-node-specific service, display, namespace, proxy or port details.

## Preconditions and authorization gate

A future Worker must stop and post a `[BLOCKER REPORT]` if any precondition is absent or ambiguous:

1. The Issue is explicitly republished as `status:ready` with `env:cloud`, has no active owner, and the Worker has claimed a new Attempt.
2. The source-runtime owner/admin provides written authorization for coexistence, naming the allowed target, maintenance window, ownership boundary and rollback contact. Authorization must not grant access to existing profile contents, cookies, credentials, VNC frames, CDP, proxy credentials or Gateway Vault.
3. The Coordinator records a sanitized slot descriptor before provisioning. It must identify the new non-root UID/GID, profile/runtime root, systemd unit or scope, cgroup/resource budget, display session, private CDP/socket endpoints, network namespace or equivalent, port allocation, EgressPolicy binding, timeout policy and cleanup owner. Secrets, raw URLs, paths that expose profile layout, and credentials are excluded from the descriptor.
4. The target access route is explicitly approved. Root Tailscale SSH, if required for provisioning, is a restricted control-plane mechanism only; it is never runtime identity, Vault authority or evidence of final ownership. Runtime and checks must run under the dedicated low-privilege slot identity.
5. A current #191 readiness result is available or the Coordinator explicitly classifies its absence. #191 remains independent and must not be edited by this Task. A provisioned slot is not automatically a clean slot.
6. Any required change to `architecture.md`, `security.md`, `runner-execution-architecture.md`, deployment policy or a public API is a design blocker. Preserve Evidence and return to Coordinator; do not silently apply the change in provisioning commands.

No authorization is inferred from a prior SSH connection, an active source-runtime session, #157 evidence, #188/#191 blocker history, or the existence of host tools.

## Scope

- Provision only the combined isolated slot described by #193 after the authorization gate passes.
- Create or use a dedicated non-root UID/GID and a fresh, independently owned profile/runtime root. Do not copy, mount, hardlink, symlink or inspect the existing source-runtime profile or Vault.
- Create an owner-scoped systemd unit/scope and cgroup with explicit CPU, memory, process-count, file-descriptor and wall-time limits. Start/stop/timeout/cleanup operations must address the exact new unit/cgroup only.
- Provide a separate display/session boundary and private, uniquely assigned browser-control endpoint. No public bind, existing VNC/CDP/X/Wayland display, remote-debugging endpoint or input channel may be reused.
- Provide an independent network namespace/container or equivalent process boundary with explicit route/DNS and EgressPolicy binding. Do not inherit proxy variables, proxy credentials or an open relay. Namespace isolation does not weaken SSRF, Secret, TLS or site allowlist rules.
- Allocate deterministic or broker-assigned private ports/sockets and prove ownership/collision checks for the new slot. No arbitrary forwarding or public exposure.
- Define bounded startup, timeout, crash and cleanup behavior. Cleanup must remove only the new slot's profile/runtime, unit/cgroup, display endpoint, sockets and temporary state, and must be safe to repeat.
- Add only the repository configuration, documentation, helper or test changes explicitly authorized by the final contract. Keep all site-specific semantics in the Site Plugin and all playback authority in Gateway.
- Produce sanitized provisioning and ownership evidence for #191 review. Do not treat provisioning as #191 PASS.

## Out of scope and prohibited actions

- Any operation before the written authorization, slot descriptor and ready/owner gate are complete.
- Inspecting contents or state of existing `source-runtime`, `source-chrome`, `source-mcp-gateway`, `source-xvfb`, its UID, profile, proxy, display, CDP endpoint, ports, logs, environment or files.
- Stopping, killing, restarting, attaching to, observing, reconfiguring, masking or cleaning any existing source-runtime or production process/service/profile/display/proxy.
- Broad `pkill`, wildcard service operations, host-wide cleanup, deleting unknown temporary files, or changing unrelated routes/firewalls/ports/namespaces.
- Using the existing VNC, CDP, Chrome profile, display, proxy, port, credentials, cookies, Vault, Gateway or production service.
- Launching Chrome, Node, VNC, CDP, X/Wayland, the browser probe, a consumer, FFmpeg or a site session as part of provisioning proof. No Bilibili/site/DNS/TLS/CONNECT/media/page/playback request.
- Downloading or executing an unapproved artifact, installing packages, compiling/building/testing on the local workspace, tx-node, phone or any other constrained host. Repository build/test/package verification must run on GitHub-hosted Actions; target provisioning must use already approved host primitives and finite commands.
- Adding a proxy relay, weakening EgressPolicy/SSRF/Secret/TLS checks, exposing public CDP/VNC, or passing profile/credential material through Control, Display or the worker.
- Phone, physical TV, Jellyfin, VNC observation, CDP observation, production Gateway mutation, Vault access, or changing #188/#191/#193/#157/#166/#182/#185 status, contract, candidate, artifact or history.

## Architecture and security invariants

- Gateway remains the sole `PlaybackSession` authority; a target slot is an execution resource, not a playback state store.
- Site Browser Worker remains generic. Core and the slot manager do not understand Bilibili DOM, URLs, cookies, private APIs or source semantics.
- Session Vault is the sole owner of persistent site sessions and profiles. The slot receives only an explicitly scoped capability or empty anonymous profile materialization.
- Display Adapter never reads Vault. Control and Display cannot download profiles, cookies, remote frames or credentials.
- Site Plugin and Browser Worker cannot bypass central `EgressPolicy`, public-web restrictions, SSRF controls, Secret classifier, TLS verification or site allowlist.
- The slot's UID, profile root, unit/cgroup, display, CDP/socket, network namespace, ports, temporary files and cleanup owner are disjoint from source-runtime and production Gateway state.
- All external control endpoints are loopback/private, uniquely allocated and short-lived. No public listener, arbitrary port forwarding or unauthenticated CDP/VNC is allowed.
- Resource, timeout, crash and cleanup operations are scoped to the slot identity and exact unit/cgroup. Failure cannot stop an already-running source-runtime or playback/display session.
- Target control access is not target runtime authority. The runtime has no root, Vault, long-lived site credential, SSH key or Tailscale auth key.

## Implementation requirements

1. Freeze the slot descriptor and authorization in the Issue before any target mutation. The Worker must record the descriptor version/hash without publishing secrets or raw profile/network details.
2. Use structured APIs and argv. Do not construct shell commands from caller-provided URLs, hosts, ports, paths, headers or credentials. Validate every fixed descriptor field against allowlists and bounded ranges.
3. Make provisioning idempotent for the new slot only. Re-running must either report the exact owned state or fail closed on collision; it must never adopt an existing resource by name or broad matching.
4. Verify UID/GID, ownership, permissions, cgroup limits, namespace identity, display/control endpoint ownership, private binding, EgressPolicy binding, proxy-variable absence and cleanup scope using sanitized fields only.
5. Keep profile/runtime roots mode-restricted and ephemeral unless the authorized descriptor explicitly requires a persistent, separately owned root. Never log contents, cookies, environment blocks, raw command lines, signed URLs or credentials.
6. Define a finite rollback that removes only resources created by this Attempt. If ownership is ambiguous, leave the resource untouched and report `BLOCKED`; do not broaden cleanup.
7. Any repository implementation/configuration change receives a Candidate SHA and required GitHub-hosted Actions evidence. Never substitute local `cargo`, npm, frontend build, package install or target compilation.
8. Do not claim browser navigation, target clean readiness, transport, source portability or playback. Those belong to #191/#188 and later independently published Tasks.

## Verification plan

### Claims

- **C1 — Authorization and ownership:** written owner/admin authorization and a sanitized slot descriptor are present; final runtime ownership is a dedicated low-privilege identity separate from source-runtime and production.
- **C2 — Complete isolation:** identity/profile, process/cgroup, display/VNC/CDP, network/egress, ports, resource and cleanup boundaries are all independent and fail closed.
- **C3 — Safe provisioning:** provisioning is idempotent, bounded and owner-scoped; collision, ambiguous ownership, capability or canonical-change gaps block without broad cleanup.
- **C4 — Repository evidence:** any implementation/configuration change is tied to the exact Candidate and passes required GitHub-hosted Actions checks; no local build/test/package occurs.
- **C5 — Coexistence readiness boundary:** the new slot can be handed to #191 for separate readiness verification without claiming #191, #188, Bilibili or playback evidence.

### Verification jobs

| Job | Claims | Plane / runner | Required evidence |
|---|---|---|---|
| J1 | C1,C2,C3 | external-codex via approved restricted control plane, target tx-node | written authorization/descriptor read-back; exact new UID/unit/cgroup/profile/display/namespace/socket/port ownership; sanitized collision and proxy/EgressPolicy checks; no source-runtime inspection |
| J2 | C2,C3 | target-specific bounded checks under new low-privilege identity | finite resource/time/namespace/display/control and cleanup checks; exact slot-only teardown/read-back; no browser or site request |
| J3 | C4 | GitHub-hosted Actions | exact Candidate checkout, static/security/contract tests and any repository build/package checks required by the changed files |
| JI1 | C4,C5 | GitHub-hosted Actions | integration checks against the exact Candidate and current accepted architecture surfaces, when contract requires them |
| J4 | C5 | Coordinator / #191 handoff | sanitized slot descriptor and provisioning evidence handed to independent #191; no automatic status change in #191 |

The Coordinator may add a required hosted job when repository surfaces change. A skipped required job, missing artifact, stale Candidate or unavailable target authority is `BLOCKED`.

### Target provisioning sequence

1. Re-read Issue, all comments, this contract, #193 report/review, #188/#191 history and current canonical docs. Confirm ready, owner, authorization, descriptor and target route.
2. Read-only admission checks must prove the target identity and generic capability without querying source-runtime contents/state. If the target route or owner boundary is ambiguous, stop.
3. Create only the declared new UID/profile/runtime/unit/cgroup/display/namespace/socket/port resources. Record each creation in a bounded private ledger owned by the new slot. Do not use existing names or adopt pre-existing resources.
4. Apply resource, timeout, namespace, private-bind, EgressPolicy and cleanup restrictions. Verify ownership and restrictions under the new identity. Do not launch a browser or make network requests.
5. Run only finite provisioning self-checks that cannot inspect or connect to source-runtime. Validate no proxy environment, no public listener and no cross-slot endpoint/ownership overlap.
6. Perform exact slot-only rollback or leave state for the authorized owner according to the descriptor. Read back zero unintended residue and preserve the source-runtime untouched.
7. Publish an execution report separating implementation result, target verification, and Coordinator decision. Hand the descriptor/evidence to #191; do not change #191 or begin #188.

## Freshness and integration contract

```text
Freshness policy: dependency-aware
Semantic authorities: #193 isolation research, docs/architecture.md, docs/security.md, docs/development-environments.md, docs/runner-execution-architecture.md
Semantic freshness domains: slot ownership/descriptor, source-runtime coexistence boundary, EgressPolicy/Secret/SSRF rules, target authorization, cleanup authority
Integration surfaces: any repository files explicitly approved by the Coordinator; no unrelated production surfaces may be touched
Task-owned surfaces: target slot provisioning contract, helper/configuration changes and sanitized ownership evidence
Authority/domain → Claim mapping: #193 design → C2; authorization/descriptor → C1; target provisioning → C3/C5; Candidate/Actions → C4
Integration verification jobs: JI1 when implementation touches shared runtime/build/security surfaces
Unrelated-main policy: unrelated docs do not invalidate target authorization; any architecture/security/routing/target-boundary change requires Coordinator review and contract republish
Strict-main reason: n/a
```

A new #193 review, #191 readiness result, target-owner change or security/architecture change may invalidate this Task's preconditions. Do not reclassify it silently; return to `status:draft` for Contract Revision and Publication Gate.

## Success criteria

1. Written owner/admin authorization and a sanitized, Coordinator-approved slot descriptor are recorded before provisioning.
2. The complete combined isolation contract is provisioned and independently evidenced under a dedicated low-privilege identity, with no shared profile, process/cgroup, display/CDP, network/proxy, port or cleanup authority.
3. The existing source-runtime session and production Gateway remain untouched; all rollback and cleanup actions are demonstrably limited to the new slot.
4. Repository changes, if any, have an exact Candidate and required GitHub-hosted Actions evidence; no local or target compilation/build/test/package substitutes for hosted verification.
5. The result is handed to #191 as separate readiness input. It does not set #191 ready, rerun #188, authorize Bilibili, prove browser/media portability or change any parent Issue.

## Evidence contract and blocked rules

Every report must record:

```text
Task / Claim / Attempt: #195 / C1–C5 / N
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high; actual Fast availability
Execution plane: GitHub-hosted Actions and/or approved external-codex control plane
Runner / Target: exact hosted runner or tx-node VM-0-11-ubuntu
Authorization / descriptor: owner/admin authority and sanitized descriptor identity
Base / Candidate: exact SHA(s), or n/a before implementation
Workflow / run / job / artifact: exact identity for each required hosted check
Commands / steps: fixed command classes only; no raw command lines or secrets
Result: PASS / CONDITIONAL PASS / FAIL / BLOCKED per claim
Cleanup: exact slot-only cleanup and residue read-back
Parent boundary: #188/#191/#193/#157/#166/#182/#185 unchanged
```

Classify the Task `BLOCKED` when authorization, descriptor, low-privilege route, exact target capability, hosted Candidate evidence or cleanup ownership is missing; when a required design change is unreviewed; when resources collide or ownership is ambiguous; or when safe evidence would require touching source-runtime. Classify `FAIL` for an actual cross-slot access, public endpoint, proxy/Secret/SSRF bypass, broad cleanup, unauthorized mutation or other policy violation. A theoretical design or host tool presence is not `PASS`.

The Worker must post `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, move the Issue to `status:review` or `status:blocked`, release ownership and stop. The Worker must not set `status:done`, close the Issue, publish a Coordinator Review, change #191 readiness, or start #188.
