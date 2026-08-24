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

### 专题架构文档 — DETAIL

- `control-ux.md`：Control 的用户场景与状态体验。
- `control-experience-architecture.md`：统一 Control 体验如何聚合多个独立领域。
- `site-plugin-architecture.md`：具体站点知识与插件边界。
- `security.md`：跨领域安全不变量和威胁模型。

专题文档不能重新定义与 canonical 架构冲突的核心对象。

### ADR — WHY

`docs/adr/*` 记录“为什么做出某个不可忽略的架构决定”。

ADR 是决策历史，不是当前系统完整规范。接受新的 ADR 后，必须检查并按需要同步：

1. `requirements.md`
2. `architecture.md`
3. `implementation-contracts.md`
4. `mvp-plan.md`
5. `security.md`

### `docs/mvp-plan.md` — WHEN

只描述实施顺序、退出条件和测试里程碑，不重新设计架构。

### 根 `README.md` — ENTRY

面向第一次进入仓库的人，只解释项目目标、顶层架构、核心原则和文档入口。

## 2. 设计变更检查表

任何会改变核心边界的设计修改，在提交前至少检查：

```text
Requirements
    ↓
Architecture
    ↓
Implementation Contracts
    ↓
MVP Plan
    ↓
Security
    ↓
相关专题文档 / ADR
```

避免只新增 ADR，而没有让实施计划与主架构真正采用该决定。

## 3. 当前最重要的不变量

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
