# Codex / Agent Working Rules

本文件定义本仓库长期 Agent 约束。Task 契约位于 `docs/tasks/`，多环境路由见 `docs/development-environments.md`，自动执行架构见 `docs/runner-execution-architecture.md`，外部 Codex fallback 入口位于 `docs/codex/`。

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

当前 P0 顺序：

```text
R007 Playback concurrency contract closure
→ R001 Media Path
→ R002 TV Browser remote audible playback / autoplay
→ R003 ARM64 resource baseline
→ R008 Egress / Secret baseline
→ Core Feasibility Review
```

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

## 9. Web-first / Runner-driven 协同

默认原则：

> **Web-first，GitHub-hosted-first，Target-runner-for-proof，Codex-for-interaction，Manual-for-UX。**

### 9.1 网页两种会话

```text
Web Coordinator
= 长生命周期、全局控制面

Web Worker
= 短生命周期、单 Task 执行者
```

Coordinator 负责优先级、拆 Task、Claims/Success Criteria/Evidence、Review、Gate Decision。

Web Worker 是默认最高优先级 Implementation Worker，并可作为 GitHub Actions Orchestrator。

### 9.2 Web Worker 可以获得真实 runtime Evidence

默认闭环：

```text
Web Worker
→ code / tests / Candidate commit
→ GitHub Actions
→ matching Runner
→ Web Worker read run/job/log/artifact
→ fix / rerun
→ Coordinator Review
```

不要因为需要 build/test 就自动路由 WSL/Codex；也不要因为需要通用 ARM64 就自动占用目标手机。

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

### 9.4 Cloud 不部署 Runner

Cloud 资源有限，不建立 self-hosted Runner，也不作为普通 long-running 默认后端。

大量 repeated race / benchmark / regression 优先使用 GitHub-hosted matrix/sharding。

Cloud Codex 只用于：

- Cloud-specific 复现；
- Actions 不适合的长期交互 state；
- 明确授权的 Tailscale remote orchestration；
- 其他需要 Cloud 主机交互能力的 Task。

### 9.5 Ubuntu ARM64 优先作为 Target Runner

通用 ARM64 compile/test 使用 GitHub-hosted ARM64。

手机 Target Runner 只用于：

- phone Ubuntu/chroot runtime；
- target FFmpeg/Chromium/Jellyfin；
- CPU/RSS/temperature/throughput；
- target media path；
- device-specific stable behavior。

只有 Actions job 无法表达的交互式 target debug、设备恢复、现场诊断才启动 Ubuntu ARM64 Codex。

### 9.6 WSL / Windows 是交互式 fallback

- WSL：Actions failure 的交互式 Linux debug、本地进程/文件、快速 failure investigation。
- Windows：ADB、Android host、手机重启/恢复/部署协调。

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

### 9.8 Task 使用 Capability，不静态绑定环境

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

### 9.9 Evidence 必须分开记录执行层次

例如：

```text
Orchestrator = web-gpt
Execution Plane = github-actions
Runner = github-hosted-arm64
Target = generic Linux ARM64
```

或：

```text
Orchestrator = web-gpt
Execution Plane = github-actions
Runner = ubuntu-arm64-self-hosted
Target = ubuntu-arm64-phone
```

“谁发起”不能替代“实际在哪里运行”。

## 10. Issue 与 task.md

Issue 是动态状态 authority：

```text
status
active owner / claim
branch
candidate commit / PR
verification status
blocker
review state
result summary
```

`docs/tasks/<issue>-<slug>/task.md` 是稳定执行契约：

```text
Task kind
Goal / Context
Base / Candidate commit
Claims to verify
Required capabilities
Execution Plane
Runner class / image / labels
Target / Trust gate
Scope
Implementation Requirements
Verification Plan
Success Criteria
Evidence Contract
Failure / Blocked Rules
Deliverables
```

## 11. Worker 协议

1. 读取 `AGENTS.md`、Issue、`task.md`；
2. 确认执行路径满足 Required Capabilities；
3. 确认 Issue `status:ready` 且无 active owner；
4. claim + `status:in-progress`；
5. 只执行 Scope；
6. 提交 candidate / Evidence；
7. 记录真实 Orchestrator/Execution Plane/Runner/Target；
8. Issue → `status:review`；
9. 停止，不自动开始下一项。

大型 Research Item 可以拆多个 Task；最终 Gate 由 Web Coordinator 汇总已接受 Evidence 决定。

## 12. 外部 Codex 入口

优先使用 GitHub Issue + `docs/tasks/<issue>-<slug>/task.md`。

阶段性 fallback：

- `docs/codex/technical-feasibility.md`

只有 GitHub Actions / Target Runner / Manual 无法提供、且需要交互式能力时才路由外部 Codex。
