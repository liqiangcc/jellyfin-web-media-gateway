# Phone management-plane readiness and recovery runbook

Status: **PREP only**. This document does not authorize a live phone mutation. A later target-phone Task must obtain its own Coordinator gate.

## Purpose

The accepted Android/Termux phone is a physical target. Cloud/Tailscale is only an external management path under `docs/security.md` section 16.5. A missing old tmux or SSH ControlMaster must not make the phone logically unrecoverable.

All target-phone Tasks that require persistent Ubuntu execution should consume one shared readiness model instead of issuing ad-hoc repeated probes.

## Canonical readiness states

```text
DEVICE_OFFLINE
  -> TAILNET_ONLY
  -> SSH_READY
  -> UBUNTU_PERSISTENT_READY
```

`scripts/phone_mgmt_readiness.py` is the pure classifier. Only `UBUNTU_PERSISTENT_READY` returns `claim_allowed=true`.

ControlMaster/control socket state is intentionally **not** an input to authority. It may reduce connection setup cost but cannot be a prerequisite.

## Bounded live probe order for a future Task

A later live management-readiness Task must preserve this exact layered order:

1. **Tailnet layer**
   - perform one bounded reachability check only against the accepted phone target;
   - if unreachable or ambiguous, classify at/below `DEVICE_OFFLINE` and STOP;
   - do not attempt SSH when this layer fails.
2. **SSH layer**
   - after Tailnet PASS, perform one bounded check of the accepted SSH endpoint and one non-interactive authentication attempt;
   - no password prompt, auth guessing, key enumeration output, ControlMaster requirement or retry-until-success;
   - if TCP/auth is not fully proven, classify `TAILNET_ONLY` and STOP.
3. **Ubuntu layer**
   - after authenticated SSH PASS, run only harmless identity/readiness checks needed to establish the accepted Ubuntu/chroot context;
   - if Ubuntu is unavailable, classify `SSH_READY` and STOP.
4. **Persistence layer**
   - prove one management context exists whose lifetime is independent from a one-shot SSH command;
   - acceptable evidence can use repository-approved persistent tmux/chroot topology;
   - a merely successful one-shot Ubuntu command is insufficient;
   - only after this proof classify `UBUNTU_PERSISTENT_READY`.

There is one bounded set. A failure at an earlier layer suppresses deeper probes. No sampling-until-success and no adaptive retry loop.

## Eventual phone-side recovery boundaries

When a separate phone recovery Task is explicitly published, it may inspect the minimum accepted management components needed to restore the failed layer. Candidate components include:

- Termux process/persistence state;
- Termux wake/persistence mechanism required by the accepted topology;
- Tailscale process/connectivity state;
- sshd process/listener state on the already accepted management endpoint;
- Ubuntu/chroot parent lifetime and its persistent tmux/session topology.

The later Task must first diagnose the failed layer. It must not blindly restart all components.

Any permitted recovery action must be idempotent, bounded and read-back verified. The Task must freeze exact allowed commands/settings before mutation and STOP after one unsuccessful bounded recovery cycle.

## Security boundary

Management recovery must preserve `docs/security.md` 16.5:

- Cloud/other external Workers may access only the accepted target and Task-scoped management port/path;
- Tailnet membership does not authorize LAN scans or unrelated peer discovery;
- no new public/LAN/`0.0.0.0` management listener is introduced;
- no Tailscale auth key, SSH private key, password, token, shell environment, profile, Runner credential or other long-lived Secret is captured in Evidence;
- no `/proc/*/environ` or credential/config dump;
- no Runner/product/browser/Bilibili action is implied by management readiness;
- no package reinstall, root/capability expansion, ADB or reboot without a later explicit Task Contract.

## Downstream claim gate

Before claiming #113, #131, or another target-phone Task requiring persistent Ubuntu execution:

1. obtain a fresh non-secret readiness snapshot;
2. run `scripts/phone_mgmt_readiness.py` locally/offline on that normalized snapshot;
3. require:

```text
state = UBUNTU_PERSISTENT_READY
claim_allowed = true
reason = authorized
```

Any other state means **do not claim** the downstream Task. The readiness probe/recovery Task reports its own bounded result and returns authority to Coordinator.

## Current #139 boundary

For Issue #139 itself:

- Live phone probe: **NOT RUN**
- Live phone mutation: **NOT RUN**
- Tailscale/SSH/sshd/wake/chroot action: **NOT RUN**
- Runner/Bilibili/#113/#131 execution: **NOT RUN**
