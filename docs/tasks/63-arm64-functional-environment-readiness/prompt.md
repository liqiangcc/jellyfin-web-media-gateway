# Session Bootstrap — ENV-ARM64-READY Functional Environment Readiness

你正在执行 `liqiangcc/jellyfin-web-media-gateway` 的独立环境验证 Task。

本文件只是 bootstrap / navigation 入口；Task Contract 在 `task.md`。

## Execution Context

```text
GitHub Issue: #63
Task Contract: docs/tasks/63-arm64-functional-environment-readiness/task.md
Expected worker: ubuntu-arm64
Expected environment label: env:ubuntu-arm64
Downstream handoff profile: docs/tasks/handoffs/ubuntu-arm64.md
```

## Start Protocol

开始前必须实际读取：

1. `AGENTS.md`
2. Issue #63 及 relevant comments
3. `docs/tasks/63-arm64-functional-environment-readiness/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/planning-priority.md`
6. task.md 引用的 Runner/Security/canonical docs

只在 live Issue 仍满足以下条件时 claim：

```text
status:ready
env:ubuntu-arm64
no active owner
```

claim 后开始新的 Attempt，严格执行 task.md 的 J0-J3。

特别提醒：

- 这是环境/功能 readiness，不是 #9 R003 性能测试；
- 不安装包、不提权、不跑 30/60 分钟 soak；
- Bilibili 只做 direct/no-proxy reachability eligibility 分类，不做 #36 ResolvedMedia/navigation 验证；
- 任何代理、Cookie、登录、challenge/CAPTCHA、指纹或访问控制绕过都不允许；
- 正常完成后 `[EXECUTION REPORT] -> status:review -> release owner -> STOP`；
- 阻塞时 `[BLOCKER REPORT] -> status:blocked -> release owner -> STOP`；
- 不自行执行 #36/#23/#9，不设置 done，不关闭 Issue。

## Authority

```text
canonical docs
→ AGENTS.md
→ task.md
→ prompt.md
→ live Issue state/comments
```

如果聊天背景与 GitHub live state 冲突，以 GitHub/canonical/task authority 为准。
