# Session Bootstrap — Issue #68 Public / No-Login Contract Revision

```text
GitHub Issue: https://github.com/liqiangcc/jellyfin-web-media-gateway/issues/68
Task Contract: docs/tasks/68-bilibili-web-e2e/task.md
Expected worker after publication: Codex Cloud, env:cloud
Handoff profile: docs/tasks/handoffs/cloud.md
Frozen public sample: BV14V411W7r5
```

This package is a formal Contract Revision. Read the live Issue and all comments, the current `main`, `AGENTS.md`, this package's `task.md`, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, and the canonical requirements, architecture, implementation-contract, security, environment and runner documents before any work.

The revised product scope is public Bilibili playback without an account, login, Auth Mode, Cookie, token, profile or test account. The intended route is production Bilibili Site Plugin + generic Browser Worker observation → server-owned `ResolvedMedia`/`MediaShapeV1` → `PlaybackSession` → generic #271 delivery when separated A/V → same-origin Web Display → bounded controls and reconnect. Keep Bilibili semantics in `plugins/bilibili`, keep the Browser Worker generic, and preserve R008/Egress, Vault, capability, PlaybackSession and Display authority.

The frozen sample is `BV14V411W7r5`; do not silently change it. Historical #67/#166/#188/#223/#226/#229/#232 anonymous diagnostics remain negative evidence: a repeated navigation `4xx`/no-media boundary must be reported `FAIL` or `BLOCKED`, never promoted to success or bypassed.

Before publication, confirm from live GitHub that Issue #68 is still `status:draft`, `env:cloud`, owner-free and that no newer Coordinator Contract Revision supersedes this package. Do not claim an Attempt while the Publication Gate in `task.md` is unsatisfied. This revision itself authorizes no tx-node access, Bilibili live request, phone/TV/VNC/CDP action, build/test/package/install, CI rerun, #246 work, #191/#195 work, merge or close. All future build/test evidence must run on GitHub-hosted Actions; a later ordinary-Linux live route requires an explicit Coordinator-frozen host, low-privilege identity, control path, budgets and sanitized Evidence contract.

If the gate is complete, use the downstream `$task-worker` entry supplied by the Coordinator. Follow the issue lifecycle exactly: claim one bounded Attempt only after `status:ready`, reuse the exact Candidate, report `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, transition to review/blocked, release ownership and stop. Do not start an authenticated route, modify #246, or automatically begin another Task.
