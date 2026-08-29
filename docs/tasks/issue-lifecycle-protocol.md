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

## 3.1 Fresh terminal-write authority guard

Claim-time authority is not sufficient for a later terminal write. Immediately before a Worker posts `[EXECUTION REPORT]` / `[BLOCKER REPORT]` or begins the coupled `status:review` / `status:blocked` + owner-release sequence, it MUST freshly read the live Issue.

The terminal sequence is authorized only while the Issue is open, `status:in-progress` is current, the Attempt/active owner still match that Worker, `status:done` is absent, no durable `[FINAL ACCEPTANCE]` exists, and no newer Coordinator gate/Attempt has superseded the Worker.

If any condition fails or authority is ambiguous, the Worker MUST fail closed: no terminal report, no status mutation, no owner release/reassignment, no reopen; STOP with `STALE_AUTHORITY`. In particular, a stale Worker must not release an owner that may belong to a newer Attempt.

This is a last-safe-point guard, not a distributed lock.