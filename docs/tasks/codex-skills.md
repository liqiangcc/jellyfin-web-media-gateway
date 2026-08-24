# Codex Task Lifecycle Skills

本文件说明仓库 Task 协作协议如何映射到 repo-scoped Codex Skills，以及它们与环境专用 handoff 的关系。

Skill 目录：

```text
.agents/skills/
├── task-publisher/
├── task-worker/
└── task-reviewer/
```

Environment Handoff Profiles：

```text
docs/tasks/handoffs/
├── web-gpt.md
├── ubuntu-arm64.md
├── wsl.md
├── windows.md
├── cloud.md
└── manual-tv.md
```

## 1. 定位

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

Environment Handoff Profile
→ 当前客户端如何启动 Task

Codex Skills
→ Codex 环境中如何可靠执行生命周期
```

Skill 和 Handoff 都不是新的 Task Contract。

## 2. Task Contract 与 Session Entry 分离

同一个 Task 可以拥有统一的 Scope / Claims / Success Criteria，但不同客户端必须使用不同启动入口：

```text
Task Contract
!=
Session Entry Syntax
```

例如：

```text
env:web-gpt
→ Web ChatGPT + GitHub connector prompt

env:ubuntu-arm64 / env:wsl / env:windows / env:cloud
→ Codex + $task-worker

env:manual-tv
→ Manual Verification instructions
```

因此发布时不得固定输出 `$task-worker` 给所有环境。

## 3. Skill 路由

### `$task-publisher`

使用者：具有 GitHub 写权限的 Coordinator Codex。

负责：

```text
Task design inputs
→ Issue status:draft
→ task.md + prompt.md
→ GitHub read-back
→ status:ready
→ per-environment queue verification
→ per-environment downstream handoff
```

`task-publisher` 必须根据 `env:*` 读取 `docs/tasks/handoffs/` 中对应 profile。

### `$task-worker`

使用者：Codex Worker 环境，例如：

```text
env:ubuntu-arm64
env:wsl
env:windows
env:cloud
```

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

`env:web-gpt` 不要求 repo-scoped Codex Skill；它使用 `docs/tasks/handoffs/web-gpt.md` 通过 GitHub connector 执行同一 Worker 生命周期。

### `$task-reviewer`

使用者：Coordinator Codex。

负责：

```text
Issue history + task.md + candidate + Evidence
→ [COORDINATOR REVIEW]
→ ACCEPT | REVISE | BLOCK | SPLIT | NOT_PLANNED
```

若 REVISE 后回到 ready，Coordinator 必须再次根据实际 eligible environment 输出对应 handoff，而不是总是输出 `$task-worker`。

## 4. 为什么 Skills explicit-only

三个 Skill 都可能修改 GitHub Issue 状态、owner、Task Package 或关闭 Issue，因此：

```yaml
policy:
  allow_implicit_invocation: false
```

Codex 中必须显式调用，避免普通讨论被误识别成真实状态变更。

## 5. 推荐入口

Coordinator Codex 发布：

```text
$task-publisher 发布 <Task>。
```

Ubuntu ARM64 / WSL / Windows / Cloud Codex Worker：

```text
$task-worker Execute Issue #<issue> using `docs/tasks/<issue>-<slug>/prompt.md`.
```

Web Worker：使用 `docs/tasks/handoffs/web-gpt.md` 渲染出的 `@GitHub` 独立复制块。

Manual TV：使用 `docs/tasks/handoffs/manual-tv.md`。

Coordinator Review：

```text
$task-reviewer Review Issue #<issue> and continue the Task lifecycle.
```

## 6. prompt.md 与 Handoff 的关系

```text
Environment Handoff Profile
= 如何把用户带进正确客户端/环境

prompt.md
= 当前 Task 的具体 bootstrap

task.md
= 当前 Task 的唯一执行契约
```

所以 Web Worker 和 Codex Worker 可以使用不同启动文本，但最终都进入同一个 Issue / prompt.md / task.md 生命周期。

## 7. 多环境 Task

如果 Task 同时允许多个环境，例如：

```text
env:wsl
env:ubuntu-arm64
```

Coordinator 发布后必须给两份独立入口：

```text
WSL copy block

Ubuntu ARM64 copy block
```

不能给一份混合提示词让用户自己删除不适用部分。

## 8. 第一阶段为什么不加 scripts

当前 Skills 仍以 instruction-first 为主。稳定后再考虑脚本化：

```text
validate-task-package
validate-publication-gate
validate-worker-claim-state
validate-final-acceptance-gate
```

脚本结果仍必须经过 GitHub read-back。

## 9. Skill / Handoff 验证

新增或修改后至少检查：

1. `SKILL.md` frontmatter name/description 正确；
2. mutation Skill explicit-only；
3. 每个稳定 `env:*` 有独立 handoff profile；
4. Web handoff 不依赖 `$task-worker`；
5. Codex handoff 指向真实 `$task-worker` + prompt path；
6. Manual profile 不伪装自动化 Evidence；
7. 多环境 Task 会输出多个独立复制块；
8. 所有 handoff placeholder 在对外输出前由发布后 GitHub read-back 的真实值替换。
