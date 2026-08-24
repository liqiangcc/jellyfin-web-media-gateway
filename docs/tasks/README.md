# 执行任务目录

本目录保存由网页 GPT + GitHub MCP 派发给具体 Codex 环境的版本化任务契约。

## 目录规则

每个 GitHub Issue 使用独立目录：

```text
docs/tasks/<issue-number>-<slug>/task.md
```

例如：

```text
docs/tasks/12-r007-playback-concurrency/task.md
```

Issue 用于状态、assignee、讨论、依赖和 PR/commit 链接；`task.md` 用于 Goal、Scope、环境、测试、Evidence 和完成条件。

## Issue 标签

网页 GPT/MCP 至少设置一个状态标签和一个或多个可执行环境标签：

```text
status:draft | status:ready | status:in-progress | status:blocked | status:review | status:done
env:web-gpt | env:windows | env:wsl | env:ubuntu-arm64 | env:cloud | env:manual-tv
```

多个 `env:*` 表示这些环境都具备领取资格，不表示默认并行执行。

## 生命周期

```text
网页 GPT/MCP 创建 Issue
→ 从 task.template.md 创建 task.md
→ task.md 提交到 main，Issue 链接路径和 base commit
→ 标记 eligible env:* + status:ready
→ 匹配环境自行查询任务队列
→ 一个环境 claim：assignee + status:in-progress
→ Codex 执行并提交结果
→ status:review
→ 网页 GPT/MCP 验收
→ status:done 或退回 status:ready
```

同一 Issue 不允许多个 Codex 同时成为 active owner。需要拆分并行工作时，建立独立子 Issue 和独立任务目录。

## 自助领取

每个环境只查询同时匹配 `status:ready` 与自身 `env:*` 的 Issue。网页 GPT + GitHub MCP 也是 `env:web-gpt` 执行环境，可以领取文档、Issue/PR、仓库分析、Review 和其工具能够完成的轻量修改。领取成功后再拉取/创建任务分支和开始写入性工作。

如果两个环境同时尝试领取，以最先成功设置 assignee 和 `status:in-progress` 的环境为 owner；另一环境必须停止。没有匹配任务时不自行从 backlog 推断工作。

## Codex 使用方式

推荐指令：

> 读取 `AGENTS.md` 和 `docs/tasks/<issue>-<slug>/task.md`，只执行任务 Scope，提交结果和 Evidence 后停止。

Codex 不自行扩大 Scope、不自动开始下一项、不自行关闭 Issue。

推荐的自动领取指令：

> 读取 `AGENTS.md`，查询匹配当前环境且标记为 `status:ready` 的 Issue；领取最高优先级任务后读取对应 `task.md`，只执行其 Scope，提交结果并转为 `status:review` 后停止。

## 完成后的文件

任务完成后保留 `task.md`，并填写 Result、Evidence 和最终 commit/PR，作为可审计历史。大型原始日志、Secret、Cookie、Token、账号信息和临时媒体 URL 不得写入任务目录。

如果任务只产生运行证据，详细结果优先写入：

```text
docs/research/<research-id>-<topic>.md
```

`task.md` 只链接该 Evidence，不重复维护两份结论。
