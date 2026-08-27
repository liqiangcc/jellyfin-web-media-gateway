# Session Bootstrap — R006-CONTRACT-PREP

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 Issue #33。

本文件只负责 Worker bootstrap/navigation；实时状态以 GitHub Issue 为准，Task Contract 以 `task.md` 为准。

## Entry

```text
GitHub Issue: #33
Task Contract: docs/tasks/33-r006-contract-prep/task.md
Expected worker: cloud-codex
Expected environment: env:cloud
Handoff: docs/tasks/handoffs/cloud.md
Parent umbrella: #27 / R006-DESIGN
Future target/runtime evidence: #9 / R003-TARGET
```

## Required reads

开始前实际读取：

- Issue #33 及 relevant comments；
- `AGENTS.md`；
- 本 prompt；
- `docs/tasks/33-r006-contract-prep/task.md`；
- `docs/tasks/issue-lifecycle-protocol.md`；
- `docs/architecture.md`；
- `docs/requirements.md`；
- `docs/security.md`；
- accepted Issue #14 / R008 Final Acceptance 与当前 Egress/security 实现；
- Issue #27 的最新 decomposition；
- live `main`；
- #9 live state 仅用于确认 target/runtime 仍不属于本 Task。

不得根据旧聊天猜测实时状态。

## Claim gate

只有 live state 同时满足：

```text
open
status:ready
env:cloud
no active owner
```

才允许：

```text
claim → status:in-progress → Attempt N
```

## Frozen execution boundary

本 Task 只做 target-neutral generic contracts + deterministic fake worker/harness：

- BrowserWorker interface；
- BrowserCommand / BrowserEvent；
- ProfileAttachmentRef；
- generic Auth Mode boundary；
- NativePanelSession / short-lived control token contract；
- R008-compatible navigation/security boundary；
- deterministic fake worker；
- generic failure taxonomy。

禁止：

- 启动 Chromium/Playwright；
- 运行 phone Target Evidence；
- 决定 always-on/on-demand/pool size；
- 写入 phone-specific CPU/RSS/timeout defaults；
- 写 Bilibili/YouTube DOM/API/login-success 逻辑；
- 真实登录/profile 获取；
- 把 Native Panel 做成 unrestricted remote desktop；
- profile/Cookie 下载；
- DRM/protected-media capture；
- 修改 R007 Playback authority。

## Target strategy rule

Issue #9 的最终 Chromium CPU/RSS/thermal Evidence 决定后续 R006-RUNTIME/R006-TARGET：

```text
phone viable
| phone viable with limits
| external host
| defer/drop
```

本 Task 不提前替它做决定。

## Evidence

至少完成：

```text
J1 generic contract/fake-worker deterministic suite
J2 security/failure/stale-ref/token cleanup suite
J3 affected workspace + R008/R007 regressions
```

所有 required Evidence 必须绑定最终 Candidate SHA。

## Completion

正常完成：

```text
[EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

阻塞：

```text
[BLOCKER REPORT]
→ status:blocked
→ release owner
→ STOP
```

Worker 不得自行 `status:done`、关闭 #33、开始 R006 target/runtime/real-site Task 或自动执行下一 Task。