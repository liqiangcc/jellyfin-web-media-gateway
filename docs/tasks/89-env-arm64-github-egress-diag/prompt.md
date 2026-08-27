# Worker Bootstrap — Issue #89 ENV-ARM64-GITHUB-EGRESS-DIAG

Execute exactly one Worker Attempt for Issue #89 in `liqiangcc/jellyfin-web-media-gateway`.

## Required read order

Before claiming, read through GitHub:

1. Issue #89 and all relevant comments.
2. `AGENTS.md`.
3. `docs/tasks/89-env-arm64-github-egress-diag/task.md`.
4. `docs/tasks/issue-lifecycle-protocol.md`.
5. `docs/tasks/handoffs/ubuntu-arm64.md`.
6. Relevant accepted environment authority referenced by `task.md`, especially #63 / #87.
7. Issue #85 latest comments and J4 run/job `33026189606 / 98367958569`.

Authority order remains:

```text
canonical docs > AGENTS.md > task.md > prompt.md
```

## Claim gate

Proceed only if live Issue #89 is:

```text
status:ready
env:ubuntu-arm64
no active owner
```

Then claim according to the repository Worker protocol and begin the next Attempt number.

## Execution focus

This is an ARM target environment diagnostic Task, not product development.

Follow the current `task.md` exactly. The required diagnostic shape is:

```text
current proxy/config state
→ Direct / Proxy A-B
→ IPv4 / IPv6 classification
→ curl / gh / git repeated comparison
→ minimal evidence-driven user-level repair if needed
→ exact #85 Candidate fresh fetch 3x
→ CHECKOUT_RESUME_ELIGIBLE_FOR_#85
```

Important boundaries:

- do not assume proxy-on or proxy-off is correct;
- sanitize proxy values and never print credentials/tokens;
- do not use root/sudo/system package installation;
- do not weaken TLS verification;
- do not modify #85 fd-isolation/product/security code or PR #86 semantics;
- do not execute #85 J4;
- do not execute #67;
- do not invent/rotate proxies;
- preserve rollback information for any user-level config change.

The exact parent Candidate used by the fetch proof is:

```text
4af64b124af4d1599a87bd211395ee832e9d7e4b
```

## End of Attempt

Normal completion:

```text
post standard [EXECUTION REPORT] to Issue #89
→ status:review
→ release active execution ownership
→ STOP
```

If blocked:

```text
post standard [BLOCKER REPORT] to Issue #89
→ status:blocked
→ release active execution ownership
→ STOP
```

Worker must not set `status:done`, close Issue #89, merge anything, or automatically start another Task/Attempt.
