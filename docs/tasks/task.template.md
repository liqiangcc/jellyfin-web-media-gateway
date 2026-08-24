# Task — <title>

## Metadata

```text
GitHub Issue: #<number>
Parent Goal / Research Item: <goal / Rxxx / issue>
Task / Research ID: <id>
Task kind: implementation | verification | combined | research
Base commit: <sha>
Candidate commit: <sha or n/a>
Preferred worker: web | cloud | wsl | windows | ubuntu-arm64 | manual-tv | capability-driven
Eligible worker environments: env:web-gpt | env:cloud | env:wsl | env:windows | env:ubuntu-arm64 | env:manual-tv
Required capabilities: <capability list>
```

> GitHub Actions / Runner 是 execution backend，不是会 claim Issue 的 Worker，因此不使用 `env:actions` / `env:runner`。
>
> 实时 `status`、assignee、claim、active branch、PR/commit、verification status 和 result summary 只保存在 GitHub Issue，不在本文件重复维护。

## Goal

用一段可验证的陈述说明本 Task 必须完成什么。

## Why / Context

说明它对应哪个产品目标、Research Gate、风险或实现目标，以及为什么现在需要执行。

## Task Decomposition Decision

先说明为什么当前工作应当是一个 Task，还是需要与 Verification 拆开。

```text
Verification mode: inline | separate-task | none
Linked implementation task: <issue/path or n/a>
Linked verification task: <issue/path or n/a>
Decision reason:
```

判断规则：

### `inline`

用于普通工程任务：

```text
Web implementation
→ Candidate SHA
→ standard GitHub Actions CI
→ Coordinator Review
```

标准 fmt/clippy/unit/contract/portable integration/x64/generic ARM64 regression 通常不需要单独 Verification Issue。

### `separate-task`

优先用于：

- 独立 Target / Manual Evidence Authority；
- Verification 生命周期、Owner 或调度时点与 Implementation 不同；
- 关键 Research Gate 要独立追踪实现完成与 claim 证明；
- 目标设备暂不可用但 Implementation 可以先完成；
- 验证需要独立重试、长期运行或多轮 target proof；
- Verification 的 PASS / FAIL / BLOCKED 本身是重要交付结果。

不要因为有 x64、ARM64、手机、TV 多个执行环境就机械拆多个 Task。

## Work Role

### Implementation

如果包含实现工作，说明候选实现必须产生什么：

- 代码/文档变化；
- contract / API / behavior；
- developer checks；
- candidate commit / PR。

### Verification

如果包含验证工作，说明需要证明哪些 claim：

```text
Claims to verify:
- C1: <claim 1>
- C2: <claim 2>
```

Verification 必须针对确定的 candidate commit；如果本 Task 同时负责实现和自动验证，则验证最终 candidate SHA。

## Task vs Job Boundary

本 Task 定义“做什么/证明什么”；Verification Job 只定义“在哪里、怎么执行一个验证切片”。

```text
Task
→ Claims
→ 0..N Jobs
→ Runner / Target
→ Evidence
```

Job 不 claim Issue，不拥有独立业务状态。

如果多个 Runner 验证的是同一个稳定 Claim 集合，应优先放在本 Task 的 Job Matrix 中，而不是按 Runner 创建多个 Task。

## Routing Rationale

默认顺序：

```text
Implementation:
Web Worker first
→ 只有缺少交互/设备能力才使用外部 Worker

Verification:
GitHub Actions first
→ GitHub-hosted x64 for portable/fast verification
→ GitHub-hosted ARM64 for generic ARM64 verification
→ Ubuntu ARM64 self-hosted runner only for phone-specific target proof
→ Manual TV for physical UX proof
→ External Codex only when interactive diagnosis/control is needed
```

Cloud **不作为 Runner**。只有 GitHub Actions 不适合表达且确实需要 Cloud 主机能力/长期交互状态/Tailscale remote orchestration 时，才把 Cloud 当外部 Worker 使用。

说明为什么当前路径足够，或者缺少什么 capability 才需要下沉。

## Preconditions

- 前置 Issue / Research Item：
- 所需服务/设备：
- 所需权限：
- 所需 workflow：
- 所需 runner / image / labels：
- 开始前检查：

## In Scope

-

## Out of Scope

-

## Architecture Invariants

只列当前 Task 最相关的不变量，不复制整份 `AGENTS.md`。

-

## Files Expected to Change

-

## Implementation Requirements

如果 `Task kind` 包含 implementation：

1.

如果不包含，写 `N/A`。

## Verification Plan

验证计划在执行前确定。

### Claims

```text
C1:
C2:
```

### Verification Job Matrix

一个 Verification Task 可以有多个 Job；按 Claim 与 Required Capability 设计，不按现有机器倒推 Task。

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1 | github-actions | github-hosted-x64 | runner-self | yes | <...> | run/log |
| J2 | C1,C2 | github-actions | github-hosted-arm64 | generic-arm64 | no/yes | <...> | run/log |
| J3 | C2 | github-actions | ubuntu-arm64-self-hosted | ubuntu-arm64-phone | no/yes | <...> | metrics/artifact |

只有 Claim / Success Criteria / lifecycle 真正不同，才拆新的 Verification Task。

### Execution Plane

```text
Execution plane: github-actions | external-codex | manual | none
```

只要能自动化，优先 `github-actions`。

### Runner Selection

当 `Execution plane = github-actions` 时：

```text
Runner class: github-hosted-x64 | github-hosted-arm64 | ubuntu-arm64-self-hosted
Runner image / labels: <image or labels>
Target: runner-self | generic-arm64 | ubuntu-arm64-phone | other
Trust gate: normal-ci | trusted-candidate-only | manual-approval
```

路由原则：

```text
portable x64 build/test/lint
→ github-hosted-x64

generic ARM64 build/test
→ github-hosted-arm64

ARM64 phone runtime / metrics / target-specific claim
→ ubuntu-arm64-self-hosted
```

大量重复测试优先使用 GitHub-hosted matrix/sharding，不建立 Cloud Runner。

### Automated verification

```text
Workflow / job:
Commands / tests:
Duration / repetitions / shards:
Artifacts / logs:
```

仓库尚未具备相应 workflow 时，如果本 Task 已经落地第一个可运行代码/测试，可以把建立最小真实 CI 纳入 Scope；不要创建无实际测试内容的空 workflow。

### Long-running / repeated verification

优先判断能否：

```text
GitHub-hosted matrix / shard / repeated jobs
```

如果 claim 要求同一进程连续运行，不能用分片伪装连续 soak。此时说明真实持续时长要求，并按 claim 选择足够的执行环境；若该验证生命周期与实现显著不同，优先拆独立 Verification Task。

Cloud 资源有限，不作为默认 long-running backend。

### Interactive debugging

只有自动化验证不足以高效定位时使用：

```text
WSL / Windows / Cloud / Ubuntu ARM64 external Codex required: yes | no
Reason:
```

交互式调试产生的是诊断能力，不自动替代最终 Verification Evidence。

### Target verification

只有 claim 依赖真实目标环境时使用：

```text
Target proof required: yes | no
Target: ubuntu-arm64 | tv | jellyfin-tv | other
Why target evidence is required:
```

### Runner Security Constraints

使用 Ubuntu ARM64 self-hosted runner 时明确：

```text
Trusted candidate only: yes | no
Dedicated low-privilege runner user: required
Vault/profile access: forbidden unless explicitly scoped
Production service mutation: forbidden unless explicitly in scope
Cleanup / timeout requirements:
```

Target Runner 默认不得读取 Gateway Vault、真实 Cookie/profile、宿主 root credential 或其他长期 Secret。

详细规则见 `docs/runner-execution-architecture.md`。

## Success Criteria

实现或实验开始前确定，不得根据结果降低标准。

### Task success

1.

### Verification claim success

```text
C1 PASS when:
C2 PASS when:
```

如果 Verification 是独立 Task，Implementation Task 的成功标准不能写成“所有 target verification PASS”；它只负责候选实现交付。Parent Goal / Research Gate 由 Coordinator 汇总独立 Task 结果后判定。

## Evidence Contract

如果需要 runtime / test / device Evidence，至少记录：

```text
Role: implementation | verification
Task / Claim:
Job ID:
Orchestrator:
Execution plane:
Executor:
Runner class / image / labels (if Actions):
Execution host:
Target host/device:
OS / architecture:
Relevant versions:
Network path:
Candidate / base commit:
Workflow / run / job (if Actions):
Commands / steps:
Duration / repetitions / shards:
Metrics / artifact / raw evidence location:
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

如果不需要 runtime Evidence：

```text
Runtime evidence required: no
Reason: <why repository/static evidence is sufficient>
```

不得提交 Secret、Cookie、Token、账号数据、完整敏感 URL 或不必要的大文件。

## Failure / Blocked Handling

说明：

- 什么结果属于 FAIL；
- 什么缺失属于 BLOCKED；
- 当前 Worker / Execution Plane / Runner 缺 capability 时应路由到哪里；
- 是否只阻塞 Verification Task，还是同时阻塞 Parent Goal；
- 恢复执行的最小条件。

不得：

- 降低 Success Criteria 制造 PASS；
- 用 GitHub-hosted generic ARM64 冒充目标手机；
- 用目标手机 Runner 承载不需要 target proof 的普通 CI；
- 用 Cloud host 冒充手机温度/家庭 LAN；
- 用桌面/模拟器冒充目标 TV。

## Deliverables

- Implementation / docs：
- Candidate commit / PR：
- Linked verification task（如分离）：
- Verification Jobs / runs：
- Target Evidence：
- Research evidence doc（如适用）：

## Completion Protocol

Worker 完成后：

1. 提交当前 Scope 的候选实现或 Verification Evidence；
2. 在 Issue 中记录 candidate SHA、linked task、实际 Jobs / Orchestrator / Execution Plane / Runner / Target、实际测试、未验证范围和 artifact/log；
3. 将当前 Issue 转为 `status:review`；
4. 停止，不自动开始下一项；
5. 由 Web Coordinator 区分 Implementation Result、Verification Result 和 Parent Goal / Gate Decision，再决定 `done / ready / blocked`。

需要长期保存正式 Research Result 时写入 `docs/research/`，不要把动态结果重新写回本执行契约。
