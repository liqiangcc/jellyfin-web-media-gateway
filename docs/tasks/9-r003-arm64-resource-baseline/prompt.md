# Session Bootstrap — R003-TARGET ARM64 Resource Baseline

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 R003-TARGET Verification Task。

本文件只是 bootstrap/navigation，不是 Task Contract，也不保存实时状态。

## Execution Context

```text
GitHub Issue: #9
Task Contract: docs/tasks/9-r003-arm64-resource-baseline/task.md
Expected worker: web
Expected environment label after publication: env:web-gpt
Downstream handoff profile: docs/tasks/handoffs/web-gpt.md
Execution plane: github-actions
Target runner: ubuntu-arm64-target-phone
Research Item: R003
Linked trusted harness/workflow: Issue #8
```

## Live Gate

```text
status:draft
→ STOP，不 claim、不 dispatch、不自行发布

status:ready + env:web-gpt + no active owner
→ 可以 claim，开始 Attempt N
```

只有 Coordinator 在确认以下条件后才能发布：

- Issue #8 已 Final Acceptance，受信 harness/workflow 已进入可执行仓库状态；
- Issue #1 Target Runner 仍可用于 target proof；
- 已记录明确的 harness/workflow SHA；
- 已记录明确的 R001/Gateway measured Candidate SHA；
- candidate 的 build/start/stop/test-media 入口足够稳定。

## Start Protocol

1. 使用 GitHub 读取 Issue #9、comments、`AGENTS.md`、本 prompt、`task.md`、`docs/tasks/issue-lifecycle-protocol.md`。
2. 读取 Issue #8 Final Acceptance、受信 workflow/harness、Issue #3/R001 被批准的 candidate，以及 `docs/technical-feasibility-validation.md` R003 和 `docs/runner-execution-architecture.md`。
3. 确认 live state 为 `status:ready + env:web-gpt`、无 owner，然后 claim → `status:in-progress` → Attempt N。
4. 只通过受信 target workflow 调度 phone-specific Evidence；Runner 是 execution backend，不是 Worker。
5. 先执行 J0 preflight。FFmpeg/Chromium/thermal metric 缺失必须真实记录；不得在 target job 内 sudo/root 安装或降低标准。
6. J1/J2/J4 的 sustained evidence 必须是真实连续运行；不能用分片伪造 60 分钟 soak。
7. 保留 raw artifact + summary；必须同时记录 harness/workflow SHA 和 measured Candidate SHA。
8. 观察到 target/candidate bug 时停止并报告，不得在同一 Candidate SHA 下偷偷修代码继续测。
9. 正常结束 `[EXECUTION REPORT]` → `status:review` → release owner → STOP；阻塞则 `[BLOCKER REPORT]` → `status:blocked` → release owner → STOP。

## Result Semantics

```text
Task execution complete
!= R003 hypothesis PASS
```

完整可信的 `R003 result: FAIL` 也是有效研究交付；不要根据结果降低低功耗/稳定性标准制造 PASS。

## Security Reminder

- no automatic untrusted PR target execution;
- no arbitrary shell input;
- no production Vault/profile/site/Jellyfin Secrets;
- no CPU governor/thermal-control tuning to improve benchmark numbers;
- target jobs serialized, bounded and cleaned up.

## Stop Boundary

完成一个 R003-TARGET Attempt 后停止。Worker 不得自行 `status:done`、关闭 Issue #9 或自动执行新的优化/修复 Task。