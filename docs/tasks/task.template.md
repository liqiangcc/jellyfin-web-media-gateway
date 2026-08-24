# Task — <title>

## Metadata

```text
GitHub Issue: #<number>
Task / Research ID: <id>
Preferred executor: web-worker | external-worker | either
Eligible environments: env:web-gpt | env:windows | env:wsl | env:ubuntu-arm64 | env:cloud | env:manual-tv
Required capabilities: <capability list>
Base commit: <sha>
```

> 实时 `status`、assignee、claimed environment、claimed at、active branch、PR/commit 和 result summary 只保存在 GitHub Issue，不在本文件重复维护。

## Goal

用一段可验证的陈述说明本任务必须完成什么。

## Why / Context

说明为什么现在需要这个任务，它对应哪个 Gate、风险或实现目标。

## Why This Executor / Capability

说明：

- 为什么 Web Worker 可以直接完成；或
- Web Worker 缺少什么 capability，因此必须路由到外部环境；
- 哪些 Evidence 只能由目标环境产生。

默认遵循：

```text
Web-first
→ Web Worker 能产生有效结果时优先 env:web-gpt
→ 否则按 Required Capabilities 路由
```

## Preconditions

- 前置 Issue / Research Item：
- 所需设备/服务：
- 所需权限：
- 开始前检查：

## In Scope

-

## Out of Scope

-

## Architecture Invariants

只列出本任务最相关的不变量；不要复制整份 `AGENTS.md`。

-

## Files Expected to Change

-

## Execution Steps

1.

## Commands / Tests

```text
<commands or test names>
```

明确哪些命令必须真实运行，哪些检查可以由 Web Worker 完成。

## Success Criteria

实验或实现开始前确定，不得根据结果降低标准。

1.

## Evidence to Capture

```text
Executor:
Execution host:
Target host/device:
OS / architecture:
Relevant versions:
Network path:
Base commit:
Commands / steps:
Metrics / raw evidence location:
```

如果本任务不需要 runtime Evidence，明确写：

```text
Runtime evidence required: no
Reason: <why repository/document evidence is sufficient>
```

不得提交 Secret、Cookie、Token、账号数据、完整敏感 URL 或不必要的大文件。

## Failure / Blocked Handling

说明何时记录 FAIL，何时记录 BLOCKED，以及恢复执行所需的最小条件。

如果当前 Worker 缺少 Required Capability：

- 不降低 Success Criteria；
- 不伪造 PASS；
- 在 Issue 中记录 blocker；
- 转 `status:blocked`，或释放回 `status:ready` 供合适环境领取。

## Deliverables

- 代码/文档：
- 测试：
- Evidence：
- commit/PR：

## Completion Protocol

Worker 完成后：

1. 提交当前 Scope 的修改；
2. 在 Issue 中写结果摘要、实际测试/Evidence、未验证范围和 commit/PR；
3. 将 Issue 转为 `status:review`；
4. 停止，不自动开始下一项；
5. 由 Web Coordinator 验收并决定 `done / ready / blocked`。

如果需要长期保存正式 Research Result，写入 `docs/research/`，不要把动态结果重新写回本执行契约。