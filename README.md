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

Jellyfin Session、浏览器 `<video>` 状态和站点 Chromium 播放器都不能覆盖 Gateway 全局状态。

## Web 入口

```text
/          智能入口：Control / Display，MVP 默认 5 秒进入 TV Display
/display   确定性 Web Display
/control   确定性 Control
/control/sites  来源站点账号管理
```

MVP 基本播放必须能在可信 LAN HTTP 下工作；Service Worker、Wake Lock 等 secure-context 能力只做渐进增强。

## Control

统一体验：

```text
Now Playing
Universal Remote
Native Site Panel（按需）
```

内部仍保持 Playback、Site、SiteSession、Display 四个领域分离。

## Site Auth

来源网站认证与 Gateway 用户身份分离。MVP 每站点最多一个活动账号，不保存网站密码；确实需要认证时返回 `SITE_AUTH_REQUIRED`，登录成功后恢复原播放意图。

## 插件化路线

第一阶段使用 Rust trait + workspace：

```text
gateway-core/
site-adapter-api/
plugins/
├── generic-direct/
├── generic-ytdlp/
└── ...
```

运行时 IPC 插件留到出现真实隔离/独立更新需求后再做。

## 当前实施顺序

先完成 Contract Freeze：

1. `SourceLocator`
2. `SiteAdapter`
3. `ResolvedMedia`
4. `PlaybackItem / PlaybackSession`
5. `DisplayAdapter`
6. scoped SiteAccess + EgressPolicy

随后：

```text
R007 Playback concurrency contract
→ R001 Media Path
→ R002 TV Browser remote audible playback
→ R003 ARM64 resource baseline
→ R008 Security boundary
→ Core Feasibility Review
```

## 文档

先阅读：

- [文档导航与权威层级](docs/README.md)
- [需求说明](docs/requirements.md)
- [系统设计](docs/architecture.md)
- [Implementation Contracts](docs/implementation-contracts.md)
- [技术预研与可行性验证](docs/technical-feasibility-validation.md)
- [MVP 实施计划](docs/mvp-plan.md)
- [开发环境与多 Agent 协同](docs/development-environments.md)
- [GitHub Actions Runner 执行架构](docs/runner-execution-architecture.md)

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

仓库长期规则见 [AGENTS.md](AGENTS.md)，调度模型见 [docs/development-environments.md](docs/development-environments.md)，Runner 执行架构见 [docs/runner-execution-architecture.md](docs/runner-execution-architecture.md)，Task 协议见 [docs/tasks/README.md](docs/tasks/README.md)。

默认执行方式：

```text
Web Coordinator
→ Web Worker implementation
→ GitHub Actions
     ├── GitHub-hosted x64: portable verification
     ├── GitHub-hosted ARM64: generic ARM64 verification
     └── Ubuntu ARM64 self-hosted: phone-specific target proof
→ WSL / Windows / Cloud / Ubuntu Codex only for interactive capability
→ Real TV / Manual for physical UX proof
→ Web Coordinator Review
```

关键边界：

- GitHub Actions 是统一 execution bus，不 claim Issue；
- Cloud **不部署 Runner**，普通验证使用 GitHub-hosted Runner；
- 大量 repeated tests 优先 GitHub-hosted matrix/sharding；
- GitHub-hosted ARM64 只证明 generic ARM64，不证明目标手机；
- Ubuntu ARM64 self-hosted Runner 只做 target-specific runtime/resource/compatibility proof；
- Target Runner 不得继承 Vault、生产 Secret、Root/ADB 权限；
- 外部 Codex 主要用于自动化难以表达的交互式诊断/设备控制；
- Implementation Result、Verification Result、Gate Decision 必须分离。

## 当前状态

设计已收敛到可编码契约阶段；技术可行性验证框架和 Web-first / runner-driven 工作流已经建立。仓库尚无正式可运行版本，也尚未建立实际 `.github/workflows/` 或 Ubuntu ARM64 Target Runner；第一个可运行 Rust workspace/测试落地后，优先启用 GitHub-hosted x64/ARM64 CI，再按需要部署目标手机 Runner。
