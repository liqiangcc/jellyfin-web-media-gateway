# Session Bootstrap — Issue #68 Public / No-Login Contract Revision

```text
GitHub Issue: https://github.com/liqiangcc/jellyfin-web-media-gateway/issues/68
Task Contract: docs/tasks/68-bilibili-web-e2e/task.md
Expected worker after publication: Codex Cloud, env:cloud
Handoff profile: docs/tasks/handoffs/cloud.md
Frozen public sample: BV14V411W7r5
Frozen Execution Base: set by the Publication Gate; final Candidate is determined during the bounded combined Attempt
```

This package is a formal Contract Revision. Read the live Issue and all comments, the current `main`, `AGENTS.md`, this package's `task.md`, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, and the canonical requirements, architecture, implementation-contract, security, environment and runner documents before any work.

The current contract revision adds the generic `BrowserAcquisitionTarget` seam. The
owning Site Plugin derives this bounded server-owned target from the opaque
`SourceLocator`; it is separate from collection `SiteAdapter::navigation()` and may
not be supplied or overridden by HTTP, Control or Display. Core validates and binds
the target, then applies R008/Egress for the initial target and every redirect/request;
Core and the generic Browser Worker do not parse Bilibili identifiers, part numbers,
site paths or private APIs. Generic/direct adapters may return unsupported/none and
direct resolution must continue without starting a Browser Worker. Observation and
server-owned handoff remain one-shot, operation/session-bound and locator-bound, and
public resolution keeps `authenticated_session=None`.

The revised product scope extends the accepted Issue #154 ordinary-Linux Control → Gateway `PlaybackSession` → Web Display authority (Final Acceptance/merge `836e220e6ba4e38377a4e40cff677c9549aa7798`). Preserve its same-origin media, play/pause/seek/stop, refresh/reconnect, stale/error/concurrency/security and Candidate-bound artifact-consumer behavior. Add only the public source ingress: production Bilibili Site Plugin + generic Browser Worker observation → server-owned `ResolvedMedia`/`MediaShapeV1` → generic #271 delivery when separated A/V → that existing Web Display authority. Keep Bilibili semantics in `plugins/bilibili`, keep the Browser Worker generic, and preserve R008/Egress, Vault, capability, PlaybackSession and Display authority.

#237/#240 and #268/#271 are the direct source/media authorities. #243/#248/#251/#257/#259 are regression surfaces on current `main`, not public-route dependencies; do not invoke Auth Mode, account registration, Vault candidate capture or authenticated routes. #255 may be used only for the smallest public/no-account composition-root adjustment if actually required.

The frozen sample is `BV14V411W7r5`; do not silently change it. Historical #67/#166/#188/#223/#226/#229/#232 anonymous diagnostics remain negative evidence: a repeated navigation `4xx`/no-media boundary must be reported `FAIL` or `BLOCKED`, never promoted to success or bypassed.

Before publication, confirm from live GitHub that Issue #68 is still `status:draft`, `env:cloud`, owner-free and that no newer Coordinator Contract Revision supersedes this package. Do not claim an Attempt while the Publication Gate in `task.md` is unsatisfied. The Gate freezes an exact Execution Base and freshness classification; it does not require a final Worker Candidate or PR. After `status:ready`, start one bounded combined Attempt from that frozen base. If implementation is needed, create one focused Candidate/PR; otherwise the final Candidate may equal the Execution Base. Bind all hosted J1/J2/J4, Candidate-bound artifact/package admission and real public J3 verification to that same final Candidate before reporting. This revision itself authorizes no tx-node access, Bilibili live request, phone/TV/VNC/CDP action, build/test/package/install, CI rerun, #246 work, #191/#195 work, merge or close. All future build/test evidence must run on GitHub-hosted Actions; a later ordinary-Linux live route requires an exact final-Candidate-bound artifact/package, manifest/digest/platform/runtime admission and fresh-consumer compile-free start, plus an explicit Coordinator-frozen host, low-privilege identity, control path, budgets and sanitized Evidence contract. The target must never compile/build/install to compensate for missing assets.

If the gate is complete, use the downstream `$task-worker` entry supplied by the Coordinator. Follow the issue lifecycle exactly: claim one bounded combined Attempt only after `status:ready`, start from the frozen Execution Base, create a focused Candidate/PR only if implementation is needed, bind every required verification to the final Candidate, report `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, transition to review/blocked, release ownership and stop. Do not start an authenticated route, modify #246, or automatically begin another Task.

Attempt 1 is blocked by the missing acquisition-target contract and remains historical.
Do not start Attempt 2 from this bootstrap until this docs revision is merged and
read back, the Publication Gate is rerun, and the same PR #278/branch is explicitly
resumed. This revision authorizes no live traffic, CI rerun, code change or workflow
change by itself.
