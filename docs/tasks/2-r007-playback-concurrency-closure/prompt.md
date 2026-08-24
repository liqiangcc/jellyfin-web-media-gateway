# Session Bootstrap — R007 Playback Concurrency Contract Closure

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 R007 Task。

本文件只是 Task 内部 bootstrap / navigation 入口，不是 Task Contract，也不保存实时 Task 状态。

## Execution Context

```text
GitHub Issue: #2
Task Contract: docs/tasks/2-r007-playback-concurrency-closure/task.md
Expected worker: web
Expected environment label: env:web-gpt
Downstream handoff profile: docs/tasks/handoffs/web-gpt.md
Research Item: R007
```

## Expected Client

本 Task 的下游执行客户端是 **Web ChatGPT Worker + GitHub connector**。

不要求 repo-scoped `$task-worker` Skill，也不要使用 Web 搜索替代 GitHub。

Coordinator 对外可复制入口由：

```text
docs/tasks/handoffs/web-gpt.md
```

生成。

## Live Gate

新 Web Worker 会话必须先实际读取 GitHub Issue #2，并以 Issue 当前 labels / owner 为实时 authority。

```text
status:draft
→ 停止，不 claim、不实现、不自行发布

status:ready + env:web-gpt + no active owner
→ 可以 claim，并开始新的 Attempt N

其他状态
→ 按 docs/tasks/issue-lifecycle-protocol.md 停止或交回 Coordinator
```

## Start Protocol

1. 必须实际使用 GitHub 读取当前仓库和 Issue，不根据聊天背景猜测状态。
2. 读取：
   - `AGENTS.md`
   - GitHub Issue #2 及 relevant comments
   - `docs/tasks/2-r007-playback-concurrency-closure/task.md`
   - `docs/tasks/issue-lifecycle-protocol.md`
   - `docs/architecture.md`
   - `docs/implementation-contracts.md`
   - `docs/technical-feasibility-validation.md`
   - `docs/mvp-plan.md`
3. 确认 Issue 当前为 `status:ready + env:web-gpt` 且无 active owner，并确认 Web Worker 当前具备 Task 要求的 GitHub read/write、code authoring 和 Actions evidence 能力。
4. claim → `status:in-progress` → 确定新的 `Attempt N`。
5. 严格执行 Task Contract，只做 R007 executable Playback model / contract closure / race tests。
6. 需要 runtime/test Evidence 时通过 GitHub Actions 获得并读取真实 run/job/log/artifact；不要把静态分析当 runtime PASS。
7. 正常结束评论 `[EXECUTION REPORT]` → `status:review`；阻塞评论 `[BLOCKER REPORT]` → `status:blocked`。
8. 释放 active execution ownership并停止，等待 Coordinator Review。

## Scope Reminder

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
- real Display networking；
- Jellyfin；
- TV；
- Site Plugin/Auth；
- Ubuntu ARM64 Target Runner。

## Test Quality Reminder

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
→ R007 bootstrap only

Issue fields / labels
→ live state

Issue comments
→ Attempt / Review / Acceptance history

web-gpt handoff profile
→ 如何启动 Web Worker，不拥有 Task Scope
```

本 prompt 不得重定义 Task Scope / Claims / Success Criteria。

## Stop Boundary

完成或阻塞一个 R007 Attempt 后立即停止。

Worker 不得自行 `status:done` 或关闭 Issue #2；Coordinator Review 可以 `REVISE` 后重新发布下一 Attempt。