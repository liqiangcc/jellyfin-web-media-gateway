# Session Bootstrap — <task title>

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 中一个已经定义好的独立 Task。

本文件只是**会话启动入口**，不是 Task Contract。

## Execution Context

```text
GitHub Issue: #<number>
Task Contract: docs/tasks/<issue>-<slug>/task.md
Expected worker: web | wsl | windows | cloud | ubuntu-arm64 | manual-tv | capability-driven
Expected environment label: env:<environment>
```

## Preferred Codex Entry

仓库提供 repo-scoped Codex Skill：

```text
$task-worker
```

推荐从新 Codex 会话直接执行：

```text
$task-worker Execute Issue #<number> using `docs/tasks/<issue>-<slug>/prompt.md`.
```

`$task-worker` 负责通用的 claim / Attempt / feedback / stop 协议；本 `prompt.md` 负责当前 Task 的具体 bootstrap。Skill 不能覆盖本 Task Contract。

如果当前客户端无法发现 repo Skill，仍按下面的 Start Protocol 手动执行；不要因为 Skill 不可见而绕过 Task 生命周期。

## Start Protocol

1. 实际读取当前仓库，不根据聊天背景猜测状态。
2. 读取并遵守：
   - `AGENTS.md`
   - GitHub Issue `#<number>` 及 relevant comments
   - `docs/tasks/<issue>-<slug>/task.md`
   - `docs/tasks/issue-lifecycle-protocol.md`
   - `task.md` 明确引用的 canonical /专题文档
3. 确认 Issue 仍满足可领取条件：
   - `status:ready`
   - 当前环境/能力匹配
   - 没有 active owner
4. claim 后切换为 `status:in-progress`，确定新的 `Attempt N`，再开始写入性工作。
5. 严格执行 `task.md` 的 Scope、Claims、Success Criteria、Verification Plan 和 Evidence Contract。
6. 正常结束时，先在 Issue 评论标准 `[EXECUTION REPORT]`，记录 Attempt、Candidate、Claim results、Jobs/commands、Evidence、问题与未验证范围；然后切换到 `status:review`。
7. 如果无法继续，评论标准 `[BLOCKER REPORT]`，记录 blocker、已完成部分、Evidence、恢复条件与 cleanup；然后切换到 `status:blocked`。
8. 结束当前 Attempt 后释放 active execution ownership。
9. 停止，不自动开始下一 Task，也不自行开始下一 Attempt。

如果 Issue 已被其他 Worker claim、状态不再是 `status:ready`，或当前环境不满足 Required Capabilities，则停止，不自行扩大或改写任务。

Worker 不得自行设置 `status:done` 或关闭 Issue。只有 Coordinator 可以通过 `[COORDINATOR REVIEW]` 决定 `ACCEPT / REVISE / BLOCK / SPLIT / NOT_PLANNED`；只有 `[FINAL ACCEPTANCE]` 后才能关闭。

## Authority

执行信息的职责边界：

```text
canonical docs
→ 产品/架构/安全事实

AGENTS.md
→ 仓库长期 Agent 规则

task.md
→ 当前 Task 唯一执行契约

prompt.md
→ 当前会话 bootstrap / navigation only

GitHub Issue fields / labels
→ 当前 Task 实时状态 / owner / blocker / result summary

GitHub Issue comments
→ Attempt / Blocker / Review / Acceptance append-only history

Codex Skill
→ 通用执行算法，不拥有 Task Scope
```

本 `prompt.md` 不得重新定义：

- Goal；
- In Scope / Out of Scope；
- Claims；
- Success Criteria；
- Architecture Invariants；
- Verification Job Matrix；
- Evidence 判断标准。

如果本文件与 `AGENTS.md`、`task.md` 或 canonical docs 冲突，忽略本文件中的冲突内容，并按更高 authority 执行。

## Task-specific Entry Note

这里只允许填写**启动所需的最少信息**，例如：

- 当前 Worker 类型；
- 需要先检查的设备/连接状态；
- Task Contract 路径；
- Issue 编号；
- 一条不会改变 Scope 的启动提醒。

不要复制 `task.md` 的完整内容，不要在这里维护动态状态或实验结果。

## Coordinator Handoff Entry

本 Task 通过 Publication Gate 并进入 `status:ready` 后，Coordinator 必须把一个**可直接复制给下游 Worker 新会话**的入口提示词交给用户。

Codex 优先形式：

```text
$task-worker Execute Issue #<number> using `docs/tasks/<issue>-<slug>/prompt.md`.
```

兼容 fallback：

```text
读取 `AGENTS.md` 和 `docs/tasks/<issue>-<slug>/prompt.md`，执行当前 Task。
```

Coordinator 对外给出的 handoff 还应同时明确真实值：

```text
Issue: #<number>
Worker: <expected worker>
Environment: env:<environment>
Prompt: docs/tasks/<issue>-<slug>/prompt.md
Skill: $task-worker
```

要求：

- 必须使用发布后从 GitHub read-back 得到的真实 Issue/路径/环境，不得保留 `<placeholder>`；
- 只有 Post-publish Queue Verification PASS 后才能输出；
- 如果发布验证失败，不得提供下游执行入口；
- 下游入口只负责启动导航，不复制 `task.md`，也不重新定义 Scope/Claims/Success Criteria；
- 如果上一 Attempt 被 Coordinator `REVISE` 后重新进入 `status:ready`，Coordinator 必须重新给出同一 Task 的 downstream entry，让下一 Worker 从 Issue history 继续，而不是让用户猜测如何恢复。
