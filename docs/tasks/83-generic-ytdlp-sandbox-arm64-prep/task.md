# Task — GENERIC-YTDLP-SANDBOX-ARM64-PREP

## Metadata

```text
GitHub Issue: #83
Task ID: GENERIC-YTDLP-SANDBOX-ARM64-PREP
Task kind: implementation + cross-architecture security verification
Planning Base: 14b001018e499a19ba8e863710438d028dbb6485
Preferred worker: cloud-codex
Eligible environment: env:cloud
Accepted runtime/security authority: #60 / #66 / #73 / R008
Accepted offline runtime authority: #79 Final Accepted
Trigger Evidence: #67 Attempt 3 SANDBOX_UNAVAILABLE on Linux aarch64
Downstream: #67 Attempt 4
Freshness policy: dependency-aware
```

> This Task closes one generic runtime portability gap only: the accepted seccomp sandbox currently fail-closes on AArch64 because its audit-architecture gate is x86_64-only. Preserve the sandbox security model; do not weaken it to make ARM64 pass.

## Trigger Evidence

#67 Attempt 3 proved all pre-sandbox prerequisites:

```text
exact Candidate: 290268c3cabe5ac16022b1ae5e4fa7716ee5deae
accepted offline bundle transfer: PASS
repository trust-anchor / wheel SHA: PASS
runtime_cache: offline-prepared
direct public HTTPS: HTTP 200
direct frozen Bilibili page: HTTP 200
process_error: SANDBOX_UNAVAILABLE
broker_request_count: 0
```

Static read-back at current main shows `plugins/generic-ytdlp/src/bin/ytdlp-sandbox.rs` contains an x86_64-only audit architecture gate:

```text
AUDIT_ARCH_X86_64
...
arch == x86_64 ? continue : KILL_PROCESS
```

The accepted target is Linux `aarch64`; therefore the current fail-closed behavior is expected but not portable enough for the accepted target.

## Goal

Make the existing `ytdlp-sandbox` security model work equivalently on Linux x86_64 and Linux AArch64:

```text
supported Linux architecture
→ validate seccomp_data.arch against exact architecture audit value
→ PR_SET_NO_NEW_PRIVS
→ SECCOMP_MODE_FILTER
→ deny creation of socket/socketpair with EPERM
→ inherited broker fd remains usable
→ Python/descendants inherit filter
```

Unknown/unsupported architecture must remain fail-closed. There is no unsandboxed fallback.

## Security invariants

1. `ytdlp-sandbox` remains mandatory for `BrokerProcessRunner` verification runtime.
2. Architecture validation happens before syscall-number policy is trusted.
3. The architecture check must use the exact Linux audit-architecture value for the compile/runtime target; do not accept multiple architectures in one running binary merely for convenience.
4. New `socket` and `socketpair` creation remains denied after filter install.
5. The already-created inherited broker IPC fd remains usable.
6. Python worker and descendants inherit `no_new_privs` + seccomp filter.
7. No direct HTTP/TCP/Unix-socket bypass is introduced.
8. No caller-selected sandbox mode, architecture, executable, syscall allowlist, proxy or bypass flag is introduced.
9. R008 remains network-policy authority above the inherited broker capability.
10. Production `GenericYtdlpAdapter::default()` remains `DisabledRunner`.

## Implementation requirements

### A. Architecture mapping

Replace the x86_64-only gate with compile-time-supported Linux architecture mapping.

Minimum supported targets:

```text
x86_64
 aarch64
```

Worker must derive/verify the AArch64 Linux audit architecture value from authoritative Linux/libc headers or equivalent build-time authority; do not guess silently.

Preferred shape:

```text
#[cfg(target_arch = "x86_64")]
const CURRENT_AUDIT_ARCH = ...;

#[cfg(target_arch = "aarch64")]
const CURRENT_AUDIT_ARCH = ...;
```

For unsupported target architectures, fail compilation or fail closed before executing the worker. Do not choose an x86_64 default.

### B. Syscall-number correctness

The seccomp filter compares syscall numbers after the architecture gate. Confirm `libc::SYS_socket` and `libc::SYS_socketpair` are correct for each supported target at compile/runtime Evidence.

Do not hard-code x86_64 syscall numbers.

### C. No security relaxation

Forbidden fixes include:

- removing the architecture check;
- accepting both audit architectures in one running binary without target binding;
- removing seccomp;
- allowing `socket`/`socketpair` on ARM64;
- sandbox bypass flags;
- running the worker outside `ytdlp-sandbox`;
- moving network access into Python/worker code;
- broadening R008 limits/policy.

### D. Tests

Existing runtime matrix must remain authoritative:

```text
python_af_inet_denied
python_af_inet6_denied
custom_handler_denied
custom_unix_handler_denied
python_af_unix_denied
child_af_inet_denied
child_af_unix_denied
broker_ipc_usable
no_new_privs
seccomp_filter
```

Add focused architecture evidence so a future regression to x86_64-only cannot pass generic hosted x86 CI unnoticed.

## Claims

```text
S1 — Exact architecture gate
x86_64 and AArch64 binaries validate their exact Linux audit architecture before syscall policy execution; unsupported targets fail closed.

S2 — Socket creation remains denied
On both x86_64 and ARM64, worker and child attempts to create AF_INET/AF_INET6/AF_UNIX sockets remain denied.

S3 — Broker IPC remains usable
The inherited pre-created broker fd works on both architectures while new socket/socketpair creation is denied.

S4 — Seccomp/no-new-privs inheritance
Python worker and descendants observe the filter/no_new_privs state on both architectures.

S5 — Runtime authority unchanged
BrokerProcessRunner/R008/Secret/lifecycle/DisabledRunner authority is unchanged except architecture portability glue.

S6 — #67 blocker specifically closed
An ARM64 deterministic runtime smoke reaches the broker-capable worker path without `SANDBOX_UNAVAILABLE`; no real Bilibili request is required in this Task.
```

## Verification matrix

### J1 — x86_64 sandbox regression

Runner: GitHub-hosted `ubuntu-latest`.

On exact Candidate:

- assert x86_64 build/runtime architecture;
- build `ytdlp-sandbox` and runtime-prep tests;
- run the existing network-matrix/seccomp tests;
- prove new socket/socketpair denied and broker IPC usable;
- prove no_new_privs + seccomp filter active;
- run lifecycle/ambient-fd regressions.

### J2 — ARM64 sandbox functional/security proof

Runner: GitHub-hosted `ubuntu-24.04-arm`.

On the **same exact Candidate**:

- assert `uname -m`/Rust target is AArch64;
- build the native `ytdlp-sandbox`;
- run the same network-matrix/seccomp tests as J1, not a weaker ARM64-only smoke;
- prove worker/child AF_INET, AF_INET6 and AF_UNIX creation denial;
- prove inherited broker IPC is usable;
- prove no_new_privs/seccomp filter active;
- prove no `SANDBOX_UNAVAILABLE` for the deterministic broker-backed runtime path.

No public site request is allowed.

### J3 — architecture fail-closed/static guards

Prove:

- source has explicit x86_64 + AArch64 architecture mapping;
- no generic x86_64 fallback for unsupported arch;
- syscall numbers remain target libc-derived;
- no sandbox bypass/environment knob;
- current security architecture guards pass.

A compile-only unsupported-target check may be used if practical; otherwise static/source guard must make unsupported default impossible.

### J4 — full affected regressions

Exact Candidate:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo test -p gateway-egress --all-targets
cargo test -p generic-ytdlp --features runtime-prep --test runtime -- --nocapture
```

Also preserve:

- offline runtime/trust-anchor tests from #79;
- R008 security regressions;
- process/descendant cleanup;
- ambient fd isolation;
- `DisabledRunner` assertion;
- Core/site boundary guard.

All required jobs must assert the exact Candidate SHA.

## Success criteria

1. S1-S6 PASS on one exact Candidate.
2. Hosted x86_64 and hosted ARM64 both execute equivalent seccomp network-matrix Evidence.
3. ARM64 deterministic runtime reaches the inherited broker-capable worker path without sandbox initialization failure.
4. No direct worker socket authority is introduced.
5. No R008/security/Secret/lifecycle/production-default boundary is weakened.
6. No real Bilibili/site request occurs.
7. Worker reports and STOPs; it does not execute #67 Attempt 4.

## Expected files

Primarily:

```text
plugins/generic-ytdlp/src/bin/ytdlp-sandbox.rs
plugins/generic-ytdlp/tests/runtime.rs
.github/workflows/generic-ytdlp-prep.yml
```

A narrowly scoped new test/helper/workflow is allowed if it gives clearer cross-architecture Evidence. Do not modify unrelated product/site/UI surfaces.

## Out of scope

- Bilibili/site extraction;
- #67 execution;
- offline wheel/version/trust-lock redesign;
- R008 policy/limit changes;
- Cookie/login/profile/auth;
- DASH/remux/FFmpeg;
- Browser/Native Panel;
- Web E2E;
- phone performance/thermal/soak;
- production generic-ytdlp enablement.

## Freshness

Semantic authorities:

```text
plugins/generic-ytdlp/src/bin/ytdlp-sandbox.rs
plugins/generic-ytdlp/src/lib.rs BrokerProcessRunner integration
gateway-egress/** / R008
plugins/generic-ytdlp/tests/runtime.rs
```

Planning Base: `14b001018e499a19ba8e863710438d028dbb6485`.

Before acceptance, Coordinator compares Candidate/current main and classifies `NONE | UNRELATED | INTEGRATION_OVERLAP | SEMANTIC_AUTHORITY | CONTRACT_INVALIDATING`.

## Completion protocol

```text
status:ready
→ Worker claim / Attempt 1
→ status:in-progress
→ J1-J4
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ status:review or status:blocked
→ release owner
→ STOP
```

Worker cannot set `status:done`, close #83, merge its own PR or auto-start #67.
