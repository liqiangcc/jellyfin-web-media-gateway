# Session Bootstrap — R002-TV Physical TV Verification

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 R002-TV physical verification Task。

本文件只是 bootstrap/navigation，不是 Task Contract，也不保存实时状态。

## Execution Context

```text
GitHub Issue: #7
Task Contract: docs/tasks/7-r002-physical-tv-verification/task.md
Expected worker: manual-tv
Expected environment label after publication: env:manual-tv
Downstream handoff profile: docs/tasks/handoffs/manual-tv.md
Research Item: R002
Linked implementation: Issue #6 / R002-PREP
```

## Live Gate

```text
status:draft
→ 停止，不执行、不自行发布

status:ready + env:manual-tv + no active owner
→ 可 claim，开始 Attempt N
```

只有 Coordinator 在确认 Issue #6 已 ACCEPT、指定 candidate/deployment 可供真实电视访问后，才能发布本 Task。

## Start Protocol

1. 读取 Issue #7、relevant comments、`docs/tasks/7-r002-physical-tv-verification/task.md`、Issue #6 的 `[FINAL ACCEPTANCE]`/candidate 信息和 `docs/tasks/issue-lifecycle-protocol.md`。
2. 确认当前 TV/browser、网络、phone/remote trigger 和音频测试媒体满足 task.md Preconditions。
3. 确认 Issue 为 `status:ready + env:manual-tv` 且无 owner，然后 claim → `status:in-progress` → Attempt N。
4. 严格执行 task.md 的 Cases A-F、10/30 分钟等待和 result classification。
5. 不用桌面浏览器、模拟器、自动化框架、autoplay bypass flag、synthetic activation 或理论判断替代真实 TV Evidence。
6. `FAIL` 是有效研究结果；不得降低标准制造 PASS。
7. 完成后把标准 `[EXECUTION REPORT]` 写回 Issue #7，包含 TV/browser 环境、每个 Case 结果和最终 R002 classification；然后 `status:review`、释放 ownership、STOP。
8. 无法继续则 `[BLOCKER REPORT]` → `status:blocked` → release → STOP。

## Important Result Semantics

```text
Task execution complete
!= R002 hypothesis PASS
```

Coordinator 可以 ACCEPT 一份完整可信的 `R002 result: FAIL` Evidence；这表示研究完成且发现产品风险，而不是把失败改写成成功。

## Stop Boundary

完成一个 Manual TV Attempt 后停止，不自行修改产品代码、不执行下一个 Task、不关闭 Issue。