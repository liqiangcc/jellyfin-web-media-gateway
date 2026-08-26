# Task — BROKER-FD-ISOLATION-LEGACY-KERNEL-PREP

## Metadata

```text
GitHub Issue: #85
Task ID: BROKER-FD-ISOLATION-LEGACY-KERNEL-PREP
Task kind: implementation + cross-kernel security verification
Planning Base: 1cbf0a21b8811a396011f6f84ce67c1ae2e5e891
Preferred worker: cloud-codex
Eligible environment: env:cloud
Execution plane: GitHub Actions
Hosted runners: x86_64 + ARM64
Target proof: ubuntu-arm64-self-hosted / Linux 4.19 phone
Trigger: #67 Attempt 4
Downstream: #67 Attempt 5
Freshness policy: dependency-aware / exact Candidate
```

> This Task owns one portability defect only: preserve the existing fail-closed `BrokerProcessRunner` ambient-file-descriptor boundary when Linux `close_range(2)` is unavailable. It does not perform Bilibili extraction and does not redesign the runtime, sandbox, R008, artifact distribution or production enablement.

## Evidence trigger

#67 Attempt 4 reached the accepted Ubuntu ARM64 target with the previously blocked layers already working:

```text
kernel: 4.19.113-964403 / aarch64
runtime user: gateway-runner uid 999 / non-root
Exact Candidate: c23b49adbe1cad8a93ff4377dfeba3f12aac7ffe
#79 offline runtime verification: PASS
runtime_cache: offline-hit
formal public HTTPS: 2xx
formal frozen Bilibili page: 2xx
AArch64 ytdlp-sandbox build: PASS
```

The final accepted harness run then stopped before worker execution:

```text
result: FAIL
process_error: SPAWN_FAILED
broker_request_count: 0
close_range_syscall: ENOSYS
```

Static read-back shows `BrokerProcessRunner::pre_exec` calls `close_unadmitted_fds()`, and the current Linux implementation has exactly one mechanism:

```text
close_range(4, UINT_MAX, 0)
failure -> return error -> spawn fails
```

This is a generic old-kernel runtime portability blocker. It is not a Bilibili compatibility result.

## Goal

Support the accepted target kernel without weakening descriptor isolation:

```text
parent process
→ determine a safe descriptor-close upper bound before fork/pre_exec
→ spawn BrokerProcessRunner child
→ setpgid
→ dup broker capability to fd 3
→ close all non-admitted descriptors
     ├─ close_range fast path on supported Linux
     └─ only when close_range returns ENOSYS:
          bounded legacy close(2) path
→ exec ytdlp-sandbox
```

The child must still begin execution with only the intentionally admitted descriptor set:

```text
fd 0 = stdin
fd 1 = stdout
fd 2 = stderr
fd 3 = inherited per-attempt R008 broker capability
fd >= 4 = not inherited
```

## Required implementation properties

### F1 — Preserve the modern fast path

On Linux where `close_range(2)` succeeds, retain the existing atomic fast path and existing behavior.

Do not replace the normal path with a slower global strategy merely to support the legacy target.

### F2 — Fallback only for syscall absence

The legacy path may activate only when the `close_range` syscall reports `ENOSYS`.

Do **not** silently fall back for:

- `EPERM` / seccomp denial;
- `EINVAL` caused by an implementation defect;
- unexpected kernel/runtime errors;
- caller-selected mode.

Those cases remain fail-closed.

### F3 — Parent-derived bounded upper limit

Any information that is unsafe or inappropriate to discover after fork must be resolved before entering `pre_exec`.

Preferred design:

```text
parent
→ derive conservative fd upper bound from RLIMIT_NOFILE or equivalent trusted process limit
→ capture plain bounded value into pre_exec closure

child pre_exec ENOSYS fallback
→ for fd in 4..<upper_bound
     close(fd)
```

Requirements:

- no small hard-coded maximum that can leave ambient descriptors open;
- infinity/overflow/unusable limit handling must fail closed or derive a conservative safe bound;
- do not depend on caller environment/configuration to choose the bound;
- no root/sudo requirement.

The exact implementation may differ if it provides equivalent or stronger evidence.

### F4 — Pre-exec safety

The fallback runs after fork and before exec. Keep that path minimal and syscall-oriented.

Do not introduce post-fork operations that rely on allocator-heavy filesystem enumeration, arbitrary callbacks, network access, logging, environment parsing or other unsafe process-global state merely to discover open descriptors.

A `/proc/self/fd` enumeration implementation inside `pre_exec` is not the preferred solution unless the Worker can prove an equally safe low-level design. A bounded `close(2)` loop using parent-derived state is preferred because `close` is async-signal-safe and preserves the existing model.

### F5 — No bypass control surface

There must be no production env var, CLI flag, caller field, source URL input or config switch that can:

- force the fallback;
- skip fd isolation;
- select an admitted fd set;
- enlarge network authority;
- disable sandbox/no_new_privs/seccomp.

Tests may inject a private deterministic strategy/helper to force the `ENOSYS` branch, but that injection must not become a production authority surface.

## Frozen security invariants

1. `BrokerProcessRunner` remains the sole parent of the sandbox/worker path.
2. The worker receives only stdio + fd 3 broker IPC capability.
3. Ambient parent descriptors at fd >= 4 do not survive into the sandbox/worker.
4. `ytdlp-sandbox` remains mandatory.
5. #83 target-bound x86_64/AArch64 seccomp architecture gate remains unchanged.
6. New socket/socketpair creation remains denied by sandbox policy.
7. `PR_SET_NO_NEW_PRIVS` / seccomp inheritance remains intact.
8. R008 remains extractor HTTP(S) authority.
9. No caller Cookie/Auth/proxy/executable/argv/fd authority is added.
10. Production `GenericYtdlpAdapter::default()` / `DisabledRunner` behavior remains unchanged.
11. No Target root/sudo/system-package requirement is introduced.
12. Safe output / Secret boundaries remain unchanged.

## Claims

```text
C1 — Modern fast path preserved
Supported Linux still uses close_range successfully and all existing runtime tests remain green.

C2 — ENOSYS legacy path is fail-closed and complete
When close_range is unavailable, all ambient descriptors >=4 are closed while 0..3 remain usable.

C3 — Unexpected close_range errors do not downgrade
Non-ENOSYS failures still prevent child execution rather than silently selecting the legacy path.

C4 — Broker IPC survives isolation
Inherited fd 3 remains usable after both fast and legacy fd-isolation paths.

C5 — Sandbox / R008 / lifecycle authority unchanged
No direct network, sandbox bypass, Secret leak, ambient-fd leak or lifecycle regression is introduced.

C6 — Accepted Linux 4.19 ARM64 target is unblocked
On the actual accepted target where close_range probe returns ENOSYS, deterministic BrokerProcessRunner execution reaches the broker-capable worker path without SPAWN_FAILED caused by fd isolation.
```

## Verification jobs

Every required automated job must assert the exact Candidate SHA.

### J1 — Hosted x86_64 modern-path regression

Runner: GitHub-hosted Linux x86_64.

Prove:

- normal `close_range` path succeeds;
- existing generic-ytdlp runtime/network matrix remains PASS;
- fd 3 inherited broker IPC remains functional;
- worker/child socket creation denial remains PASS;
- lifecycle / timeout / cancel / descendant cleanup remains PASS;
- offline runtime and `DisabledRunner` regressions remain PASS.

No public Bilibili request.

### J2 — Hosted ARM64 modern-path regression

Runner: GitHub-hosted `ubuntu-24.04-arm`.

Run the equivalent relevant runtime/security matrix natively on AArch64, including:

- exact target architecture assertion;
- current sandbox mapping test;
- broker-backed deterministic worker path;
- fd isolation / ambient-fd negative;
- socket/socketpair denial;
- no_new_privs/seccomp state.

No public Bilibili request.

### J3 — Deterministic forced-legacy isolation proof

Use an internal test seam or helper that deterministically exercises the `ENOSYS` fallback without adding a production bypass knob.

At minimum:

1. open multiple sentinel descriptors above fd 3, including descriptors far enough apart to detect partial-range mistakes;
2. enter the same isolation logic used by `BrokerProcessRunner`;
3. prove fd 0/1/2 remain admitted as expected;
4. prove broker fd 3 remains usable end-to-end;
5. prove every sentinel fd >=4 is closed/not inherited;
6. prove a simulated non-ENOSYS `close_range` failure does **not** select fallback and fails closed;
7. prove no env/CLI/caller-selected bypass was introduced.

The test must validate actual descriptor state, not only static source strings.

### J4 — Real legacy-kernel target proof

Execution plane: GitHub Actions on the accepted `ubuntu-arm64-self-hosted` target runner / phone.

This is a deterministic runtime proof, **not a Bilibili/site request**.

Required Evidence:

```text
uname/kernel class: Linux 4.19.113-964403 / aarch64 or current accepted equivalent
runtime uid privilege: gateway-runner / non-root
close_range probe: ENOSYS (if target remains unchanged)
exact Candidate SHA
BrokerProcessRunner deterministic outcome
ambient fd isolation result
broker IPC result
sandbox initialization result
cleanup result
```

Success requires the actual target to pass beyond the #67 Attempt-4 `SPAWN_FAILED` point and reach a deterministic broker-capable worker execution while preserving fd isolation and sandbox policy.

Do not contact Bilibili in this Task. Real-site compatibility remains #67.

## Required regression commands

Exact Candidate must include at least the applicable equivalents of:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo test -p gateway-egress --all-targets
cargo test -p generic-ytdlp --features runtime-prep --test runtime -- --nocapture
cargo test -p generic-ytdlp --features runtime-prep --bin ytdlp-sandbox -- --nocapture
```

If additional focused tests are introduced for fd isolation, list and run them explicitly.

## Success criteria

1. C1-C6 are PASS with exact-Candidate Evidence.
2. Modern hosted x86_64 behavior is not weakened.
3. Native hosted AArch64 behavior remains equivalent.
4. Forced `ENOSYS` fallback proves actual ambient-fd closure and fd 3 preservation.
5. Non-ENOSYS errors still fail closed.
6. The accepted Linux 4.19 ARM64 target completes deterministic BrokerProcessRunner startup past the former `SPAWN_FAILED` point.
7. No site request, security-policy weakening, root/sudo, runtime-bundle redesign or production enablement is used.
8. Worker posts one standard `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, releases ownership and STOPs.

## Out of scope

- Bilibili or any real-site extraction;
- #67 execution;
- #68 Web E2E;
- changes to R008 policy/body limits;
- changes to ytdlp-sandbox seccomp policy except tests required to prove preservation;
- yt-dlp version/provenance/offline artifact changes;
- alternate proxy/direct-network worker path;
- Cookie/login/profile/Auth;
- DASH/separate A/V/remux/FFmpeg;
- Browser/NativePanel work;
- resource/performance/thermal/soak;
- production generic-ytdlp enablement;
- broad process-supervision refactoring.

## Freshness / integration

Semantic authority surfaces for this Task are narrow:

```text
plugins/generic-ytdlp/src/runtime.rs
plugins/generic-ytdlp runtime tests
plugins/generic-ytdlp sandbox integration only as preservation evidence
gateway-egress / R008 only as preservation evidence
#79/#83 accepted runtime/security identities
```

#67 planning/docs, BrowserWorker/R006, navigation, Auth and roadmap-only changes are normally `UNRELATED`.

If another accepted change touches `BrokerProcessRunner`, fd inheritance, sandbox execution or R008 process authority before Worker claim, Coordinator must reclassify freshness and may need to re-freeze the Task.

## Completion protocol

```text
status:ready
→ claim / Attempt 1
→ status:in-progress
→ implement + J1-J4
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ status:review or status:blocked
→ release active owner
→ STOP
```

Worker must not execute #67 Attempt 5, set `status:done`, merge its own PR, or close #85.
