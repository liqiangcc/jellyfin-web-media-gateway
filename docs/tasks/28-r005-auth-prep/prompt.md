# Session Bootstrap — R005-AUTH-PREP

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的 Issue #28。

本文件只是 Worker bootstrap/navigation；实时状态以 GitHub Issue 为准，Task Contract 以 `task.md` 为准。

## Entry

```text
GitHub Issue: #28
Task Contract: docs/tasks/28-r005-auth-prep/task.md
Expected worker: cloud-codex
Expected environment: env:cloud
Handoff: docs/tasks/handoffs/cloud.md
Parent umbrella: #26 / R005-AUTH
Parallel task: #23 / R005-PUBLIC
```

## Required reads

开始前必须实际读取：

- Issue #28 及 relevant comments；
- `AGENTS.md`；
- 本 prompt；
- `docs/tasks/28-r005-auth-prep/task.md`；
- `docs/tasks/issue-lifecycle-protocol.md`；
- `docs/security.md`；
- `docs/requirements.md`；
- `docs/architecture.md`；
- accepted Issue #14 / R008 Final Acceptance 与当前安全实现；
- accepted Issue #2 / R007 authority；
- live `main` 与并行 Issue #23 状态。

不得根据旧聊天猜测实时状态。

## Claim gate

只有当前 Issue 同时满足：

```text
open
status:ready
env:cloud
no active owner
```

才允许 claim。

然后：

```text
claim
→ status:in-progress
→ Attempt N
→ execute task.md
```

## Frozen execution boundary

本 Task 只实现确定性服务器侧 auth infrastructure：

- SiteAccount / SiteSessionRef / AccountState；
- Session Vault API/test storage；
- scoped SiteAccessCapability；
- R008-controlled authenticated HTTP injection boundary；
- non-secret PendingIntent；
- candidate-session validate + atomic swap；
- expiry/logout/rotation primitives；
- deterministic security/regression Evidence。

禁止：

- 真实 Bilibili/其他站点登录；
- production Cookie/profile/Secret；
- Browser Worker / Native Site Panel；
- 手机/电视 Target Evidence；
- CAPTCHA/验证码/二维码自动化；
- DRM/paywall/access-control bypass；
- 修改 R007 concurrency/revision 语义；
- 为了登录功能给 Site Plugin 增加 private-network/Egress bypass。

## Parallel/freshness rule

Issue #23 可以并行执行。

如果 `main` 在最终验证前前进：

- 判断 delta 是否与本 Task 的 auth/security/core surfaces 有关；
- final Candidate 必须满足 task.md 的 current integration/security boundary；
- 不得因为纯文档变化无限制造 Attempt；
- 但若 accepted runtime/security 变化影响本 Task，必须集成后再收集 exact-SHA Evidence。

## Evidence

至少完成：

```text
J1 auth-domain deterministic tests
J2 security/failure/Secret-sentinel tests
J3 affected workspace + R008/R001/R007 regressions
```

所有 Actions Evidence 必须绑定最终 Candidate SHA。

## Completion

正常完成：

```text
[EXECUTION REPORT]
→ status:review
→ release active owner
→ STOP
```

阻塞：

```text
[BLOCKER REPORT]
→ status:blocked
→ release active owner
→ STOP
```

Worker 不得自行 `status:done`、关闭 #28、执行 #26/R005-AUTH-REAL 或自动开始下一 Task。