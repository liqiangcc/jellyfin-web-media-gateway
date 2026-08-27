# Session Bootstrap — R005-PUBLIC Real Site Resolution PoC

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 Issue #23 / `R005-PUBLIC`。

本文件只是 bootstrap/navigation，不是 Task Contract，也不保存动态 Attempt 或 Evidence。

## Execution Context

```text
GitHub Issue: #23
Task Contract: docs/tasks/23-r005-public-real-site/task.md
Expected worker after publication: cloud
Expected environment: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Research Item: R005
Phase: public/no-login only
Frozen target: Bilibili / BV14V411W7r5
```

## Live Gate

先实际读取 Issue #23：

```text
status:draft
→ STOP，不 claim、不自行发布

status:ready + env:cloud + no active owner
→ claim，开始新的 Attempt N
```

## Required Reading

使用 GitHub 读取：

- Issue #23 及 relevant comments；
- `AGENTS.md`；
- 本 prompt；
- `docs/tasks/23-r005-public-real-site/task.md`；
- `docs/tasks/issue-lifecycle-protocol.md`；
- `docs/tasks/handoffs/cloud.md`；
- `docs/requirements.md` 的 Site Plugin / SourceLocator / ResolvedMedia / Egress 部分；
- `docs/architecture.md`；
- `docs/implementation-contracts.md`；
- `docs/technical-feasibility-validation.md` R005；
- Issue #2/R007 Final Acceptance；
- Issue #3/R001 Final Acceptance；
- Issue #14/R008 当前 live state、最新 Coordinator Review/Execution Report；若执行期间 R008 Final Acceptance 合入，按 task.md 做 final integration/freshness。

动态 GitHub 状态优先于聊天背景。

## Start Protocol

1. 确认 Issue 当前冻结 target 为 `Bilibili / BV14V411W7r5`，且满足 `status:ready + env:cloud + no owner`。
2. claim → `status:in-progress` → Attempt N。
3. 只执行 `task.md` 的公开/无需登录阶段；不得扩大到账号、Cookie、Vault、Browser Worker。
4. concrete site knowledge 只进入 `plugins/<site>/` 和相应 fixture/docs；Core 不增加站点 special case。
5. 从第一步遵守 canonical central `public_web` egress/Secret boundary；不得为了真实站点可用而增加私网例外或 open proxy。
6. Issue #14/R008 可以并行。若其 Final Acceptance 在本 Task final verification 前合入，必须集成 accepted R008 并重跑受影响 J2；不要把并行候选当成已接受 authority。
7. re-resolve/expiry 必须消费 accepted R007 freshness 语义；不要定义第二套 item/media revision。
8. 保留 deterministic J1/J2 和 bounded real-site J3 Evidence；真实站点不可用、出现 challenge/login wall/access-control 时真实报告 BLOCKED/FAIL，不做绕过，也不用 fixture 冒充 real-site PASS。
9. 正常结束：标准 `[EXECUTION REPORT]` → `status:review` → release owner → STOP。
10. 阻塞：标准 `[BLOCKER REPORT]` → `status:blocked` → release owner → STOP。

## Boundary Reminder

本 Task 的 Goal、Claims、Success Criteria、Evidence Contract 仅由：

```text
docs/tasks/23-r005-public-real-site/task.md
```

定义。

本 prompt 不得重新定义 Scope，也不得自行更换 target site/sample。Candidate 变化后旧 real-site smoke 不自动证明新 Candidate。

## Stop Boundary

Worker 不得：

- 自行发布 draft Issue；
- 执行 R005-AUTH；
- 保存/提交登录信息或 Secret；
- 运行 R006 Native Site Panel/Browser Worker；
- 修改 Core 加 concrete-site 规则；
- 自行 `status:done` 或关闭 Issue；
- 自动开始下一 Task/Attempt。
