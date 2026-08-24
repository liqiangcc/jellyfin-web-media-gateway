# Session Bootstrap — R001 Media Path Proof

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 R001 Task。

本文件只是 Task 内部 bootstrap / navigation 入口，不是 Task Contract，也不保存实时 Task 状态。

## Execution Context

```text
GitHub Issue: #3
Task Contract: docs/tasks/3-r001-media-path-proof/task.md
Expected worker: web
Expected environment label: env:web-gpt
Downstream handoff profile: docs/tasks/handoffs/web-gpt.md
Research Item: R001
Hard publication dependencies: none
Parallel sibling: Issue #2 / R007
```

## Expected Client

本 Task 的下游执行客户端是 **Web ChatGPT Worker + GitHub connector**。

不要求 repo-scoped `$task-worker` Skill，也不要使用 Web 搜索替代 GitHub。

## Live Gate

新 Web Worker 会话必须先实际读取 GitHub Issue #3，并以 Issue 当前 labels / owner 为实时 authority。

```text
status:draft
→ 停止，不 claim、不实现、不自行发布

status:ready + env:web-gpt + no active owner
→ 可以 claim，并开始新的 Attempt N

其他状态
→ 按 docs/tasks/issue-lifecycle-protocol.md 停止或交回 Coordinator
```

Worker 不得因为看到本 prompt 就自行改变发布状态。

## Parallel R007 Boundary

Issue #2 / R007 可以与本 Task 同时执行。

必须保持职责分离：

```text
R001
→ Source / ResolvedMedia / Media Gateway / media capability / Web media consumption

R007
→ Playback command CAS / telemetry revision / item refresh freshness / display generation / handoff authority
```

因此：

- 不等待 R007 完成才开始 R001；
- 不在 R001 中实现/重定义 R007 的 Playback 并发状态机；
- media capability 中的 session/item identity 可以来自确定性的测试上下文；
- 如果两个 candidate 同时修改 root Cargo/workspace metadata，正常 branch/rebase/merge，不把文件冲突当业务依赖；
- 只有新的具体 R007 Evidence 真正推翻 R001 假设时，才交回 Coordinator 评审是否需要 Contract revision。

## Start Protocol

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
capability expiry/session/item/resource binding
bounded streaming cleanup
Jellyfin not required
```

不要进入：

- R007 Playback command/revision/handoff implementation；
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