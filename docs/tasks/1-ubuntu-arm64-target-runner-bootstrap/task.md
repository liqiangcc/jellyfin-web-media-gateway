# Task — INFRA-001 Bootstrap Ubuntu ARM64 Target Runner

## Metadata

```text
GitHub Issue: #1
Parent Goal / Research Item: development infrastructure / target-proof execution plane
Task / Research ID: INFRA-001
Task kind: combined
Base commit: 3f4f8061e660e1561b9c571abfd6231de0a82415
Candidate commit: n/a (live state belongs in Issue)
Session bootstrap prompt: docs/tasks/1-ubuntu-arm64-target-runner-bootstrap/prompt.md
Preferred worker: ubuntu-arm64
Eligible worker environments: env:ubuntu-arm64
Required capabilities: github-read-write, arm64-target-runtime, interactive-linux-debug
```

> GitHub Actions / Runner 是 execution backend，不是会 claim Issue 的 Worker。
>
> 实时 status、owner、candidate、run/result 只保存在 Issue；Attempt / Review / Acceptance 使用 `docs/tasks/issue-lifecycle-protocol.md`。

## Goal

把当前 Ubuntu ARM64 手机准备成一个安全、稳定、可由 GitHub Actions 调度的 self-hosted **Target Runner**，并通过一个真实、受信、最小化的 target-runner smoke job 证明调度、ARM64 身份、低权限、workspace 隔离和基础生命周期都成立。

本 Task 只建立 Target Runner 基础设施，不得把结果解释为 R001、R003 或任何产品可行性 Gate PASS。

## Why / Context

当前自动执行架构要求：

```text
GitHub-hosted x64/ARM64
→ portable / generic proof

Ubuntu ARM64 self-hosted Target Runner
→ phone-specific target proof
```

后续 R001 / R003 等目标设备验证依赖一个可信、受限、可调度的手机 Runner。Runner 尚未存在，因此 bootstrap 本身必须先由 Ubuntu ARM64 外部 Codex/Operator 在目标机上完成；一旦 Runner 注册成功，最终 smoke 必须转到 GitHub Actions 真实调度，而不是只看本地进程启动。

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: provisioning + infrastructure smoke are one tightly coupled infrastructure outcome; this Task does not perform product verification.
```

如果实际执行发现“持久化启动机制”或“GitHub 管理权限/registration authorization”本身需要独立生命周期，可由 Coordinator `SPLIT`；不得因为 local/bootstrap 和 Actions 两个 execution plane 就机械拆 Task。

## Work Role

### Implementation

必须完成：

- 发现当前真实 Ubuntu/ARM64/chroot/container/init 环境；
- 创建或确认专用低权限 Runner 用户；
- 安装官方 GitHub Actions Linux ARM64 self-hosted runner；
- 使用执行时获取的短期 registration token 注册到 `liqiangcc/jellyfin-web-media-gateway`；
- 配置稳定的 service/supervisor/autostart；
- 设置 Target Runner labels；
- 隔离 Runner workdir 与 Gateway production/Vault；
- 如果仓库尚无可信 smoke workflow，创建最小 `workflow_dispatch` target-runner smoke workflow。

### Verification

Claims：

```text
C1: GitHub 能看到且调度这个 Ubuntu ARM64 Target Runner，labels/target 唯一且符合契约。
C2: Target smoke 真实运行在 ARM64 手机上，并且 job 用户不是 root。
C3: Runner workspace 与 Gateway production/Vault 分离，job 默认无 sudo/root；若真实 Vault 路径存在则默认不可读。
C4: Runner 的 service/supervisor 能被安全 stop/start/restart，并恢复 online；当前 Ubuntu runtime 的 autostart 已启用并可解释。
C5: Bootstrap/verification 没有把 registration token、PAT、Cookie、SSH key、Tailscale auth key 或其他长期 Secret 写入仓库/Issue/log/history。
```

## Task vs Job Boundary

本 Task 定义“Target Runner 基础设施是否准备好”；local bootstrap 与 GitHub Actions smoke 是不同执行 Job，不是两个业务 Task。

```text
INFRA-001
→ C1..C5
→ bootstrap/preflight jobs
→ target-runner smoke job
→ Evidence
→ Coordinator Review
```

## Routing Rationale

Bootstrap 前 Runner 不存在，因此：

```text
Implementation / preflight
→ external-codex on ubuntu-arm64-phone

Final infrastructure proof
→ github-actions
→ ubuntu-arm64-self-hosted
→ ubuntu-arm64-phone
```

GitHub-hosted ARM64 不能证明该手机 Runner；Cloud 不能证明该手机；本地 `run.sh` 在线也不能替代 GitHub Actions 调度成功。

## Preconditions

- 当前设备能访问 GitHub；
- 当前 Worker 能对目标 Ubuntu 环境进行必要的安装/用户/service 操作；
- 能通过安全方式在执行时获得一次性/短期 GitHub Actions runner registration token；
- registration token 不写入仓库、Issue、prompt、持久 shell history 或日志；
- 如需新增 smoke workflow，当前 Worker 具有 GitHub repo 写权限；
- 如果 repo policy 要求 PR/Review 才能让 workflow 进入受信 default branch，则先提交 candidate 并转 `status:review`，由 Coordinator Review 后再进行下一 Attempt 的 target smoke。

## In Scope

- 目标环境发现：OS、architecture、init/supervisor、container/chroot 情况；
- 专用 Runner OS 用户，例如 `gateway-runner`；
- GitHub Actions Linux ARM64 runner 安装和 repo registration；
- labels：
  - `self-hosted`
  - `linux`
  - `ARM64`
  - `ubuntu-arm64`
  - `target-device`
- Runner binary/config/workspace 与 `/var/lib/web-media-gateway/` 分离；
- service/supervisor/autostart；
- 最小 target-runner smoke workflow；
- GitHub Actions 真实调度与 Evidence；
- 安全清理、token unset/history cleanup；
- Issue `[EXECUTION REPORT]` / `[BLOCKER REPORT]`。

## Out of Scope

- R001 Media Path；
- R003 CPU/RSS/temperature baseline；
- FFmpeg media-path benchmark；
- Chromium/Jellyfin compatibility；
- Bilibili/其他 Site Plugin；
- Gateway production deployment；
- TV/manual UX；
- 给 Cloud 配 Runner；
- 把普通 generic ARM64 CI 移到手机；
- 为了 smoke 读取真实站点 Cookie、Jellyfin API key 或 Gateway Vault 内容；
- Android/ADB host-level reboot orchestration（如后续需要，单独评审）。

## Architecture Invariants

- GitHub Actions 是 execution bus；target self-hosted only for target proof。
- Runner 不是 Agent，不 claim Issue。
- Target Runner 使用专用低权限用户，不是 Gateway production service account。
- 最终 Runner process/job 用户不得是 root，默认不得拥有 sudo。
- Runner workspace 不得位于 `/var/lib/web-media-gateway/`。
- Target workflow 不由任意 untrusted PR/fork 自动触发。
- registration token / PAT / SSH key / Cookie / Vault Secret 不持久化到仓库、Issue 或 workflow log。
- 本 Task 的 bootstrap operator 可以在 `task.md` 明确允许的安装步骤中使用必要特权；**operator 是 root 不等于最终 Runner 可以是 root**。最终 runtime identity 必须单独验证。

## Files Expected to Change

仓库内预计最多：

- `.github/workflows/target-runner-smoke.yml`（仅当不存在等价可信 workflow 时）

目标机预计创建/修改：

- 专用 Runner 用户/home；
- GitHub Actions runner installation/config/work directory；
- 当前 Ubuntu runtime 合适的 service/supervisor/autostart 配置。

不要把 token、`.credentials` 内容或真实 Secret 复制进仓库。

## Implementation Requirements

1. **环境发现先于安装**
   - 记录 `uname -a` / `uname -m`；
   - 记录 `/etc/os-release`；
   - 记录当前 user；
   - 判断 PID 1 / systemd / chroot/container 状态；
   - 不因为文档写了 systemd 就假设 systemd 可用。

2. **专用身份**
   - 优先用户：`gateway-runner`；
   - 不加入 sudo/admin 类 group；
   - 不复用 Gateway production account；
   - Runner service 和 Actions jobs 以该低权限用户运行。

3. **目录隔离**
   - 推荐安装/work 根位于 `/home/gateway-runner/...`；
   - 不把 Runner `_work` 放到 `/var/lib/web-media-gateway/`；
   - 不授予 Runner 用户对 production Vault/profile 的额外访问权。

4. **Runner 安装/注册**
   - 使用 GitHub 官方 Linux ARM64 runner package；
   - 使用当前官方/仓库建议版本，不固定过期版本；
   - 可验证 checksum 时应验证；
   - registration token 仅在执行时获取和使用，使用后立即 unset/清理；
   - 不把 token 留在 shell history、Issue、repo 文件或日志；
   - 避免重复注册同一物理 Target；若已有 runner，先识别并安全复用/修复。

5. **Labels**
   - 至少：`self-hosted`, `linux`, `ARM64`, `ubuntu-arm64`, `target-device`；
   - 只有真实具备能力后，未来 Task 才可增加 `device-metrics` / `ffmpeg-runtime` / `chromium-runtime` / `jellyfin-runtime` / `lan-target`。

6. **持久化运行**
   - systemd 真可用时可以使用 runner service helper/systemd；
   - systemd 不可用时使用当前 Ubuntu runtime 中可审计、可自动启动、可安全 stop/start 的 supervisor 方案；
   - 不以 root 常驻 `run.sh` 作为最终方案；
   - 若当前 target runtime 无法建立符合要求的 autostart/supervisor，报告 BLOCKED，不自创不受控 root daemon。

7. **可信 smoke workflow**
   - 使用 `workflow_dispatch`；
   - `runs-on` 精确要求 target labels；
   - 不使用 `pull_request` 自动触发 target runner；
   - smoke 不需要 checkout/执行仓库业务代码即可证明基础设施；
   - 如果 workflow 必须先经过 repo Review/merge 才能成为受信 default-branch workflow，则本 Attempt 到 `status:review` 停止，下一 Attempt 再触发。

8. **Smoke 必须实际检查**
   - `uname -a`；
   - `uname -m` 为 ARM64/aarch64；
   - `id` / `id -u`，必须非 root；
   - `pwd` / workspace 不在 production path；
   - `$RUNNER_TEMP` 或等价目录可创建并清理临时文件；
   - 如果 `sudo` 存在，`sudo -n true` 必须失败；
   - 如果 `/var/lib/web-media-gateway/vault` 真实存在，job 用户默认不得读取；若不存在，明确记录 N/A，不伪造 Vault Evidence；
   - 输出不得包含 registration token / PAT / Secret。

9. **生命周期验证**
   - service/supervisor stop/start/restart 一次；
   - runner 能重新回到 GitHub online/idle；
   - autostart 已启用/配置并记录机制；
   - Android/宿主级完整 reboot 不属于本 Task 的硬性成功标准。

## Verification Plan

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J0 | C1,C4,C5 | external-codex | ubuntu-arm64-phone operator | ubuntu-arm64-phone | yes | environment/user/init/runner/service preflight + bootstrap | Issue report / command evidence |
| J1 | C1,C2,C3,C5 | github-actions | ubuntu-arm64-self-hosted | ubuntu-arm64-phone | yes | `target-runner-smoke` / `workflow_dispatch` | run + job + logs |
| J2 | C4 | external-codex | ubuntu-arm64-phone operator | ubuntu-arm64-phone | yes | safe service/supervisor stop/start/restart + GitHub online check | Issue report / runner state |

### Execution Plane

```text
Bootstrap: external-codex
Final smoke: github-actions
```

### Runner Selection

```text
Runner class: ubuntu-arm64-self-hosted
Runner labels: self-hosted, linux, ARM64, ubuntu-arm64, target-device
Target: ubuntu-arm64-phone
Trust gate: trusted-candidate-only / manual-dispatch
```

### Interactive debugging

```text
Ubuntu ARM64 external Codex required: yes for bootstrap and local recovery
Reason: the self-hosted runner does not exist before this Task.
```

交互式诊断不能替代 J1 GitHub Actions smoke。

### Runner Security Constraints

```text
Trusted candidate only: yes
Dedicated low-privilege runner user: required
Vault/profile access: forbidden
Production service mutation: forbidden
Cleanup: registration token unset; temporary files removed; no secret in logs/history
```

## Success Criteria

### Task success

1. `gateway-runner` 或等价专用用户存在，最终 Runner/job 非 root，默认无 sudo/admin 权限。
2. Runner binary/config/workspace 与 `/var/lib/web-media-gateway/` production/Vault 路径分离。
3. GitHub repo 中能看到唯一、预期的 Target Runner，包含 required labels，并能进入 online/idle。
4. `target-runner-smoke` 通过 GitHub Actions 实际调度到该手机并 PASS。
5. Smoke 证明 ARM64、non-root、workspace 可写/可清理、无 sudo；真实 Vault 存在时默认不可读，不存在时如实记录 N/A。
6. service/supervisor 可安全 stop/start/restart，Runner 能恢复 online；当前 Ubuntu runtime 的 autostart 机制已经配置并记录。
7. 没有 registration token、PAT、Cookie、SSH key、Tailscale auth key 或其他长期 Secret 泄露到 repo/Issue/log/history。
8. Issue 中有完整 Attempt Evidence，能区分 Bootstrap operator、Execution Plane、Runner、Execution Host、Target 和 Actions run/job。

### Verification claim success

```text
C1 PASS when: GitHub runner state + labels + successful scheduled J1 prove registration/scheduling.
C2 PASS when: J1 reports ARM64/aarch64 and non-root identity on the phone target.
C3 PASS when: J1 proves workspace separation/no-sudo and does not obtain forbidden Vault access when Vault exists.
C4 PASS when: service/supervisor restart restores runner online and autostart mechanism is configured/documented.
C5 PASS when: secret scan/review and execution logs show no persisted registration/long-lived secret material.
```

## Evidence Contract

每个 Attempt 至少记录：

```text
Role: implementation | verification | combined
Task / Claim: INFRA-001 / C1..C5
Attempt:
Job ID: J0 | J1 | J2
Orchestrator:
Execution plane:
Executor:
Runner class / labels (if Actions):
Execution host:
Target host/device:
OS / architecture:
Init / supervisor:
Runner version:
Network path:
Base / candidate commit:
Workflow / run / job:
Commands / steps:
Artifacts / logs:
Secret-cleanup check:
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

不得贴出 registration token、runner `.credentials` 内容、Cookie、PAT、SSH key 或真实 Vault Secret。

## Failure / Blocked Handling

### FAIL

- 最终 Runner/job 以 root 运行；
- Runner 用户实际拥有无需交互的 sudo/root；
- target workflow 可由 untrusted PR/fork 自动命中；
- workspace 放在 production/Vault 下；
- ARM64 identity 不符合；
- J1 调度到错误机器/错误 runner；
- registration token / 长期 Secret 被持久化或输出；
- service restart 后无法恢复，且原因属于当前实现错误。

### BLOCKED

- 当前 GitHub 权限无法安全获取 runner registration authorization/token；
- repo policy 要求 Coordinator Review/merge smoke workflow 后才能进行 J1；
- target runtime 没有可接受的 service/supervisor/autostart 能力；
- GitHub/网络故障导致无法注册或调度；
- 当前 Worker 缺少完成明确安装步骤所必需的 host capability。

BLOCKED 时必须按 `issue-lifecycle-protocol.md` 评论 `[BLOCKER REPORT]` 并停止，不降低安全标准。

## Deliverables

- 目标机低权限 Runner 安装/注册；
- service/supervisor/autostart；
- `.github/workflows/target-runner-smoke.yml`（如需要）；
- GitHub Actions smoke run/job Evidence；
- Issue Attempt report；
- candidate commit/PR（如仓库文件有修改）。

## Issue Feedback / Iteration Protocol

完整协议：`docs/tasks/issue-lifecycle-protocol.md`。

普通安装错误、workflow bug、Evidence 不足、同一 Claim 重测：保持本 Contract 不变，由 Coordinator `REVISE` 后进入下一 Attempt。

如果发现当前 Ubuntu runtime 根本需要不同的持久化架构、安全前提或独立 host-level bootstrap Scope，则由 Coordinator 评审是否修改 Contract 或 `SPLIT`。

## Completion Protocol

Worker 每个 Attempt 结束必须：

1. 正常结束评论 `[EXECUTION REPORT]`，阻塞评论 `[BLOCKER REPORT]`；
2. 正常结束 → `status:review`；阻塞 → `status:blocked`；
3. 释放 active execution ownership；
4. 停止，不自行开始下一 Attempt；
5. 不自行 `status:done` 或关闭 Issue。

只有 Coordinator `$task-reviewer` 在 Final Acceptance Gate 全部满足后，才能 `[FINAL ACCEPTANCE] → status:done → close Issue`。
