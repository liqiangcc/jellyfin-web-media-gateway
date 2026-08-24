# Session Bootstrap — R007 Playback Concurrency Contract Closure

你正在查看 `liqiangcc/jellyfin-web-media-gateway` 的 R007 Task Package。

本文件只是会话启动入口，不是 Task Contract，也不保存实时 Task 状态。

## Execution Context

```text
GitHub Issue: #2
Task Contract: docs/tasks/2-r007-playback-concurrency-closure/task.md
Expected worker: web
Expected environment label: env:web-gpt
Preferred Skill: $task-worker
Research Item: R007
```

## Live Gate

新会话必须先重新读取 Issue #2，并以 Issue 当前 labels / owner 为实时 authority。

```text
status:draft
→ 停止，不 claim、不实现、不自行发布

status:ready + env:web-gpt + no active owner
→ 可以按 $task-worker 流程 claim

其他状态
→ 按 docs/tasks/issue-lifecycle-protocol.md 停止或交回 Coordinator
```

只有 Coordinator 通过 `$task-publisher` / Publication Gate 才能把 draft Task 发布为 ready；Worker 不自行改变发布状态。

## Preferred Codex Entry

```text
$task-worker Execute Issue #2 using `docs/tasks/2-r007-playback-concurrency-closure/prompt.md`.
```

如果 repo Skill 不可见，则按下面 Start Protocol 手动执行。

## Start Protocol

1. 同步并实际读取当前仓库，不根据聊天背景猜测状态。
2. 读取：
   - `AGENTS.md`
   - GitHub Issue #2 及 relevant comments
   - `docs/tasks/2-r007-playback-concurrency-closure/task.md`
   - `docs/tasks/issue-lifecycle-protocol.md`
   - `docs/architecture.md`
   - `docs/implementation-contracts.md`
   - `docs/technical-feasibility-validation.md`
   - `docs/mvp-plan.md`
3. 确认 Issue 当前为 `status:ready + env:web-gpt` 且无 active owner。
4. claim → `status:in-progress` → 确定新的 `Attempt N`。
5. 严格执行 Task Contract，只做 R007 executable Playback model / contract closure / race tests。
6. 正常结束评论 `[EXECUTION REPORT]` → `status:review`；阻塞评论 `[BLOCKER REPORT]` → `status:blocked`。
7. 释放 active execution ownership并停止，等待 Coordinator `$task-reviewer`。

## Scope reminder

R007 当前任务的核心是：

```text
command CAS revision != high-frequency telemetry
same-item async media refresh has explicit freshness generation/ticket
handoff candidate authority != committed active_display authority
stale async work never overwrites newer authority
```

最小并发测试覆盖：

```text
duplicate request_id
stale expected revision
stale item callback
stale re-resolve result
stale display generation
overlapping handoff
two-Control concurrent mutation
```

不要进入：

- R001 HLS/MP4 Media Gateway；
- FFmpeg；
- `/control` 完整 UI；
- real Display/networking；
- Jellyfin；
- TV；
- Site Plugin/Auth；
- Ubuntu ARM64 Target Runner。

## Test quality reminder

竞态测试必须能强制关键交错顺序：barrier/channel/manual scheduling/model checking 均可。

`sleep()` + “多跑几次”不能作为唯一竞态证据。Repeated stress 只是 deterministic tests 的补充。

## Authority

```text
canonical docs
→ architecture / implementation contracts / feasibility / MVP facts

AGENTS.md
→ long-term collaboration rules

task.md
→ R007 unique execution contract

prompt.md
→ bootstrap only

Issue fields / labels
→ live state

Issue comments
→ Attempt / Review / Acceptance history

$task-worker
→ common claim / Attempt / feedback algorithm
```

本 prompt 不得重定义 Task Scope / Claims / Success Criteria。

## Stop Boundary

完成或阻塞一个 R007 Attempt 后立即停止。

Worker 不得自行 `status:done` 或关闭 Issue #2；Coordinator 必须通过 `$task-reviewer` Review，并可 `REVISE` 后启动下一 Attempt。