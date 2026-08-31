# INFRA-PHONE-MGMT-READINESS-REFRESH

## Identity

- Issue: #144
- Kind: verification-only / management-plane readiness
- Preferred Worker: cloud
- Eligible environment: `env:cloud`
- Accepted readiness authority: #139 Final Acceptance / merge `feaafaae6ac81d01ea8d0a1a1582bec31074d418`
- Downstream consumers: #113 R3 Attempt 4 and #131 Attempt 6
- Frozen target: accepted physical phone management endpoint
- Repository mutation: none expected

## Goal

Execute one bounded layered management-readiness observation set and classify the normalized non-secret snapshot through accepted `scripts/phone_mgmt_readiness.py`.

Canonical states:

```text
DEVICE_OFFLINE
  -> TAILNET_ONLY
  -> SSH_READY
  -> UBUNTU_PERSISTENT_READY
```

Only `UBUNTU_PERSISTENT_READY / claim_allowed=true / reason=authorized` returns downstream authority.

## Exact bounded observation contract

Run exactly one layered set in this order. Failure/ambiguity at an earlier layer suppresses all deeper layers.

1. **Tailnet**
   - one bounded reachability check against the accepted phone only;
   - suppress raw target/route/latency/peer output;
   - if false/unknown: stop deeper probes.
2. **SSH TCP/auth**
   - only after Tailnet true;
   - one bounded accepted SSH endpoint check plus one non-interactive authentication attempt;
   - `BatchMode=yes`, one connection attempt, bounded timeout, no password prompt, no auth guessing, no ControlMaster requirement;
   - suppress raw stderr, host/IP/user/key/fingerprint output.
3. **Ubuntu context**
   - only after authenticated SSH true;
   - harmless architecture/context readiness checks only;
   - no Runner/product/device mutation.
4. **Persistent context**
   - only after Ubuntu context true;
   - prove one management context whose lifetime is independent of the one-shot SSH command;
   - a successful one-shot Ubuntu command is insufficient;
   - ControlMaster/tmux metadata may support diagnosis but cannot itself authorize readiness.
5. Normalize only these observations:
   - `tailnet_reachable`
   - `ssh_tcp_reachable`
   - `ssh_authenticated`
   - `ubuntu_context_reachable`
   - `persistent_context_proven`
6. Run accepted classifier offline and report only its bounded state/reason/claim flag.

## Result semantics

### PASS

Requires exactly:

```text
state=UBUNTU_PERSISTENT_READY
claim_allowed=true
reason=authorized
PHONE_MGMT_READY_FOR_TARGET_TASK=yes
```

PASS returns authority to Coordinator only. Worker MUST NOT claim or execute #113/#131 itself.

### BLOCKED

Any lower/ambiguous/contradictory state returns:

```text
PHONE_MGMT_READY_FOR_TARGET_TASK=no
```

Report the normalized booleans/classes and classifier state/reason, then STOP. Do not retry-until-success and do not consume #113/#131 Attempts.

## Hard boundaries

- verification-only; no phone/Tailscale/sshd/wake/chroot configuration or restart;
- no package install/root/ADB/reboot;
- no Runner/workflow/product/browser/Bilibili mutation;
- no LAN scan or Tailnet peer enumeration;
- no key/password/token/config/profile/environment/credential output;
- no `/proc/*/environ` or credential dump;
- no ControlMaster requirement or creation loop;
- no retries, sampling-until-success, adaptive auth/request variation;
- no #113/#131 claim/execution by Worker;
- terminal Issue mutations use current fresh terminal-write authority guard.

## Evidence

Durable report may include only:

```text
Attempt / Worker / Environment / UTC
Tailnet reachable: yes|no|unknown
SSH TCP reachable: yes|no|unknown
SSH authenticated: yes|no|unknown
Ubuntu context reachable: yes|no|unknown
Persistent context proven: yes|no|unknown
Classifier state
Classifier reason
claim_allowed=true|false
PHONE_MGMT_READY_FOR_TARGET_TASK=yes|no
Overall: PASS|BLOCKED
```

Never publish target IP/hostname/user, key paths/fingerprints, raw SSH/Tailscale output, route/latency, peer metadata, command history, credentials or environment secrets.

## Lifecycle

PASS:

`status:ready -> claim -> bounded layered observation -> [EXECUTION REPORT] -> status:review -> release owner -> STOP`

BLOCKED:

`status:ready -> claim -> bounded layered observation -> [BLOCKER REPORT] -> status:blocked -> release owner -> STOP`

Worker must not merge/close/done, create a recovery Task, or execute downstream Tasks.