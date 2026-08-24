# 执行任务目录

本目录保存由 Web Coordinator 为独立 Worker 准备的版本化执行契约。

核心原则：

> **任务定义工作和证据，不把任务固定绑定到环境。**

默认调度：

```text
Web Worker first
→ GitHub Actions for automated verification
→ Cloud for long-running
→ WSL / Windows for interactive debugging / host capability
→ Ubuntu ARM64 / TV for target proof
```

## 1. 任务目录

需要版本化执行契约的 Issue 使用：

```text
docs/tasks/<issue-number>-<slug>/task.md
```

例如：

```text
docs/tasks/12-r007-playback-concurrency/task.md
```

`task.md` 是稳定执行契约，不是实时状态文件。

## 2. Task 类型

模板支持：

```text
implementation
verification
combined
research
```

### implementation

产生候选实现：

```text
Goal / Contract
→ Worker implementation
→ Candidate commit / PR
```

### verification

针对确定 commit 验证 claim：

```text
Candidate SHA
→ Verification backend
→ Evidence
```

### combined

适用于普通工程任务：Web Worker 实现后，由 GitHub Actions 自动验证，不必为了形式拆两个 Issue。

### research

用于需要明确 Hypothesis、Success Criteria、真实环境 Evidence 和 Gate Decision 的工作。

逻辑上始终保持：

```text
Implementation Result
!= Verification Result
!= Coordinator Gate Decision
```

## 3. Issue = 实时状态 authority

以下动态状态只在 GitHub Issue 中维护：

```text
status
assignee / active owner
claimed environment
claimed at
active branch
candidate commit / PR
verification status
blocker
review state
result summary
```

状态：

```text
status:draft
status:ready
status:in-progress
status:blocked
status:review
status:done
```

不要把这些字段重新写回 `task.md`。

## 4. task.md = 执行契约

保存：

```text
Task kind
Goal / Context
Base commit
Candidate commit (verification task)
Claims to verify
Preferred execution path
Eligible environments
Required capabilities
Preconditions
Scope
Architecture Invariants
Implementation Requirements
Verification Plan
Success Criteria
Evidence Contract
Failure / Blocked Rules
Deliverables
```

任务契约发生实质变化时由 Coordinator 修改并重新 Review；Worker 不静默改变 Success Criteria。

## 5. Web-first

`env:web-gpt` 表示独立 Web Worker Session 的执行资格。

Web Worker 可以：

- 写代码、测试和文档；
- 修改 GitHub；
- 提交 commit / PR；
- Review Actions logs / artifacts；
- 基于真实自动化结果完成大量工程闭环。

因此不要因为任务包含代码或测试就自动交给外部 Codex。

先问：

> Web Worker + GitHub Actions 是否已经能够完成并产生有效 Evidence？

## 6. GitHub Actions = 默认自动验证平面

当任务需要可自动化的：

```text
build
test
fmt
clippy
contract test
concurrency test
security test
integration test
```

优先由 Actions 执行。

Web Worker 可以作为 Orchestrator 读取 run/job/log/artifact 并继续修复。

注意：

```text
Orchestrator = web-gpt
Executor = github-actions
```

真实 Evidence 来源是 Actions runner，不是网页分析。

当前仓库尚未建立 `.github/workflows/`；等第一个 Rust workspace/真实测试落地时再创建有实际意义的 CI。

## 7. Cloud = 长时间执行优先

需要：

- 6h/24h soak；
- 大量重复 race；
- benchmark matrix；
- failure injection；
- 内存泄漏观察；
- 无人值守持续执行；

优先考虑 `env:cloud`，而不是让 WSL/网页会话长期占用。

Cloud 默认不能证明：手机温度、家庭 LAN、真实 TV。若 Cloud 远程在目标设备执行，Evidence 必须记录真实 Target。

## 8. Local / Target 环境

### WSL

用于交互式 Linux debug：CI 失败后需要快速改代码、加日志、启动本地进程时最合适。

### Windows

用于 ADB、Android host、手机重启/恢复/部署协调。

### Ubuntu ARM64

只在 claim 依赖 ARM64、目标 runtime、CPU/RSS/temperature 或设备特有兼容性时使用。

### Real TV / Manual

只在最终 claim 依赖 audible autoplay、遥控器、TV UX 或 Jellyfin Android TV 行为时使用。

## 9. Claim

一个具体 Task 任一时刻只允许一个 active owner。大型 Research Item 可以拆成多个独立 Task 并行。

Worker：

1. 查询 `status:ready + env:<current-environment>`；
2. 确认 Required Capabilities；
3. 确认无 active owner；
4. claim + `status:in-progress`；
5. 读取 `AGENTS.md` / Issue / `task.md`；
6. 只执行当前 Scope；
7. 提交候选实现或 Evidence；
8. Issue → `status:review`；
9. 停止，不自动开始下一项。

Web Coordinator 负责最终 Review 和 `done / ready / blocked`。

## 10. Evidence

runtime / Research 结果至少记录：

```text
Role
Orchestrator
Executor
Execution host
Target
OS / architecture
Relevant versions
Network path
Candidate / base commit
Workflow / run / job (if Actions)
Commands / steps
Metrics / artifact / logs
Result
```

不得把：

- Web 静态分析当 runtime PASS；
- Actions x86 runner 当 ARM64；
- Cloud host 当手机热环境/家庭 LAN；
- 模拟器当真实 TV。

大型日志、Secret、Cookie、Token、账号数据和临时媒体 URL 不得进入任务目录。

## 11. 推荐指令

### Web Worker

> 读取 `AGENTS.md`、对应 Issue 和 `docs/tasks/<issue>-<slug>/task.md`；claim `env:web-gpt` Task，优先在 Web 完成 Implementation，并通过可用 GitHub Actions 获取自动验证 Evidence；只执行当前 Scope，提交后转 `status:review` 并停止。

### 外部 Worker

> 读取 `AGENTS.md`、对应 Issue 和 `docs/tasks/<issue>-<slug>/task.md`；确认当前环境提供 Task 缺失的 Required Capabilities 后 claim，只执行当前 Scope并记录真实 Executor/Target Evidence，提交后转 `status:review` 并停止。
