# Task — INFRA-002 Recover Stuck Ubuntu ARM64 Target Runner

## Metadata

```text
GitHub Issue: #21
Task ID: INFRA-002
Task kind: verification/operations
Parent Goal: restore trusted target execution before Issue #9 / R003-TARGET
Preferred worker: ubuntu-arm64
Eligible environment: env:ubuntu-arm64
Required capabilities: github-read-write, arm64-target-runtime, interactive-linux-debug, runner-lifecycle-control
Accepted authority: Issue #1 / INFRA-001
Downstream consumer: Issue #9 / R003-TARGET
Handoff: docs/tasks/handoffs/ubuntu-arm64.md
Hard publication dependencies: none
```

> Live state, owner, Attempt and recovery Evidence belong in Issue #21.
>
> This is an execution-plane recovery Task. It does not execute or classify R003.

## Triggering Incident

Coordinator reran the accepted INFRA-001 smoke to prove current target schedulability before publishing Issue #9:

```text
workflow: target-runner-smoke
run: 32727443950
attempt: 2
job: 97505873310
run_started_at: 2026-08-24T16:22:20Z
configured job timeout: 5 minutes
```

GitHub moved the job from `queued` to `in_progress`, proving that the existing `ubuntu-arm64-target-phone` listener accepted scheduling. The job then remained `in_progress` beyond its 5-minute timeout window and exposed no completed steps/logs through Coordinator read-back.

Treat this as a Runner lifecycle incident, not as R003 Evidence.

## Goal

Recover the **existing** accepted Target Runner identity and lifecycle with the smallest safe intervention, preserving INFRA-001 boundaries:

```text
ubuntu-arm64-target-phone
labels: self-hosted, Linux, ARM64, ubuntu-arm64, target-device
runtime user: gateway-runner
workspace: /home/gateway-runner/actions-runner/_work
production Gateway state: separate
```

After recovery, the runner should be online/idle and able to accept a fresh Coordinator smoke without stale Runner.Worker/job processes remaining.

## Canonical Reading

Before action read:

- Issue #21 and comments
- Issue #1 Execution Report / Coordinator Review / Final Acceptance
- `AGENTS.md`
- `docs/tasks/issue-lifecycle-protocol.md`
- `docs/tasks/handoffs/ubuntu-arm64.md`
- `docs/runner-execution-architecture.md`
- `docs/security.md` Target Runner sections
- current `.github/workflows/target-runner-smoke.yml`
- Issue #9 publication snapshot

## Invariants

1. Recover the existing runner first. Do not delete/re-register the runner or rotate credentials merely because one job is stuck.
2. Do not print/cat `.credentials`, registration tokens, PATs, SSH keys, Tailscale auth material or other Secrets into Issue/log Evidence.
3. Keep runtime execution as dedicated low-privilege `gateway-runner`; operator/root shell may only operate the already accepted supervisor/control plane and inspect host state.
4. Do not add sudo/root capability to `gateway-runner`.
5. Do not move the Runner workspace into `/var/lib/web-media-gateway` or production Vault/runtime paths.
6. Do not run R003 heavy scenarios from this Task.
7. Do not change CPU governor, thermal controls, Android kernel/root policy, firewall or network architecture to make the Runner look healthy.
8. Do not kill unrelated host processes. Limit cleanup to the accepted Runner supervisor/listener/worker process tree and the identified stale job.
9. If recovery requires a durable supervisor implementation change beyond restoring the accepted INFRA-001 setup, report it explicitly; do not silently broaden this operational Task.

## Recovery Procedure

### J0 — Preserve incident evidence

Before restart record non-secret diagnostics:

```text
date -u
uname -a
id
free -h
df -h /home/gateway-runner /tmp 2>/dev/null || true
ps -eo pid,ppid,pgid,sid,user,stat,etime,cmd --sort=pid | grep -E 'Runner\.(Listener|Worker)|run-helper|gateway-runner' || true
```

Inspect the existing control plane when present:

```text
command -v gateway-runnerctl
command -v gateway-runner-supervise
gateway-runnerctl status || true
```

Inspect only non-secret diagnostic metadata under:

```text
/home/gateway-runner/actions-runner/_diag/
```

Record filenames/timestamps and bounded tails of relevant diagnostic logs. Redact any token-like value before Issue Evidence. Do **not** print credential/config secret files.

Determine whether the stuck job corresponds to a live `Runner.Worker` / helper process, an orphaned process group, listener loss, network loss, or a control-plane state mismatch.

### J1 — Controlled recovery

Prefer the accepted lifecycle command:

```text
gateway-runnerctl restart
```

If restart itself cannot complete, use the accepted `stop` then `start` path and capture why.

After stop, verify the previous Runner process group does not leave stale `Runner.Listener`, `Runner.Worker`, `run-helper` or task child processes owned by the runner.

Do not use broad `killall`/system-wide process killing. If the accepted control tool fails to reap a specific runner-owned stale child, identify exact PID/PGID and preserve evidence before any narrow cleanup.

### J2 — Runtime-boundary verification

After recovery verify at minimum:

```text
Runner.Listener exists
Runner.Listener user = gateway-runner
runner workspace remains under /home/gateway-runner/actions-runner
no runner process executes as root
no new sudo/admin membership
no production Vault/runtime overlap
```

Where existing tooling permits, re-check the accepted zero-capability runtime boundary. Absence of optional inspection tooling is not a reason to install packages during this Task.

Verify outbound GitHub connectivity using the existing runner/listener behavior; do not expose credentials to test it.

### J3 — GitHub-side state

Use GitHub to confirm the repository still has the expected runner identity/labels when that API is available. At minimum record whether the existing runner returns to online/idle/claimable state without re-registration.

**Do not dispatch another smoke from this Worker.** Coordinator owns the post-recovery fresh smoke and Issue #9 publication decision.

## Claims

```text
C1 — Incident is classified with concrete process/control/log evidence rather than guessed.
C2 — Existing runner identity/configuration is preserved; no unnecessary re-registration/credential rotation occurs.
C3 — Stale runner/job processes are deterministically removed by bounded lifecycle recovery.
C4 — Runner.Listener returns under gateway-runner with accepted workspace/privilege boundaries intact.
C5 — Runner returns to an online/idle/claimable state suitable for Coordinator smoke.
C6 — No R003 workload/result is claimed and no production Secret/state boundary is weakened.
```

## Success Criteria

1. J0 incident diagnostics are recorded without Secrets.
2. Existing runner identity is preserved unless irrecoverable registration corruption is proven.
3. Accepted control plane can stop/restart the runner or a concrete blocker is reported.
4. No stale runner Worker/helper/job process remains after the recovery stop/restart boundary.
5. Listener returns as `gateway-runner`, non-root, in the accepted isolated workspace.
6. GitHub-side runner state is online/idle/claimable when observable.
7. Worker posts `[EXECUTION REPORT]`, moves Issue to `status:review`, releases owner and stops.
8. Worker does not rerun smoke, publish #9, execute R003, close #21 or mark `status:done`.

## Evidence Contract

The report must include:

```text
Attempt:
Execution host/target:
Incident run/job:
Observed stuck process/control state:
Relevant non-secret _diag files/timestamps:
Memory/disk/network observations:
Control command used:
Stop cleanup result:
Restart/start result:
Runner.Listener PID/user/workspace:
Runner.Worker/helper leftovers after recovery:
Privilege/capability boundary result:
GitHub runner identity/labels/state:
Credential/re-registration changes: none | explain proven necessity
Claims C1-C6:
Limitations:
```

## Blocker Conditions

Return `[BLOCKER REPORT]` rather than improvising if, for example:

- runner process is stuck in uninterruptible kernel state and accepted control cannot recover it;
- filesystem/workspace corruption prevents safe startup;
- GitHub runner registration is demonstrably invalid and re-registration would be required;
- target network cannot reach GitHub after local network sanity checks;
- accepted supervisor/control scripts are missing/corrupt and cannot be restored from known accepted configuration without broader work;
- recovery would require weakening the low-privilege/security boundary.

## Completion Protocol

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ diagnose + bounded recovery
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

If blocked:

```text
[BLOCKER REPORT]
→ status:blocked
→ release owner
→ STOP
```

Only Coordinator performs Final Acceptance/close and the post-recovery smoke.