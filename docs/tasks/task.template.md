# Task — <title>

## Metadata

```text
GitHub Issue: #<number>
Task / Research ID: <id>
Task kind: implementation | verification | combined | research
Base commit: <sha>
Candidate commit: <sha or n/a>
Preferred execution path: web | web+actions | cloud | wsl | windows | ubuntu-arm64 | manual-tv | capability-driven
Eligible environments: env:web-gpt | env:actions | env:cloud | env:wsl | env:windows | env:ubuntu-arm64 | env:manual-tv
Required capabilities: <capability list>
```

> 实时 `status`、assignee、claim、active branch、PR/commit、verification status 和 result summary 只保存在 GitHub Issue，不在本文件重复维护。

## Goal

用一段可验证的陈述说明本 Task 必须完成什么。

## Why / Context

说明它对应哪个产品目标、Research Gate、风险或实现目标，以及为什么现在需要执行。

## Work Role

明确本 Task 的逻辑职责，而不是先绑定环境。

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
- <claim 1>
- <claim 2>
```

Verification 必须针对确定的 candidate commit；如果本 Task 同时负责实现和自动验证，则验证最终提交的 candidate SHA。

## Routing Rationale

默认遵循：

```text
Web Worker first
→ GitHub Actions for automated verification
→ Cloud for long-running / repeated execution
→ WSL / Windows for interactive local capability
→ Ubuntu ARM64 / Real TV only for target proof
```

说明为什么当前 Preferred execution path 足够，或者 Web / Actions 缺少什么 capability 才需要下沉。

不要用“这是代码任务，所以交给 Codex”作为路由理由。

## Preconditions

- 前置 Issue / Research Item：
- 所需服务/设备：
- 所需权限：
- 所需 workflow / runner：
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

### Automated verification

优先判断 GitHub Actions 能否产生有效 Evidence：

```text
Workflow / job:
Runner requirements:
Commands / tests:
Artifacts / logs:
```

仓库尚未具备相应 workflow 时，如果本 Task 已经落地第一个可运行代码/测试，可以把建立最小 CI 纳入 Scope；不要为了流程形式创建无实际测试内容的空 workflow。

### Long-running verification

如果需要长时间或大量重复执行：

```text
Cloud required: yes | no
Duration / repetitions:
Metrics:
```

### Interactive debugging

只有自动化验证不足以高效定位时使用：

```text
WSL / Windows required: yes | no
Reason:
```

### Target verification

只有 claim 依赖真实目标环境时使用：

```text
Target required: none | ubuntu-arm64 | tv | jellyfin-tv | other
Why target evidence is required:
```

## Success Criteria

实现或实验开始前确定，不得根据结果降低标准。

1.

## Evidence Contract

如果需要 runtime / test / device Evidence，至少记录：

```text
Role: implementation | verification
Orchestrator:
Executor:
Execution host:
Target host/device:
OS / architecture:
Relevant versions:
Network path:
Candidate / base commit:
Workflow / run / job (if Actions):
Commands / steps:
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
- 当前执行后端缺 capability 时应路由到哪里；
- 恢复执行的最小条件。

不得：

- 降低 Success Criteria 制造 PASS；
- 用 Actions x86 runner 冒充 ARM64；
- 用 Cloud 冒充家庭 LAN/手机热环境；
- 用桌面/模拟器冒充目标 TV。

## Deliverables

- Implementation / docs：
- Candidate commit / PR：
- Automated verification：
- Long-running / target Evidence：
- Research evidence doc（如适用）：

## Completion Protocol

Worker 完成后：

1. 提交当前 Scope 的候选实现或 Verification Evidence；
2. 在 Issue 中记录 candidate SHA、实际 Executor/Target、实际测试、未验证范围和 artifact/log；
3. 将 Issue 转为 `status:review`；
4. 停止，不自动开始下一项；
5. 由 Web Coordinator 区分 Implementation Result、Verification Result 和 Gate Decision，再决定 `done / ready / blocked`。

需要长期保存正式 Research Result 时写入 `docs/research/`，不要把动态结果重新写回本执行契约。
