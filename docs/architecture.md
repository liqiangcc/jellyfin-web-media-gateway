# 系统设计

## 1. 架构决策摘要

系统采用独立 Web Media Gateway，不修改 Jellyfin Server 核心。Gateway 是网页媒体播放任务的权威状态源：负责网页媒体解析、站点会话保护、媒体生命周期、播放状态、`active_display` 和跨显示端 handoff。

最终显示通过可插拔 `DisplayAdapter` 接入。首批支持：

- `WebDisplayAdapter`：浏览器直接消费 Gateway 媒体，不经过 Jellyfin。
- `JellyfinDisplayAdapter`：通过动态 M3U、媒体代理和 Jellyfin API 把同一播放任务交给 Jellyfin 客户端。

Jellyfin 继续负责其自身用户、设备、客户端兼容、媒体库和内部播放会话，但不再充当整个 Gateway 系统的全局播放状态源。

```mermaid
flowchart LR
    C[Control PWA] -->|HTTPS / WebSocket| G[Web Media Gateway]
    G --> PC[Playback Coordinator]
    G --> R[Resolver / yt-dlp / site adapters]
    R --> O[来源站点]
    G --> MG[Media Gateway / Remux]
    PC --> WD[Web Display Adapter]
    WD --> B[Browser HTML5 Player]
    PC --> JD[Jellyfin Display Adapter]
    JD --> J[Jellyfin Server]
    J --> T[Jellyfin Clients / Android TV]
    WD --> MG
    JD --> MG
```

## 2. 核心领域模型

### 2.1 ResolvedMedia

Resolver 输出与显示端无关的标准媒体描述：

```text
ResolvedMedia
├── title
├── duration
├── video variants
├── audio variants
├── subtitle tracks
├── required headers (server only)
├── expires_at
└── drm / unsupported reason
```

`ResolvedMedia` 不包含 Jellyfin 特有字段，也不假设最终由浏览器、电视或其他客户端消费。

### 2.2 PlaybackSession

Gateway 为每次网页媒体播放创建 `PlaybackSession`：

```text
PlaybackSession
├── id
├── resolved_media
├── state
├── position
├── subtitle_selection
├── active_display
├── media_url_expiry
└── adapter_capability_snapshot
```

Gateway 是这些状态的 authority。Display Adapter 可以上报播放状态，但 adapter 的本地会话不是全局真相。

### 2.3 DisplayInstance

显示端统一表示为：

```text
DisplayInstance
├── id
├── adapter_type
├── label
├── online
├── capabilities
└── adapter_metadata
```

例如：

```text
adapter_type = web
  → 当前 Chromium 页面

adapter_type = jellyfin
  → 小米电视 Jellyfin Android TV
```

核心代码只依赖标准能力，不使用“如果是电视 / 如果是浏览器”的分支表达业务规则。

## 3. 组件

### 3.1 Control Console

- 单页 PWA，由 Gateway 托管。
- 负责 URL 输入、站点登录、解析进度、当前播放内容、显示端选择、handoff、播放状态和错误展示。
- 不直接接触上游媒体 Cookie；控制台只持有 Gateway 会话。
- 通过 Gateway WebSocket 恢复并同步 `PlaybackSession`。
- 当前浏览器可以注册为 Web Display，并使用 HTML5 播放器直接呈现 Gateway 媒体与字幕。
- 可提供“打开 Jellyfin Web”入口，但不依赖 Jellyfin Web 完成自身核心流程。

### 3.2 Gateway API

- 建议使用 Rust + Tokio + Axum。
- 负责鉴权、播放任务生命周期、显示端注册、handoff、限流、SSRF 防护和状态事件。
- 使用 SQLite 保存非敏感配置、站点信息、审计事件和必要元数据。
- 临时 `PlaybackSession` 可主要驻留内存；如果持久化，只用于故障诊断或未来恢复能力，不作为首个 MVP 要求。

### 3.3 Playback Coordinator

Playback Coordinator 管理 `PlaybackSession` 与 `DisplayAdapter` 之间的状态机。

职责：

- 创建和停止播放任务；
- 维护当前进度、字幕选择和 `active_display`；
- 根据 `ResolvedMedia` 与显示端能力选择 direct / remux / unsupported；
- 协调 adapter 的 prepare、start、pause、seek、stop 和状态回报；
- 执行跨显示端 handoff；
- 处理显示端断线、URL 过期和 adapter 失败。

它不得把 Jellyfin Session API 直接作为全局状态模型。

### 3.4 Display Adapter Interface

逻辑接口示意：

```text
DisplayAdapter
├── list_or_register_displays()
├── probe(display, media)
├── prepare(session, display)
├── start(session, display, position)
├── pause(display)
├── seek(display, position)
├── stop(display)
├── status(display)
└── subtitle_capabilities(display)
```

具体方法名在实现前通过 Rust trait / OpenAPI 契约确定；这里约束的是边界和职责。

### 3.5 WebDisplayAdapter

- 浏览器通过 Control PWA 注册为一个显示实例。
- adapter 根据 User-Agent / MediaCapabilities / 运行时探测记录浏览器能力。
- Gateway 向浏览器签发任务绑定、短期媒体 URL。
- 浏览器直接使用 HTML5 media element 或必要的轻量 HLS 播放层。
- 上游 Cookie、Authorization、真实源 URL 中的敏感参数不下发。
- 播放进度、暂停、结束和错误通过 WebSocket/HTTP 回报 Playback Coordinator。
- Web Display 不要求 Jellyfin 在线。

### 3.6 JellyfinDisplayAdapter

- 可选 adapter；启用后连接已有 Jellyfin Server。
- 生成动态 M3U Tuner 或其他经验证的稳定媒体入口。
- 调用 Jellyfin API 发现客户端、发送播放命令并读取 Jellyfin 内部会话状态。
- 把 Jellyfin 状态转换成统一 Display Adapter 状态后交给 Playback Coordinator。
- 使用专用、最小权限 Jellyfin 用户或 API Key。
- Jellyfin 故障不得破坏 Web Display、Resolver 或 Session Vault。

### 3.7 Session Vault

- 按站点隔离 Cookie 与必要凭据。
- 密钥不与数据库放在同一文件中。
- 仅解析子进程按最小权限读取目标站点会话。
- 所有 Display Adapter 都只能获得 Gateway 生成的临时媒体能力，不能读取 Vault。

### 3.7.1 Auth Browser Worker

- Chromium/Playwright 实际运行在 Ubuntu 服务端，按站点使用独立持久化 profile。
- 控制台通过一次性授权的远程画面通道显示浏览器，并转发键盘、鼠标和触摸输入。
- 用户在 Windows 或其他手机上完成账号密码、验证码、扫码和二次认证，但网站 Cookie、localStorage 与设备令牌始终留在服务端。
- 登录确认成功后关闭 Chromium 进程，只持久化加密的站点 profile；下次失效时再按需启动。
- 同一用户、同一站点默认只允许一个交互登录会话，并设置空闲超时和总时长上限。

```text
Control PWA
  ↔ 一次性登录通道（画面 + 输入）
Auth Browser Worker（服务端 Chromium）
  ↔ 目标网站
Session Vault
  ← 加密保存站点 profile / Cookie
```

不能通过 iframe 直接嵌入目标网站并复制登录状态：站点通常受 SameSite、HttpOnly、CSP 和跨域策略保护。远程操作服务端浏览器可以在不把凭据下发给控制设备的前提下保留完整登录上下文。

### 3.8 Resolver

- 统一接口屏蔽 yt-dlp 与未来站点适配器差异。
- 输出标准 `ResolvedMedia`。
- 检测 DRM 与不支持的保护方式。
- Resolver 不知道 Jellyfin 设备 ID、浏览器实例或 handoff 规则。

### 3.9 Media Gateway

- 能直接播放时代理原始媒体，避免处理内容字节。
- 分离音视频或显示端要求时可用 FFmpeg remux，不默认重新编码。
- 对 Display Adapter 暴露短期、签名化、本地 URL。
- URL 绑定用户、任务、允许的方法与生命周期。
- 上游 Cookie 和 Authorization 永不下发给显示端。

## 4. 单显示端所有权与 Handoff

每个播放任务只有一个 `active_display`。

```text
当前显示端 A
  → snapshot 已确认进度/字幕/状态
  → adapter B probe + prepare
  → B 确认可启动
  → B 从 snapshot position 启动
  → 停止 A
  → commit active_display = B
```

实现时可以根据 adapter 的能力调整“停止 A”和“启动 B”的精确顺序，但必须满足两个不变量：

1. 在 B 未确认可播放前，不得因为 handoff 请求静默杀死 A。
2. 只有 Gateway 可以提交新的 `active_display`。

若 B 启动后无法确认状态，Coordinator 必须返回明确的 handoff failure，并尝试保持或恢复 A，而不是让两个显示端长期竞争。

首个版本不实现多显示端同步播放。

## 5. 主要数据流

### 5.1 Web Display Direct Path

```mermaid
sequenceDiagram
    participant U as 用户/Control PWA
    participant G as Gateway
    participant R as Resolver
    participant P as Media Gateway
    participant W as WebDisplayAdapter
    participant B as Browser Player
    U->>G: URL
    G->>R: 解析（携带服务器端站点会话）
    R-->>G: ResolvedMedia
    G->>G: 创建 PlaybackSession
    U->>G: 选择当前浏览器显示
    G->>W: probe + prepare
    W-->>G: 可播放
    G->>P: 生成签名媒体/字幕 URL
    G->>B: start(position, signed URLs)
    B->>P: 拉取媒体/字幕
    B-->>G: progress / pause / error
```

该路径完全不要求 Jellyfin 运行。

### 5.2 Jellyfin Display Path

```mermaid
sequenceDiagram
    participant U as Control PWA
    participant G as Gateway
    participant JAD as JellyfinDisplayAdapter
    participant J as Jellyfin
    participant T as Jellyfin Client
    participant P as Media Gateway
    U->>G: handoff 到 Jellyfin 显示端
    G->>JAD: probe + prepare(session, display)
    JAD->>J: 检查设备和能力
    J-->>JAD: ready
    JAD-->>G: prepared
    G->>P: 准备签名媒体入口 / M3U
    G->>JAD: start(position)
    JAD->>J: 发起播放
    J->>P: 拉取媒体/字幕
    J-->>T: Direct Play 或 Remux
    JAD-->>G: Jellyfin session state
    G->>G: commit active_display
```

### 5.3 从 Web Handoff 到 Jellyfin

```text
Web 当前 00:18:24
→ Gateway snapshot = 00:18:24
→ JellyfinDisplayAdapter prepare
→ Jellyfin 确认可播放
→ Jellyfin 从约 00:18:24 启动
→ Web 停止
→ Gateway commit active_display = jellyfin:<device>
```

进度精度受 adapter 与媒体协议能力影响，Gateway 记录确认值并对偏差可观测。

### 5.4 浏览器捕获（非 MVP）

浏览器捕获需要 Chromium 渲染、音画捕获和 H.264 实时编码。在当前 ARM64 Ubuntu chroot 环境中缺少已验证的稳定硬件编码路径，因此只保留技术实验，不作为 Web Display 的默认实现。

Web Display 指“浏览器直接播放已解析媒体”，不是“服务器把任意网页录屏再推给浏览器”。浏览器捕获不能绕过 DRM。

## 6. API 草案

```text
POST   /api/v1/sessions                      创建播放任务
GET    /api/v1/sessions/{id}                 获取任务状态
DELETE /api/v1/sessions/{id}                 停止并清理任务
POST   /api/v1/sessions/{id}/refresh         刷新短期媒体能力

GET    /api/v1/displays                      获取所有显示端
POST   /api/v1/displays/web/register         注册当前浏览器显示端
DELETE /api/v1/displays/web/{id}             注销浏览器显示端
POST   /api/v1/sessions/{id}/handoff         接管到指定显示端
POST   /api/v1/sessions/{id}/control         播放/暂停/停止/跳转

GET    /api/v1/sites                         获取站点会话状态
POST   /api/v1/sites/{site}/login            启动服务器端登录流程

GET    /live/channels.m3u                    Jellyfin Adapter 动态入口
GET    /stream/{token}/...                   临时媒体代理
WS     /api/v1/events                        任务、显示端与播放状态
GET    /api/v1/now-playing                   当前播放任务与活动显示端
```

旧的 `/devices/{id}/...` 语义不再作为核心 API；Jellyfin device 是 `DisplayInstance` 的一种 adapter metadata。

所有路径仅为设计草案，实施前需完成 OpenAPI 契约和威胁审查。

## 7. 状态与存储

```text
/var/lib/web-media-gateway/
├── gateway.sqlite          # 配置、站点、adapter、审计等非敏感元数据
├── sessions/               # 加密的站点会话
├── cache/                  # 有界元数据/清单缓存
└── runtime/                # 临时播放状态、M3U、分片与 sockets
```

- 运行目录可放 tmpfs，重启后允许丢失。
- 站点会话和必要数据库必须持久化。
- 首个 MVP 不要求恢复正在播放的 `PlaybackSession`。
- 大型媒体默认不落盘；如需缓存必须设置容量和过期策略。

## 8. 部署

```text
受信任 LAN / Tailscale 管理网络
├── Ubuntu ARM64 手机
│   ├── Web Media Gateway
│   └── Jellyfin Server（可选 Display Adapter 依赖）
├── Windows / 手机浏览器
│   └── Control PWA + Web Display
└── 小米电视
    └── Jellyfin Android TV（启用 Jellyfin Adapter 时）
```

- Control PWA 和 API 只允许 LAN/Tailscale 管理网络访问。
- 媒体流优先走局域网地址，不绕 Tailscale。
- Jellyfin 与 Gateway 独立启动、独立日志和独立故障域。
- Jellyfin 未启动时，Gateway 的 Web Display 路径仍可用。

## 9. 可观察性

记录：

- 解析阶段和持续时间；
- 站点适配器；
- 媒体格式；
- Display Adapter 类型与能力选择；
- handoff 阶段、耗时与失败点；
- Direct / Remux / Transcode；
- 资源使用与稳定错误码。

禁止记录：

- 完整 URL 查询 Secret；
- Cookie；
- Authorization；
- 临时媒体签名；
- 登录输入；
- 字幕正文。
