# Session Bootstrap — BROKER-FD-ISOLATION-LEGACY-KERNEL-PREP

Execute GitHub Issue #85 using the repository Worker protocol.

## Read first

1. `AGENTS.md`
2. Issue #85 and all relevant comments
3. `docs/tasks/85-broker-fd-isolation-legacy-kernel-prep/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/freshness-integration-protocol.md`
6. #67 Attempt 4 `[BLOCKER REPORT]`
7. #83 Final Acceptance / PR #84
8. #79 Final Acceptance / PR #82
9. #60 / accepted R008 runtime-security authority
10. `docs/product-roadmap.md`
11. `docs/planning-priority.md`

Claim only if live Issue #85 is:

```text
status:ready
env:cloud
no active owner
```

Use `task.md` as the execution contract. Do not infer requirements from this bootstrap when the contract is more specific.

Critical focus:

```text
#67 Attempt 4
→ Linux 4.19 close_range == ENOSYS
→ BrokerProcessRunner SPAWN_FAILED

#85
→ preserve modern close_range fast path
→ ENOSYS-only fail-closed legacy fd-isolation fallback
→ preserve only fd 0..3
→ hosted x86_64 + ARM64 proof
→ deterministic forced-legacy proof
→ real Linux 4.19 target proof
```

Do not contact Bilibili or start #67. Do not add a production bypass switch, weaken sandbox/R008, require root/sudo, redesign the yt-dlp artifact, or broaden scope.

Normal completion:

```text
[EXECUTION REPORT]
→ status:review
→ release active ownership
→ STOP
```

Blocker:

```text
[BLOCKER REPORT]
→ status:blocked
→ release active ownership
→ STOP
```

Worker must not set `status:done`, close #85, merge its own PR, or execute #67 Attempt 5.
