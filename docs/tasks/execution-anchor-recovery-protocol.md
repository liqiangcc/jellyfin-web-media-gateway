# Durable Execution Anchor / Recovery Protocol

本文件是 `docs/tasks/issue-lifecycle-protocol.md` 的执行可观察性 / 中断恢复补充协议。

它只规定 **Attempt 执行过程中如何尽早留下可恢复锚点，以及 Worker 中断后 Coordinator 如何恢复**；它不改变 `task.md` 的 Goal、Scope、Claims、Success Criteria、Evidence Authority，也不改变 Worker 必须在 Attempt 结束时提交 `[EXECUTION REPORT]` / `[BLOCKER REPORT]` 的正式闭环。

核心目标：

> **不要用频繁 heartbeat 制造噪声；要用 branch / commit / draft PR / workflow Evidence 留下真正可恢复的 durable state。**

---

## 1. Adoption / non-retroactive rule

本协议合入 `main` 之前已经处于 `status:in-progress` 的 Attempt：

- 不因为没有早期 checkpoint / draft PR 被追溯判错；
- 不要求重新 claim、重启 Attempt 或为了满足本协议制造空 commit；
- 可以在不干扰当前 Task Contract 的前提下自愿采用 durable anchor；
- 最终仍按其已发布 Task Contract 和 Issue lifecycle 完成。

本协议默认适用于之后新 claim 的 Attempt。

---

## 2. Durable anchor 是什么

Durable anchor 是 Coordinator / 下一 Worker 不依赖旧聊天即可找到并复用的执行状态。

### 2.1 Repository mutation Attempt

优先锚点：

```text
Issue / Attempt N
+
worker branch
+
至少一个已经 push 的 coherent in-scope commit
+
适合时的 draft PR
```

推荐 branch：

```text
worker/issue-<number>-<short-slug>
```

已有 Task branch / PR 时必须优先复用，不为了本协议重复创建。

### 2.2 Verification-only / workflow Attempt

如果 Task 不需要代码修改，durable anchor 可以是：

```text
exact Candidate / harness SHA
+
workflow run / job / artifact
+
Issue checkpoint
```

Target / long-running verification 必须先把 harness / candidate identity 固定到 durable Git ref / SHA，再开始不可轻易重建的执行。

### 2.3 Manual / physical Evidence Attempt

Manual TV / 现场验证没有必要制造 Git commit。可恢复锚点是：

```text
Issue checkpoint
+
frozen candidate/deployment identity
+
已获得的非敏感 Evidence 引用
```

---

## 3. 什么时候建立 anchor

不要在 claim 后立即创建空 commit / 空 PR。

Repository mutation Attempt：

```text
claim
→ status:in-progress
→ first coherent in-scope change
→ commit
→ push worker branch
→ durable anchor exists
```

如果后续还会继续较多实现 / CI 迭代，而且 PR 是该 Task 的正常交付形式，则尽早创建 draft PR，并在 PR body 引用：

```text
Refs #<issue>
Attempt: <N>
```

在以下工作之前尤其应该先有 durable anchor：

- 长时间 GitHub Actions / target workflow；
- 多轮 CI fix；
- 大范围 integration / rebase；
- Worker session 可能因客户端/环境中断而丢失本地状态的工作。

---

## 4. `[EXECUTION CHECKPOINT]`：每个 Attempt 最多一次

当第一个 durable anchor 已经存在、但 Attempt 仍会继续执行时，Worker 在 Issue 留 **一次**：

```text
[EXECUTION CHECKPOINT]

Attempt: <N>
Worker: <worker>
Environment: env:<environment>

Branch: <branch or n/a>
Durable commit: <sha or n/a>
Draft PR: <number/url or n/a>
Workflow / Evidence anchor: <run/job/ref or n/a>

Current stage:
- <brief stage; not a result claim>

Recovery note:
- <what a replacement Worker should reuse if this session disappears>
```

规则：

- 这是恢复锚点，不是 `[EXECUTION REPORT]`；
- 不把 checkpoint commit 当作 Final Candidate；
- 不把尚未完成的 Claim 写成 PASS；
- 不需要周期性“仍在执行” heartbeat；
- 正常进展不重复发 checkpoint；
- 只有出现真正的 recovery-relevant durable identity 变化且旧锚点已不可用时，才用新的 `[CORRECTION]` / recovery note 说明。

---

## 5. 禁止为了可观察性制造噪声

不得：

- 创建空 commit / no-op commit 只为了有 SHA；
- 创建无内容 PR 只为了显示“在工作”；
- 每隔固定分钟发 Issue heartbeat；
- 把本地未 push commit 当 durable state；
- 把 draft PR 存在本身当 Task 完成证据；
- 为了早开 PR 而提交 Secret、临时调试垃圾、大型未清理 artifact。

原则：

```text
meaningful durable artifact > periodic status text
```

---

## 6. Worker 正常结束 / Blocked 时如何使用 anchor

正常结束时 `[EXECUTION REPORT]` 必须记录**最终** Candidate / PR / Evidence，而不是只引用早期 checkpoint。

如果 checkpoint 后 Candidate 已变化：

```text
early durable commit
!= final Candidate
```

最终 exact-SHA Verification 仍以 `task.md` 为准。

Blocked 时 `[BLOCKER REPORT]` 应额外说明：

```text
Reusable branch / PR:
Last durable commit:
Reusable Evidence:
```

让下一 Attempt 可以继续，而不是从零重建。

---

## 7. 什么叫 stale ownership

`status:in-progress + owner` 表示当前执行租约，不是永久锁。

但 **仅仅经过了一段时间，不能自动判定 stale**。

Coordinator 只有在存在具体信号时才做 recovery，例如：

- Worker/session 明确终止、崩溃或失联；
- 用户/执行环境明确报告该 Worker 已停止；
- Worker 已不能继续，但没有机会完成正式 report；
- durable branch/PR/Actions 已存在，而当前 owner 明确不再执行。

缺少 PR / checkpoint 本身不等于 Worker stale。

---

## 8. Coordinator Recovery

发现 execution ownership 中断时，Coordinator **不做正常 Review，也不假装 Worker 已提交结果**。

先读取：

```text
Issue + comments
→ current owner/status
→ worker branch
→ commits
→ draft/open PR
→ Actions runs/jobs/artifacts
→ current main / Task Contract
```

然后评论：

```text
[COORDINATOR RECOVERY]

Interrupted Attempt: <N>

Reason:
- <why active execution is known to have stopped>

Durable anchor found:
- Branch: <... or none>
- Last durable commit: <... or none>
- PR: <... or none>
- Evidence/run: <... or none>

Reusable work:
- <what should be preserved>

Not accepted / still required:
- <what remains unverified/incomplete>

Contract change required: yes | no

Recovery action:
- release stale active owner
- next state: status:ready | status:draft | status:blocked
- next Attempt: <N+1 or n/a>
- reuse existing branch/PR: yes | no
```

### 8.1 Contract unchanged

最常见路径：

```text
Attempt N interrupted
→ [COORDINATOR RECOVERY]
→ release owner
→ status:ready
→ next Worker claim
→ Attempt N+1
→ reuse existing branch / PR / Evidence
```

**新的 Worker 不继续冒用 Attempt N。** 新 claim 必须是 Attempt N+1，但应复用 durable work。

### 8.2 Contract 已失效

如果中断暴露的是 Scope / Claims / architecture / Evidence Authority 问题：

```text
status:draft
→ Contract Revision / Publication Gate
```

不要把 execution recovery 当作静默 Contract change。

### 8.3 外部 blocker

如果 Worker 中断的真实原因是外部条件缺失：

```text
[COORDINATOR RECOVERY]
→ status:blocked
```

并记录最小 unblock 条件。

---

## 9. Replacement Worker 规则

恢复后的下一 Worker必须：

1. 读取上一 Attempt / `[EXECUTION CHECKPOINT]` / `[COORDINATOR RECOVERY]`；
2. 获取并检查已有 branch / PR，而不是默认重新实现；
3. 确认哪些 commit / Evidence 可以复用，哪些需要对 current main 重新验证；
4. 新 claim 后使用新的 Attempt N+1；
5. 最终仍跑当前 Task Contract 要求的 exact-Candidate Evidence。

不得因为换 Worker 就新建重复业务 Issue 或复制 PR。

---

## 10. Security / privacy

Checkpoint / recovery comment 不得记录：

- Cookie / Authorization / Token / API Key；
- Secret-bearing URL / signed media URL；
- Vault/profile 内容；
- password/code/QR/login frame；
- 本地 credential 文件内容。

只记录可公开的 branch / SHA / PR / run/job / artifact identity 和非敏感状态。

---

## 11. 与 Task Contract / Evidence 的关系

本协议定义的是：

```text
execution observability
+
interrupted-session recovery
```

它不定义：

```text
Task Scope
Claims
Success Criteria
Verification PASS
Coordinator ACCEPT
```

因此：

```text
checkpoint exists
!= Candidate accepted
!= Verification PASS
!= Task ACCEPT
```

Task-specific Contract 如果对 branch/PR/Evidence 生命周期有更严格要求，按更高的 Task-specific 要求执行，但不得降低本协议的安全/可恢复性原则。
