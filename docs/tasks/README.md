# 执行任务目录

本目录保存由 Web Coordinator 为独立 Worker 准备的版本化执行契约。

核心原则：

> **任务定义工作和证据，不把任务固定绑定到环境。**

默认调度：

```text
Web Worker implementation
→ GitHub-hosted x64 / ARM64 verification
→ Ubuntu ARM64 Target Runner only for phone-specific proof
→ WSL / Windows / Cloud Codex only for interactive capability
→ Real TV / Manual for physical UX proof
```

Cloud 不部署 Runner。

## 1. 任务目录

需要版本化执行契约的 Issue 使用：

```text
docs/tasks/<issue-number>-<slug>/task.md
```

`task.md` 是稳定执行契约，不是实时状态文件。

## 2. Task 类型

```text
implementation
verification
combined
research
```

- `implementation`：产生 candidate commit / PR。
- `verification`：针对确定 Candidate SHA 验证 claim。
- `combined`：普通工程任务可在一个 Task 内完成 Web implementation + Actions verification。
- `research`：需要 Hypothesis、Success Criteria、真实 Evidence、Gate Decision。

始终保持：

```text
Implementation Result
!= Verification Result
!= Coordinator Gate Decision
```

## 3. Issue = 实时状态 authority

动态状态只在 Issue 中维护：

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

## 4. task.md = 稳定执行契约

保存：

```text
Task kind
Goal / Context
Base / Candidate commit
Claims to verify
Required capabilities
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

## 5. Web-first

`env:web-gpt` 表示独立 Web Worker Session。

Web Worker 可以写代码、测试、文档、workflow、commit/PR，并读取 Actions run/job/log/artifact 完成真实验证闭环。

先问：

> Web Worker + GitHub Actions 是否已经能够完成并产生有效 Evidence？

## 6. GitHub Actions = 统一自动验证平面

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

## 7. 大量重复 / 长时间任务

大量 race、benchmark、regression 优先使用：

```text
GitHub-hosted matrix / sharding / repeated jobs
```

Cloud 资源有限，不作为 Runner，也不作为普通 long-running 默认后端。

如果 claim 要求同一进程连续运行，不能用分片伪装连续 soak；此时按 claim 选择真实足够环境。

## 8. 外部 Worker

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

## 9. Claim

一个 Task 任一时刻只允许一个 active owner；Research Item 可以拆多个 Task 并行。

Worker：

1. 查询 `status:ready + env:<current-environment>`；
2. 确认 Required Capabilities / Execution Plane；
3. 确认无 active owner；
4. claim + `status:in-progress`；
5. 读取 `AGENTS.md` / Issue / `task.md`；
6. 只执行当前 Scope；
7. 提交 candidate / Evidence；
8. Issue → `status:review`；
9. 停止，不自动开始下一项。

## 10. Evidence

至少记录：

```text
Role
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

## 11. 推荐指令

### Web Worker

> 读取 `AGENTS.md`、对应 Issue 和 `docs/tasks/<issue>-<slug>/task.md`；claim `env:web-gpt` Task，优先在 Web 完成 Implementation，并通过 GitHub-hosted x64/ARM64 或受控 Ubuntu ARM64 Target Runner 获取 Verification Evidence；只执行当前 Scope，提交后转 `status:review` 并停止。

### 外部 Worker

> 读取 `AGENTS.md`、对应 Issue 和 `docs/tasks/<issue>-<slug>/task.md`；确认 Actions/Runner 无法提供而当前环境正好提供所需交互 capability 后 claim，只执行当前 Scope，提交后转 `status:review` 并停止。
