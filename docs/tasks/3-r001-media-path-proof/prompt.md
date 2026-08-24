# Session Bootstrap — R001 Media Path Proof

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 R001 Task。

本文件只是 Task bootstrap/navigation，不是 Task Contract，也不保存实时 Task 状态或 Candidate 结果。

## Execution Context

```text
GitHub Issue: #3
Task Contract: docs/tasks/3-r001-media-path-proof/task.md
Expected worker: cloud-codex
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Research Item: R001
Hard publication dependencies: none
Accepted concurrency authority: Issue #2 / R007 is done and merged to main
```

## Preferred Codex Entry

```text
$task-worker Execute Issue #3 using `docs/tasks/3-r001-media-path-proof/prompt.md`.
```

Skill 不可见时按 `docs/tasks/handoffs/cloud.md` fallback 执行。

## Live Gate

Codex Worker 必须先实际读取 GitHub Issue #3、全部 relevant comments 和当前 PR/candidate 状态，以 Issue labels / owner 为实时 authority。

```text
status:draft
→ STOP，不 claim、不实现、不自行发布

status:ready + env:cloud + no active owner
→ 可以 claim，并开始新的 Attempt N

其他状态
→ 按 docs/tasks/issue-lifecycle-protocol.md 停止或交回 Coordinator
```

Worker 不得因为看到本 prompt 就自行改变发布状态。

## Recovery / Continuation Rule

本 Task 可能已有前一 Attempt 的 durable branch / PR / Actions Evidence。

开始新 Attempt 时必须：

1. 读取 Issue #3 的最新 `[COORDINATOR REVIEW]`；
2. 找到该 Review 指向的现有 candidate branch / PR / Actions run；
3. 优先继续、rebase、修复现有工作，不机械从零重写；
4. 读取当前 `main`，确认已包含 accepted R007；
5. final Candidate 必须基于/集成 current main，且不得重定义 R007 Playback authority；
6. 旧 Actions PASS 只证明旧 Candidate SHA；rebase/fix 后必须重新跑 Task Contract 要求的 J1-J4。

## Start Protocol

1. 同步最新仓库并实际使用 GitHub 读取 Issue #3、comments、current PR/candidate、Actions 状态。
2. 读取并遵守：
   - `AGENTS.md`
   - `docs/tasks/3-r001-media-path-proof/task.md`
   - `docs/tasks/issue-lifecycle-protocol.md`
   - `docs/architecture.md`
   - `docs/implementation-contracts.md`
   - `docs/technical-feasibility-validation.md`
   - `docs/mvp-plan.md`
   - `docs/security.md`
3. 确认 Issue 当前为 `status:ready + env:cloud` 且无 active owner。
4. claim → `status:in-progress` → 确定新的 `Attempt N`。
5. 严格执行 Task Contract，只做 R001 Media Path Proof / current-main integration。
6. runtime/browser/test Evidence 必须通过真实 GitHub Actions run/job/log/artifact 获得；Codex 本地测试可用于开发，但不能替代 required Actions Evidence。
7. 正常结束评论 `[EXECUTION REPORT]` → `status:review`；阻塞评论 `[BLOCKER REPORT]` → `status:blocked`。
8. 释放 active execution ownership 并停止，等待 Web Coordinator Review。

## Scope Reminder

```text
R001
→ Source / Registry / generic-direct
→ ResolvedMedia
→ scoped media capability
→ Media Gateway
→ Web Display
→ MP4 Range/play/pause/seek
→ HLS concrete result
→ Secret boundary / no open proxy
→ bounded cleanup

R007 (already accepted)
→ Playback command CAS / telemetry revision / media-refresh freshness
→ display generation / handoff authority
```

R001 不得进入 R002 TV UX、R003 target-phone resource acceptance、Jellyfin、real-site auth/plugin business logic、Native Site Panel、software video transcode 或完整 R008 security proof。

## Authority

```text
canonical docs
→ architecture / implementation contracts / feasibility / MVP / security facts

AGENTS.md
→ long-term collaboration and Codex-first routing rules

task.md
→ R001 execution contract

prompt.md
→ bootstrap only

Issue fields / labels
→ live state

Issue comments
→ Attempt / recovery / Review / Acceptance history

cloud handoff profile
→ how to start Codex Worker, not Task scope
```

## Stop Boundary

完成或阻塞一个 R001 Attempt 后立即停止。

Worker 不得自行 `status:done` 或关闭 Issue #3；Coordinator Review 可 `REVISE` 后开始下一 Attempt。