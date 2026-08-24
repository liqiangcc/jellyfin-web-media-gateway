# Codex Task Lifecycle Skills

本文件说明仓库 Task 协作协议如何映射到 repo-scoped Codex Skills。

Skill 目录：

```text
.agents/skills/
├── task-publisher/
├── task-worker/
└── task-reviewer/
```

## 1. 定位

Skills 是**可执行流程层**，不是新的事实源。

```text
canonical docs
→ 产品 / 架构 / 安全事实

AGENTS.md
→ 长期 Agent 规则

task.md
→ 当前 Task 唯一执行契约

prompt.md
→ 当前 Task bootstrap

Issue fields / labels
→ 实时状态

Issue comments
→ Attempt / Review / Acceptance history

Codex Skills
→ 如何可靠执行上述协议
```

因此 Skill 不复制完整 Task Contract，也不能为了方便修改 Scope / Claims / Success Criteria。

## 2. Skill 路由

### `$task-publisher`

使用者：Web Coordinator / 具有 GitHub 写权限的 Coordinator Codex。

负责：

```text
Task design inputs
→ Issue status:draft
→ task.md + prompt.md
→ GitHub read-back
→ status:ready
→ target queue verification
→ downstream handoff
```

不负责实际 Task implementation，也不负责最终 Review。

### `$task-worker`

使用者：Web / WSL / Windows / Cloud / Ubuntu ARM64 等实际 Worker Codex。

负责：

```text
read Issue + comments + task package
→ verify ready / env / owner
→ claim
→ Attempt N
→ execute Scope
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ review / blocked
→ release ownership
→ STOP
```

Worker 不得自行 `status:done` 或关闭 Issue。

### `$task-reviewer`

使用者：Web Coordinator / Review Codex。

负责：

```text
Issue history + task.md + candidate + Evidence
→ [COORDINATOR REVIEW]
→ ACCEPT | REVISE | BLOCK | SPLIT | NOT_PLANNED
```

其中：

```text
REVISE
→ same Task / next Attempt / $task-worker

BLOCK
→ blocked / UNBLOCK / $task-worker

SPLIT
→ child Task / $task-publisher

ACCEPT
→ [FINAL ACCEPTANCE]
→ status:done
→ close Issue
```

## 3. 为什么 explicit-only

三个 Skill 都可能修改 GitHub Issue 状态、owner、Task Package 或关闭 Issue，因此：

```yaml
policy:
  allow_implicit_invocation: false
```

必须显式调用：

```text
$task-publisher
$task-worker
$task-reviewer
```

避免普通讨论被语义匹配成一次真实状态变更。

## 4. 推荐用户入口

发布任务：

```text
$task-publisher 发布 <Task>。
```

下游执行：

```text
$task-worker Execute Issue #<issue> using `docs/tasks/<issue>-<slug>/prompt.md`.
```

Coordinator Review：

```text
$task-reviewer Review Issue #<issue> and continue the Task lifecycle.
```

`prompt.md` 仍然保留，因为它是具体 Task 的版本化 bootstrap，Skill 是通用流程。两者职责不同，不互相替代。

## 5. Skill 与 prompt.md 的关系

```text
$task-worker
= 通用执行算法

prompt.md
= 当前 Issue / task.md / expected environment 的入口
```

所以推荐 handoff 是：

```text
$task-worker Execute Issue #123 using `docs/tasks/123-example/prompt.md`.
```

而不是重新粘贴 Task 的完整背景。

## 6. 第一阶段为什么不加 scripts

当前 Skill 先使用 instruction-only 形式。

原因：

- Task/Issue 状态协议刚完成收敛；
- GitHub write backend 在不同 Codex 环境可能是 `gh`、MCP 或其他连接能力；
- 过早固定 shell script 会把 backend 假设写死；
- 先通过真实 INFRA / implementation / verification Attempt 找出稳定的机械重复点。

后续优先脚本化候选：

```text
validate-task-package
validate-publication-gate
validate-worker-claim-state
validate-final-acceptance-gate
```

只有当这些输入/输出稳定后再加入 `scripts/`，并要求脚本结果仍经过 GitHub read-back 验证。

## 7. Skill 验证

新增/修改 Skill 后至少检查：

1. `SKILL.md` frontmatter 含唯一 `name` 和清晰 `description`；
2. repo 路径位于 `.agents/skills/<skill>/SKILL.md`；
3. explicit invocation 名称与 frontmatter 一致；
4. Skill 不复制/重定义 Task Contract；
5. mutation Skill 默认 explicit-only；
6. 新 Codex 会话能通过 `$` 或 `/skills` 发现；
7. 若 Codex 未刷新，重启 Codex 后再次验证。
