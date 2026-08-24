# Jellyfin Web Media Gateway

把 Ubuntu 手机变成低功耗网页媒体网关：服务器集中完成来源站点会话、媒体解析和媒体代理，再把同一个播放任务交给 Web、Jellyfin 等不同显示端。

Jellyfin 是首批支持的 Display Adapter，不是整个系统的强制依赖。

## 顶层架构

```text
手机 / Windows
     ↓
 /control
     ↓
Control Experience
     ↓
Gateway Core
├── Playback Coordinator
├── Resolution Service
├── Media Gateway
├── SiteAdapterRegistry
├── Session Vault
└── Display Adapters
     ↓                 ↓
Site Plugins         Displays
├── generic-ytdlp    ├── Web Display
├── bilibili         └── Jellyfin
└── ...
```

最重要的边界：

> **Gateway 可以识别站点，但不理解站点。**

Core 可以知道 `site_id = bilibili` 用于路由、会话隔离和 UI 标签，但 BV/EP/Season、Cookie、DOM selector、私有 API、清晰度枚举和下一集算法必须留在 Site Plugin。

Generic yt-dlp 也通过 SiteAdapter Registry 接入，不是 Core 的特殊后门。

## 播放状态

Gateway 是 `PlaybackSession` authority。

```text
PlaybackSession
├── session_revision
├── current_item: PlaybackItem
│   ├── item_revision
│   ├── SourceLocator
│   └── ResolvedMedia
├── PlaybackContext
├── position
├── active_display
└── display_generation
```

这样可以明确处理：

- 下一集；
- URL 过期重新 resolve；
- 多 Control 并发；
- 旧 item callback；
- 跨 Display handoff。

Jellyfin Session、浏览器 `<video>` 状态和站点 Chromium 播放器都不能覆盖 Gateway 的全局状态。

## Web 入口

```text
/          智能入口：Control / Display，MVP 默认 5 秒进入 TV Display
/display   确定性 Web Display
/control   确定性 Control
/control/sites  来源站点账号管理
```

`/display` 默认使用 TV-oriented viewport 沉浸布局。

MVP 的基本播放必须能在可信 LAN HTTP 环境工作，例如：

```text
http://10.0.0.116/
```

Service Worker、installable PWA、Screen Wake Lock 等 secure-context 能力只作为 HTTPS 部署下的渐进增强，不是基本播放成功条件。

## Control

Control 对用户是统一体验：

```text
Now Playing
Universal Remote
Native Site Panel（按需）
```

内部仍保持 Playback、Site、SiteSession、Display 四个领域分离。

Native Site Panel 使用服务器端 Site Browser Worker，而不是依赖普通 iframe。Site Browser Worker 只是通用 Chromium runtime；具体站点页面语义仍由对应 Site Plugin 解释。

## Site Auth

来源网站认证与 Gateway 用户身份分离。

MVP：

- 可信 LAN / 单用户；
- 暂不实现 Gateway 用户/RBAC；
- 每站点最多一个活动账号；
- Gateway 不保存网站密码；
- 解析确实需要登录时才触发 `SITE_AUTH_REQUIRED`；
- 登录成功自动恢复原 `SourceLocator`、目标显示端和播放意图；
- `/control/sites` 支持登录、重新登录和退出。

## 插件化路线

### 第一阶段：架构插件化

```text
gateway-core/
site-adapter-api/
plugins/
├── generic-direct/
├── generic-ytdlp/
└── ...
```

Rust trait + workspace，一起编译发布。重点是先证明 SiteAdapter Contract 和变化隔离边界成立。

### 第二阶段：进程插件

只有出现独立更新、依赖隔离、崩溃隔离或资源沙箱的真实需求后，再演进为版本化 IPC 插件。

优先进程插件，不优先 Rust `.so`。

## 当前实施顺序

当前不再继续扩展功能设计，先完成 Contract Freeze：

1. `SourceLocator`
2. `SiteAdapter`
3. `ResolvedMedia`
4. `PlaybackItem / PlaybackSession`
5. `DisplayAdapter`
6. scoped SiteAccess + EgressPolicy

随后进入风险驱动技术可行性验证：

```text
R007 Playback concurrency contract
→ R001 Media Path
→ R002 TV Browser remote audible playback
→ R003 ARM64 resource baseline
→ R008 Security boundary
→ Core Feasibility Review
```

Jellyfin Display、真实站点和 Native Site Panel 分别继续通过 R004/R005/R006 验证；Jellyfin 或 Native Site Panel 失败不能阻塞 Web-only Core。

详细实验、指标、成功标准和 Go / No-Go Gate 见 `technical-feasibility-validation.md`，具体实施顺序见 `mvp-plan.md`。

## 文档

先阅读：

- [文档导航与权威层级](docs/README.md)
- [需求说明](docs/requirements.md)
- [系统设计](docs/architecture.md)
- [Implementation Contracts](docs/implementation-contracts.md)
- [技术预研与可行性验证](docs/technical-feasibility-validation.md)
- [MVP 实施计划](docs/mvp-plan.md)
- [开发环境与多 Agent 协同](docs/development-environments.md)

专题：

- [Control UX](docs/control-ux.md)
- [Control 统一体验架构](docs/control-experience-architecture.md)
- [Site Plugin Architecture](docs/site-plugin-architecture.md)
- [安全设计](docs/security.md)

ADR：

- [ADR-0001：旁路网关](docs/adr/0001-sidecar-gateway.md)
- [ADR-0002：Gateway Playback Authority / Display Adapter](docs/adr/0002-gateway-playback-display-adapters.md)
- [ADR-0003：统一入口](docs/adr/0003-unified-entry-display-default.md)
- [ADR-0004：Site Auth / Account](docs/adr/0004-site-auth-account-management.md)
- [ADR-0005：统一 Control 体验](docs/adr/0005-unified-control-experience.md)
- [ADR-0006：Site Plugin Boundary](docs/adr/0006-site-plugin-boundary.md)

## Agent / 多环境开发

仓库长期规则见 [AGENTS.md](AGENTS.md)，完整调度模型见 [docs/development-environments.md](docs/development-environments.md)，Task 协议见 [docs/tasks/README.md](docs/tasks/README.md)。

默认执行方式：

```text
Web Coordinator
→ Web Worker implementation（默认最高优先级）
→ GitHub Actions automated verification
→ Cloud long-running verification
→ WSL / Windows interactive debugging（按需）
→ Ubuntu ARM64 / Real TV target proof（按需）
→ Web Coordinator Review
```

网页明确区分：

- **Web Coordinator Session**：长生命周期、项目全局控制面；
- **Web Worker Session**：短生命周期、单 Task 执行者。

重要边界：

- GitHub Actions 是 execution/verification backend，不是会 claim Issue 的 Worker；
- Cloud 主要承担长时间、无人值守、重复执行；
- WSL/Windows 主要承担交互式调试和 host-specific 能力；
- ARM64/TV 只在 claim 必须依赖目标环境真实性时使用；
- Implementation Result、Verification Result、Coordinator Gate Decision 必须区分。

具体跨会话任务优先使用 GitHub Issue + `docs/tasks/<issue>-<slug>/task.md`。只有 Web + Actions/Cloud 缺少所需 capability 时，才进入 [docs/codex/](docs/codex/) 的外部 Codex Worker 路径。

## 当前状态

设计收敛完成到可编码契约阶段；技术可行性验证框架和 Web-first / automated-verification 多环境工作流已经建立。尚未把真实设备/真实媒体路径标记为已验证，也尚无可运行正式版本；仓库当前也尚未建立实际 `.github/workflows/`，应在第一个可运行 Rust workspace/测试落地时再建立真实 CI。
