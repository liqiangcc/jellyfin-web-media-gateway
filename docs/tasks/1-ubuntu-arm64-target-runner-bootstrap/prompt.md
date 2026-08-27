# Session Bootstrap — INFRA-001 Bootstrap Ubuntu ARM64 Target Runner

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的独立 Task `INFRA-001`。

本文件只是**会话启动入口**，不是 Task Contract。

## Execution Context

```text
GitHub Issue: #1
Task Contract: docs/tasks/1-ubuntu-arm64-target-runner-bootstrap/task.md
Expected worker: ubuntu-arm64
Expected environment label: env:ubuntu-arm64
Preferred Skill: $task-worker
```

## Preferred Codex Entry

```text
$task-worker Execute Issue #1 using `docs/tasks/1-ubuntu-arm64-target-runner-bootstrap/prompt.md`.
```

如果当前 Codex 无法发现 repo Skill，按下面 Start Protocol 手动执行，不得绕过 Issue 生命周期。

## Start Protocol

1. `git pull` / 同步并实际读取当前仓库，不根据聊天背景猜测状态。
2. 读取并遵守：
   - `AGENTS.md`
   - GitHub Issue `#1` 及 relevant comments
   - `docs/tasks/1-ubuntu-arm64-target-runner-bootstrap/task.md`
   - `docs/tasks/issue-lifecycle-protocol.md`
   - `docs/runner-execution-architecture.md`
   - `docs/security.md`
3. 确认 Issue：
   - `status:ready`
   - `env:ubuntu-arm64`
   - 没有 active owner
4. claim 后切换 `status:in-progress`，确定新的 `Attempt N`。
5. 严格执行 `task.md`，只做 Target Runner bootstrap + infrastructure smoke。
6. 正常结束先评论 `[EXECUTION REPORT]`，再转 `status:review`；阻塞则评论 `[BLOCKER REPORT]`，再转 `status:blocked`。
7. 释放 active execution ownership，然后停止。

## Important Bootstrap Note

当前 Codex/operator shell **可以是 root**，只要 `task.md` 明确允许该安装步骤需要特权。

这本身不是 BLOCKED。

必须区分：

```text
Bootstrap operator
→ 可以在明确安装步骤中使用必要特权

Final GitHub Actions Runner/service/job
→ 必须使用专用低权限用户
→ 不得 root
→ 默认无 sudo
→ workspace 与 Gateway production/Vault 分离
```

不要因为当前 shell 是 root 就提前停止；真正要验证的是最终 Runner runtime identity 和安全边界。

## Runtime Secret Note

Runner registration token 只能在执行时短期获取和使用：

- 不写入 repo；
- 不贴到 Issue；
- 不写入本 prompt/task；
- 不保留在 shell history；
- 使用后 unset/清理；
- 不在日志中回显。

如果当前 GitHub 权限无法安全获取 registration authorization/token，按 Task Contract 报 `BLOCKED`，不要绕过。

## Authority

```text
canonical docs / security
→ 产品与安全事实

AGENTS.md
→ 长期 Agent 规则

task.md
→ INFRA-001 唯一执行契约

prompt.md
→ 本会话 bootstrap only

Issue fields / labels
→ 实时状态

Issue comments
→ Attempt / Blocker / Review / Acceptance 历史

$task-worker
→ 通用 claim / Attempt / feedback / stop 算法
```

本 prompt 不得重新定义 Scope、Claims、Success Criteria 或 Evidence 标准；冲突时服从更高 authority。

## Stop Boundary

本 Task 不进入：

- R001 media path；
- R003 resource baseline；
- FFmpeg/Chromium/Jellyfin 产品验证；
- Site Plugin；
- TV UX。

完成或阻塞 INFRA-001 当前 Attempt 后立即停止，等待 Coordinator `$task-reviewer`。
