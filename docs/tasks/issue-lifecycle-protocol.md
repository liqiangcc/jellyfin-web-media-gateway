# Issue Feedback / Review / Iteration / Closure Protocol

本文件定义独立 Task 从发布、领取、执行、反馈、Review、修订、复测、阻塞、解阻到最终关闭的统一闭环协议。

核心原则：

> **Issue 是 Task 的实时协调状态与 append-only 执行历史；`task.md` 是稳定执行契约；`prompt.md` 是会话启动入口；Worker 报告结果，Coordinator 决定是否接受。**

---

## 1. Authority

```text
canonical docs = 产品 / 架构 / 安全事实
AGENTS.md = 长期 Agent 规则
task.md = 当前 Task 稳定执行契约
prompt.md = 会话 bootstrap / navigation only
GitHub Issue fields / labels = 当前状态快照
GitHub Issue comments = Attempt / Blocker / Review / Acceptance 历史
PR / commit / Actions / artifact / research doc = Evidence
```

Issue Comment 可以记录执行结果和协调决定，但不能通过评论静默重定义 architecture、Scope、Claims 或 Success Criteria。改变 Task Contract 必须更新 `task.md`；影响 canonical architecture/security/requirements 时先更新对应 canonical docs。

## 2. Task Lifecycle

```text
status:draft
→ Publication Gate PASS → status:ready
→ Worker claim → status:in-progress
   ├─ authorized [EXECUTION REPORT] → status:review → Coordinator Review
   └─ authorized [BLOCKER REPORT] → status:blocked → Coordinator UNBLOCK → ready
Coordinator ACCEPT → [FINAL ACCEPTANCE] → status:done → close
Coordinator REVISE → status:ready → next Attempt
Coordinator SPLIT → child Tasks
```

`status:done` 必须紧邻最终关闭，仅由 Coordinator 在 Final Acceptance 后设置。Worker 不得自行将 Task 设为 `done` 或关闭 Issue。

## 3. Attempt

每次成功的 `status:ready → status:in-progress` + claim 都开始一个新的递增 Attempt。Attempt 是可审计单元：claim → execution → candidate/evidence → report → review。Goal/Scope/Claims/Success Criteria 未变时，失败优先留在同一 Issue 进入下一 Attempt。

## 3.1 Fresh terminal-write authority guard

Claim-time authority 不能授权 Worker 在未来任意时刻写 terminal state。Worker 在以下任何 terminal mutation sequence 的**第一项不可逆写入之前**，必须重新读取 live Issue：

- `[EXECUTION REPORT]` + `status:review` + owner release；
- `[BLOCKER REPORT]` + `status:blocked` + owner release。

Fresh snapshot 只有同时满足以下条件才授权写入：

```text
Issue OPEN
status:in-progress current
Attempt == current Worker Attempt
active owner/claim == current Worker
status:done absent
no durable [FINAL ACCEPTANCE]
no newer Coordinator gate / Attempt supersedes this Worker
```

任一条件失败或 authority 有歧义时必须 **fail closed**：不得发 terminal report、不得改 status、不得 release/reassign owner、不得 reopen Issue；立即以 `STALE_AUTHORITY` 停止。旧 Worker 尤其不得“清理”可能已经属于新 Attempt 的 owner。

这是 last-safe-point guard，不声称 GitHub 多操作原子性。如果在较早写入后已经知道 authority 失效，后续 status/owner mutation 必须停止，由 Coordinator reconcile。Coordinator Final Acceptance/close 不属于 Worker terminal write，继续由 Final Acceptance Gate 管理。

Repository-owned pure decision helper: `scripts/task-worker-terminal-guard.py`. 它只判断 freshly fetched normalized snapshot，不执行 GitHub mutation，也不能替代 live GitHub read。

## 4. Worker Feedback Rule

Worker 在 Attempt 结束时必须先通过 3.1 guard。只有 PASS 才能写结果并改变状态。

正常结束：

```text
prepare [EXECUTION REPORT]
→ fresh terminal authority guard
→ post report
→ status:review
→ release active execution ownership
→ durable read-back
→ STOP
```

阻塞：

```text
prepare [BLOCKER REPORT]
→ fresh terminal authority guard
→ post report
→ status:blocked
→ release active execution ownership unless explicitly retained by protocol
→ durable read-back
→ STOP
```

Guard REJECT 时两个流程都变为：`STALE_AUTHORITY → zero Issue mutations → STOP`。

Worker 不等待聊天里的隐式 Review，也不自动开始下一 Attempt。不得降低 Success Criteria、绕过安全边界或把未验证结果写成 PASS 来逃避 blocker。

## 5. Worker Result vs Verification Result vs Coordinator Decision

```text
Worker execution outcome != Verification claim result != Coordinator Task decision != Parent Goal / Research Gate decision
```

Worker outcome: `COMPLETED | PARTIAL | FAILED | BLOCKED`.
Verification Claim: `PASS | CONDITIONAL PASS | FAIL | BLOCKED | NOT RUN`.
Coordinator decision only: `ACCEPT | REVISE | BLOCK | SPLIT | NOT_PLANNED`.
Worker `COMPLETED` / Claim `PASS` 不等于 Task ACCEPT。

## 6. Execution Report

```text
[EXECUTION REPORT]
Attempt: <N>
Worker: <web | wsl | windows | cloud | ubuntu-arm64 | manual-tv>
Environment: env:<environment>
Role: implementation | verification | combined | research
Base commit: <sha>
Candidate commit: <sha or n/a>
PR: <number/url or n/a>
Execution outcome: COMPLETED | PARTIAL | FAILED
Implementation result:
- <what changed>
Verification claim results:
- C1: PASS | CONDITIONAL PASS | FAIL | BLOCKED | NOT RUN
Jobs / commands:
- <workflow/run/job or command>
Evidence:
- <run/artifact/log/research/device observation>
Problems found:
- <problem or none>
Unverified / limitations:
- <item or none>
Suggested next action:
- <optional; Coordinator decides>
```

必须填写实际 Candidate/run/environment；不能用理论判断替代 runtime Evidence。Report posting is subject to 3.1.

## 7. Blocker Report

```text
[BLOCKER REPORT]
Attempt: <N>
Worker: <worker>
Environment: env:<environment>
Blocked at: <step/claim/job>
Completed before blocker:
- <item>
Blocker:
- <exact missing condition/failure>
Evidence:
- <bounded evidence>
Minimal resume condition:
- <condition>
Cleanup / safe state:
- <state>
Reusable durable anchor:
- <branch/commit/PR/evidence or n/a>
```

Blocker posting is subject to 3.1. A stale Worker does not post a blocker merely to announce that it