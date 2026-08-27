# Codex / Agent Working Rules

本文件定义本仓库长期 Agent 约束。Task 契约位于 `docs/tasks/`，多环境路由见 `docs/development-environments.md`，自动执行架构见 `docs/runner-execution-architecture.md`，Codex Task 入口位于 `docs/tasks/handoffs/` 与 `.agents/skills/`。

## 1. 开始任何任务前

先读取：

1. `docs/README.md`
2. `docs/requirements.md`
3. `docs/architecture.md`
4. `docs/implementation-contracts.md`
5. `docs/technical-feasibility-validation.md`
6. `docs/mvp-plan.md`
7. `docs/security.md`
8. `docs/development-environments.md`
9. `docs/runner-execution-architecture.md`
10. 当前 Task / Research Item 相关专题文档 / ADR
11. 当前独立 Task 的 `docs/tasks/issue-lifecycle-protocol.md`

不要从旧聊天或旧代码猜测架构事实；冲突按 `docs/README.md` 权威层级处理。

## 2. 架构不变量

- Gateway 是 `PlaybackSession` authority。
- Jellyfin 只是可选 `DisplayAdapter`。
- Control 是统一体验层，不是第二份业务状态库。
- Core 可以识别 `site_id`，但不理解具体站点 URL、Cookie、DOM、私有 API、清晰度枚举、下一集算法或登录成功规则。
- 第一版即通过 `SiteAdapterRegistry`；`generic-ytdlp` 也是 Site Plugin，Core 不直接 fallback 到 yt-dlp。
- `SourceLocator` 是插件拥有的版本化 opaque 内容定位契约；短期 CDN/HLS URL 不是内容身份。
- Site Browser Worker 是通用 Chromium runtime；具体站点语义由 Site Plugin 解释。
- Display Adapter 不读取 Session Vault。
- Site Plugin 不直接读取 Vault，也不能绕过 `EgressPolicy`。
- Native Site Panel 故障不得停止已开始播放。
- Jellyfin 故障不得阻塞 Web Display。
- MVP 是可信 LAN / 单用户，但仍必须守住 Origin/CSRF、SSRF、Secret、token 边界。
- Self-hosted Target Runner 不得因为与 Gateway 同机而继承 Vault、生产 Secret、Root/ADB 权限。

若实现需要破坏任一不变量，停止实现，保留 Evidence，先走设计变更流程。

## 3. Research 结果规则

Research 结果只使用：

- `PASS`
- `CONDITIONAL PASS`
- `FAIL`
- `BLOCKED`

“理论上支持”“应该可行”“看起来没问题”不能当 PASS。

当前 P0 证据聚合顺序：

```text
R007 Playback concurrency contract closure
→ R001 Media Path
→ R002 TV Browser remote audible playback / autoplay
→ R003 ARM64 resource baseline
→ R008 Egress / Secret baseline
→ Core Feasibility Review
```

这个顺序不自动等于 Task publication dependency；hard dependency 以具体 Task Contract 为准。

R004 Jellyfin 可并行但不是 Core blocker；R005 验证真实 Site Plugin Contract；R006 Native Panel 非 Core blocker。

## 4. 实现与实验规则

- PoC 小而真实，不为 Demo 绕过架构/安全边界。
- Core 不直接调用具体站点或 yt-dlp。
- 不关闭 SSRF、任意私网限制，不下发 Cookie/Authorization，不建立开放代理。
- FFmpeg、yt-dlp、Chromium 使用 argv/structured API，禁止 shell 字符串拼接。
- 实验代码和正式代码边界清晰。
- 具体站点主要 diff 在 `plugins/<site>/`；若要向 Core/Playback/Display/Control 加站点业务分支，先评审。

## 5. Playback 并发测试最低集

实现/修改 Playback 至少覆盖：

- duplicate `request_id`；
- stale expected revision；
- stale item callback；
- stale re-resolve result；
- stale display generation；
- overlapping handoff；
- two-Control concurrent mutation。

旧异步结果不得覆盖已确认的新 `PlaybackItem`、`active_display` 或媒体解析结果。

## 6. 测试与 Evidence

优先维护：

- SiteAdapter conformance；
- ResolvedMedia Secret boundary；
- SourceLocator version；
- Playback revision / stale callback；
- Display generation / handoff rollback；
- EgressPolicy / SSRF；
- Web Display baseline；
- 当前 Research Item 对应实验。

必须区分：

```text
Implementation Result
!= Verification Result
!= Coordinator Gate Decision
```

Verification 明确 Candidate SHA。不能运行的测试必须说明原因，不得写成“通过”。

## 7. 设计变更流程

```text
Evidence
→ requirements.md（如产品目标改变）
→ architecture.md
→ implementation-contracts.md
→ technical-feasibility-validation.md
→ mvp-plan.md
→ security.md
→ 相关专题文档
→ 必要时 ADR
```

## 8. Git 规则

- 一个清晰研究/实现单元一个聚焦 commit/PR。
- 不把不相关 Spike 混在一起。
- 不 force push / 重写他人已引用历史，除非明确要求。
- 提交前检查 Secret、账号数据、Cookie、Token、敏感 URL、大型实验文件。
- Verification Evidence 能回指 Candidate SHA 和实际 run/job/环境。
- 已有可恢复 Candidate/PR 的 Task 发生 Worker 卡死时，优先保留同一 Issue/PR 进入下一 Attempt，不从头重建。

## 9. Codex-first / Runner-driven 协同

默认原则：

> **Codex-first，GitHub-hosted-first，Target-runner-for-proof，Web-for-coordination/review，Manual-for-UX。**

这里的 `Codex-first` 指 **Worker/client 路由**；它不改变 GitHub Actions/Runner/Target 的验证职责。

### 9.1 会话角色

```text
Web Coordinator
= 长生命周期、项目级控制面
= priority / Task split / publication / Review / Gate / recovery

Codex Worker
= 默认短生命周期、单 Task 仓库执行者
= code / tests / CI authoring / PR integration / fix / interactive repo work

Web Worker
= fallback / GitHub-only 轻量 Worker
= 仅当 Task 不值得启动 coding workspace、Codex 不合适或 Coordinator 明确指定
```

普通仓库 implementation / combined Task 默认先考虑 Codex Cloud (`env:cloud`)；不要仅因为历史上 Web Worker 可写 GitHub 就继续把代码任务默认派给 Web。

### 9.2 默认 Codex 开发闭环

```text
Codex Worker
→ read Issue / task.md / current main / existing candidate PR
→ code / tests / Candidate commit
→ GitHub Actions
→ matching Runner
→ read run/job/log/artifact
→ fix / rerun
→ [EXECUTION REPORT]
→ status:review
→ Coordinator Review
```

已有 Candidate/PR 时，下一 Attempt 优先续用并 rebase/integrate；不要因为 Worker 切换而复制业务 Task。

GitHub Actions 仍是真实 runtime Evidence 的默认执行面。Codex 本地命令可以用于开发/诊断，但 required Verification Evidence 仍以 `task.md` 为准。

### 9.3 GitHub Actions 是统一自动执行总线

Runner 路由：

```text
GitHub-hosted x64
→ portable x64 build/test/fmt/clippy/integration

GitHub-hosted ARM64
→ generic ARM64 build/test

Ubuntu ARM64 self-hosted Target Runner
→ phone-specific runtime / resource / compatibility proof

Real TV / Manual
→ physical UX/autoplay/remote/Jellyfin TV proof
```

Runner 不是 Agent，不 claim Issue。

### 9.4 Codex Cloud 是默认通用代码 Worker，不是 Runner

`env:cloud` 默认用于不需要设备专属交互能力的仓库实现/修复/重构/test/CI authoring。

Cloud Codex 可以：

- 修改代码/文档/tests/workflows；
- 维护 candidate branch/PR；
- 读取/触发 GitHub Actions；
- 分析 artifacts/logs 并迭代；
- 执行需要 coding workspace 的 repository integration/rebase。

但：

```text
Cloud Codex
!= GitHub Actions Runner
!= Ubuntu ARM64 phone
!= 家庭 LAN
!= TV Evidence
```

大量 repeated race / benchmark / regression 仍优先使用 GitHub-hosted matrix/sharding；不要让 Cloud shell 代替可审计 Actions Evidence。

### 9.5 Ubuntu ARM64 优先作为 Target Runner

通用 ARM64 compile/test 使用 GitHub-hosted ARM64。

手机 Target Runner 只用于：

- phone Ubuntu/chroot runtime；
- target FFmpeg/Chromium/Jellyfin；
- CPU/RSS/temperature/throughput；
- target media path；
- device-specific stable behavior。

只有 Actions job 无法表达的交互式 target debug、设备恢复、现场诊断才启动 Ubuntu ARM64 Codex (`env:ubuntu-arm64`)。

### 9.6 WSL / Windows 是能力路由，不是默认代码环境

- WSL (`env:wsl`)：本地 Linux 交互 debug、本地进程/文件、Actions failure investigation。
- Windows (`env:windows`)：ADB、Android host、手机重启/恢复/部署协调。

Task 不需要这些能力时优先 Codex Cloud，而不是为了“本地更真实”机械路由。

### 9.7 Target Runner 安全不变量

详见 `docs/runner-execution-architecture.md` 与 `docs/security.md`。

至少：

- 专用低权限用户；
- 默认无 root/sudo；
- work dir 与 Gateway Vault/生产 runtime 分离；
- 不持有长期站点 Secret、SSH key、Tailscale auth key、Root credential；
- 不可信 PR/fork 不直接命中 Target Runner；
- target job 只验证明确 Candidate SHA；
- 高 CPU/FFmpeg/Chromium/长跑 job 有 timeout、cleanup、资源限制。

### 9.8 Task 拆分先于环境路由

Coordinator 拆工作时必须按下面顺序思考：

```text
Goal / Research Item
↓
Task
↓
Claims
↓
Required Worker capabilities
↓
Verification Jobs
↓
Runner / Target
```

禁止反过来：

```text
“有 Cloud / x64 / ARM64 / phone 三个环境”
→ 所以创建多个相同业务 Task
```

Task 表示“做什么/证明什么”，有独立 Scope、Owner、Success Criteria 和 Review 生命周期。

Job 表示“在哪里、用什么命令验证某个 Claim 切片”，不 claim Issue，不拥有独立业务状态。

一个 Verification Task 可以包含多个 Jobs。

### 9.9 combined 与独立 Verification Task

默认普通工程任务使用 `combined`：

```text
Codex implementation
→ Candidate SHA
→ standard GitHub Actions Jobs
→ Coordinator Review
```

以下情况优先拆独立 Verification Task：

- Evidence Authority 独立，例如目标手机、真实 TV；
- Verification 生命周期/Owner/调度时点与 Implementation 不同；
- 关键 Research Gate 要独立追踪实现完成与 claim 证明；
- Target 暂不可用但实现可以先完成；
- Verification 需要独立重试、多轮 target proof 或独立 PASS/FAIL/BLOCKED。

Implementation Task 完成只代表候选实现已接受，不自动表示 Parent Goal / Research Gate 已通过。

### 9.10 Task 使用 Capability，再选择 Codex 环境

Task kind：

```text
implementation | verification | combined | research
```

常用 capability：

```text
github-read-write
repository-static-analysis
code-authoring
automated-build
automated-test
rust-build
rust-test
generic-arm64
high-repetition
interactive-linux-debug
cloud-interactive
adb
arm64-target-runtime
device-metrics
thermal-metrics
lan-access
tv-browser
remote-control
jellyfin-tv
manual-observation
```

默认环境映射：

```text
generic repo coding → env:cloud
interactive Linux-specific → env:wsl
Windows/ADB → env:windows
target-phone interactive → env:ubuntu-arm64
physical TV → env:manual-tv
GitHub-only lightweight fallback → env:web-gpt
```

### 9.11 Evidence 必须分开记录执行层次

例如：

```text
Orchestrator = codex-cloud
Execution Plane = github-actions
Runner = github-hosted-arm64
Target = generic Linux ARM64
```

或：

```text
Orchestrator = codex-cloud
Execution Plane = github-actions
Runner = ubuntu-arm64-self-hosted
Target = ubuntu-arm64-phone
```

“谁发起”不能替代“实际在哪里运行”。

## 10. Task Package：Issue + task.md + prompt.md

进入独立 Worker 队列的标准 Task Package：

```text
GitHub Issue
+
docs/tasks/<issue>-<slug>/
├── task.md
└── prompt.md
```

职责必须分开：

```text
Issue
= 实时 status / owner / blocker / branch / result summary
+ append-only Attempt / Review / Acceptance history

task.md
= 当前 Task 唯一执行契约

prompt.md
= 新会话 bootstrap / navigation only
```

`prompt.md` 不能重新定义或复制 Goal、Scope、Claims、Success Criteria、Architecture Invariants、Verification Job Matrix、Evidence 判断标准。

独立 Worker Task 进入 `status:ready` 前，默认先提交 `task.md + prompt.md`，并让 Issue 链接两者和 base commit。

Prompt 模板：

- `docs/tasks/prompt.template.md`

Issue 闭环协议：

- `docs/tasks/issue-lifecycle-protocol.md`

冲突时：

```text
canonical docs
→ AGENTS.md
→ task.md
→ prompt.md
```

Issue 是实时状态 authority，但不能通过评论静默改写 canonical architecture 或 Task Contract。

### 10.1 Task Publication Gate

Coordinator 发布独立 Worker Task 必须执行：

```text
status:draft
→ materialize Issue + task.md + prompt.md
→ read-back verify from GitHub
→ set status:ready + eligible env
→ read-back/search target worker queue
→ output downstream handoff entry
→ only then declare handoff complete
```

硬性规则：

- create/update 工具返回成功只说明写操作成功，不等于 Task 已发布；
- `status:ready` 必须是发布流程的最后写入步骤之一，不在 Issue 刚创建时提前设置；
- 在切 `status:ready` 前，必须重新读取 Issue、`task.md`、`prompt.md`，确认真实存在且互相引用正确；
- 切 `status:ready` 后，必须使用与目标 Worker 等价的队列查询（例如 `status:ready + env:cloud` / `env:ubuntu-arm64`）确认目标 Task 实际可见、无 active owner；
- 如果读回或队列查询失败，任务发布失败，保持/退回 `status:draft`，修复后重新验证；
- 在完整 read-back PASS 前，不得告诉用户“任务已创建 / 已发布 / 可以领取”；
- 发布验证 PASS 后，必须向用户给出真实的下游 Worker 执行入口提示词，不能只说“任务已发布”。

最小发布证明：

```text
Issue read-back PASS
+ task.md read-back PASS
+ prompt.md read-back PASS
+ ready/env state read-back PASS
+ target worker queue search PASS
```

发布后的最小 handoff 输出：

```text
Task: <real title>
Issue: #<real issue>
Worker: <real worker>
Environment: env:<real environment>
Prompt: docs/tasks/<real-issue>-<real-slug>/prompt.md
```

并提供该环境可直接复制的入口。Codex 环境优先使用 `$task-worker`；Web/Manual 使用各自 handoff profile。

所有值必须来自发布后的 GitHub read-back，不得保留 placeholder。如果 queue verification 失败，不得输出该入口。

详细检查见 `docs/tasks/README.md` 的 `Task Publication Gate`。

原则：

> **Plan is not execution. Write success is not publication. Publication requires independent read-back from GitHub, and handoff is incomplete until the downstream entry prompt is delivered.**

### 10.2 Issue Feedback / Review / Iteration / Closure

每次 Worker 成功 claim：

```text
status:ready
→ status:in-progress
→ starts Attempt N
```

Worker 正常结束：

```text
post [EXECUTION REPORT]
→ status:review
→ release active execution ownership
→ STOP
```

Worker 阻塞：

```text
post [BLOCKER REPORT]
→ status:blocked
→ release active execution ownership unless explicitly resolving blocker
→ STOP
```

Coordinator 必须读取 Issue history + `task.md` + candidate/PR + required Evidence，并把决定评论到 Issue：

```text
[COORDINATOR REVIEW]
Decision: ACCEPT | REVISE | BLOCK | SPLIT | NOT_PLANNED
```

规则：

- `REVISE`：如果 Contract 没变，`task.md` / `prompt.md` 不变，Issue → `status:ready`，输出下一轮 downstream entry；
- `BLOCK`：Issue → `status:blocked`；解阻后评论 `[COORDINATOR UNBLOCK]`，再 `status:ready`；
- `SPLIT`：只有独立 Scope/lifecycle/Success Criteria/Evidence Authority 才建 child Task；不能按 Runner/环境拆；
- Contract/routing bootstrap 改变：先回 `status:draft`，更新 canonical/process docs / `task.md` / `prompt.md`，再走 read-back / ready / queue verification；
- `ACCEPT`：只有 Final Acceptance Gate 全部满足后才允许 `status:done` + close。

Issue Comment 标准类型：

```text
[EXECUTION REPORT]
[BLOCKER REPORT]
[COORDINATOR REVIEW]
[COORDINATOR UNBLOCK]
[SPLIT]
[FINAL ACCEPTANCE]
[COORDINATOR REOPEN]
[CORRECTION]
```

Issue comments 默认 append-only；不要删除旧 Attempt / Review 来隐藏失败历史。

Worker execution outcome、Verification Claim Result、Coordinator Task Decision、Parent Goal / Research Gate Decision 必须分开。

最终关闭顺序：

```text
Success Criteria accepted
+ required Claims/Evidence accepted
+ no unresolved blocker
+ required child Tasks complete
→ post [FINAL ACCEPTANCE]
→ status:done
→ close Issue as completed
```

Worker 不得自行 `done` 或关闭 Issue。

完整模板与 reopen/split/contract revision 规则见 `docs/tasks/issue-lifecycle-protocol.md`。

## 11. Worker 协议

1. 读取 `AGENTS.md`；如果 Task Package 有 `prompt.md`，先用它完成 bootstrap；
2. 读取 Issue、全部 relevant comments、`task.md`、`docs/tasks/issue-lifecycle-protocol.md` 以及 Task 引用的适用 canonical /专题文档；
3. 确认 Task kind、Scope 和执行路径满足 Required Capabilities；
4. 确认 Issue `status:ready`、当前 environment eligible 且无 active owner；
5. claim + `status:in-progress`，确定新的 `Attempt N`；
6. 如果 Issue 历史存在前一 Attempt Candidate/PR，先评估复用/rebase/继续，不机械重建；
7. 只执行 Scope；
8. 提交 candidate / Evidence；
9. 正常完成时评论 `[EXECUTION REPORT]`，阻塞时评论 `[BLOCKER REPORT]`；
10. 记录真实 Task/Claim/Attempt/Job、Orchestrator/Execution Plane/Runner/Target；
11. 正常完成 → Issue `status:review`；阻塞 → `status:blocked`；
12. 释放 active execution ownership；
13. 停止，不自动开始下一项。

GitHub Actions Job 不参与 claim。大型 Research Item 可以拆多个 Task；最终 Gate 由 Web Coordinator 汇总已接受 Evidence 决定。

Codex 环境推荐入口：

```text
$task-worker Execute Issue #<issue> using `docs/tasks/<issue>-<slug>/prompt.md`.
```

Skill 不可见时，使用对应 `docs/tasks/handoffs/<env>.md` fallback。

## 12. Coordinator Review / Closure 协议

Coordinator 不能只在聊天中 Review。

每次 Review 必须：

1. 读取当前 Issue 与 relevant comments；
2. 读取 `task.md`；
3. 读取 candidate commit / PR / required Actions / artifact / target Evidence；
4. 评论 `[COORDINATOR REVIEW]`；
5. 明确 `ACCEPT / REVISE / BLOCK / SPLIT / NOT_PLANNED`；
6. 执行对应状态转换；
7. 如果回到 `status:ready`，重新给用户真实 downstream entry；
8. 只有 `[FINAL ACCEPTANCE]` 后才能 `status:done` 并关闭 Issue。

如果 Worker session 卡死但 GitHub 已存在 Candidate/PR/Evidence，Coordinator 应保留这些 durable 结果，结束僵死 ownership，并在同一 Issue 下一 Attempt 重新路由；不能依赖旧聊天恢复状态。

聊天只是操作界面；Task 的可恢复协作历史必须留在 GitHub。

## 13. Codex Task 入口

普通仓库代码 Task 的首选入口是对应 Codex 环境 handoff，默认：

```text
env:cloud
→ docs/tasks/handoffs/cloud.md
→ $task-worker Execute Issue #<issue> using `docs/tasks/<issue>-<slug>/prompt.md`.
```

需要特定交互能力时按 capability 改用 WSL / Windows / Ubuntu ARM64 Codex。

`docs/codex/technical-feasibility.md` 仅保留为没有标准 Task Package 时的阶段性 fallback；已有 Issue + task.md + prompt.md 时不得绕过 Task lifecycle。