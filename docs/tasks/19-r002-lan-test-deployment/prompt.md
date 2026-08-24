# Session Bootstrap — R002-DEPLOY Trusted LAN Test Deployment Preparation

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 Issue #19 / R002-DEPLOY。

本文件只是 bootstrap/navigation，不是 Task Contract，也不保存实时状态。

## Execution Context

```text
GitHub Issue: #19
Task Contract: docs/tasks/19-r002-lan-test-deployment/task.md
Expected worker: cloud-codex
Expected environment label after publication: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Research Item: R002
Downstream physical verification: Issue #7
Accepted R002 implementation: Issue #6 / main merge 132c2747d736f9af72d9c06cfc08660876619029
Target infrastructure: Issue #1 / INFRA-001
```

## Preferred Codex Entry

```text
$task-worker Execute Issue #19 using `docs/tasks/19-r002-lan-test-deployment/prompt.md`.
```

Skill 不可见时按 `docs/tasks/handoffs/cloud.md` fallback。

## Live Gate

Worker 必须先实际读取 Issue #19：

```text
status:draft
→ STOP，不 claim、不实现、不自行发布

status:ready + env:cloud + no active owner
→ 可以 claim，开始新的 Attempt N

其他状态
→ 按 docs/tasks/issue-lifecycle-protocol.md 停止或交回 Coordinator
```

## Required Reading

先通过 GitHub 读取：

- Issue #19 及 relevant comments
- `AGENTS.md`
- 本 prompt
- `docs/tasks/19-r002-lan-test-deployment/task.md`
- `docs/tasks/issue-lifecycle-protocol.md`
- Issue #6 Final Acceptance / accepted R002 candidate
- Issue #1 Final Acceptance / Target Runner constraints
- `gateway-core/src/bin/r001-server.rs`
- `docs/security.md`（特别是 trusted LAN、Web Secure Context、Target Runner）
- `docs/runner-execution-architecture.md`
- `docs/tasks/7-r002-physical-tv-verification/task.md`

动态状态必须从 GitHub 重新读取，不得根据本 prompt 猜测。

## Start Protocol

1. 确认 live state 是 `status:ready + env:cloud` 且没有 active owner。
2. claim → `status:in-progress` → 新 Attempt N。
3. 从 current `main` 集成，不要基于旧 #6 分支重建。
4. 只实现 Task Contract 冻结的两个面：
   - 默认 loopback、不默认公网暴露的显式 IP bind 配置；
   - manual-only、trusted Candidate、low-privilege 的 Target Runner 临时 LAN deployment workflow。
5. 不得从未合并 PR 自动调度手机 Target Runner。
6. 本 Attempt required Evidence 只跑 GitHub-hosted J1/J2；Target deployment 必须留给 Coordinator ACCEPT/merge 后手动 dispatch。
7. 不读取/注入生产 Vault、站点 Cookie/profile、Jellyfin key、SSH/Tailscale Secret；不 sudo/root-install。
8. 不修改 R007 Playback command/revision/handoff 语义；不重定义 R001 media/Secret/open-proxy 语义。
9. required Actions Evidence 必须绑定 exact final Candidate SHA。
10. 正常完成后发标准 `[EXECUTION REPORT]` → `status:review` → release owner → STOP。
11. 阻塞则发 `[BLOCKER REPORT]` → `status:blocked` → release owner → STOP。

## Important Network Boundary

当前接受的 server 硬编码 `127.0.0.1`。本 Task 允许增加显式 IP bind 配置，但：

```text
default = loopback only
LAN deployment = explicit test configuration
public/WAN exposure = forbidden
```

Target workflow 应优先自动发现并验证一个具体 RFC1918/private LAN IPv4，再绑定该地址；不要为了方便把产品默认改成 `0.0.0.0`。

## Stop Boundary

完成 Issue #19 当前 Attempt 后立即停止。

Worker 不得：

- dispatch 新的 target deployment workflow 到手机；
- 执行 Issue #7；
- 宣称 R002 PASS/FAIL；
- 自行 merge；
- 自行 `status:done` 或关闭 Issue #19。