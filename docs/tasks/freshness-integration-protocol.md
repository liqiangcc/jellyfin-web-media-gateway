# Dependency-Aware Freshness / Integration Gate Protocol

本协议定义并行 Task 在 `main` 推进后的 Evidence freshness、integration verification 与最终 merge 规则。

目标：

> **并行 Task 不因为任意 `main` 变化就全部失效；只对真正影响其语义 authority、Claim 或集成面的变化重新验证。**

同时保持：

> **已有 Evidence 只能在其证明范围内复用；不能为了提高并行度而把真实 semantic/integration 风险当成无关变化。**

---

## 1. Adoption / non-retroactive rule

本协议默认适用于：

- 本协议合入后新 materialize / republish 的 Task Package；
- 已有 draft Task 在正式 materialize 时；
- Coordinator 明确通过 Contract Revision 迁移到本协议的已有 Task。

本协议**不追溯降低已经冻结 Task Contract 的 freshness 要求**。

如果一个已发布 Task 明确写着：

```text
Final Candidate must integrate live accepted main
```

或等价 strict-main 规则，则该 Attempt 仍按原 Contract 执行，除非先走正式 Contract Revision；不能因为新协议更宽松就在 Review 时临时降低要求。

---

## 2. 四种必须区分的关系

### 2.1 Hard dependency

没有上游结果，本 Task 就无法正确实现或验证。

```text
A Final Acceptance
→ B 才可 publish/execute
```

Hard dependency 属于 Publication Gate，不等于所有未来 upstream merge 都自动使 B 失效。

### 2.2 Semantic / authority dependency

本 Task 的实现或 Claim 直接依赖某个已接受 contract / API / authority / security invariant。

例如：

```text
Playback revision authority
Display generation authority
SiteAdapter schema
R008 Egress/Secret policy
```

如果这些 authority 在 Task Evidence 之后发生变化，必须判断受影响 Claims，不能只做普通 mergeability 检查。

### 2.3 Integration overlap

两个 Task 没有业务/语义依赖，但共享 build/workspace/dependency/integration surface，例如：

```text
Cargo.toml
Cargo.lock
workspace members
shared crate public type layout
shared generated code
common router composition
workflow/build image
```

它可能需要 composition proof，但**不自动使原 Task-specific semantic Evidence 失效**。

### 2.4 Unrelated change

变化既不触及 Task 的 semantic freshness domain，也不触及 integration surface。

例如对一个 Rust runtime Task：

- 不相关 docs；
- 另一独立插件且不改变共享 workspace/dependency surface；
- 不相关 UI asset；
- 不影响该 Task Claim 的治理/计划文本。

Unrelated change 不应触发 rebase/full rerun。

---

## 3. Task Freshness Contract

新 Task Package 必须在 `task.md` 中包含 `Freshness / Integration Contract`，格式来自：

```text
docs/tasks/freshness-contract.template.md
```

至少声明：

```text
Freshness policy
Semantic authorities
Semantic freshness domains
Integration surfaces
Task-owned surfaces
Authority/domain → Claim mapping
Integration verification jobs
Unrelated-main policy
Strict-main reason (when applicable)
```

### 3.1 默认 policy

默认：

```text
Freshness policy: dependency-aware
```

禁止默认写：

```text
main advances
→ full Task invalidation
```

### 3.2 strict-main 只用于确实需要的 Task

只有 Task 正确性本身要求“必须验证当前完整 main snapshot”时才允许：

```text
Freshness policy: strict-main
```

例如：

- release candidate；
- schema/migration chain 必须针对当前主线；
- lockfile/dependency closure 本身就是 Claim；
- repo-wide architecture rewrite；
- Task Contract 明确证明“整个 current main”的性质。

`strict-main` 必须在执行前冻结理由；不能在看到并行 merge 后临时添加。

---

## 4. SHA / Evidence 术语

### Planning Base

Task materialize 时记录的基线，用于解释最初 Scope/仓库状态。

```text
Planning Base != 永久 freshness lock
```

### Task Candidate SHA

包含本 Task 语义实现、并由 Task-specific Claims/Evidence 验证的 commit。

### Evidence Base SHA

Task Candidate 开始 required Evidence 时所基于/已包含的 accepted main snapshot。

### Current Main SHA

Coordinator Review / merge gate 时 live `main` HEAD。

### Integration Base SHA

Coordinator 开 Integration Slot 时冻结的 current main SHA。

### Integration Candidate SHA

Task Candidate 与 Integration Base 组合后的 PR head / merge commit candidate。

Task Candidate 与 Integration Candidate 可以相同，也可以不同。

---

## 5. Freshness classification algorithm

Coordinator 在 Review 时必须读取：

```text
Task Freshness Contract
Task Candidate / PR
Evidence Base
Current Main
compare(Evidence Base, Current Main)
actual changed files/patches when needed
```

然后按最高影响级别分类：

```text
CONTRACT_INVALIDATING
SEMANTIC_AUTHORITY
INTEGRATION_OVERLAP
UNRELATED
NONE
```

### 5.1 NONE

Current Main 没有比 Evidence Base 更新。

动作：正常 Review。

### 5.2 UNRELATED

main 有更新，但不触及 Task 的 semantic/integration surface。

动作：

- Task-specific Evidence 保持有效；
- 不要求为了 freshness rebase/merge main；
- 不重跑 J1/J2/...；
- PR clean/mergeable 时可以进入最终 merge gate；
- merge 前仍使用 expected head SHA 防止 PR head 漂移。

### 5.3 INTEGRATION_OVERLAP

main 更新只触及 composition surface，不改变本 Task 的 semantic authority/Claim 前提。

动作：

- 已接受的 Task-specific semantic Evidence 可以复用；
- 不重跑全部 Task-specific Jobs；
- 进入 Integration Gate，只运行 Task Contract 声明的 `JI*` integration jobs；
- 如果 integration 过程出现 semantic conflict，升级为 `SEMANTIC_AUTHORITY`。

### 5.4 SEMANTIC_AUTHORITY

main 更新改变了本 Task 依赖的 authority/API/invariant/semantic domain。

动作：

- integrate/reconcile 当前 accepted authority；
- 根据 `authority/domain → Claim mapping` 只重跑受影响 Claims/Jobs；
- 如果不能安全证明哪些 Claims 不受影响，使用保守更广验证；
- 不得复用已经被 authority change 否定的 Evidence。

### 5.5 CONTRACT_INVALIDATING

变化使 Scope、Claims、Success Criteria、Evidence Authority、decomposition 或安全/架构前提本身不再可执行。

动作：

```text
status:draft
→ Contract Revision
→ Publication Gate
```

不能把它当普通 freshness retry。

---

## 6. Path overlap 不是唯一判断依据

不能只靠文件名机械分类。

例如：

- `Cargo.lock` 通常至少是 Integration Overlap；
- `gateway-core/src/control.rs` 对 Control Task 可能是 Semantic Authority；
- 同一个文件里的独立测试注释可能仍是 Unrelated；
- canonical security doc 改变可能没有代码 path overlap，但会成为 Semantic Authority。

因此：

```text
path/symbol overlap
= classification evidence
!= classification authority by itself
```

Coordinator 必须在重要 Task 中查看实际 patch/contract impact。

---

## 7. Integration Gate

当：

```text
Task-specific semantic review = acceptable
+
Freshness classification = INTEGRATION_OVERLAP
```

Coordinator 不应把整个 Task 当普通 implementation failure 重做，而应开启 Integration Gate。

Issue 评论：

```text
[INTEGRATION GATE]

Task Attempt reviewed: <N>
Task Candidate: <sha>
Semantic Evidence: accepted / preserved

Integration Base: <current-main-sha>
Freshness classification: INTEGRATION_OVERLAP
Overlap reason:
- <workspace/shared dependency/build surface>

Required integration jobs:
- JI1: <selector>
- JI2: <selector or n/a>

Protected integration surfaces:
- <paths/domains that must not change during slot without reclassification>

Next state: status:ready
Next attempt: <N+1>
Revision class: INTEGRATION_ONLY
Reuse PR/branch: yes
```

Integration-only Attempt 仍使用正常 claim/owner 生命周期，便于恢复和审计，但它**不重新打开已经接受的 semantic Claims**。

---

## 8. Integration Slot / manual merge queue

`[INTEGRATION GATE]` 同时建立一个轻量 Integration Slot。

在 slot 存续期间，Coordinator：

- 不合入会触及该 Task `Protected integration surfaces` 的其他 Task；
- 可以继续合入经分类为 `UNRELATED` 的变化；
- 不因 unrelated merge 重新失效 slot；
- 如果不得不合入 overlapping/semantic change，必须先关闭/重分类当前 slot。

目标是避免：

```text
A integration test running
→ B overlapping merge
→ A stale
→ A rerun
→ C overlapping merge
→ A stale again
```

而变成：

```text
A semantic Evidence accepted
→ A gets integration slot
→ compose with frozen Integration Base
→ JI pass
→ merge A
→ release slot
→ next overlapping Task
```

这是一种 Coordinator-managed merge queue，不要求引入新的业务 Task。

---

## 9. Integration-only Worker rules

收到 `Revision class: INTEGRATION_ONLY` 后：

1. 复用原 Issue / branch / PR；
2. 保留原 Task Candidate；
3. **优先使用 merge commit** 把冻结的 `Integration Base SHA` 合入 worker branch，使原 Task Candidate 保持祖先关系；
4. 不做与 integration 冲突无关的实现修改；
5. clean merge 时运行 `JI*`；
6. 如果发生 conflict 且 conflict 触及 semantic/task-owned surface，停止把它当 integration-only，报告给 Coordinator 重新分类；
7. 如果只存在机械 integration fix，也必须记录具体差异；
8. 产生 `Integration Candidate SHA`；
9. `[EXECUTION REPORT]` 同时记录 Task Candidate、Integration Base、Integration Candidate、JI Evidence。

不得因为 integration-only Attempt 就重新声称 C1-Cn 全部是由新 SHA 独立重新证明的；应明确哪些 semantic Evidence 被复用，哪些是新的 integration Evidence。

---

## 10. Evidence reuse conditions

Task Candidate 的 semantic Evidence 可以跨 Integration Candidate 复用，仅当 Coordinator 能确认：

- Integration Candidate 包含原 Task Candidate ancestry（推荐 merge，而非 rewrite）；
- integration 没有改变 Task-owned semantic diff；
- 没有 semantic authority conflict；
- required JI 在 exact Integration Candidate 上 PASS。

如果 merge/rebase/conflict resolution 改写了 Task semantic implementation，则旧 Evidence 不自动复用，升级为 Semantic Authority / affected-claim reverify。

---

## 11. Coordinator Review fields

在采用本协议的 Task Review 中，必须记录：

```text
Task Candidate:
Evidence Base:
Current Main:
Freshness policy:
Freshness classification:
Changed main surface reviewed:
Semantic Evidence reuse: yes | no | partial
Affected Claims requiring reverify:
Integration Gate required: yes | no
Integration Base / Candidate: <sha or n/a>
Integration Evidence: <jobs or n/a>
```

这可以放在 `[COORDINATOR REVIEW]` 内；若进入 integration-only，则再发 `[INTEGRATION GATE]`。

---

## 12. Final Acceptance Gate

采用 dependency-aware policy 的 Task 可以 Final Accept，当且仅当：

```text
Task Success Criteria accepted
+
semantic Claims/Evidence accepted
+
Freshness classification resolved
+
required Integration Gate PASS (if any)
+
PR head / expected head verified
+
no unresolved blocker
```

Final Acceptance 应记录：

```text
Accepted Task Candidate: <sha>
Accepted Integration Candidate: <sha or same/n/a>
Freshness classification at merge: <...>
Integration Base: <sha or n/a>
```

---

## 13. Default examples

### Example A — independent plugin vs Web Display

```text
Task A: Web Display runtime
Task B: independent plugin
```

B 只改独立插件文件：

```text
A freshness = UNRELATED
→ no A rerun
```

B 同时改 `Cargo.toml/Cargo.lock`：

```text
A freshness = INTEGRATION_OVERLAP
→ preserve A display semantic Evidence
→ run A declared workspace integration job only
```

B 改 `gateway-core` Display/Playback authority：

```text
A freshness = SEMANTIC_AUTHORITY
→ affected A Claims reverified
```

### Example B — docs/governance commit

普通 runtime Task 执行时合入一份不改变其 Task Contract 的治理文档：

```text
freshness = UNRELATED
```

不应仅因为 `main` SHA 变化就重跑 runtime matrix。

### Example C — R008 security authority changes

Task 声明 R008 为 semantic authority，期间 R008 policy/API 改变：

```text
freshness = SEMANTIC_AUTHORITY
```

必须重新检查受影响安全 Claims，不能降为 integration-only。

---

## 14. Publication rule

新 Task 发布前必须冻结 Freshness Contract。

默认禁止含糊语句：

```text
must always integrate latest main before acceptance
```

除非 `Freshness policy: strict-main` 且已有明确 reason。

推荐写：

```text
Freshness policy: dependency-aware
Unrelated-main policy: existing exact-Candidate Evidence remains valid
Integration overlap: run declared JI jobs
Semantic authority change: reverify mapped affected Claims
```

---

## 15. Safety principle

本协议优化的是：

```text
parallelism
+ Evidence reuse
+ bounded integration work
```

它绝不允许：

- 忽略真实 authority change；
- 用 mergeability 代替 semantic review；
- 把 integration failure 当 unrelated；
- 在 Evidence 之后临时降低 Success Criteria；
- 为了减少 CI 而跳过 Task Contract 明确要求的 target/runtime proof。

最终原则：

> **Freshness 由依赖和影响面决定，不由“main SHA 是否变化”这一事实单独决定。**
