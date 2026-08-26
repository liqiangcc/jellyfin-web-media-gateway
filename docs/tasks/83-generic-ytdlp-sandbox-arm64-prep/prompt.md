# Session Bootstrap — GENERIC-YTDLP-SANDBOX-ARM64-PREP

Execute Issue #83 using the repository Worker protocol only after Coordinator Publication Gate PASS.

## Read first

1. `AGENTS.md`
2. Issue #83 and relevant comments
3. `docs/tasks/83-generic-ytdlp-sandbox-arm64-prep/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/freshness-integration-protocol.md`
6. #67 Attempt 3 `[BLOCKER REPORT]`
7. #60/#66/#73 accepted generic-ytdlp runtime/security authority
8. #79 Final Acceptance
9. `plugins/generic-ytdlp/src/bin/ytdlp-sandbox.rs`
10. `plugins/generic-ytdlp/tests/runtime.rs`
11. `.github/workflows/generic-ytdlp-prep.yml`

Claim only if live #83 is:

```text
status:ready
env:cloud
no active owner
```

## Goal

Close the exact #67 ARM64 sandbox blocker without weakening security:

```text
x86_64 native sandbox
→ exact audit-arch check
→ no_new_privs + seccomp
→ deny new socket/socketpair
→ inherited broker fd works

AArch64 native sandbox
→ exact audit-arch check
→ no_new_privs + seccomp
→ deny new socket/socketpair
→ inherited broker fd works
```

Unsupported architecture must fail closed.

## Required Evidence

Use one exact Candidate and prove:

- J1 GitHub-hosted x86_64 full sandbox/network-matrix PASS;
- J2 GitHub-hosted ARM64 (`ubuntu-24.04-arm`) equivalent sandbox/network-matrix PASS;
- worker/child AF_INET, AF_INET6 and AF_UNIX creation remain denied;
- inherited broker IPC remains usable on both architectures;
- no_new_privs + seccomp filter remain active;
- ARM64 deterministic broker-backed runtime no longer fails as `SANDBOX_UNAVAILABLE`;
- R008, lifecycle, ambient-fd isolation, offline-runtime trust tests and production `DisabledRunner` regressions PASS;
- exact Candidate SHA asserted by every required job.

## Critical boundaries

- do not disable/bypass seccomp;
- do not remove the audit-architecture gate;
- do not allow socket/socketpair on ARM64;
- do not introduce caller architecture/sandbox/proxy/syscall knobs;
- do not change R008 policy/limits or BrokerProcessRunner authority;
- no Bilibili/real-site request;
- no yt-dlp version/bundle/Auth/DASH/remux/Browser/Web E2E/performance work;
- normal completion: `[EXECUTION REPORT] -> status:review -> release owner -> STOP`;
- blocker: `[BLOCKER REPORT] -> status:blocked -> release owner -> STOP`;
- never set `status:done`, close #83, merge own PR, or start #67 Attempt 4.
