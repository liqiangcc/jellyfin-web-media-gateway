# Task — <title>

## Metadata

```text
GitHub Issue: #<number>
Parent Goal / Research Item: <goal / Rxxx / issue>
Task / Research ID: <id>
Task kind: implementation | verification | combined | research
Base commit: <sha>
Candidate commit: <sha or n/a>
Session bootstrap prompt: docs/tasks/<issue>-<slug>/prompt.md | n/a
Preferred worker: cloud | web | wsl | windows | ubuntu-arm64 | manual-tv | capability-driven
Eligible worker environments: env:cloud | env:web-gpt | env:wsl | env:windows | env:ubuntu-arm64 | env:manual-tv
Required capabilities: <capability list>
Hard publication dependencies: <none or explicit dependencies>
```

> GitHub Actions / Runner 是 execution backend，不是会 claim Issue 的 Worker，因此不使用 `env:actions` / `env:runner`。
>
> 实时 `status`、assignee、claim、active branch、PR/commit、verification status 和 result summary 只保存在 GitHub Issue，不在本文件重复维护。
>
> Attempt / Blocker / Coordinator Review / Final Acceptance 使用 Issue comments，格式见 `docs/tasks/issue-lifecycle-protocol.md`。

## Session Bootstrap

如果本 Task 会进入 `status:ready` 并由独立 Worker 新会话领取，从：

```text
docs/tasks/prompt.template.md
```

生成同目录 `prompt.md`。

`prompt.md` 只负责：

- 指向 GitHub Issue；
- 指向本 `task.md`；
- 声明预期 Worker / environment / handoff profile；
- 提醒读取 `AGENTS.md`、Issue history 和相关 canonical docs；
- 提醒 claim / `status:in-progress` / feedback / `status:review` / stop 协议；
- 提供不会改变 Scope 的最少启动提醒。

`prompt.md` **不得复制或重新定义**本文件中的 Goal、Scope、Claims、Success Criteria、Architecture Invariants、Verification Job Matrix 或 Evidence 判断标准。

如果 Prompt 与本 Task Contract、`AGENTS.md` 或 canonical docs 冲突，Prompt 的冲突内容无效。

## Goal

用一段可验证的陈述说明本 Task 必须完成什么。

## Why / Context

说明它对应哪个产品目标、Research Gate、风险或实现目标，以及为什么现在需要执行。

## Task Decomposition Decision

```text
Verification mode: inline | separate-task | none
Linked implementation task: <issue/path or n/a>
Linked verification task: <issue/path or n/a>
Decision reason:
```

### `inline`

普通工程任务：

```text
Codex implementation
→ Candidate SHA
→ standard GitHub Actions CI
→ Coordinator Review
```

标准 fmt/clippy/unit/contract/portable integration/x64/generic ARM64 regression 通常不需要单独 Verification Issue。

### `separate-task`

优先用于：

- 独立 Target / Manual Evidence Authority；
- Verification 生命周期、Owner 或调度时点与 Implementation 不同；
- 关键 Research Gate 要独立追踪实现完成与 Claim 证明；
- 目标设备暂不可用但 Implementation 可以先完成；
- 验证需要独立重试、长期运行或多轮 target proof；
- Verification 的 PASS / FAIL / BLOCKED 本身是重要交付结果；
- 受信 target workflow/harness 必须先 Review/merge，随后再允许 self-hosted target execution。

不要因为有 Cloud/x64/ARM64/手机/TV 多个环境就机械拆多个业务 Task。

## Worker Routing Decision

先按 capability 选择 Worker/client，再选择 Verification Runner。

默认：

```text
ordinary repository implementation / fix / refactor / test/workflow authoring / PR integration
→ cloud-codex
→ env:cloud

GitHub-only lightweight fallback / Coordinator explicit Web choice
→ web
→ env:web-gpt

local Linux-specific interactive
→ wsl
→ env:wsl

Windows / ADB
→ windows
→ env:windows

target-phone interactive install/recovery/debug
→ ubuntu-arm64
→ env:ubuntu-arm64

physical TV / remote / audible observation
→ manual-tv
→ env:manual-tv
```

Capability wins over preference。Codex-first 不意味着把 Windows/ADB、目标手机现场能力或 TV 人工 Evidence 塞进 generic Cloud。

## Work Role

### Implementation

如果包含实现工作，说明候选实现必须产生什么：

- 代码/文档变化；
- contract / API / behavior；
- developer checks；
- candidate commit / PR；
- 如果前一 Attempt 已有 Candidate/PR，说明是否必须继续/rebase/复用。

### Verification

如果包含验证工作，说明需要证明哪些 Claim：

```text
Claims to verify:
- C1: <claim 1>
- C2: <claim 2>
```

Verification 必须针对确定的 Candidate SHA；如果实现变化，旧 Evidence 不自动证明新 Candidate。

## Task vs Job Boundary

```text
Task
→ Claims
→ Required Capabilities
→ Worker/client
→ 0..N Verification Jobs
→ Runner / Target
→ Evidence
```

Job 不 claim Issue，不拥有独立业务状态。

如果多个 Runner 验证的是同一个稳定 Claim 集合，应优先放在本 Task 的 Job Matrix 中，而不是按 Runner 创建多个 Task。

## Routing Rationale

默认顺序：

```text
Implementation / repository work:
Codex Cloud first
→ only use Web/WSL/Windows/Ubuntu ARM64 when Task capability requires it

Verification:
GitHub Actions first
→ GitHub-hosted x64 for portable/fast verification
→ GitHub-hosted ARM64 for generic ARM64 verification
→ Ubuntu ARM64 self-hosted runner only for phone-specific target proof
→ Manual TV for physical UX proof
```

Cloud **不作为 Runner**。Worker 是 Codex Cloud 时，重 build/test/benchmark 仍优先通过 GitHub Actions。

## Preconditions

- 前置 Issue / Research Item：
- hard dependency：
- 所需服务/设备：
- 所需权限：
- 所需 workflow：
- 所需 runner / image / labels：
- 开始前检查：
- 需要复用的 existing candidate/PR（live reference 从 Issue 读取）：

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

### Claims

```text
C1:
C2:
```

### Verification Job Matrix

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

只要能自动化且 required Evidence 不依赖交互式 host，优先 `github-actions`。

### Runner Selection

```text
portable x64 build/test/lint
→ github-hosted-x64

generic ARM64 build/test
→ github-hosted-arm64

ARM64 phone runtime / metrics / target-specific claim
→ ubuntu-arm64-self-hosted
```

大量重复测试优先使用 GitHub-hosted matrix/sharding，不在 Codex Cloud shell 中硬跑。

### Long-running / repeated verification

如果 Claim 要求同一进程连续运行，不能用分片伪装连续 soak；按 Claim 选择真实环境。若生命周期与实现显著不同，拆独立 Verification Task。

### Interactive debugging

只有自动化验证不足以高效定位时使用：

```text
WSL / Windows / Ubuntu ARM64 external Codex required: yes | no
Reason:
```

交互式调试产生的是诊断能力，不自动替代最终 Verification Evidence。

### Target verification

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

如果 Verification 是独立 Task，Implementation Task 的成功标准不能写成“所有 target verification PASS”；Parent Goal / Research Gate 由 Coordinator 汇总独立 Task 结果后判定。

## Evidence Contract

如果需要 runtime / test / device Evidence，至少记录：

```text
Role: implementation | verification
Task / Claim:
Attempt:
Worker / Orchestrator:
Job ID:
Execution plane:
Runner class / image / labels (if Actions):
Execution host:
Target host/device:
OS / architecture:
Relevant versions:
Network path:
Base / Candidate commit:
Workflow / run / job (if Actions):
Commands / steps:
Duration / repetitions / shards:
Metrics / artifact / raw evidence location:
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
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
- Session bootstrap prompt：
- Linked verification task（如分离）：
- Verification Jobs / runs：
- Target Evidence：
- Research evidence doc（如适用）：

## Issue Feedback / Iteration Protocol

完整协议见：

```text
docs/tasks/issue-lifecycle-protocol.md
```

```text
ready
→ claim / Attempt N
→ in-progress
→ Execution Report / Blocker Report
→ review / blocked
→ Coordinator ACCEPT / REVISE / BLOCK / SPLIT
→ next Attempt or Final Acceptance
```

如果只是实现 bug、integration/rebase、测试失败、漏实现、Evidence 不足或同一 Claim 重测，不创建新 Issue；优先同一 Task 下一 Attempt。

如果 Worker session 卡死但已有 durable branch/PR/Evidence，Coordinator 释放 stale ownership 并让下一 Worker 继续同一 Issue；不要从零重建。

如果 Scope、Claims、Success Criteria、Task decomposition、Evidence Authority、architecture/security 前提或 eligible Worker/environment routing 本身改变，则正式更新 Contract / prompt 并重新 Publication Gate。

## Completion Protocol

Worker 每次 Attempt 结束：

1. 提交当前 Scope 的候选实现或 Verification Evidence；
2. 正常结束在 Issue 评论 `[EXECUTION REPORT]`；
3. 阻塞评论 `[BLOCKER REPORT]`；
4. 正常结束 → `status:review`；阻塞 → `status:blocked`；
5. 释放 active execution ownership；
6. 停止，不自动开始下一项。

Web Coordinator Review：

1. 读取 Issue history、本 `task.md`、candidate/PR 和所有 required Evidence；
2. 评论 `[COORDINATOR REVIEW]`；
3. 明确 `ACCEPT | REVISE | BLOCK | SPLIT | NOT_PLANNED`；
4. `REVISE` 且 Contract 不变时 → `status:ready` → 输出下一轮 environment-specific entry；
5. Contract/routing 改变时 → `status:draft` → 更新 docs/task/prompt → Publication Gate；
6. 只有所有 Task Success Criteria、required Claims/Evidence、candidate/PR、blocker 和 required child Tasks 满足后，评论 `[FINAL ACCEPTANCE]`；
7. Final Acceptance 后才设置 `status:done` 并关闭 Issue as completed。

Worker 不得自行 `done` 或关闭 Issue。