# Codex / Agent Working Rules

本文件定义在本仓库工作的长期 Agent 约束。阶段性执行契约位于 `docs/tasks/`，外部 Codex 阶段入口位于 `docs/codex/`。

## 1. 开始任何任务前

先读取并遵守当前 canonical 文档：

1. `docs/README.md` — 文档权威层级
2. `docs/requirements.md` — WHAT
3. `docs/architecture.md` — CURRENT ARCHITECTURE
4. `docs/implementation-contracts.md` — CODING CONTRACTS
5. `docs/technical-feasibility-validation.md` — EVIDENCE / FEASIBILITY GATES
6. `docs/mvp-plan.md` — WHEN
7. `docs/security.md` — SECURITY INVARIANTS
8. 当前 Task / Research Item 直接相关的专题文档 / ADR

不要从旧聊天、README 摘要或旧代码猜测新的架构事实；发生冲突时按 `docs/README.md` 的权威层级处理。

## 2. 当前架构不变量

- Gateway 是 `PlaybackSession` authority。
- Jellyfin 只是可选 `DisplayAdapter`，不能反向定义 Core。
- Control 是统一体验层，不是第二份业务状态库。
- Gateway Core 可以识别 `site_id`，但不能理解具体站点 URL、Cookie、DOM、私有 API、清晰度枚举、下一集算法或登录成功规则。
- 第一版实现即通过 `SiteAdapterRegistry`；`generic-ytdlp` 也是 Site Plugin，Core 不允许直接 fallback 到 yt-dlp。
- `SourceLocator` 是插件拥有的版本化 opaque 内容定位契约；短期 CDN/HLS URL 不是内容身份。
- Site Browser Worker 是通用 Chromium runtime；具体站点语义由 Site Plugin 解释。
- Display Adapter 不读取 Session Vault。
- Site Plugin 不直接读取 Vault，也不能绕过 `EgressPolicy`。
- Native Site Panel 故障不得停止已经开始的 Gateway 播放。
- Jellyfin 故障不得阻塞 Web Display。
- MVP 面向可信 LAN / 单用户；这不等于可以关闭 Origin/CSRF、SSRF、Secret、token 等安全边界。

如果实现需要破坏任一不变量，停止当前实现路径，保留证据并走设计变更流程；不要在代码里偷偷引入例外。

## 3. 风险验证优先于扩大实现

当前阶段采用 risk-driven feasibility validation。

研究结果只使用：

- `PASS`
- `CONDITIONAL PASS`
- `FAIL`
- `BLOCKED`

禁止把以下内容当作 PASS：

- “理论上支持”
- “应该可行”
- “看起来没问题”
- “预计不会有问题”

实验失败是有效结果。优先 `Change / Defer / Drop` 可选能力，而不是为了 Demo 绕过架构或安全边界。

## 4. 当前 P0 Gate

以 `docs/technical-feasibility-validation.md` 与 `docs/mvp-plan.md` 为准：

```text
R007 Playback concurrency contract closure
→ R001 Media Path
→ R002 TV Browser remote audible playback / autoplay
→ R003 ARM64 resource baseline
→ R008 Egress / Secret baseline
→ Core Feasibility Review
```

- R004 Jellyfin PoC 可以并行，但失败不阻塞 Web-only Core。
- R005 Real Site 用于验证 Site Plugin Contract。
- R006 Site Browser Worker / Native Site Panel 非 Core blocker。

不要因为实现更有趣而跳过更高优先级 Gate。

## 5. 实现与实验规则

- PoC 可以小，但必须真实验证目标风险。
- 不为了 PoC 让 Core 直接调用具体站点或 yt-dlp。
- 不为了 PoC 关闭 SSRF、允许任意私网访问、下发 Cookie/Authorization 或建立开放代理。
- FFmpeg、yt-dlp、Chromium 等子进程使用 argv/structured API，禁止 shell 字符串拼接。
- 实验代码与正式代码边界清楚；未经验证的 PoC 不自动升级为产品实现。
- 新增具体站点的主要 diff 应位于 `plugins/<site>/`；如果需要在 PlaybackCoordinator/DisplayAdapter/Control 中加入具体站点业务分支，先做架构评审。
- 运行时第三方插件、动态 `.so`、插件市场、完整 Native Site Panel、完整 Jellyfin handoff 都不是首批 Core 前置条件。

## 6. 并发与状态规则

实现/修改 Playback 必须覆盖：

- `request_id` 幂等；
- command revision/CAS；
- stale item callback；
- stale display generation callback；
- re-resolve race；
- handoff transition race；
- 多 Control 并发 mutation。

任何旧异步结果都不得覆盖已确认的新 `PlaybackItem`、`active_display` 或新媒体解析结果。

## 7. 测试要求

优先维护：

- SiteAdapter conformance tests；
- ResolvedMedia schema / Secret boundary tests；
- SourceLocator version tests；
- Playback revision / stale callback tests；
- Display generation / handoff rollback tests；
- EgressPolicy / SSRF tests；
- Web Display 基线测试；
- 当前 Research Item 对应的可复现实验。

测试必须区分：

```text
Implementation Result
!= Verification Result
!= Coordinator Gate Decision
```

不能运行的测试必须明确说明原因；不得把“未运行”写成“通过”。

## 8. 设计变更流程

真实证据推翻当前假设时：

```text
Evidence
→ requirements.md（如果产品目标/非目标改变）
→ architecture.md
→ implementation-contracts.md
→ technical-feasibility-validation.md
→ mvp-plan.md
→ security.md
→ 相关专题文档
→ 必要时 ADR
```

不要只修改 PoC/专题文档，让 canonical 文档继续漂移。

## 9. Git 工作方式

- 一个清晰研究/实现单元一个聚焦提交。
- 不把多个不相关 Spike 塞进一个巨大 commit。
- 提交信息说明意图，不只写 `update` / `fix`。
- 不重写用户已有历史，不 force push，除非任务明确要求。
- 提交前检查 Secret、账号信息、Cookie、Token、完整敏感 URL、大型实验文件。
- Verification 必须明确所验证的 candidate commit SHA。

## 10. 多环境协同

本项目采用：

> **Web-first，Automation-second，Cloud-for-duration，Local-for-debug，Target-for-proof，Manual-for-UX。**

完整规则见 `docs/development-environments.md`；本节定义 Agent 必须遵守的最小调度不变量。

### 10.1 网页两种会话

```text
Web Coordinator Session
= 长生命周期、项目全局控制面

Web Worker Session
= 短生命周期、单 Task 执行者
```

Web Coordinator 负责优先级、拆 Task、定义 Claims / Success Criteria / Evidence、Review 和 Gate Decision。

Web Worker 是默认最高优先级 Implementation Worker。一个已经定义为独立 Task 的实现工作默认交给独立 Worker Session，避免 Coordinator 被局部实现上下文吞噬。

### 10.2 Web Worker 不等于“没有 runtime 能力”

Web Worker 自身没有任意本地进程，但可以调度或消费真实执行后端的 Evidence。

默认工程闭环：

```text
Web Worker
→ implementation / tests / commit / PR
→ GitHub Actions
→ Web Worker review run/job/log/artifact
→ fix if needed
→ Coordinator Review
```

因此不要使用错误规则：

```text
“需要 build/test，所以必须交给 WSL/Codex”
```

先判断 GitHub Actions 是否可以产生有效自动化 Evidence。

### 10.3 GitHub Actions 是默认 Automated Verification Plane

适合：

- build；
- test；
- fmt；
- clippy；
- contract/concurrency/security suite；
- 短中时长 integration/regression；
- artifact/log 采集。

Evidence 必须记录实际 runner OS/arch、workflow/run/job 和 commit SHA。

GitHub-hosted x86 runner 不得冒充 ARM64、手机热环境、家庭 LAN 或真实 TV。

仓库尚未建立 `.github/workflows/`；第一个 Rust workspace/真实自动化测试落地时建立有实际内容的 CI，不为了流程形式提前创建空 workflow。

### 10.4 Cloud 是长时间执行优先后端

需要长时间、无人值守、重复运行时，Cloud 优先于 WSL：

- 6h/24h soak；
- 大量 repeated race；
- benchmark matrix；
- failure injection；
- 长时间内存/稳定性观察；
- 持续自动化复现。

Cloud host 本身不能证明手机温度、家庭 LAN 或真实 TV。如果 Cloud 通过 Tailscale 在目标设备执行，Evidence 必须记录真实 Execution host / Target。

### 10.5 Local 环境用于交互式能力

- WSL：interactive Linux debug、反复加日志、启动本地进程、快速 failure investigation。
- Windows：ADB、Android host、手机重启/恢复/部署协调。

不要因为 WSL 能运行 `cargo test` 就默认把所有测试路由到 WSL；能自动化的先 Actions。

### 10.6 Target 环境只用于 target proof

- Ubuntu ARM64：ARM64 compatibility、Gateway/FFmpeg/Chromium/Jellyfin target runtime、CPU/RSS/temperature/throughput、目标稳定性。
- Real TV / Manual：audible autoplay、遥控器、TV UX、Jellyfin Android TV。

尽量把普通开发、测试编写、portable verification 留在 Web/Actions/Cloud/WSL，只把必须的最后 Proof 放到目标设备。

### 10.7 Implementation / Verification 逻辑分离

Task 可定义为：

```text
implementation | verification | combined | research
```

- Implementation 输出 candidate commit / PR。
- Verification 对确定 candidate SHA 验证 Claims。
- 普通工程任务可用 combined：Web 实现 + Actions 自动验证，不强制拆两个 Issue。
- ARM64/TV/资源/关键 Research 必须能独立追踪 target verification。

### 10.8 Capability-driven routing

Task 用 Required Capabilities 描述需要，而不是静态写“属于某环境”。

常用 capability：

```text
github-read-write
repository-static-analysis
code-authoring
automated-build
automated-test
rust-build
rust-test
long-running
interactive-linux-debug
adb
arm64-runtime
device-metrics
thermal-metrics
lan-access
tv-browser
remote-control
jellyfin-tv
manual-observation
```

默认决策：

```text
Web implementation?
→ Actions automated verification?
→ Cloud long-running?
→ WSL/Windows interactive capability?
→ ARM64/TV target proof?
```

### 10.9 Orchestrator / Executor / Target 必须分开记录

例如 Web Worker读取 Actions：

```text
Orchestrator = web-gpt
Executor = github-actions
Execution host = github runner
Target = runner
```

Cloud 远程在手机执行：

```text
Orchestrator = cloud-codex
Executor / Execution host = ubuntu-arm64-phone
Target = ubuntu-arm64-phone
```

“谁发起”不能替代“实际在哪里运行”。

### 10.10 Issue 与 task.md

GitHub Issue 是动态状态 authority：

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
Base / candidate commit
Claims to verify
Preferred execution path
Eligible environments
Required capabilities
Scope
Architecture Invariants
Implementation Requirements
Verification Plan
Success Criteria
Evidence Contract
Failure / Blocked Rules
Deliverables
```

`task.md` 不重复维护实时 status/owner/result。

### 10.11 Worker 协议

1. 读取 `AGENTS.md`、Issue、`task.md`；
2. 确认当前执行路径满足 Required Capabilities；
3. 确认 Issue `status:ready` 且无 active owner；
4. claim + `status:in-progress`；
5. 只执行 Scope；
6. 提交 candidate implementation 或 Evidence；
7. 记录真实 Orchestrator/Executor/Target、测试与未验证范围；
8. Issue → `status:review`；
9. 停止，不自动开始下一项。

大型 Research Item 可以拆多个 Task 并行，但每个 Task 只有一个 active owner；最终 Research Gate 由 Web Coordinator 汇总已接受 Evidence 决定。

### 10.12 Evidence 最小字段

runtime / Research Evidence 至少记录：

```text
Role: implementation | verification
Orchestrator
Executor
Execution host
Target host/device
OS / architecture
Relevant versions
Network path
Candidate / base commit
Workflow / run / job (if Actions)
Commands / steps
Metrics / artifact / raw evidence
Result
```

网页静态分析不能冒充 runtime PASS；Actions/Cloud/WSL 不能冒充 ARM64/TV target Evidence。

## 11. 阶段任务入口

优先使用 GitHub Issue + `docs/tasks/<issue>-<slug>/task.md`。

外部 Codex 的阶段性技术预研入口：

- `docs/codex/technical-feasibility.md`

只有 Web + Actions/Cloud 无法满足当前 Task Required Capabilities，或者明确需要交互式/目标环境时，才路由到相应外部 Worker。

推荐外部 Worker 指令：

> 读取 `AGENTS.md`、对应 GitHub Issue 和 `docs/tasks/<issue>-<slug>/task.md`；确认当前环境提供缺失 Required Capabilities 后 claim，只执行当前 Scope，记录真实 Executor/Target Evidence，提交后转 `status:review` 并停止。
