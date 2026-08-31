# INFRA-PHONE-MGMT-READINESS-LIVE

## Identity

- Issue: #142
- Kind: live infrastructure readiness verification
- Preferred Worker: cloud
- Eligible environment: `env:cloud`
- Planning Base: `feaafaae6ac81d01ea8d0a1a1582bec31074d418`
- Accepted readiness authority: #139 Final Acceptance / `feaafaae6ac81d01ea8d0a1a1582bec31074d418`
- Downstream consumers: #113 R3 Attempt 4 and #131 Attempt 6
- Accepted classifier: `scripts/phone_mgmt_readiness.py`

## Goal

Run exactly one bounded layered observation set against the accepted phone management plane, normalize only non-secret readiness booleans/unknowns, and classify the snapshot through the accepted #139 classifier.

Canonical states:

```text
DEVICE_OFFLINE
  -> TAILNET_ONLY
  -> SSH_READY
  -> UBUNTU_PERSISTENT_READY
```

Only the final state with:

```text
state = UBUNTU_PERSISTENT_READY
claim_allowed = true
reason = authorized
```

returns authority to Coordinator to consider claiming #113 or #131.

## Attempt 1 probe contract

Attempt 1 is verification-only. It does not mutate the phone management plane.

Use one layered bounded set:

1. **Tailnet layer**
   - run exactly one bounded reachability check to the already-accepted phone target;
   - suppress raw peer/route/latency output;
   - if not proven reachable, set `tailnet_reachable=false|unknown`, leave all deeper observations unknown, classify, report BLOCKED and STOP.
2. **SSH layer**
   - only after Tailnet PASS, run one bounded TCP/readiness check and one non-interactive authentication attempt to the accepted SSH endpoint;
   - no password prompt, auth guessing, key enumeration output, ControlMaster requirement or retry;
   - if TCP/auth is not fully proven, set normalized observations accordingly, classify, report BLOCKED and STOP.
3. **Ubuntu layer**
   - only after authenticated SSH PASS, run harmless checks sufficient to prove the accepted Ubuntu/chroot execution context;
   - no Runner/product/device mutation;
   - if unavailable, classify `SSH_READY` or lower, report BLOCKED and STOP.
4. **Persistence layer**
   - only after Ubuntu PASS, prove one existing/created Task-scoped management context whose lifetime is independent from the one-shot command;
   - a one-shot successful Ubuntu command alone is insufficient;
   - this proof may use the already-accepted persistent tmux/chroot topology but ControlMaster state itself is not authority.
5. Run the accepted classifier offline on the normalized snapshot.

## Observation schema

Durable evidence may retain only:

```text
tailnet_reachable: true|false|unknown
ssh_tcp_reachable: true|false|unknown
ssh_authenticated: true|false|unknown
ubuntu_context_reachable: true|false|unknown
persistent_context_proven: true|false|unknown
state: DEVICE_OFFLINE|TAILNET_ONLY|SSH_READY|UBUNTU_PERSISTENT_READY
claim_allowed: true|false
reason: bounded classifier reason
```

Never publish raw host/IP, peer list, route/latency, SSH command, username, key path/fingerprint, stderr containing connection details, environment, credentials or shell history.

## Result semantics

### PASS

Only when the accepted classifier returns `UBUNTU_PERSISTENT_READY / claim_allowed=true / reason=authorized`.

PASS proves management readiness only. It does not execute or prove #113/#131/#67/#68/R002 product behavior.

### BLOCKED

Any lower/ambiguous state is BLOCKED. The Worker reports the failed layer and STOPs.

Attempt 1 MUST NOT perform recovery mutation. A later Coordinator Contract Revision may authorize one bounded layer-specific recovery only if current evidence identifies a remotely recoverable layer under #139 boundaries.

## Hard boundaries

- exactly one bounded layered observation set;
- no retries-until-success or second set;
- no Tailscale/sshd/wake/chroot start/stop/config mutation;
- no ControlMaster requirement;
- no password prompt/auth guessing/key-enumeration output;
- no LAN scan/Tailnet peer enumeration/public listener;
- no credentials/keys/tokens/environment/profile/config dump;
- no package install/root/capability expansion/ADB/reboot;
- no Runner/workflow mutation;
- no Bilibili/#67/#68/#113/#131 execution;
- no product/media/browser/site/security-runtime change.

## Lifecycle

PASS:

```text
status:ready
-> claim
-> one layered observation set
-> classifier PASS
-> [EXECUTION REPORT]
-> status:review
-> release owner
-> STOP
```

BLOCKED:

```text
status:ready
-> claim
-> one layered observation set
-> classifier non-final state
-> [BLOCKER REPORT]
-> status:blocked
-> release owner
-> STOP
```

Before every terminal Issue mutation use the current fresh terminal-write authority guard.

## Success criteria

1. exact #139 classifier authority is used;
2. at most one observation is made per layer and deeper layers are suppressed after failure;
3. normalized durable output contains no sensitive network/auth details;
4. ControlMaster is not used as readiness authority;
5. only final classifier state returns claim authority downstream;
6. no recovery or downstream Task mutation occurs in Attempt 1.