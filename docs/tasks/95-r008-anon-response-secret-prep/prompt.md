# Session Bootstrap — R008-ANON-RESPONSE-SECRET-PREP

Execute Issue #95 using the repository Worker protocol.

## Read first

1. `AGENTS.md`
2. Issue #95 and all relevant comments
3. `docs/tasks/95-r008-anon-response-secret-prep/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/freshness-integration-protocol.md`
6. `docs/adr/0007-r008-anonymous-response-secret-containment.md`
7. #67 Attempt 5 authoritative corrected `[BLOCKER REPORT]`
8. #14 / R008 Final Acceptance and `docs/research/r008-security-boundary.md`
9. #39 Secret classifier/conformance authority
10. #50 `docs/research/generic-ytdlp-egress-research.md`
11. #60 Final Acceptance / brokered worker runtime
12. #66 Final Acceptance / extraction path
13. `docs/security.md`

Claim only if live #95 is:

```text
status:ready
env:cloud
no active owner
```

## Frozen goal

Implement ADR 0007 on one exact Candidate:

```text
request Secret material
→ REJECT before prohibited side effects

origin response
→ R008 accepted network authority
→ Secret response headers remain Secret
→ CONTAIN before broker IPC
→ no cookie/auth store or replay
→ safe status/body/non-Secret headers continue when valid
```

## Critical prohibitions

- Do not remove `Set-Cookie` or other accepted Secret classes from the Secret classifier.
- Do not weaken request Cookie/Auth/token/Basic/Bearer rejection.
- Do not add a cookie jar or response credential replay.
- Do not change R008 SSRF/DNS/public-IP/pinning/TLS/redirect authority.
- Do not add CONNECT/open proxy/raw tunnel behavior.
- Do not expand body/frame/time/cancel limits to make tests pass.
- Do not run Bilibili or another real public-site compatibility test in this Task.
- Do not modify #67 or start its next Attempt.
- Do not enable production generic-ytdlp; `GenericYtdlpAdapter::default()` remains disabled.

## Required verification

Run the exact-Candidate J1-J3 matrix from `task.md` and provide durable run/job IDs.

At minimum prove:

```text
request Secret reject                   PASS
Set-Cookie response containment         PASS
Auth/challenge response containment     PASS
Bearer/Basic-valued response containment PASS
no Secret value crosses broker IPC      PASS
no cookie/auth state replay              PASS
safe public response continuity          PASS
redirect/R008 regression                 PASS
bounds/malformed regression              PASS
Secret-sentinel leak scan                PASS
#39/#60/#66/R008/workspace regressions   PASS
Default DisabledRunner                   PASS
docs/security.md sync                    PASS
```

Use deterministic synthetic fixtures only for response-secret policy. The real #67 report must not be expanded to reveal the real origin Secret header.

## Lifecycle

Normal completion:

```text
claim
→ status:in-progress
→ implementation + exact-Candidate J1-J3
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

If blocked:

```text
[BLOCKER REPORT]
→ status:blocked
→ release owner
→ STOP
```

Worker must not merge, set `status:done`, close #95, republish #67 or execute #67/#68.
