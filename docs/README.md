# 文档导航与权威层级

本仓库当前仍处于设计与功能闭环阶段。为避免 README、需求、架构、专题设计和 ADR 逐渐漂移，按下面权威层级解释。

## 1. 权威层级

### `docs/requirements.md` — WHAT

规范性需求：系统必须做到什么、不做什么、验收条件是什么。

### `docs/architecture.md` — CURRENT ARCHITECTURE

当前 canonical 系统架构：分层、状态所有权、依赖方向。

### `docs/implementation-contracts.md` — CODING CONTRACTS

编码前必须稳定的核心数据与行为契约，包括：

- `SourceLocator` / `SourceDescriptor`
- `SiteAdapter`
- `ResolvedMedia`
- `PlaybackSession` / `PlaybackItem`
- `DisplayAdapter`
- command/revision
- Session Vault / scoped capability / EgressPolicy

### `docs/technical-feasibility-validation.md` — EVIDENCE / FEASIBILITY GATES

风险驱动预研、PoC、真实设备验证和 Go / No-Go Gate。

P0 至少关注：Media Path、TV audible autoplay、ARM64 resource、Playback concurrency、Egress/Secret。

若实验推翻假设，必须反馈到 canonical 文档，而不是让研究文档静默覆盖架构。

### `docs/product-roadmap.md` — CURRENT PRODUCT DELIVERY ROUTE

当前用户能力 → 架构能力 → Task → Acceptance Gate 的交付映射。

它用于回答：

```text
现在先做什么？
某个功能由哪条能力线承接？
哪些 Issue 是当前执行路线，哪些只是历史 Evidence？
```

它不覆盖 requirements / architecture / implementation-contracts / security；只负责当前产品交付拆分和 Task 路由。

如果 `mvp-plan.md` 中较早的具体 Task 路由与 `product-roadmap.md` 的当前路线不同，以 `product-roadmap.md` 作为当前 Task sequencing/decomposition authority；`mvp-plan.md` 的通用 Phase/退出条件仍保留参考价值。

### 专题架构文档 — DETAIL

- `control-ux.md`
- `control-experience-architecture.md`
- `site-plugin-architecture.md`
- `security.md`
- `development-environments.md`：Web Coordinator / Worker、Implementation / Verification、capability routing。
- `runner-execution-architecture.md`：GitHub Actions execution bus、GitHub-hosted x64/ARM64、Ubuntu ARM64 Target Runner、安全与路由。
- `planning-priority.md`：当前执行优先级；环境就绪 → 功能闭环 → 真实场景兼容 → 性能/容量 → 生产加固。

### ADR — WHY

`docs/adr/*` 记录不可忽略架构决策的原因。新的 ADR 被接受后检查 requirements / architecture / contracts / feasibility / MVP / security。

### `docs/mvp-plan.md` — PHASE FRAMEWORK / WHEN

描述总体实施阶段、退出条件和测试里程碑。当前具体产品 Task 路由由 `product-roadmap.md` 补充并保持更新。

### 根 `README.md` — ENTRY

只解释项目目标、顶层架构、核心原则和入口。

## 2. Agent / Worker 工作入口

长期规则：`../AGENTS.md`。

默认模型：

```text
Web Coordinator
→ Codex/Web Worker implementation
→ GitHub Actions execution bus
     ├── GitHub-hosted x64: portable verification
     ├── GitHub-hosted ARM64: generic ARM64 verification
     └── Ubuntu ARM64 self-hosted: phone-specific target proof
→ WSL / Windows / Cloud / Ubuntu Codex only for required interactive capability
→ Real TV / Manual only for physical UX proof
→ Web Coordinator Review
```

Cloud **不部署 Runner**。

当前入口：

- `../AGENTS.md`：长期 Agent 规则。
- `product-roadmap.md`：当前用户功能 / Task 交付路线。
- `planning-priority.md`：当前项目执行优先级。
- `development-environments.md`：工作/环境解耦、Evidence 路由。
- `runner-execution-architecture.md`：Actions/Runner 自动执行架构。
- `tasks/README.md`：Issue + task 协议。
- `tasks/task.template.md`：Execution Plane / Runner / Target / Trust Gate 模板。
- `codex/README.md`：外部 Codex fallback。
- `codex/technical-feasibility.md`：外部环境技术预研 fallback。

新 Worker 会话优先：

> 读取 `AGENTS.md`、对应 GitHub Issue 和 `docs/tasks/<issue>-<slug>/task.md`；只在 live Issue 可 claim 时执行当前 Scope；优先通过 GitHub Actions/匹配 Runner 完成 Verification，提交报告后释放 ownership 并停止。

只有自动化无法表达、或需要人工交互/设备控制时，才启动对应外部 Worker。

## 3. 设计变更检查表

```text
Requirements
↓
Architecture
↓
Implementation Contracts
↓
Technical Feasibility / Evidence
↓
Product Roadmap / MVP Plan
↓
Security
↓
相关专题文档 / ADR
```

## 4. 当前最重要的不变量

- Gateway 是 `PlaybackSession` authority。
- Jellyfin 只是 `DisplayAdapter`。
- Control 是统一体验层，不是第二份业务状态。
- Core 可以识别 `site_id`，但不能理解 concrete site knowledge。
- Site Browser Worker 是通用 Chromium runtime。
- Generic yt-dlp 也是 Site Plugin。
- MVP 可信 LAN / 单用户，不实现 Gateway Identity/RBAC。
- SiteAccount 只代表来源站点会话。
- Native Site Panel 故障不得破坏已开始播放。
- GitHub-hosted ARM64 不等于目标手机 Evidence。
- Self-hosted Target Runner 不得继承 Vault/生产 Secret/Root 权限。
- P0 未完成前，不把 Web-only Core 写成已验证可行。
