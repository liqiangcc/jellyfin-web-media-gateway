# INFRA-005-TARGET-RUNNER-CONTROL-PLANE-RECOVERY

## Identity

- Issue: #130
- Kind: infrastructure incident verification / bounded recovery
- Environment: `env:ubuntu-arm64`
- Target: existing runner id `2` / `ubuntu-arm64-target-phone`
- Trigger deployment run: `33317481859`
- Trigger deployment job: `99273465231`
- Trigger R002 Candidate: `b5d7c3c26b6e4839ada7ce41ad0ba92ee0955d36`
- Planning Base: `c51a1ed4d6389beb708d23e331b9bc1583a38195`
- Historical authorities: #21 INFRA-002 and #87 INFRA-003

## Problem

The accepted phone is online and reachable over Tailscale/SSH, but the trusted R002 deployment job remains queued because the accepted Target Runner control plane is not running. Before this Task:

- no `Runner.Listener` / `Runner.Worker` was present;
- one accepted `gateway-runnerctl restart` was attempted;
- read-back remained `stopped`;
- no Listener appeared;
- the already-created deployment job remained queued.

This is a new incident. It must not reopen or rewrite #21/#87 historical acceptance.

## Goal

Classify why the existing accepted runner control plane cannot start and perform the smallest safe recovery that preserves the existing runner registration, identity and low-privilege boundary.

## Frozen boundaries

- preserve runner id/name/configuration and labels;
- no re-registration, credential rotation, registration-token creation, or runner Secret-file inspection;
- no full environment or `/proc/*/environ` dump;
- no Authorization/Cookie/token/proxy credential output;
- no sudo/package installation/ADB/boot/kernel change;
- no product/Gateway/media/site/security code change;
- no second R002 deployment dispatch;
- do not cancel run `33317481859` unless Coordinator later revises this Task;
- diagnostics are bounded to `gateway-runnerctl` state/output, process tree, safe filesystem metadata, and redacted/latest runner `_diag` error/session/transport lines;
- preserve uid 999 `gateway-runner`, zero-capability/no-sudo boundary and existing `_work` workspace;
- no Bilibili request, no #67/#68, no R003 workload.

## Execution sequence

### J0 — current incident read-back

Record only bounded non-secret evidence:

- `gateway-runnerctl status`;
- `Runner.Listener` / `Runner.Worker` / accepted supervisor-helper process inventory;
- accepted runner directory/control-script path metadata without credentials;
- latest bounded `_diag` filenames/timestamps and only error/session/transport/control lines needed to classify start failure;
- phone architecture/user/disk/memory sanity as needed;
- GitHub status of run `33317481859` / job `99273465231`.

### J1 — classify

Classify one of:

- stale/conflicting runner session/process state;
- control-script/supervisor start failure;
- missing executable/runtime file or permission within the existing installation;
- GitHub transport/session failure;
- unknown / insufficient evidence.

Do not guess and do not inspect Secrets.

### J2 — bounded recovery

Use only the smallest intervention supported by J0/J1 evidence and historical accepted control plane. Allowed examples are bounded stop/start/restart of the **existing** `gateway-runnerctl` control plane and cleanup of clearly stale process/PID state owned by that control plane. Do not re-register or reinstall.

At most one recovery cycle after J1 classification. If it remains stopped, STOP and report BLOCKED.

### J3 — recovery proof

If recovery succeeds, prove:

- `Runner.Listener` exists as uid 999 `gateway-runner`;
- accepted workspace/labels/identity are unchanged;
- no stale unexpected `Runner.Worker` remains before job assignment;
- GitHub naturally assigns the already-queued job `99273465231` without another dispatch;
- if the job starts, normal steps/logs become visible.

Do not wait for the 45-minute deployment hold to finish merely to prove runner recovery. Once the job has normal steps and the trusted workflow publishes/derives its LAN entry, downstream #7 authority belongs to Coordinator.

## Claims

- C1 incident classified from bounded non-secret evidence.
- C2 existing runner identity/configuration preserved.
- C3 no re-registration/credential/security expansion.
- C4 recovery restores low-privilege Listener or produces explicit irrecoverable blocker.
- C5 already-queued job is used as schedulability proof; no second dispatch.
- C6 product/runtime/site semantics unchanged.

## Success criteria

PASS when C1-C6 are evidenced and the existing runner is restored enough for job `99273465231` to leave queued with normal target-runner execution. BLOCKED when one bounded classified recovery cycle cannot restore the Listener. Worker must terminal-report using the fresh terminal-write authority guard from `docs/tasks/issue-lifecycle-protocol.md` and then STOP.