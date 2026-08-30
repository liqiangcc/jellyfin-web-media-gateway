# Issue Feedback / Review / Iteration / Closure Protocol

本文件定义独立 Task 从发布、领取、执行、反馈、Review、修订、复测、阻塞、解阻到最终关闭的统一闭环协议。

核心原则：

> **Issue 是 Task 的实时协调状态与 append-only 执行历史；`task.md` 是稳定执行契约；`prompt.md` 是会话启动入口；Worker 报告结果，Coordinator 决定是否接受。**

---

## 1. Authority

```text
canonical docs
= 产品 / 架构 / 安全事实

AGENTS.md
= 长期 Agent 规则

task.md
= 当前 Task 稳定执行契约

prompt.md
= 会话 bootstrap / navigation only

GitHub Issue fields / labels
= 当前状态快照

GitHub Issue comments
= Attempt / Blocker / Review / Acceptance 历史

PR / commit / Actions / artifact / research doc
= Evidence
```

Issue Comment 可以记录执行结果和协调决定，但不能通过评论静默重定义 architecture、Scope、Claims 或 Success Criteria。

如果需要改变 Task Contract，必须更新 `task.md`；如果影响 canonical architecture / security / requirements，则先更新对应 canonical docs。

---

## 2. Task Lifecycle

标准状态机：

```text
status:draft
   ↓ Task Publication Gate PASS
status:ready
   ↓ Worker claim / Attempt N starts
status:in-progress
   │
   ├── Worker Execution Report
   │       ↓
   │   status:review
   │       │
   │       ├── Coordinator ACCEPT
   │       │      ↓
   │       │   status:done
   │       │      ↓
   │       │   close Issue
   │       │
   │       ├── Coordinator REVISE
   │       │      ↓
   │       │   status:ready
   │       │      ↓
   │       │   Attempt N+1
   │       │
   │       ├── Coordinator BLOCK
   │       │      ↓
   │       │   status:blocked
   │       │      ↓ blocker resolved
   │       │   status:ready
   │       │
   │       └── Coordinator SPLIT
   │              ↓ child Task(s)
   │              ↓ required Evidence returns
   │              ↓ review / ready / accept
   │
   └── Worker Blocker Report
           ↓
       status:blocked
           ↓ Coordinator UNBLOCK
       status:ready
```

`status:done` 必须紧邻最终关闭，仅由 Coordinator 在 Final Acceptance 后设置。

Worker 不得自行将 Task 设为 `done` 或关闭 Issue。

---

## 3. Attempt

每次从：

```text
status:ready → status:in-progress
```

并成功 claim，都开始一个新的 Attempt。

Attempt 使用递增编号：

```text
Attempt: 1
Attempt: 2
Attempt: 3
```

Attempt 的目标不是保证成功，而是形成一个可审计的执行单元：

```text
claim
→ execution
→ candidate / evidence
→ report
→ review decision
```

同一个 Task 可以经过多个 Attempt，直到 Success Criteria 被接受或 Coordinator 明确终止。

不要因为一次失败就机械创建新 Issue；如果 Goal、Scope、Claims 和 Success Criteria 没变，优先在同一 Issue 中继续下一 Attempt。

---

## 3.1. Fresh terminal-write authority guard

Worker 的 claim-time read-back 不能作为之后 terminal write 的永久 authority。就在 Worker **每一个**不可逆 terminal mutation 之前，必须重新读取 live Issue。terminal mutation 包括 `[EXECUTION REPORT]` / `[BLOCKER REPORT]` comment、`status:review` / `status:blocked` 变更和 owner release/reassignment。

只有 fresh snapshot 同时证明以下条件时才允许当前 mutation：

```text
Issue is open
status:in-progress is current
Attempt still matches this Worker
active owner still matches this Worker
status:done is absent
no [FINAL ACCEPTANCE] newer than the current claim/checkpoint authority exists
no newer Coordinator gate / Attempt supersedes this Worker
```

历史 `[FINAL ACCEPTANCE]` 如果早于明确的 `[COORDINATOR REOPEN]` 和当前 fresh claim，本身不会阻塞新的合法 Attempt。时间顺序或 authority 无法无歧义确认时必须 fail closed。

如果 guard 拒绝：

```text
STALE_AUTHORITY
→ do not perform the pending terminal mutation
→ no reopen
→ STOP
```

如果 stale 是在 report 已经 append-only 写入后才出现，则保留该历史 comment，但不得继续后续 status/owner write。特别地，stale Worker 不得以“cleanup”为由释放可能已经属于新 Attempt 的 owner。

这是 repeated last-safe-point guard，不是 distributed lock，也不声称 GitHub 多次 mutation 具有原子性。Coordinator 的 Final Acceptance / close 仍只受第 13 节约束。

## 4. Worker Feedback Rule

Worker 在 Attempt 结束时必须按 3.1 在每个 terminal mutation 前 fresh-read authority；只有当前 mutation 的 guard PASS 才能继续。

### 4.1 正常结束

```text
execute
→ prepare [EXECUTION REPORT]
→ fresh guard → post [EXECUTION REPORT]
→ fresh guard → status:review
→ fresh guard → release active execution ownership
→ STOP
```

Worker 不等待聊天里的隐式 Review，也不自动开始下一 Attempt。

### 4.2 阻塞

如果缺少权限、设备、Secret、Runner、外部条件或必需 capability，不能继续：

```text
prepare [BLOCKER REPORT]
→ fresh guard → post [BLOCKER REPORT]
→ fresh guard → status:blocked
→ fresh guard → release active execution ownership unless explicitly resolving blocker
→ STOP
```

不得通过降低 Success Criteria、绕过安全边界或把未验证结果写成 PASS 来逃避 blocker。

---

## 5. Worker Result vs Verification Result vs Coordinator Decision

必须始终区分：

```text
Worker execution outcome
!= Verification claim result
!= Coordinator Task decision
!= Parent Goal / Research Gate decision
```

Worker 可以报告：

```text
Execution outcome:
COMPLETED | PARTIAL | FAILED | BLOCKED
```

Verification Claim 可以报告：

```text
PASS | CONDITIONAL PASS | FAIL | BLOCKED | NOT RUN
```

只有 Coordinator 可以给 Task Review Decision：

```text
ACCEPT | REVISE | BLOCK | SPLIT | NOT_PLANNED
```

Worker 写 `COMPLETED` 或某些 Claims `PASS` 不等于整个 Task 已 ACCEPT。

---

## 6. Execution Report

正常 Attempt 结束使用以下 Issue Comment：

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
- <what was implemented / changed>

Verification claim results:
- C1: PASS | CONDITIONAL PASS | FAIL | BLOCKED | NOT RUN
- C2: ...

Jobs / commands:
- <workflow/run/job or command selector>

Evidence:
- <run / artifact / log / research doc / device observation>

Problems found:
- <problem or none>

Unverified / limitations:
- <item or none>

Suggested next action:
- <optional recommendation; Coordinator decides>
```

必须填写实际 Candidate SHA / run / environment；不能用理论判断替代 runtime Evidence。

---

## 7. Blocker Report

阻塞使用：

```text
[BLOCKER REPORT]

Attempt: <N>
Worker: <worker>
Environment: env:<environment>

Blocked at:
- <step / claim>

Missing capability / dependency:
- <what is unavailable>

Completed before blocker:
- <what is safely complete>

Evidence:
- <logs / command output / run / observation>

Required to resume:
- <minimal concrete condition>

Safe state / cleanup:
- <what was cleaned or intentionally left>

Result: BLOCKED
```

如果 blocker 本身需要独立工作和生命周期，Coordinator 可以 SPLIT 出基础设施 / 修复 Task；原 Task 记录 linked blocker Task。

---

## 8. Coordinator Review

Coordinator 必须读取：

- 当前 Issue 与全部 relevant comments；
- 当前 `task.md`；
- Candidate commit / PR；
- Required Actions run / artifact / target Evidence；
- linked Task Evidence（如有）。

然后在 Issue 评论：

```text
[COORDINATOR REVIEW]

Review of Attempt: <N>

Decision: ACCEPT | REVISE | BLOCK | SPLIT | NOT_PLANNED

Accepted:
- <criteria / claims accepted>

Failed / missing:
- <criteria / evidence missing>

Required changes:
1. <change>
2. <change>

Contract change required: yes | no
Canonical doc change required: yes | no

Linked child/blocker tasks:
- <issue or n/a>

Next state:
- status:done | status:ready | status:blocked

Next attempt:
- <N+1 or n/a>

Expected worker / capability:
- <env / capabilities or n/a>
```

Coordinator 的 Review 必须写入 Issue，不能只留在聊天中。

---

## 9. REVISE：什么时候不改 task.md

如果发现的是：

- 实现 bug；
- 测试失败；
- 漏实现既定要求；
- Evidence 不足；
- candidate 需要修正；
- 同一 Claim 需要重新验证；

则：

```text
task.md unchanged
prompt.md unchanged
→ Coordinator Review = REVISE
→ status:ready
→ no active owner
→ publish downstream entry for next Attempt
```

下一 Worker 读取 Issue 历史即可知道上一轮失败原因。

---

## 10. Contract Revision：什么时候必须改 task.md

如果 Review 发现：

- Scope 本身错误或缺失；
- Claims 需要改变；
- Success Criteria 需要合法重定义；
- Task decomposition decision 改变；
- Required Evidence Authority 改变；
- Architecture / security 前提被真实 Evidence 推翻；

则不能只在 Issue 评论里修改要求。

必须：

```text
Coordinator
→ status:draft (when contract is not executable as published)
→ update canonical docs when required
→ update task.md
→ update prompt.md only if bootstrap changed
→ read-back verify
→ status:ready
→ queue verify
→ output a new downstream handoff entry
```

禁止为了让已有结果通过而事后降低 Success Criteria。

---

## 11. UNBLOCK

Blocker 被解决后，Coordinator 评论：

```text
[COORDINATOR UNBLOCK]

Blocker from Attempt: <N>

Resolved:
- <what changed>

Evidence / linked task:
- <reference>

Resume condition satisfied: yes

Resume from:
- <step / claim>

Next attempt: <N+1>
Next state: status:ready
Expected worker: <worker / capability>
```

然后保持无 active owner，并重新给出下游执行入口。

如果 blocker 解决改变了 Task Contract，则先走第 10 节 Contract Revision，而不是直接 `ready`。

---

## 12. SPLIT 与父子 Task

Coordinator 只有在新的工作具有独立：

- Scope；
- Owner / lifecycle；
- Success Criteria；
- Evidence Authority；
- 或明确的独立交付物；

时才创建 child Task。

不要因为不同 Runner / 环境而 SPLIT。

父 Issue 评论必须记录：

```text
[SPLIT]

Reason:
- <why this is a separate Task, not a Job>

Child Task(s):
- #<issue> <purpose>

Parent blocked by child: yes | no
Required Evidence to return:
- <what the parent needs>
```

如果 parent 必须等待 child 才能继续，设置 `status:blocked`；child 完成后 Evidence 回流父 Issue，再由 Coordinator Review。

---

## 13. Final Acceptance / Close Gate

Issue 只能在以下全部成立时关闭：

```text
Task Success Criteria accepted
+
all required Claims accepted
+
required Verification Evidence reviewed
+
required Candidate / PR accepted
+
no unresolved blocker
+
no required linked child Task still open
+
Coordinator Final Acceptance comment posted
```

Final comment：

```text
[FINAL ACCEPTANCE]

Task: <task id / title>
Accepted candidate: <sha or n/a>
Accepted PR: <pr or n/a>

Accepted attempts:
- Attempt <N>: <summary>

Success Criteria:
- SC1: ACCEPTED
- SC2: ACCEPTED

Required Claims / Verification:
- C1: PASS / accepted evidence
- C2: PASS / accepted evidence

Known remaining limitations:
- <non-blocking limitation or none>

Linked tasks:
- <state / n/a>

Parent Goal / Research Gate impact:
- <what this Task completion does and does not prove>

Decision: ACCEPT
Final state: status:done
Issue close reason: completed
```

顺序：

```text
post [FINAL ACCEPTANCE]
→ status:done
→ close Issue as completed
```

`Task Issue closed` 不自动等于 Parent Goal / Research Gate PASS。Parent 由 Coordinator 根据其 required Tasks / Claims 单独决定。

---

## 14. Reopen

如果关闭后发现新 Evidence 直接否定了本 Task 已接受的 Success Criteria，可以由 Coordinator reopen：

```text
[COORDINATOR REOPEN]

Reason:
- <new contradictory evidence>

Previously accepted evidence affected:
- <claim / criterion>

Contract change required: yes | no
Next state: status:draft | status:ready
Next attempt: <N+1>
```

如果只是出现一个新的、不同 Scope 的需求，不应复用旧 Issue；创建新 Task 并链接原 Issue。

---

## 15. Append-only History

Issue comments 默认视为 append-only 历史。

- 不删除旧 Attempt / Review 来“整理状态”；
- 重大纠正使用新的 `[CORRECTION]` 评论并引用被纠正 Attempt / Review；
- Issue body / labels 保存当前快照；
- comments 保存发生过什么以及为什么改变；
- `task.md` 不记录动态 Attempt 结果；
- `prompt.md` 不记录执行结果。

目标是让任何新的 Coordinator / Worker 只通过 GitHub 就能重建：

```text
Task Contract
→ Attempts
→ Evidence
→ Failures / Blockers
→ Review Decisions
→ Final Acceptance
```

而不依赖旧聊天。

---

## 16. End-to-End Closed Loop

```text
Web Coordinator
→ publish Task
→ Publication Gate PASS
→ downstream entry

Worker
→ claim
→ Attempt N
→ Issue Execution / Blocker Report
→ review / blocked
→ STOP

Coordinator
→ read Issue + Candidate + Evidence
→ ACCEPT / REVISE / BLOCK / SPLIT

REVISE
→ status:ready
→ downstream entry
→ Attempt N+1

BLOCK
→ resolve blocker
→ UNBLOCK
→ status:ready
→ downstream entry

SPLIT
→ child Task(s)
→ child Evidence returns
→ parent Review

ACCEPT
→ Final Acceptance
→ status:done
→ close Issue
```

最终目标：

> **聊天只负责操作和解释；GitHub Issue 保存实时协调与迭代历史；Task 可以跨多个 Worker / 会话反复执行和验证，直到 Success Criteria 真正满足并由 Coordinator 关闭。**
