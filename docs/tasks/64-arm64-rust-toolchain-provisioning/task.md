# Task — ENV-ARM64-RUST User-level Rust Toolchain Provisioning

## Metadata

```text
GitHub Issue: #64
Task ID: ENV-ARM64-RUST
Task kind: operations / environment provisioning
Parent blocker: #63 ENV-ARM64-READY Attempt 1
Planning Base: 447e58b36690cc5958323d507321efd1f54689f3
Preferred worker: ubuntu-arm64
Eligible environment: env:ubuntu-arm64
Target: Ubuntu ARM64 phone
Runtime user: gateway-runner (uid 999)
Accepted infrastructure authority: #1 INFRA-001, #21 INFRA-002
Downstream consumer: #63 Attempt 2
Repository Rust requirement: rust-version = 1.85
Handoff: docs/tasks/handoffs/ubuntu-arm64.md
```

> Live Attempt/status/Evidence belongs in Issue #64. This task owns only the stable provisioning contract.

## Goal

Make a persistent Rust toolchain available to the existing low-privilege `gateway-runner` without weakening the accepted Target Runner security boundary.

Required end state:

```text
/home/gateway-runner/.cargo
/home/gateway-runner/.rustup
→ owned by gateway-runner
→ cargo + rustc usable as gateway-runner
→ rustc satisfies repository rust-version >= 1.85
→ Runner.Listener environment can resolve /home/gateway-runner/.cargo/bin
→ no sudo/root/admin/capability added to gateway-runner
```

This is an environment operation. It does not run #63 Gateway functional verification and does not run #9 performance scenarios.

## Triggering Evidence

#63 Attempt 1 established:

- target Linux/aarch64 and low-privilege boundary PASS;
- direct public HTTPS PASS;
- frozen Bilibili sample direct/no-proxy HTTP 200;
- `gateway-runner` has bash/git/curl/python3;
- `cargo` and `rustc` are absent from the non-root PATH;
- a root-local Rust installation exists but is not accessible to `gateway-runner`;
- no package install/privilege escalation was attempted.

The correct repair is a user-owned toolchain, not exposing root-private state.

## Authority / Existing Runner Boundary

Preserve #1/#21 accepted facts:

- runner identity `ubuntu-arm64-target-phone`;
- `Runner.Listener` runs as `gateway-runner`;
- runner install/workspace under `/home/gateway-runner`;
- runtime has no sudo/wheel/admin and capability sets remain zero;
- root-owned `gateway-runner-supervise` / `gateway-runnerctl` are the accepted control plane;
- production Gateway/Vault state remains separate;
- Runner normal execution does not inherit proxy variables merely from operator shell state.

## Invariants

1. Never symlink/copy a root-private `~root/.cargo` or `~root/.rustup` tree into the runner.
2. Never grant `gateway-runner` sudo, root login, admin groups or Linux capabilities.
3. Rustup/Cargo/Rust files must be owned by `gateway-runner` and live under `/home/gateway-runner/.cargo` and `/home/gateway-runner/.rustup` unless an equivalent user-owned path is justified.
4. Operator/root may only perform the minimum control-plane actions needed to invoke the installer as `gateway-runner`, repair ownership of runner-owned home paths if objectively wrong, and update/restart the accepted root-owned supervisor so future Runner jobs inherit the toolchain PATH.
5. Do not add production Secrets, GitHub registration credentials, Vault paths, Cookie/token material, or proxy credentials to shell profiles/supervisor environment.
6. Do not install FFmpeg/Chromium/Node in this task.
7. Do not run #63 J2 or #9 performance workloads automatically.
8. Do not change product code.
9. Do not change CPU governor, kernel/Android policy, firewall or network routing.
10. If user-level Rust installation requires weakening these boundaries, BLOCK instead.

## Provisioning Procedure

### J0 — Re-read target and existing toolchain state

Record bounded non-secret output:

```text
uname -a / uname -m
id gateway-runner
/home/gateway-runner ownership/mode
command -v cargo/rustc as gateway-runner
current Runner.Listener PID/user
current Runner.Listener CapInh/Prm/Eff/Bnd/Amb
current root-owned supervisor/control paths
```

Inspect whether `/home/gateway-runner/.cargo` or `.rustup` already exists. Do not delete a valid user-owned installation just to reinstall. If partial/corrupt state exists, classify it first.

### J1 — Install Rust as gateway-runner

Use the official Rustup distribution path and execute the installer **as `gateway-runner`**, not as root.

Preferred characteristics:

- architecture: `aarch64-unknown-linux-gnu`;
- Rustup profile: `minimal`;
- default toolchain: stable;
- install roots: `CARGO_HOME=/home/gateway-runner/.cargo`, `RUSTUP_HOME=/home/gateway-runner/.rustup`;
- verify official Rustup download integrity using the official published checksum/digest mechanism before execution when available;
- do not pipe an unverified response directly into a privileged shell;
- installation network may use the target's ordinary permitted outbound path, but no site/auth Secrets are involved.

After install, as `gateway-runner`, verify:

```text
cargo --version
rustc --version
rustup --version
rustc -Vv
```

`rustc` must satisfy the repository minimum `1.85`. Do not silently install an older toolchain.

### J2 — Persistent PATH for interactive user and Target Runner

Ensure `/home/gateway-runner/.cargo/bin` is available in a deterministic non-interactive execution environment.

Do not rely only on an interactive `.bashrc` that GitHub Runner jobs may not source.

Preferred accepted solution:

- keep the Rust files user-owned;
- update the accepted root-owned Runner supervisor environment to prepend `/home/gateway-runner/.cargo/bin` to a bounded system PATH before it execs `Runner.Listener` as `gateway-runner`;
- preserve the supervisor's existing proxy stripping and capability-drop behavior;
- do not add arbitrary operator environment inheritance.

If the existing supervisor already has an approved environment hook that provides this safely, reuse it instead of creating a second control plane.

Restart through the accepted `gateway-runnerctl` mechanism only after preserving current state.

### J3 — Post-provision boundary verification

After restart, prove:

- `Runner.Listener` is again running as uid 999 `gateway-runner`;
- capability sets remain all zero as accepted in #1;
- runner workspace remains under `/home/gateway-runner/actions-runner`;
- no sudo/admin group change occurred;
- only sanitized PATH evidence is reported from the Listener environment, showing `/home/gateway-runner/.cargo/bin` is present;
- as `gateway-runner` in a clean/non-interactive environment, `command -v cargo`, `cargo --version`, `command -v rustc`, `rustc --version` succeed;
- toolchain directories/files are not root-owned in a way that prevents normal use;
- runner returns online/idle/claimable;
- no stale Runner.Worker/helper process remains after restart.

Do not print full `/proc/<pid>/environ`; extract/report only PATH or specifically approved non-secret variables.

## Claims

```text
C1 — User-owned toolchain
Rustup/Cargo/Rust are installed under gateway-runner-owned storage, not root-private state.

C2 — Required compiler availability
cargo/rustc execute successfully as gateway-runner and rustc satisfies >= 1.85.

C3 — Durable non-interactive availability
Future Target Runner jobs inherit a bounded PATH that can resolve ~/.cargo/bin without interactive shell assumptions.

C4 — Security boundary preserved
gateway-runner remains non-root/no-sudo/no-admin with zero accepted capability sets and no production Secret/Vault authority.

C5 — Runner lifecycle preserved
After the control-plane restart, ubuntu-arm64-target-phone returns online/idle/claimable with no stale worker process.
```

## Success Criteria

1. C1-C5 all PASS.
2. No root-private Rust tree is exposed to the runtime user.
3. `cargo` and `rustc` work as `gateway-runner` from a clean non-interactive environment.
4. `rustc` is at least repository `rust-version = 1.85`.
5. Runner Listener PATH includes the user Cargo bin directory through the accepted supervisor/control plane or equivalent durable mechanism.
6. Runner runtime privilege/capability/workspace boundaries remain unchanged.
7. No FFmpeg/Chromium/Node/product/performance scope is added.
8. Worker posts `[EXECUTION REPORT]`, transitions to `status:review`, releases owner and STOPs; if blocked, posts `[BLOCKER REPORT]`, sets `status:blocked`, releases owner and STOPs.

## Evidence Contract

Report:

```text
Attempt:
Worker/environment:
Target identity:
Pre-install cargo/rustc state:
Installation method/source/integrity verification:
CARGO_HOME/RUSTUP_HOME:
Ownership/modes summary:
cargo version:
rustc version / rustc -Vv:
rustup version:
Repository rust-version comparison:
Supervisor PATH change or reused hook:
Runner restart result:
Runner.Listener PID/user/workspace:
Runner.Listener PATH sanitized result:
Capability sets after restart:
Admin/sudo/group result:
Online/idle/claimable result:
Stale worker/helper cleanup result:
Claims C1-C5:
Secret/sensitive-data scan:
Downstream: #63 can/cannot be unblocked
```

Do not include registration tokens, PATs, credential file contents, proxy credentials, Cookies, Authorization, or unrelated environment values.

## Freshness

This is target environment provisioning. Product `main` movement does not invalidate a successfully provisioned toolchain unless repository Rust requirements change beyond the installed compiler or accepted Runner security authority changes.

## Completion Protocol

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ J0-J3
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Worker must not execute #63 Attempt 2 automatically, close #64, set `status:done`, or start #36/#23/#9.