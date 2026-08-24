# 文档导航与权威层级

本仓库当前仍处于设计阶段。为了避免同一原则在 README、需求、架构、专题设计和 ADR 中重复维护后逐渐漂移，后续文档按下面的权威层级解释。

## 1. 权威层级

### `docs/requirements.md` — WHAT

规范性需求。回答“系统必须做到什么、明确不做什么、验收条件是什么”。

如果专题设计与需求冲突，以当前 `requirements.md` 为准，并通过新的设计变更同步修正其他文档。

### `docs/architecture.md` — CURRENT ARCHITECTURE

当前 canonical 系统架构。回答“现在决定怎么分层、状态归谁、组件之间允许怎样依赖”。

这里应保持精炼，不复制每个专题的全部细节。

### `docs/implementation-contracts.md` — CODING CONTRACTS

开始编码前必须稳定的核心数据与行为契约，包括：

- `SourceLocator` / `SourceDescriptor`
- `SiteAdapter`
- `ResolvedMedia`
- `PlaybackSession` / `PlaybackItem`
- `DisplayAdapter`
- command/revision 语义
- Session Vault / scoped capability / EgressPolicy 边界

代码实现与测试首先对齐本文件。

### `docs/technical-feasibility-validation.md` — EVIDENCE / FEASIBILITY GATES

风险驱动技术预研、最小 PoC、真实设备验证和 Go / No-Go Gate。

它回答：

> 当前架构假设是否已经在真实浏览器、电视、Ubuntu ARM64、媒体协议、Jellyfin 和真实来源站点上得到证据支持？

该文档不重新定义 canonical architecture。若实验推翻当前假设，必须把结论反馈到 requirements / architecture / implementation contracts / MVP plan / security，必要时新增 ADR。

P0 Core Feasibility 至少关注：

- Media Path；
- TV Browser remote audible playback / autoplay；
- ARM64 resource baseline；
- Playback concurrency contract；
- Egress / Secret security boundary。

### 专题架构文档 — DETAIL

- `control-ux.md`：Control 的用户场景与状态体验。
- `control-experience-architecture.md`：统一 Control 体验如何聚合多个独立领域。
- `site-plugin-architecture.md`：具体站点知识与插件边界。
- `security.md`：跨领域安全不变量和威胁模型。
- `development-environments.md`：Web Coordinator / Web Worker、Implementation / Verification、环境 capability 与 Evidence 协同规则。
- `runner-execution-architecture.md`：GitHub Actions 统一执行平面、GitHub-hosted / Cloud / Ubuntu ARM64 Runner 分层、路由与安全边界。

专题文档不能重新定义与 canonical 架构冲突的核心对象。

### ADR — WHY

`docs/adr/*` 记录“为什么做出某个不可忽略的架构决定”。

ADR 是决策历史，不是当前系统完整规范。接受新的 ADR 后，必须检查并按需要同步：

1. `requirements.md`
2. `architecture.md`
3. `implementation-contracts.md`
4. `technical-feasibility-validation.md`（如果影响待验证假设或 Gate）
5. `mvp-plan.md`
6. `security.md`

### `docs/mvp-plan.md` — WHEN

只描述实施顺序、退出条件和测试里程碑，不重新设计架构。

技术风险验证的实验定义和判断标准由 `technical-feasibility-validation.md` 提供；`mvp-plan.md` 负责把这些 Gate 放进实施顺序。

### 根 `README.md` — ENTRY

面向第一次进入仓库的人，只解释项目目标、顶层架构、核心原则和文档入口。

## 2. Agent / Worker 工作入口

Agent 的长期仓库规则由根 `AGENTS.md` 定义；多环境调度规则由 `development-environments.md` 定义；自动执行与 Runner 规则由 `runner-execution-architecture.md` 定义。

默认模型：

```text
Web Coordinator
→ Web Worker implementation
→ GitHub Actions execution plane
     ├── GitHub-hosted runner: portable/fast verification
     ├── Cloud self-hosted runner: long-running/repeated verification
     └── Ubuntu ARM64 self-hosted runner: target proof
→ WSL / Windows external worker only for interactive capability
→ Real TV / Manual only for physical UX proof
→ Web Coordinator Review
```

网页包含两种独立会话：

- **Web Coordinator Session**：长期项目控制面；
- **Web Worker Session**：短期单 Task 执行者，使用 `env:web-gpt`。

当前入口：

- `../AGENTS.md`：长期架构、安全、测试、Git 和 Agent 规则。
- `development-environments.md`：Web-first、Implementation/Verification 分离、capability routing、Evidence 边界。
- `runner-execution-architecture.md`：Actions/Runner 执行架构、Runner 分层和安全约束。
- `tasks/README.md`：Issue + `task.md` 任务协议。
- `tasks/task.template.md`：稳定执行契约模板，包含 Execution Plane / Runner / Target / Trust Gate。
- `codex/README.md`：外部 Codex Worker fallback 入口说明。
- `codex/technical-feasibility.md`：需要交互式或目标环境能力时的阶段性技术预研入口。

新 Web Worker 会话优先读取对应 Issue / Task：

> 读取 `AGENTS.md`、对应 GitHub Issue 和 `docs/tasks/<issue>-<slug>/task.md`；claim `env:web-gpt` 任务，只执行当前 Scope；优先通过 GitHub Actions/匹配 Runner 完成自动验证，提交结果并转为 `status:review` 后停止。

只有 Web + Actions Runner 能力仍不足，或者确实需要交互式人工诊断时，才启动相应外部 Worker。

`AGENTS.md`、`docs/tasks/*` 与 `docs/codex/*` 是 Agent 工作指令，不高于本文件定义的 canonical 产品/架构文档；若任务 Prompt 与 canonical 文档冲突，应修复 Prompt 漂移而不是覆盖架构。

## 3. 设计变更检查表

任何会改变核心边界的设计修改，在提交前至少检查：

```text
Requirements
    ↓
Architecture
    ↓
Implementation Contracts
    ↓
Technical Feasibility / Evidence（如涉及外部假设）
    ↓
MVP Plan
    ↓
Security
    ↓
相关专题文档 / ADR
```

避免只新增 ADR，而没有让实施计划与主架构真正采用该决定；也避免实验已经证明某个假设失败，但 canonical 文档仍继续把它写成既定事实。

## 4. 当前最重要的不变量

- Gateway 是 `PlaybackSession` authority。
- Jellyfin 只是一个 `DisplayAdapter`。
- Control 是统一体验层，不是统一业务状态层。
- Gateway Core 可以识别 `site_id`，但不能理解具体站点规则。
- 所有 concrete site knowledge 必须停留在 Site Plugin Boundary 外侧。
- `Site Browser Worker` 是通用 Chromium runtime；具体 DOM/API/登录成功判定属于 Site Plugin。
- Generic yt-dlp 也是 Site Plugin，不允许成为 Core 特例。
- MVP 是可信 LAN / 单用户，不实现 Gateway Identity/RBAC。
- SiteAccount 只代表来源网站会话，不代表 Gateway 用户身份。
- Native Site Panel 失败不得破坏已经开始的 Gateway 播放。
- Self-hosted Runner 不得因为与目标 Gateway 同机而继承 Vault/生产 Secret/Root 权限。
- 尚未完成 P0 技术验证前，不把 Web-only Core 的真实设备可行性写成已验证事实。
