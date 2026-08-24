# 执行任务目录

本目录保存由 Web Coordinator 为独立 Worker 准备的版本化执行契约和会话启动入口。

核心原则：

> **Task 按工作与 Claim 拆；Job 按执行能力拆；环境和 Runner 不决定业务 Task 边界。**

默认层级：

```text
Goal / Research Item
        ↓
Task
├── implementation
├── verification
├── combined
└── research
        ↓
Claims to verify
        ↓
Verification Jobs
├── GitHub-hosted x64
├── GitHub-hosted ARM64
├── Ubuntu ARM64 Target Runner
└── Manual TV（如需要）
        ↓
Evidence
        ↓
Coordinator Decision
```

默认调度：

```text
Web Worker implementation
→ GitHub-hosted x64 / ARM64 verification
→ Ubuntu ARM64 Target Runner only for phone-specific proof
→ WSL / Windows / Cloud Codex only for interactive capability
→ Real TV / Manual for physical UX proof
```

Cloud 不部署 Runner。

## 1. Task Package 目录

进入任务队列、可由独立 Worker 领取的 Task 使用：

```text
docs/tasks/<issue-number>-<slug>/
├── task.md
└── prompt.md
```

职责：

```text
task.md
= 当前 Task 的唯一执行契约

prompt.md
= 新 Worker 会话的 bootstrap / navigation 入口
```

模板：

```text
docs/tasks/task.template.md
docs/tasks/prompt.template.md
```

对于一个准备进入 `status:ready`、需要独立 Worker 新会话执行的 Task，`task.md` 与 `prompt.md` 应先提交到仓库，Issue 再链接两者及 base commit。

如果只是 Coordinator 当前会话内完成、且不进入独立 Worker 队列的极小协调性修改，可以不建立 Task Package。

## 2. Authority 与状态所有权

必须避免同一信息在 Issue、`task.md`、`prompt.md` 中重复维护。

```text
canonical docs
= 产品 / 架构 / 安全事实

AGENTS.md
= 长期 Agent 规则

GitHub Issue
= 实时状态 authority

task.md
= 当前 Task 稳定执行契约

prompt.md
= 会话启动入口，最低 authority
```

Issue 动态状态包括：

```text
status
assignee / active owner
claimed environment
claimed at
active branch
candidate commit / PR
verification status
linked implementation / verification task
blocker
review state
result summary
```

这些字段不写回 `task.md` 或 `prompt.md`。

如果 `prompt.md` 与 `task.md`、`AGENTS.md` 或 canonical docs 冲突，忽略 Prompt 中的冲突内容。

## 3. prompt.md 规则

`prompt.md` 的目标是让用户启动新 Web/Codex/设备会话时，只需给出非常短的入口，例如：

> 读取 `AGENTS.md` 和 `docs/tasks/123-example/prompt.md`，执行当前 Task。

Worker 应自行从 GitHub 获取 Issue、Task Contract 和相关 canonical 文档，不要求用户重复粘贴完整项目背景。

### 3.1 prompt.md 应包含

- GitHub Issue 编号；
- `task.md` 路径；
- 预期 Worker / environment 提示；
- 必须先读取哪些入口文件；
- claim 前置条件；
- `status:in-progress` / `status:review` 生命周期提醒；
- 完成后停止、不自动开始下一项；
- 必要的最少启动提醒，例如先检查设备连接状态。

### 3.2 prompt.md 不得包含

- 一套复制出来的 Goal / Scope；
- 重新定义的 Claims；
- 另一套 Success Criteria；
- 另一套 Architecture Invariants；
- 另一份 Verification Job Matrix；
- 动态 claim/status/result；
- Secret、Cookie、Token、registration token；
- 为了“方便”而复制整份 `task.md`。

原则：

> **Prompt tells the Worker where and how to start; task.md tells the Worker what must be done.**

### 3.3 Prompt 更新规则

通常 Task 执行过程中不需要修改 `prompt.md`。

只有以下内容发生变化时才更新：

- Issue / Task 路径发生变化；
- 预期 Worker 类型发生实质变化；
- bootstrap 前置步骤发生变化；
- 原 Prompt 本身存在错误或与更高 authority 漂移。

claim、block、review、done、Evidence 变化不触发 Prompt 更新。

## 4. Task 类型

```text
implementation
verification
combined
research
```

- `implementation`：产生 candidate commit / PR，不自动等于 runtime claim 已验证。
- `verification`：针对确定 Candidate SHA 验证一个或一组稳定 Claims。
- `combined`：普通工程任务可在一个 Task 内完成 Web implementation + 标准 Actions verification。
- `research`：需要 Hypothesis、Success Criteria、真实 Evidence、Gate Decision。

始终保持：

```text
Implementation Result
!= Verification Result
!= Coordinator Gate Decision
```

## 5. Task 拆分规则

### 5.1 默认使用 combined，而不是机械拆两个 Issue

如果验证只是候选实现的标准、快速、可自动化 CI，例如：

```text
fmt
clippy
unit test
contract test
portable integration
普通 x64 / generic ARM64 regression
```

默认使用一个 `combined` Task：

```text
Web Worker implementation
→ Candidate SHA
→ GitHub Actions Jobs
→ Verification Result
→ Coordinator Review
```

不为了“开发/测试分离”形式化地创建两个 Issue。

### 5.2 何时拆独立 Verification Task

出现以下任一情况时，优先把 Verification 拆成独立 Task：

- Verification 有独立 Evidence Authority，例如 Ubuntu ARM64 Target Runner、Real TV、真实 Jellyfin TV；
- Verification 生命周期明显晚于 Implementation，例如目标设备暂不可用；
- 验证成本、风险或持续时间需要独立调度与重试；
- 验证结论需要单独 PASS / CONDITIONAL PASS / FAIL / BLOCKED；
- Research Gate 需要把“候选实现已完成”和“关键 claim 已证明”独立追踪；
- Verification 可能由不同 Worker/Manual operator 执行；
- 同一候选实现需要多轮 target proof，而 Implementation 不应反复 reopen。

拆分后：

```text
Implementation Task
→ Candidate SHA
→ done/review as implementation result

Verification Task
→ references Candidate SHA
→ Claims
→ Jobs / Target / Evidence
→ verification result

Parent Goal / Research Gate
→ Coordinator 汇总两者后决定
```

`Implementation Task = done` 只表示实现交付已接受，不表示 Parent Goal 或 Research Gate 已通过。

### 5.3 不按环境拆 Task

不要因为需要：

```text
x64
ARM64
Ubuntu phone
TV
```

就机械创建四个业务 Task。

如果它们验证的是同一组 Claims，可以属于同一个 Verification Task，并映射为多个 Job：

```text
Verification Task: playback concurrency safety

Job A → GitHub-hosted x64
Job B → GitHub-hosted ARM64
Job C → Ubuntu ARM64 Target Runner（仅 target-specific claim）
```

只有当 Claim、生命周期、Owner、Success Criteria 或 Evidence Authority 真正不同，才拆成新的 Task。

## 6. Task 与 Job 的边界

```text
Task
= 要完成/证明什么
= 有 Scope、Owner、Success Criteria、Review 生命周期

Job
= 在哪里、用什么命令执行某个验证切片
= 没有独立业务 owner
= 不 claim Issue
```

一个 Verification Task 可以拥有 0..N 个 Jobs。

Job 应由 Claim 推导：

```text
Claim
→ Required Capabilities
→ Execution Plane
→ Runner / Target
→ Commands
→ Evidence
```

禁止反向设计：

```text
“我有一个 ARM64 Runner”
→ 所以创建一个 ARM64 Task
```

## 7. task.md = 稳定执行契约

保存：

```text
Task kind
Parent Goal / Research Item
Goal / Context
Base / Candidate commit
Task decomposition decision
Claims to verify
Required capabilities
Verification Job Matrix
Execution Plane
Runner class / image / labels
Target / Trust gate
Scope
Architecture Invariants
Implementation Requirements
Verification Plan
Success Criteria
Evidence Contract
Failure / Blocked Rules
Deliverables
```

Worker 不静默改变 Success Criteria。

## 8. Web-first

`env:web-gpt` 表示独立 Web Worker Session。

Web Worker 可以写代码、测试、文档、workflow、commit/PR，并读取 Actions run/job/log/artifact 完成真实验证闭环。

先问：

> Web Worker + GitHub Actions 是否已经能够完成并产生有效 Evidence？

## 9. GitHub Actions = 统一自动验证平面

优先：

```text
portable x64 build/test/lint
→ GitHub-hosted x64

generic ARM64 build/test
→ GitHub-hosted ARM64

phone-specific ARM64/runtime/resource proof
→ Ubuntu ARM64 self-hosted Target Runner
```

Runner 不是 Worker，不 claim Issue。

当前仓库尚未建立 `.github/workflows/`；等第一个 Rust workspace/真实测试落地时再创建有实际意义的 CI。

## 10. 大量重复 / 长时间任务

大量 race、benchmark、regression 优先使用：

```text
GitHub-hosted matrix / sharding / repeated jobs
```

Cloud 资源有限，不作为 Runner，也不作为普通 long-running 默认后端。

如果 claim 要求同一进程连续运行，不能用分片伪装连续 soak；此时按 claim 选择真实足够环境，并视其生命周期决定是否拆独立 Verification Task。

## 11. 外部 Worker

### WSL

Actions 失败后需要交互式 Linux debug、反复加日志、启动本地进程时使用。

### Windows

ADB、Android host、手机重启/恢复/部署协调。

### Cloud

不部署 Runner。只用于 Cloud-specific 复现、长期交互 state、Tailscale remote orchestration 等 Actions 不适合的场景。

### Ubuntu ARM64 Codex

只有 Target Runner 无法表达的交互式 target debug、设备恢复、现场诊断时使用。

### Real TV / Manual

最终 audible autoplay、遥控器、TV UX、Jellyfin Android TV。

## 12. Claim

一个 Task 任一时刻只允许一个 active owner；Research Item 可以拆多个 Task 并行。

Worker：

1. 查询 `status:ready + env:<current-environment>`；
2. 确认 Task kind、Required Capabilities / Execution Plane；
3. 确认无 active owner；
4. claim + `status:in-progress`；
5. 读取 `AGENTS.md` / Issue / `task.md`；如果存在 `prompt.md`，先用它完成会话 bootstrap；
6. 只执行当前 Scope；
7. 提交 candidate / Evidence；
8. Issue → `status:review`；
9. 停止，不自动开始下一项。

GitHub Actions Job 不参与 claim。

## 13. Evidence

至少记录：

```text
Role
Task / Claim
Orchestrator
Execution Plane
Executor / Runner class
Runner image / labels
Execution host
Target
OS / architecture
Relevant versions
Network path
Candidate commit
Workflow / run / job
Commands / test selector
Duration / repetitions / shards
Metrics / artifact / logs
Result
```

不得把：

- Web 静态分析当 runtime PASS；
- GitHub-hosted generic ARM64 当目标手机；
- Cloud host 当手机热环境/家庭 LAN；
- 模拟器当真实 TV。

## 14. 推荐启动方式

### Web Worker

给新会话优先只提供：

> 读取 `AGENTS.md` 和 `docs/tasks/<issue>-<slug>/prompt.md`，执行当前 Task。

`prompt.md` 再导航到 Issue / `task.md`。如果 Prompt 与 Task Contract 冲突，以 `task.md` 和更高 authority 为准。

### 外部 Worker

同样优先使用对应 Task Package 的 `prompt.md`，避免用户重复粘贴整套背景。

如果某个旧 Task 尚无 `prompt.md`，Worker 仍可以直接读取 `AGENTS.md` + Issue + `task.md` 执行；Prompt 是标准 bootstrap 入口，不是新的业务 authority。
