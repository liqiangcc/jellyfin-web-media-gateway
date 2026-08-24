# Session Bootstrap — INFRA-002 Target Runner Recovery

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 Issue #21 / INFRA-002。

本文件只是 bootstrap/navigation，不保存实时状态。

## Execution Context

```text
GitHub Issue: #21
Task Contract: docs/tasks/21-infra-target-runner-recovery/task.md
Expected worker: ubuntu-arm64
Expected environment: env:ubuntu-arm64
Handoff: docs/tasks/handoffs/ubuntu-arm64.md
Accepted authority: Issue #1 / INFRA-001
Downstream consumer: Issue #9 / R003-TARGET
Incident run: 32727443950 attempt 2
Incident job: 97505873310
```

## Preferred Entry

```text
$task-worker Execute Issue #21 using `docs/tasks/21-infra-target-runner-recovery/prompt.md`.
```

## Live Gate

先实际读取 Issue #21：

```text
status:draft
→ STOP，不 claim、不自行发布

status:ready + env:ubuntu-arm64 + no active owner
→ claim，开始新的 Attempt N
```

## Required Reading

通过 GitHub 读取：

- Issue #21 及 comments
- Issue #1 INFRA-001 Execution Report / Coordinator Review / Final Acceptance
- `AGENTS.md`
- 本 prompt
- `docs/tasks/21-infra-target-runner-recovery/task.md`
- `docs/tasks/issue-lifecycle-protocol.md`
- `docs/tasks/handoffs/ubuntu-arm64.md`
- `docs/runner-execution-architecture.md`
- `docs/security.md` Target Runner sections
- `.github/workflows/target-runner-smoke.yml`
- Issue #9 当前 publication snapshot

动态 GitHub 状态优先于本 prompt。

## Start Protocol

1. 确认 live state 为 `status:ready + env:ubuntu-arm64` 且无 owner。
2. claim → `status:in-progress` → Attempt N。
3. 先保存 stuck job 对应的非敏感进程、control、`_diag`、内存/磁盘/网络证据。
4. 不读取/输出 `.credentials`、注册 token、PAT、SSH/Tailscale Secret。
5. 优先恢复 **现有** runner：使用已接受的 `gateway-runnerctl restart`；必要时 bounded `stop` → `start`。
6. 不先删 runner、不重新注册、不轮换凭据，除非证据证明注册已经不可恢复；遇到这种情况应 BLOCKER REPORT，而不是擅自扩大。
7. 验证旧 Runner.Worker/helper/job process 被清理，新的 Runner.Listener 仍以 `gateway-runner`、non-root、原隔离 workspace 运行。
8. 不执行 R003 heavy job，不 dispatch 新 smoke；Coordinator 在 Review 后负责 fresh smoke。
9. 完成后标准 `[EXECUTION REPORT]` → `status:review` → release owner → STOP。
10. 无法安全恢复则 `[BLOCKER REPORT]` → `status:blocked` → release owner → STOP。

## Important Boundary

operator shell 即使是 root，也只用于已接受的 supervisor/control-plane 操作。最终 Runner runtime 仍必须是低权限 `gateway-runner`；不要为了恢复 runner 给它 sudo/root/capability。

## Stop Boundary

Worker 不得：

- 执行/发布 Issue #9；
- 重新跑 target smoke；
- 跑 R003 60 分钟任务；
- 修改产品代码；
- 自行 close #21 / `status:done`；
- 自动开始下一 Task。