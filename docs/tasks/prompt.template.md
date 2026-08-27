# Session Bootstrap — <task title>

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 中一个已经定义好的独立 Task。

本文件只是**Task 内部 bootstrap / navigation 入口**，不是 Task Contract，也不是某个客户端的复制命令模板。

## Execution Context

```text
GitHub Issue: #<number>
Task Contract: docs/tasks/<issue>-<slug>/task.md
Expected worker: web | wsl | windows | cloud | ubuntu-arm64 | manual-tv | capability-driven
Expected environment label: env:<environment>
Downstream handoff profile: docs/tasks/handoffs/<environment-profile>.md
```

## Environment-specific Entry

Coordinator 对用户给出的可复制启动命令由对应 environment handoff profile 生成，而不是由本文件统一假设为 Codex。

例如：

```text
env:web-gpt
→ docs/tasks/handoffs/web-gpt.md

env:ubuntu-arm64
→ docs/tasks/handoffs/ubuntu-arm64.md
```

Task Contract 可以相同，但 Web ChatGPT、Codex、Manual TV 的启动语法不同。

如果一次 Task 有多个 eligible environments，Coordinator 必须分别输出多个独立复制块；不要把多个客户端入口揉进一个提示词。

## Start Protocol

无论使用哪种客户端入口，Worker 都必须：

1. 实际读取当前仓库/GitHub，不根据聊天背景猜测状态。
2. 读取并遵守：
   - `AGENTS.md`
   - GitHub Issue `#<number>` 及 relevant comments
   - `docs/tasks/<issue>-<slug>/task.md`
   - `docs/tasks/issue-lifecycle-protocol.md`
   - `docs/tasks/execution-anchor-recovery-protocol.md`
   - `docs/tasks/freshness-integration-protocol.md`
   - `task.md` 明确引用的 canonical /专题文档
3. 确认 Issue 当前满足可领取条件：
   - `status:ready`
   - 当前 `env:*` eligibility 匹配
   - 没有 active owner
   - 当前环境具备 Required Capabilities
4. claim 后切换为 `status:in-progress`，确定新的 `Attempt N`，再开始写入性工作。
5. 严格执行 `task.md` 的 Scope、Claims、Success Criteria、Verification Plan、Freshness / Integration Contract 和 Evidence Contract。
6. 不要仅因为 `main` SHA 变化就自行宣布旧 Evidence 失效或全量重跑；按 `task.md` 的 Freshness Contract 和 freshness protocol 执行，最终 freshness classification 由 Coordinator Review 决定。
7. 正常结束时先在 Issue 评论标准 `[EXECUTION REPORT]`，再转 `status:review`。
8. 无法继续时评论标准 `[BLOCKER REPORT]`，再转 `status:blocked`。
9. 结束当前 Attempt 后释放 active execution ownership。
10. 停止，不自动开始下一 Task，也不自行开始下一 Attempt。

如果 Issue 已被其他 Worker claim、状态不再是 `status:ready`，或当前环境不满足 Required Capabilities，则停止，不自行扩大、改写或发布任务。

Worker 不得自行设置 `status:done` 或关闭 Issue。只有 Coordinator 可以通过 `[COORDINATOR REVIEW]` 决定 `ACCEPT / REVISE / BLOCK / SPLIT / NOT_PLANNED`；只有 `[FINAL ACCEPTANCE]` 后才能关闭。

## Freshness / Integration Reminder

对于采用 `Freshness policy: dependency-aware` 的 Task：

```text
main advanced
!= automatically stale
```

Worker 应保留并报告真实：

- Task Candidate SHA；
- Task-specific exact-SHA Evidence；
- 自己实际基于/观察到的 accepted main snapshot；
- 如果 Coordinator 已发布 `[INTEGRATION GATE]`，则还要记录 Integration Base / Integration Candidate / JI Evidence。

如果当前 Attempt 是 `Revision class: INTEGRATION_ONLY`：

- 必须复用原 Issue/branch/PR；
- 优先把 Coordinator 冻结的 Integration Base 以 merge commit 合入，保留原 Task Candidate ancestry；
- 只执行声明的 `JI*` integration jobs，除非出现 semantic conflict；
- conflict 触及 Task-owned semantic surface 时停止按 integration-only 猜测，交回 Coordinator 重新分类。

如果 Task 明确是 `Freshness policy: strict-main`，则继续遵守其冻结的 strict-main 要求；不能用 dependency-aware 默认值降低已发布 Contract。

## Authority

```text
canonical docs
→ 产品/架构/安全事实

AGENTS.md
→ 仓库长期 Agent 规则

task.md
→ 当前 Task 唯一执行契约

prompt.md
→ 当前 Task bootstrap / navigation only

GitHub Issue fields / labels
→ 当前 Task 实时状态 / owner / blocker / result summary

GitHub Issue comments
→ Attempt / Blocker / Review / Acceptance append-only history

Environment Handoff Profile
→ 当前客户端如何启动这个 prompt/task，不拥有 Scope

Codex Skill (when applicable)
→ 通用执行算法，不拥有 Task Scope
```

本 `prompt.md` 不得重新定义：

- Goal；
- In Scope / Out of Scope；
- Claims；
- Success Criteria；
- Architecture Invariants；
- Verification Job Matrix；
- Freshness / Integration Contract；
- Evidence 判断标准。

如果本文件与 `AGENTS.md`、`task.md` 或 canonical docs 冲突，忽略本文件中的冲突内容，并按更高 authority 执行。

## Task-specific Entry Note

这里只允许填写启动所需的最少 Task-specific 信息，例如：

- 当前 Worker 类型；
- 需要先检查的设备/连接状态；
- Task Contract 路径；
- Issue 编号；
- 一条不会改变 Scope 的启动提醒。

不要复制 `task.md` 的完整内容，不要在这里维护动态状态或实验结果。

## Coordinator Handoff Rule

本 Task 通过 Publication Gate 后，Coordinator 必须：

1. 从 GitHub read-back 获得真实 Issue / environment / prompt path；
2. 对每个 eligible environment 选择 `docs/tasks/handoffs/` 中对应 profile；
3. 替换 profile 中 placeholder；
4. 每个环境分别输出一个可直接复制的新会话入口；
5. queue verification 失败时不输出 handoff；
6. `REVISE` 后重新进入 `status:ready` 时，再次输出对应环境的 handoff。

通用 Task bootstrap 不等于通用客户端命令。