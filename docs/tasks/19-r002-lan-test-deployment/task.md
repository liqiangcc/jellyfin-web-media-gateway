# Task — R002-DEPLOY Trusted LAN Test Deployment Preparation

## Metadata

```text
GitHub Issue: #19
Parent Goal / Research Item: R002 / physical-TV remote audible playback feasibility
Task / Research ID: R002-DEPLOY
Task kind: implementation
Planning / integration base commit: 132c2747d736f9af72d9c06cfc08660876619029
Session bootstrap prompt: docs/tasks/19-r002-lan-test-deployment/prompt.md
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Preferred worker: cloud-codex
Eligible worker environments after publication: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, rust-build, rust-test, workflow-authoring, automated-test, security-static-analysis
Accepted upstream: Issue #1 / INFRA-001; Issue #6 / R002-PREP
Downstream verification task: Issue #7 / R002-TV
Hard publication dependencies: none beyond accepted #1 and #6
Execution plane prepared by this Task: github-actions
Target deployment backend after Coordinator merge/approval: ubuntu-arm64-self-hosted / target-device
```

> Live status, owner, Attempt, PR, Candidate SHA and Evidence belong in Issue #19.
>
> This Task prepares the deployment mechanism. It does not execute or classify the physical-TV R002 experiment. Issue #7 remains the real-device Evidence Authority.

## Goal

Prepare the smallest auditable path for a **trusted, bounded, LAN-reachable test deployment** of the accepted R002 Web Display probe on the Ubuntu ARM64 Target Runner.

The current accepted server binds only to `127.0.0.1`, so simply starting it on the target phone cannot satisfy Issue #7. This Task must make LAN binding an explicit deployment/test configuration while preserving loopback as the product/test-server default, and must add a trusted manual Target Runner workflow that can later keep an isolated test instance alive long enough for physical-TV verification.

The intended post-merge sequence is:

```text
Issue #19 accepted + merged trusted workflow
→ Coordinator dispatches trusted deployment workflow
→ workflow starts exact approved Candidate on target phone
→ workflow reports bounded LAN /display + /control entry
→ Coordinator records reachable deployment in Issue #7
→ Issue #7 Publication Gate
→ real TV/manual Evidence
```

## Why / Context

Issue #6 is accepted/merged and provides the R002 probe mechanics. Issue #7 cannot yet publish because it requires an accepted candidate on a specific reachable test deployment.

Current `gateway-core/src/bin/r001-server.rs` binds `TcpListener` to `127.0.0.1`. The change here is deliberately narrow: make the bind address explicit and safe for a trusted LAN deployment without changing R001 media semantics, R007 Playback authority, or the default exposure model.

Canonical security allows the MVP on a trusted family LAN and requires basic playback to work over LAN HTTP, while still forbidding default public exposure. Target Runner jobs must stay low privilege, use separate test workspace/ports, contain no production Secrets, and untrusted PRs must not automatically gain target shell authority.

## Decomposition / Trust Decision

```text
Issue #19 / R002-DEPLOY
→ cloud-codex implements bind configuration + trusted workflow
→ GitHub-hosted CI proves code/workflow mechanics
→ Coordinator reviews/merges

Post-merge Coordinator deployment setup
→ manual dispatch of trusted workflow from accepted repository state
→ Ubuntu ARM64 Target Runner starts bounded test instance
→ produces actual deployment entry

Issue #7 / R002-TV
→ real physical TV + phone/control path
→ R002 PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

An unreviewed PR must not run the new deployment workflow on the target phone.

## Canonical Sources to Read

Before implementation read at least:

- `AGENTS.md`
- Issue #19 and relevant comments
- Issue #6 Final Acceptance and accepted/merged R002 Candidate
- Issue #1 Final Acceptance / Target Runner constraints
- `gateway-core/src/bin/r001-server.rs`
- `docs/security.md` sections 1, 14, 16
- `docs/runner-execution-architecture.md`
- `docs/technical-feasibility-validation.md` R002
- `docs/tasks/7-r002-physical-tv-verification/task.md`
- `docs/tasks/issue-lifecycle-protocol.md`

## Architecture / Security Invariants

1. Default listen behavior remains loopback-only.
2. LAN listening requires explicit deployment/test configuration; no implicit `0.0.0.0` default.
3. The trusted target workflow should bind to a concrete target LAN address derived/validated on the target when practical, rather than blindly exposing all interfaces.
4. Public/WAN exposure is out of scope and must not be introduced.
5. Test instance uses a dedicated test port and Runner workspace, never `/var/lib/web-media-gateway/` production state.
6. Target job remains non-root/no-sudo and does not install packages.
7. No Vault, site Cookie/token/profile, Jellyfin API key, SSH key, Tailscale auth key or other production/long-term Secret is required.
8. Target workflow is `workflow_dispatch` only; no automatic `pull_request`/`push` target trigger.
9. Candidate SHA is explicit full 40-hex and separately recorded from the trusted workflow/harness SHA.
10. Workflow inputs are strict/bounded and must not become arbitrary shell command interpolation.
11. R001 Media Gateway / Secret/open-proxy behavior remains authoritative and unchanged.
12. R007 Playback command/revision/handoff semantics remain authoritative and unchanged.
13. Issue #19 does not claim R002 physical-TV success.

## Work Role

### A. Configurable listen address

Extend the R001/R002 server entrypoint with an explicit listen-address configuration, expected equivalent:

```text
R001_BIND_ADDR=<IP address>
R001_PORT=<port>
```

Requirements:

- default address is `127.0.0.1` when unset;
- parse as an IP address, not a hostname or arbitrary socket string;
- invalid values fail closed with an explicit startup error;
- do not change the existing default port unless already configured through `R001_PORT`;
- server startup log may print its actual listen address, but must not print temporary media capability URLs or Secrets;
- unit/integration tests prove default loopback and explicit bind configuration semantics.

The implementation may choose an equivalent env/config name if it is clearer, but the default/security behavior is frozen.

### B. Trusted bounded deployment workflow

Add a dedicated workflow, expected equivalent:

```text
.github/workflows/r002-target-tv-deployment.yml
```

Required properties:

```text
trigger: workflow_dispatch only
runs-on: [self-hosted, linux, ARM64, ubuntu-arm64, target-device]
permissions: contents: read
candidate_sha: required full 40-hex
hold_minutes: strict bounded choice/validation
port: fixed repository-defined test port or strict safe choice
production state: forbidden
sudo/root/package install: forbidden
```

The workflow must:

1. checkout the **trusted workflow revision** separately from the measured/deployed Candidate checkout when needed to keep control logic trusted;
2. verify Candidate checkout HEAD equals the requested full SHA;
3. verify target user is non-root;
4. use a dedicated test workspace/runtime and a fixed non-production port (recommended `18788` unless conflict requires another documented test port);
5. derive a concrete LAN-reachable IPv4 address from the target's normal network route/interface, validate it is a private LAN address suitable for the manual TV experiment, and fail safely if no acceptable address exists;
6. start the exact Candidate server bound to that concrete LAN address;
7. use the accepted public/non-secret R002 media path; do not require production Secrets or protected fixture credentials;
8. self-smoke `/healthz`, `/display`, `/control` against the bound address before declaring the deployment ready;
9. record clearly:

```text
trusted workflow SHA
candidate SHA
runner name / labels / arch / OS
bind address
port
display entry
control entry
start UTC
planned expiry/end UTC
```

10. keep the deployment alive for a bounded manual-verification window; supported values should cover at least a 30-minute idle scenario plus setup overhead (for example 45/75/90 minutes, with an upper bound);
11. keep the Gateway process alive for the same bounded window and fail if it exits unexpectedly;
12. always stop the test process and clean temporary runtime on success/failure/cancel/timeout;
13. upload non-secret deployment/server diagnostics as artifact when useful.

Do not make the workflow a persistent production service manager. This is a temporary verification deployment.

### C. Hosted verification

Because the target deployment workflow is untrusted until merge, Issue #19 required CI runs only on GitHub-hosted runners.

Hosted tests must validate:

- loopback remains the default;
- explicit bind semantics work on a safe hosted test address/interface;
- invalid bind value fails closed;
- deployment workflow is manual-only;
- target labels/permissions are correct;
- candidate SHA and hold duration are strictly validated;
- no PR/push target trigger;
- no sudo/package-install/production Vault path;
- cleanup/timeout logic exists and is deterministic where testable;
- R001/R007 regressions remain passing if affected.

## Verification Claims

```text
C1 — Default exposure preserved
Without explicit bind configuration, the server listens only on loopback exactly as before.

C2 — Explicit LAN bind mechanics
A valid explicit IP can be used to bind the test server; invalid/non-IP configuration fails closed and is test-covered.

C3 — Trusted target dispatch
The target deployment workflow is manual-only, explicit-Candidate, low-privilege and cannot be automatically triggered by an untrusted PR/push.

C4 — Target isolation
Deployment uses a dedicated test workspace/port, no production Vault/runtime and no production/long-term Secrets.

C5 — Bounded deployment lifecycle
Workflow supports a bounded manual-TV window, detects early server exit, and guarantees cleanup on normal/failure/cancel/timeout paths as far as the platform permits.

C6 — Reachability setup contract
Workflow derives/validates a private LAN address and emits stable `/display` + `/control` deployment entries after self-smoke, without exposing temporary media capabilities/Secrets.

C7 — Existing authority preserved
No R001 media/Secret/open-proxy semantics or R007 Playback concurrency/handoff semantics are redefined.

C8 — Issue #7 readiness mechanism
After this workflow is Coordinator-accepted/merged, Coordinator can dispatch an exact approved Candidate and obtain the concrete deployment entry required to publish Issue #7 without inventing new deployment behavior.
```

## Verification Job Matrix

| Job | Claims | Execution plane | Runner/target | Required | Evidence |
|---|---|---|---|---|---|
| J1 | C1,C2,C7 | GitHub Actions | github-hosted x64 | yes | Rust build/test + bind integration tests + affected regressions |
| J2 | C3-C6,C8 | GitHub Actions | github-hosted x64 / static workflow validation | yes | workflow trust/input/cleanup/static tests |
| J3 | none for this Attempt | Ubuntu ARM64 Target Runner | target phone | forbidden before Coordinator merge | post-merge deployment setup belongs to Coordinator |

## Success Criteria

1. Default server remains loopback-only.
2. Explicit IP bind is implemented, validated and test-covered.
3. A manual-only Target Runner deployment workflow exists with explicit Candidate SHA, low privilege, strict inputs and no untrusted automatic trigger.
4. Workflow binds only to a validated concrete private LAN address for the experiment rather than changing the global default to public/all-interface listening.
5. Deployment uses dedicated non-production workspace/port and no production Secrets.
6. Bounded deployment lifetime and cleanup are deterministic/reviewable.
7. Workflow emits enough non-secret metadata for Coordinator to record a concrete `/display` and `/control` entry after post-merge dispatch.
8. J1/J2 pass on the exact final Issue #19 Candidate SHA.
9. Affected R001/R007 regressions pass on the final Candidate when relevant.
10. No target phone deployment is executed from the unreviewed Issue #19 PR.
11. No R002 physical-TV result is claimed.

## Evidence Contract

The `[EXECUTION REPORT]` must include:

```text
Attempt:
Base commit:
Candidate commit:
PR:
Worker / Environment:
Bind config name/default:
Default-loopback test:
Explicit-bind test:
Invalid-bind test:
Target workflow path:
Target trigger/labels/permissions validation:
Candidate-SHA validation:
Hold-duration bounds:
LAN-address validation strategy:
Test port/workspace:
Secret/production-path checks:
Cleanup/failure tests:
J1 run/job:
J2 run/job:
R001/R007 regression runs if applicable:
Claims C1-C8:
Limitations:
```

Do not include real production Secrets, signed media capability URLs or account data.

## Post-Merge Coordinator Handoff

After ACCEPT/merge, the Coordinator — not the implementation Worker — will:

1. read back the trusted workflow from `main`;
2. approve an exact Candidate SHA for the physical-TV deployment;
3. manually dispatch the target deployment workflow;
4. verify target self-smoke and obtain the actual LAN `/display` and `/control` entries;
5. record deployment identity/expiry in Issue #7;
6. complete Issue #7 Publication Gate and hand the real-device procedure to Manual TV Evidence Authority.

If the Target Runner is unavailable or no acceptable private LAN address exists, that is a deployment blocker to report before publishing #7; do not weaken the network/security rules.

## Out of Scope

- persistent production deployment/service management;
- public Internet exposure, reverse proxy, port forwarding or router configuration;
- Tailscale auth/bootstrap;
- changing home firewall/router rules automatically;
- physical-TV R002 classification;
- real site authentication or Vault use;
- R003 resource measurement;
- Jellyfin;
- R007 concurrency changes;
- R001 media-path redesign.

## Completion Protocol

Worker follows `docs/tasks/issue-lifecycle-protocol.md`:

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ implementation + exact-SHA hosted J1/J2
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Worker must not dispatch the target deployment workflow before Coordinator acceptance/merge, must not execute Issue #7, and must not set `status:done` or close Issue #19.