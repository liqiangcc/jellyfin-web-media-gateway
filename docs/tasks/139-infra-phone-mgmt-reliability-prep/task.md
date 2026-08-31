# INFRA-PHONE-MGMT-RELIABILITY-PREP

## Identity

- Issue: #139
- Kind: infrastructure management-plane reliability PREP
- Preferred Worker: cloud
- Eligible environment: `env:cloud`
- Planning Base: `11266ba77eff04977e64913f0f2352af98781127`
- Downstream blockers: #113 R3 Attempt 4, #131 Attempt 6
- Security authority: `docs/security.md` section 16.5
- Live phone/network mutation: forbidden in this Task

## Problem

The accepted phone execution plane repeatedly disappears between valid task windows. The latest bounded check returned `tailnet_device_reachable=no`, while target-phone work remains queued/blocked. Existing repository security rules define Cloud/Tailscale as an external management path but do not define a persistent readiness/recovery protocol.

Previous successful access sometimes reused an already-established SSH ControlMaster/tmux context. That is not a durable recovery contract.

## Goal

Prepare one repository-owned, deterministic, fail-closed readiness model and recovery runbook for the phone management plane.

Canonical states:

```text
DEVICE_OFFLINE
  -> TAILNET_ONLY
  -> SSH_READY
  -> UBUNTU_PERSISTENT_READY
```

Only `UBUNTU_PERSISTENT_READY` may authorize claim of a target-phone Task that needs persistent Ubuntu execution.

## Readiness semantics

The classifier consumes only normalized non-secret observations. It performs no network or device operations.

Required observations:

- `tailnet_reachable: bool|unknown`
- `ssh_tcp_reachable: bool|unknown`
- `ssh_authenticated: bool|unknown`
- `ubuntu_context_reachable: bool|unknown`
- `persistent_context_proven: bool|unknown`

Classification rules:

1. `tailnet_reachable=false` -> `DEVICE_OFFLINE`.
2. Tailnet true but SSH TCP/auth not fully proven -> `TAILNET_ONLY`.
3. SSH authenticated true but Ubuntu/persistence not fully proven -> `SSH_READY`.
4. All required observations true -> `UBUNTU_PERSISTENT_READY`.
5. Ambiguous/contradictory observations fail closed and must never return the final ready state.

The classifier must also emit a bounded reason and `claim_allowed: true|false`.

## Management-plane contract

### ControlMaster

SSH ControlMaster/control socket may be used as an optimization only. Its absence must never make a healthy phone unrecoverable if ordinary non-interactive SSH authentication is available.

### Cloud probe order

A later live execution Task must use a bounded layered order:

1. one Tailnet reachability check;
2. only if reachable, one bounded SSH TCP/auth readiness check;
3. only after authenticated SSH, one harmless Ubuntu context check;
4. only after Ubuntu is reachable, prove a persistent management context whose lifetime is independent from a one-shot command.

Failure at an earlier layer stops deeper probing. No retries-until-success.

### Phone-side eventual recovery boundaries

A later phone execution Task may inspect/recover only the minimum already-accepted management components needed for persistence, such as Tailscale process/connectivity, sshd availability, Termux wake/persistence state, and Ubuntu/chroot lifetime. This PREP does not authorize those mutations.

Any later recovery must be:

- idempotent and bounded;
- limited to the accepted phone management plane;
- no LAN scan or broad Tailnet enumeration;
- no new public listener beyond the existing management boundary;
- no Tailscale auth-key, SSH key, password, token, environment or credential dump;
- no Runner/product/browser/Bilibili mutation unless a separate Task explicitly authorizes it;
- no blind restart loop, package reinstall, root expansion, ADB or reboot without a later explicit gate.

## Claims

- C1: pure classifier performs no network/subprocess/device mutation.
- C2: `DEVICE_OFFLINE` is returned when Tailnet is explicitly unreachable.
- C3: `TAILNET_ONLY` requires Tailnet reachability but incomplete SSH readiness.
- C4: `SSH_READY` requires authenticated SSH but incomplete persistent Ubuntu readiness.
- C5: `UBUNTU_PERSISTENT_READY` requires all readiness evidence true and is the only `claim_allowed=true` state.
- C6: unknown/contradictory inputs fail closed and never authorize claim.
- C7: ControlMaster presence/absence does not participate in final readiness authority.
- C8: runbook preserves `docs/security.md` 16.5: Cloud/Tailscale remains an external management path with target/port scope only.
- C9: runbook forbids retry loops and deeper probes after earlier-layer failure.
- C10: #113/#131 remain unclaimed until a later live Task proves `UBUNTU_PERSISTENT_READY`.

## Deterministic verification

Offline tests must cover at least:

- each canonical state;
- fully ready state -> `claim_allowed=true`;
- every unknown field family -> fail-closed/non-ready;
- contradictory combinations such as SSH authenticated while Tailnet false;
- ControlMaster metadata ignored for readiness authority;
- input purity/no mutation;
- bounded output schema/field lengths;
- static proof classifier imports/uses no network, DNS, SSH, subprocess or filesystem mutation path;
- documentation scan proving no auth-key/private-key/password/token literals or retry-until-success guidance.

## Boundaries

- no live Tailnet/SSH probe by this Task;
- no phone SSH or device mutation;
- no Tailscale/sshd/wake-lock/chroot start/stop/configuration;
- no package install/root/ADB/reboot;
- no Runner/workflow/product/media/browser/site/security-runtime change;
- no Bilibili/#67/#68/#113/#131 live execution;
- no credential material in tests/docs/artifacts.

## Evidence required

Worker report must include:

- exact Candidate SHA + PR;
- offline test count and result;
- syntax/compile/lint result;
- diff-scope review;
- targeted secret/leak scan and tooling limitations;
- explicit `Live phone probe: NOT RUN`;
- explicit `Live phone mutation: NOT RUN`.

## Success criteria

PASS requires C1-C10, deterministic offline tests, coherent bounded runbook, compatibility with `docs/security.md` 16.5, no live phone/network mutation and a narrow reviewable Candidate.
