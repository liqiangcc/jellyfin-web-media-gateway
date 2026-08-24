# Session Bootstrap — R001 Media Path Proof

你正在查看 `liqiangcc/jellyfin-web-media-gateway` 的 R001 Task Package。

本文件只是 Task 内部 bootstrap / navigation 入口，不是 Task Contract，也不保存实时 Task 状态。

## Execution Context

```text
GitHub Issue: #3
Task Contract: docs/tasks/3-r001-media-path-proof/task.md
Expected worker: web
Expected environment label: env:web-gpt
Downstream handoff profile: docs/tasks/handoffs/web-gpt.md
Research Item: R001
Publication dependency: R007 Issue #2 Final Acceptance
```

## Expected Client

本 Task 的计划执行客户端是 **Web ChatGPT Worker + GitHub connector**。

不要求 repo-scoped `$task-worker` Skill，也不要使用 Web 搜索替代 GitHub。

## Live Gate

新 Web Worker 会话必须先实际读取 GitHub Issue #3，并以 Issue 当前 labels / owner 为实时 authority。

```text
status:draft
→ 停止，不 claim、不实现、不自行发布

status:ready + env:web-gpt + no active owner
→ 再检查 R007 publication dependency 已由 Coordinator 在发布时确认
→ 可以 claim，并开始新的 Attempt N

其他状态
→ 按 docs/tasks/issue-lifecycle-protocol.md 停止或交回 Coordinator
```

Worker 不得因为看到本 prompt 就自行把 draft 改成 ready。

## Publication Dependency Reminder

R001 在规划阶段可以存在，但正式发布前 Coordinator 必须先确认：

- Issue #2 / R007 已 `[FINAL ACCEPTANCE]`；
- R007 接受的代码和 `docs/implementation-contracts.md` 已读回；
- Issue #3 / `task.md` 的 Publication base 已刷新；
- R001 没有绕过 R007 的 stale media/callback authority 语义；
- Publication Gate 已重新执行。

如果 Issue #3 已经是 `status:ready`，Worker 仍应从 GitHub 读取当前 Issue/body/task.md，而不是根据本文件猜测 R007 状态。

## Start Protocol after Publication

1. 必须实际使用 GitHub 读取当前仓库和 Issue，不根据聊天背景猜测状态。
2. 读取并遵守：
   - `AGENTS.md`
   - GitHub Issue #3 及 relevant comments
   - `docs/tasks/3-r001-media-path-proof/task.md`
   - `docs/tasks/issue-lifecycle-protocol.md`
   - `docs/architecture.md`
   - `docs/implementation-contracts.md`
   - `docs/technical-feasibility-validation.md`
   - `docs/mvp-plan.md`
   - `docs/security.md`
3. 确认 Issue 当前为 `status:ready + env:web-gpt` 且无 active owner，并确认当前 Web Worker 具备 Task 要求的 GitHub write/code/Actions/browser-evidence 能力。
4. claim → `status:in-progress` → 确定新的 `Attempt N`。
5. 严格执行 Task Contract，只做 R001 Media Path Proof。
6. runtime/browser/test Evidence 必须通过真实 GitHub Actions run/job/log/artifact 获得；不要把静态分析当 runtime PASS。
7. 正常结束评论 `[EXECUTION REPORT]` → `status:review`；阻塞评论 `[BLOCKER REPORT]` → `status:blocked`。
8. 释放 active execution ownership并停止，等待 Coordinator Review。

## Scope Reminder

R001 核心链路：

```text
Source input
→ SiteAdapterRegistry
→ generic-direct
→ SourceLocator / ResolvedMedia
→ scoped Media Gateway capability
→ Media Gateway
→ Web Display
```

Primary required browser proof：direct HTTP MP4。

HLS 必须形成明确 manifest/segment/result，不得保持“理论支持”。

重点证明：

```text
Range / seek semantics
Secret stays server-side
/stream is not an arbitrary open proxy
capability expiry/session/item binding
bounded streaming cleanup
Jellyfin not required
```

不要进入：

- R002 TV audible autoplay / physical UX；
- R003 target-phone CPU/RSS/temperature acceptance；
- Jellyfin；
- concrete site auth/plugin business logic；
- Native Site Panel；
- software video transcoding；
- full R008 security proof。

## Authority

```text
canonical docs
→ architecture / implementation contracts / feasibility / MVP / security facts

AGENTS.md
→ long-term collaboration rules

task.md
→ R001 unique execution contract

prompt.md
→ R001 bootstrap only

Issue fields / labels
→ live state

Issue comments
→ Attempt / Review / Acceptance history

web-gpt handoff profile
→ how to start Web Worker, not Task scope
```

## Stop Boundary

完成或阻塞一个 R001 Attempt 后立即停止。

Worker 不得自行 `status:done` 或关闭 Issue #3；Coordinator Review 可 `REVISE` 后开始下一 Attempt。