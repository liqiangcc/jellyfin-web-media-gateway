# Task — GENERIC-YTDLP-SANDBOX-AVAILABILITY-PREP

## Metadata

```text
GitHub Issue: #99
Task ID: GENERIC-YTDLP-SANDBOX-AVAILABILITY-PREP
Task kind: implementation + cross-architecture verification
Planning Base: e48d6041dc6b0a307b2cca8fcc29493558ac15bf
Preferred worker: cloud-codex
Eligible environment: env:cloud
Trigger Evidence: #67 R7 / Attempt 7 SANDBOX_UNAVAILABLE on exact Candidate d9c038547ed2df695571f8dd4f732bdcdd4d5c19
Accepted authorities: #79 / #83 / #85 / #95 / #97 / R008
Downstream: #67 next Contract Revision / Attempt
Freshness policy: dependency-aware
```

> This Task closes one clean-build/runtime wiring defect only. It must make the
> repository-owned real smoke bind the exact-Candidate `ytdlp-sandbox` binary
> deterministically. It must not weaken or replace the accepted sandbox.

## Trigger Evidence

#67 Attempt 7 proved:

```text
exact Candidate: d9c038547ed2df695571f8dd4f732bdcdd4d5c19
target: Linux 4.19.113-964403 / aarch64 / uid 999 / no capabilities
runtime_cache: offline-hit
direct/no-proxy public HTTPS: 2xx
direct/no-proxy frozen Bilibili page: 2xx
process_error: SANDBOX_UNAVAILABLE
broker_request_count: 0
cleanup: PASS
```

Static read-back shows:

```text
scripts/generic-ytdlp-real-smoke.sh
→ cargo run ... --bin generic-ytdlp-real-smoke

generic-ytdlp-real-smoke::sandbox_path()
→ current_exe().parent().join("ytdlp-sandbox")
→ require sibling file
```

From a clean target directory, building only the smoke binary does not prove
that the sibling sandbox binary exists. This can report
`SANDBOX_UNAVAILABLE` before the accepted #83 sandbox is attempted.

## Goal

Make the real-site smoke path deterministic from a clean exact-Candidate build:

```text
clean target directory
→ build exact-Candidate real-smoke + ytdlp-sandbox artifacts
→ bind only the repository-built sibling ytdlp-sandbox
→ install seccomp/no_new_privs
→ inherited broker fd remains usable
→ new socket/socketpair creation remains denied
→ broker-capable worker path reached
```

No real site request is required or allowed in this Task. The downstream #67
verification remains authority for the frozen Bilibili sample.

## Security invariants

1. `ytdlp-sandbox` remains mandatory; no unsandboxed fallback exists.
2. The smoke caller cannot select an arbitrary sandbox executable or path.
3. The sandbox artifact is built from the same exact Candidate as the smoke
   binary.
4. #83 exact architecture gate, seccomp filter and socket/socketpair denial are
   unchanged unless a focused test proves a genuine defect.
5. #85 fd isolation, #95 Secret containment, #97 framing and R008 egress
   authority remain unchanged.
6. Production `GenericYtdlpAdapter::default()` remains `DisabledRunner`.
7. No site URL, Cookie, Authorization, profile, proxy or bypass is introduced.

## Implementation requirements

### A. Clean-build artifact closure

The repository-owned smoke script or an equivalently narrow build helper must
build both required binaries from the same checkout before execution:

```text
generic-ytdlp-real-smoke
ytdlp-sandbox
```

The implementation must not depend on a stale binary left by a previous test,
workflow or developer build.

### B. Exact sandbox binding

The smoke binary must continue to bind only the expected repository-built
sandbox artifact. Do not add a caller-controlled environment variable, CLI
argument or search through arbitrary `PATH` entries to select the sandbox.

### C. Bounded failure semantics

Missing, non-file or non-executable sandbox artifacts must remain a bounded
`SANDBOX_UNAVAILABLE` result before broker/site traffic. A clean supported build
must no longer hit that result.

### D. Regression coverage

Add a deterministic regression starting from an isolated/clean target
directory. It must fail if only the real-smoke binary is available and pass
only after the exact sibling sandbox artifact is built and bound.

Preserve the full #83 matrix:

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

## Claims

```text
A1 — Clean artifact closure
A clean exact-Candidate checkout deterministically produces both required
binaries without relying on prior target state.

A2 — Exact sandbox binding
The real smoke binds the same-Candidate sibling sandbox and exposes no caller
selection or unsandboxed fallback.

A3 — Sandbox security unchanged
Equivalent x86_64 and AArch64 socket-denial, broker-IPC, no_new_privs and
seccomp evidence remains PASS.

A4 — Bounded negative behavior
Missing/invalid sandbox artifacts still fail before broker traffic with stable
SANDBOX_UNAVAILABLE semantics.

A5 — Authority preservation
#79/#83/#85/#95/#97/R008 and production DisabledRunner boundaries remain
unchanged.

A6 — #67 blocker prepared for re-verification
A clean ARM64 deterministic runtime reaches the broker-capable worker path
without SANDBOX_UNAVAILABLE; no real site request occurs in this Task.
```

## Verification matrix

### J1 — hosted x86_64 clean-build regression

- assert exact Candidate checkout;
- remove/use an isolated Cargo target directory;
- prove the smoke build produces and binds both exact artifacts;
- run the complete sandbox/runtime security matrix;
- run focused missing-sandbox negative coverage.

### J2 — hosted native ARM64 equivalent proof

Runner: `ubuntu-24.04-arm`.

- assert native AArch64 identity and exact Candidate;
- repeat the same isolated clean-build and security matrix as J1;
- prove deterministic broker-backed runtime does not return
  `SANDBOX_UNAVAILABLE`;
- no public site request.

### J3 — security/static guards

- no caller-selected sandbox path or bypass knob;
- no direct worker egress;
- no weakening of architecture/seccomp/socket policy;
- production DisabledRunner unchanged;
- real-smoke workflow/script cannot silently depend on a stale sibling binary.

### J4 — full affected regressions

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo test -p gateway-egress --all-targets
cargo test -p generic-ytdlp --features runtime-prep --test runtime -- --nocapture
```

Preserve offline runtime/trust, R008, lifecycle, ambient-fd and cleanup tests.

## Success criteria

1. A1–A6 PASS on one exact Candidate.
2. J1–J4 PASS; hosted ARM64 executes the same security matrix as x86_64.
3. A clean ARM64 build reaches broker-capable deterministic runtime without a
   real site request and without `SANDBOX_UNAVAILABLE`.
4. No accepted security or production-default boundary is weakened.
5. Worker reports, releases ownership and stops; it does not execute #67.

## Expected files

Primarily:

```text
scripts/generic-ytdlp-real-smoke.sh
plugins/generic-ytdlp/src/bin/generic-ytdlp-real-smoke.rs
plugins/generic-ytdlp/tests/runtime.rs
.github/workflows/generic-ytdlp-prep.yml
```

Only the smallest subset needed by the accepted fix should change.

## Out of scope

- real Bilibili/site extraction;
- #67 execution or contract revision;
- sandbox bypass or sandbox redesign;
- R008/Secret/HTTP-limit changes;
- yt-dlp version/bundle/trust redesign;
- Cookie/login/profile/proxy;
- DASH/remux/FFmpeg/Browser/Web E2E;
- production generic-ytdlp enablement.

## Completion protocol

```text
status:ready
→ Worker claim / Attempt 1
→ status:in-progress
→ implementation + J1-J4
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ status:review or status:blocked
→ release owner
→ STOP
```

Coordinator alone reviews, merges, closes and republishes #67.
