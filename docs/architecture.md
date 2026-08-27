# 系统设计

## 1. 架构摘要

系统采用独立 Web Media Gateway，不修改 Jellyfin Server 核心。

Gateway 是网页媒体播放任务的权威状态源；具体网站逻辑位于 Site Plugin；最终显示通过 Display Adapter 接入。

```text
Control / Display Web UI
        ↓
Gateway Core
├── Control Experience
├── Playback Coordinator
├── Resolution Service
├── Media Gateway
├── Display Adapter Registry
├── SiteAdapterRegistry
├── Session Vault
└── EgressPolicy
        ↓
Site Plugins                  Display Adapters
├── generic-direct           ├── Web
├── generic-ytdlp            └── Jellyfin
├── bilibili
└── ...
```

核心原则：

- Gateway 是 `PlaybackSession` authority。
- Jellyfin 只是一个可选 `DisplayAdapter`。
- Control 是 View + Intent 聚合层，不保存第二份业务真状态。
- Gateway Core 可以识别 `site_id`，但不理解具体站点 URL、Cookie、DOM、私有 API、下一集算法。
- 第一版实现就通过 `SiteAdapterRegistry` 解析；Generic yt-dlp 也是插件。
- Site Browser Worker 是通用 Chromium runtime；站点语义由 Site Plugin 解释。
- Native Site Panel 是站点能力兜底，不是全局播放器。

详细数据契约见 `implementation-contracts.md`。

## 2. Web 入口

```text
GET /
  → 显示模式 / 控制模式
  → MVP 默认 5 秒无操作
  → /display?profile=tv

GET /display
  → 确定性 Web Display

GET /control
  → 确定性 Control

GET /control/sites
  → 站点账号管理
```

`PageRole` 与 `DisplayProfile` 分离：

```text
PageRole = control | display
DisplayProfile = tv | desktop | mobile | auto
```

分辨率只影响布局，不决定业务角色、站点账号或未来可能引入的 Gateway Identity。

MVP 基线必须在可信 LAN HTTP 下工作；Service Worker、Wake Lock 等 secure-context 增强能力不可作为基本播放成功条件。部署提供 HTTPS 时再启用增强能力。

## 3. 核心领域

### 3.1 Source Domain

用户输入 URL 后，不直接进入 yt-dlp。

```text
Input URL
→ SiteAdapterRegistry.recognize
→ SourceLocator
→ Resolution Service
→ SiteAdapter.resolve
→ ResolvedMedia
```

`SourceLocator` 是可重新定位内容的稳定、插件拥有的 opaque contract。Core 不解释 payload。

### 3.2 Playback Domain

一个 `PlaybackSession` 可以连续播放多个 item：

```text
PlaybackSession
├── session_id
├── session_revision
├── current_item: PlaybackItem
├── playback_context
├── position
├── subtitle_selection
├── active_display
└── display_generation
```

`PlaybackItem` 包含：

```text
item_id
item_revision
source_locator
resolved_media
metadata
```

切换下一集属于同一 Session 内的 `PlaybackItemTransition`，不属于 Display Handoff。

旧 item / 旧 display generation 的延迟回调不能覆盖当前状态。

### 3.3 Display Domain

首批 Adapter：

- `WebDisplayAdapter`
- `JellyfinDisplayAdapter`

Web Display 不依赖 Jellyfin；Jellyfin 不可用时 Web 路径仍工作。

跨显示端 handoff：

```text
snapshot current state
→ target probe / prepare
→ target confirms playable
→ target start
→ source stop
→ CAS commit active_display
```

目标未确认前不得静默停止当前显示端。

### 3.4 Site Session Domain

来源网站账号与 Gateway 用户身份完全分离。

```text
SiteAccount
→ SiteSessionRef
→ Session Vault
```

MVP 每站点最多一个活动账号，不实现 Gateway 用户/RBAC。

登录、重新登录、账号状态检查都通过 Site Plugin + Site Browser Worker + Session Vault 协作完成。

### 3.5 Control Experience

`/control` 对用户是一个完整体验：

```text
Now Playing
Universal Remote
Native Site Panel（可选）
```

内部仍分离：

```text
Playback Domain
Source / Site Domain
Site Session Domain
Display Domain
```

Control 只聚合 `ControlView` 并发送 command，不建立第二个状态 authority。

## 4. Site Plugin Boundary

### 4.1 Registry 是 Core 唯一入口

```text
Gateway Core
→ SiteAdapterRegistry
→ SiteAdapter Contract
→ concrete plugin
```

禁止：

```text
PlaybackCoordinator → BilibiliAdapter
Control → BilibiliPanel
Core → yt-dlp special case
if site == "bilibili" { ... }
```

允许 Core 使用 `site_id` 做路由、会话隔离、健康状态和 UI 标签。

### 4.2 Resolution Service

`Resolution Service` 是 Core 编排组件，不包含站点知识。

职责：

1. 接受 `SourceLocator`。
2. 通过 Registry 找到插件。
3. 获取 scoped `SiteAccessCapability`。
4. 调用 `adapter.resolve()`。
5. 校验 `ResolvedMedia` 输出。
6. 将结果交给 Playback Domain。

它从不直接调用 yt-dlp。

### 4.3 Site Browser Worker

Site Browser Worker 只负责通用 runtime：

- Chromium/Playwright 生命周期；
- profile attach/materialization；
- 远程画面和输入；
- navigation/URL/title/browser event；
- timeout、并发、资源限制。

它不理解 Bilibili/YouTube DOM。

```text
Browser Event
→ Site Plugin.browser_interpret
→ SourceContext / AccountState / NativePanelState
```

### 4.4 Native Site Panel

Native Site Panel 是 `/control` 对 Site Browser Worker 的呈现，不依赖普通 iframe。

站点原生操作分类：

1. `Source-changing`：选集/选择新视频 → 新 `SourceLocator` → resolve → item transition。
2. `PlaybackPreference-mappable`：清晰度/音轨/字幕等，只有稳定映射后进入 Gateway 通用能力。
3. `Site-only`：收藏、评论、页面设置、站点弹幕开关等，只影响 Site Domain。

站点 Chromium 内部 pause/seek 不能自动覆盖远端 Display 状态。

## 5. Media Gateway

Media Gateway 接收 `ResolvedMedia`，向 Display 暴露任务绑定的短期媒体 URL。

原则：

- 优先 Direct Stream。
- 分离音视频或兼容需要时允许 FFmpeg remux。
- 不默认视频重新编码。
- 上游 Cookie/Authorization 不下发给 Display。
- Secret 通过 `upstream_access_ref` / scoped capability 注入。
- 大型媒体默认不落盘。

## 6. Session Vault 与存储

Session Vault 是逻辑安全边界：

```text
/var/lib/web-media-gateway/
├── gateway.sqlite
├── vault/
│   ├── accounts/
│   └── browser-profiles/
├── cache/
└── runtime/
```

- `gateway.sqlite`：非敏感配置、插件/adapter 元数据、审计等。
- `vault/`：来源网站会话和 profile 的唯一所有者。
- `runtime/`：临时播放状态、临时 profile materialization、M3U、sockets 等，可丢失。
- 首个 MVP 不要求服务重启后恢复正在播放的 Session。

## 7. 网络与 EgressPolicy

SSRF 和内网访问例外由 Core 中央策略管理。

```text
EgressPolicy
├── public_web
└── configured_local_service
```

`public_web`：拒绝 loopback/private/link-local/metadata/reserved，并在每次 redirect 重检。

`configured_local_service`：只用于显式配置的内部集成，例如 Jellyfin；不接受任意用户 URL。

Site Plugin / Site Browser Worker 默认只获得 public web 或更窄的站点 allowlist，不能自行关闭 SSRF 检查。

## 8. Command / API 边界

资源生命周期：

```text
POST   /api/v1/sessions
GET    /api/v1/sessions/{id}
DELETE /api/v1/sessions/{id}
```

所有 Playback mutation 统一：

```text
POST /api/v1/sessions/{id}/commands
```

command 包括 play/pause/seek/stop/next/previous/subtitle/handoff。

每个 command 使用 `request_id`，并可携带 `expected_session_revision`；冲突返回 `REVISION_CONFLICT`。

Site 侧操作独立：

```text
GET  /api/v1/sites
POST /api/v1/sites/{site_id}/commands
```

Display 注册、媒体流与事件：

```text
GET  /api/v1/displays
POST /api/v1/displays/web/register
WS   /api/v1/events
GET  /stream/{token}/...
```

具体 JSON schema 由实现契约继续细化，但不再同时保留 `/intent`、`/handoff`、`/control` 多套 mutation 入口。

## 9. 主要数据流

### 9.1 URL → Web Display

```text
Control URL input
→ Registry.recognize
→ SourceLocator
→ Resolution Service
→ Site Plugin.resolve
→ ResolvedMedia
→ new PlaybackSession / PlaybackItem
→ Media Gateway
→ WebDisplayAdapter
→ Browser Player
```

### 9.2 下一集

```text
NextItem command
→ PlaybackContext.next SourceLocator
→ Resolution Service
→ fresh ResolvedMedia
→ create PlaybackItem(item_revision + 1)
→ CAS commit current_item
→ keep active_display
→ start new item
```

### 9.3 Native Site Panel 选集

```text
Site Browser Worker emits browser event
→ Site Plugin interprets event
→ SourceLocator
→ Site command SelectSource
→ Resolution Service
→ Playback Item Transition
```

### 9.4 Site Auth

```text
resolve
→ SITE_AUTH_REQUIRED
→ save PendingIntent
→ Site Browser Worker Auth Mode
→ Site Plugin validates account state
→ Session Vault atomically updates active session
→ retry PendingIntent
```

重新登录失败时尽可能保留旧 SiteSession。

## 10. 可观察性

允许记录：

- plugin/adapter id 和版本；
- `site_id`；
- 解析阶段/耗时；
- session/item revision；
- command 类型（不含敏感参数）；
- handoff 阶段；
- Direct/Remux/Transcode；
- worker 生命周期和稳定错误码。

禁止记录：

- Cookie / Authorization / API Key；
- 完整敏感 URL query；
- 临时媒体签名；
- 登录输入、二维码、远程画面；
- browser profile 内容。

## 11. 架构不变量

1. Core 不直接调用具体站点实现或 yt-dlp。
2. Site Browser Worker 不包含 concrete site knowledge。
3. ControlView 不成为第二份业务状态库。
4. 旧 item revision / display generation 不覆盖当前状态。
5. Display Adapter 不读取 Session Vault。
6. Site Plugin 不直接读取其他站点 Session。
7. 插件不能绕过 EgressPolicy。
8. Native Site Panel 故障不得停止已解析媒体播放。
9. Jellyfin 故障不得阻塞 Web Display。
10. HTTP LAN 下基本播放必须成立；secure-context 增强能力只能渐进增强。
