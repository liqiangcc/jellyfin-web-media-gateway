# Jellyfin Web Media Gateway

把 Ubuntu 手机变成低功耗网页媒体网关：服务器集中完成网页媒体解析、站点登录和媒体代理，再把同一个播放任务交给不同显示端。显示端既可以是当前浏览器，也可以是 Jellyfin Android TV；Jellyfin 是首批支持的 Display Adapter，而不是整个系统的强制依赖。

本项目不修改 Jellyfin 核心。网关把受支持网页中的视频、音频和字幕解析为受控的 HLS/DASH/MP4 媒体源，并集中保管网站登录会话。浏览器显示端可直接播放网关签发的媒体地址；Jellyfin 显示端则通过动态 M3U、媒体代理和 Jellyfin API 接入。

## 目标体验

```text
Windows / 手机
  Control PWA：选择内容、登录、控制、也可直接显示
                         │
                         ▼
Ubuntu 手机
  Web Media Gateway
  ├── Resolver / Session Vault / Media Proxy
  ├── Playback Coordinator
  └── Display Adapters
       ├── Web Display ───────────────→ 浏览器 HTML5 Player
       └── Jellyfin Display → Jellyfin → Android TV
```

用户在控制台粘贴网页地址，网关优先提取原始 HLS/DASH/MP4 与字幕，避免重新编码。创建播放任务后，用户可以让当前网页直接成为显示端，也可以把任务发送给 Jellyfin 客户端；两条路径共享同一个解析结果、媒体生命周期和播放进度模型。

## 统一入口与模式选择

Gateway 的根 URL 是面向人的统一入口，例如：

```text
http://10.0.0.116/
        ↓
   选择运行模式
   ├── 显示模式
   └── 控制模式
        ↓
   无操作超时
        ↓
默认进入电视显示模式
```

MVP 默认倒计时为 5 秒，并保留配置能力。显式入口始终可用：

- `/`：智能入口；允许选择模式，超时进入 Display + TV profile。
- `/display`：确定性 Display 入口，适合电视书签、kiosk 和自动化测试。
- `/control`：确定性 Control 入口，适合手机、Windows 和自动化测试。

角色和显示布局分开建模：路径/用户选择决定当前页面是 `control` 还是 `display`；`DisplayProfile` 再决定显示布局，例如 `tv`、`desktop`、`mobile`。屏幕尺寸可以影响 profile，但不能单独决定页面角色。

专用 Display 页面默认采用沉浸式电视布局：占满 viewport、黑色背景、视频 `contain`、大字幕、遥控器友好焦点和自动隐藏控制层。浏览器真正的 Fullscreen 受用户手势策略限制，因此页面先做到 viewport 级全屏，并在允许时通过一次用户交互进入 Fullscreen。

空闲 Display 可以显示设备名称、连接状态以及控制入口/二维码，便于手机扫码后进入控制模式。

## 播放会话与单显示端模型

Gateway 是播放任务的权威状态源。每个 `PlaybackSession` 任一时刻只有一个 `active_display`，显示端由统一的 `DisplayAdapter` 抽象表示。

```text
PlaybackSession
├── resolved media
├── position / subtitle / playback state
└── active_display
     ├── adapter = web
     └── adapter = jellyfin
```

切换显示端采用“准备、确认、提交”流程：先保存当前进度并验证新显示端可播放，再停止旧显示端，由新显示端从确认位置接管。新显示端启动失败时保留旧显示端，避免网页刷新、设备离线或格式不兼容导致播放被静默中断。

默认不做同一任务的多端同步播放，以避免双倍网络流量、进度竞争和额外解码负载。

## 与 Jellyfin 的边界

Jellyfin 保留其擅长的用户、设备、客户端兼容、媒体库和 Jellyfin 内部播放会话能力，但只负责 `JellyfinDisplayAdapter` 这一条显示路径。

- 浏览器直接显示时不需要经过 Jellyfin。
- Jellyfin Android TV 仍是首批重点支持的电视显示端。
- Jellyfin Web 可继续独立用于媒体库浏览、服务端验证和普通 Jellyfin 播放。
- 不 fork Jellyfin Server 或 Jellyfin Web，不把网站会话与解析器注入 Jellyfin。

## 核心原则

- Gateway 拥有网页播放任务、媒体生命周期、`active_display` 和跨显示端 handoff。
- Display Adapter 可插拔；Web 与 Jellyfin 是首批实现，未来可扩展其他显示端。
- `/` 是统一的人机入口，`/display` 与 `/control` 是稳定的确定性入口。
- 页面角色与 Display Profile 分离，不以分辨率推断控制权限或页面角色。
- Jellyfin 上游可持续升级，不维护大型私有分支。
- 网站账号、Cookie 和解析逻辑只存在于服务器。
- 登录时按需启动服务端浏览器，控制设备只远程操作其画面；完成后关闭浏览器进程并保留隔离会话。
- 上游 Cookie、Authorization 和站点 Token 永不下发给显示端。
- 优先 Direct Stream / Remux，保持低功耗和原始画质。
- DRM、无法合法解析的内容明确拒绝，不尝试绕过。
- 浏览器画面捕获仅作为独立实验，不进入首个 MVP。

## 文档

- [需求说明](docs/requirements.md)
- [系统设计](docs/architecture.md)
- [安全设计](docs/security.md)
- [MVP 实施计划](docs/mvp-plan.md)
- [ADR-0001：使用旁路网关而非修改 Jellyfin 核心](docs/adr/0001-sidecar-gateway.md)
- [ADR-0002：Gateway 持有播放会话并使用 Display Adapter](docs/adr/0002-gateway-playback-display-adapters.md)
- [ADR-0003：统一入口、角色选择与默认电视显示模式](docs/adr/0003-unified-entry-display-default.md)

## 当前状态

设计阶段，尚未提供可运行版本。
