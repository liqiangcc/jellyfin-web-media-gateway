# Freshness / Integration Contract Template

每个新 materialize / republish 的 Worker Task，应把下面这一节复制进自己的 `task.md` 并替换所有 placeholder。

```markdown
## Freshness / Integration Contract

Freshness policy: dependency-aware | strict-main

Semantic authorities:
- <Issue / canonical doc / API / accepted contract that this Task semantically relies on>

Semantic freshness domains:
- <path / crate / symbol / contract / invariant>

Integration surfaces:
- <workspace/build/dependency/router/shared schema surfaces that may require composition proof>

Task-owned surfaces:
- <paths/domains this Task is expected to change semantically>

Authority/domain → Claim mapping:
- <authority/domain>: C1,C2
- <authority/domain>: C3

Integration verification:
- JI1: <exact integration selector / command / workflow job>
- JI2: <optional>

Unrelated-main policy:
- existing exact-Candidate semantic Evidence remains valid; no rebase/full rerun solely because main advanced

Integration-overlap policy:
- preserve accepted semantic Evidence; compose Task Candidate with Coordinator-frozen Integration Base and run only declared JI jobs unless conflict changes Task semantics

Semantic-authority-change policy:
- integrate/reconcile current accepted authority and rerun mapped affected Claims; use broader verification only when impact cannot be bounded safely

Strict-main reason:
- n/a | <why this Task correctness genuinely requires validating the complete current main snapshot>
```

## Rules

- 默认使用 `dependency-aware`。
- `strict-main` 必须在执行前给出具体 correctness reason。
- `Base commit` 是 planning/execution baseline，不自动等于“任何未来 main 变化都使 Candidate 失效”。
- `Semantic authorities` 与 `Hard publication dependencies` 是两个不同概念：前者控制 Evidence freshness，后者控制 Task 是否可发布。
- `Integration surfaces` 不能机械写成整个仓库；只声明真实 composition risk。
- `Authority/domain → Claim mapping` 应尽可能具体，让后续 semantic change 只重跑 affected Claims。
- 如果无法安全界定影响面，Coordinator 可以保守扩大验证，但必须在 Review 中解释原因。
- 详细算法见 `docs/tasks/freshness-integration-protocol.md`。
