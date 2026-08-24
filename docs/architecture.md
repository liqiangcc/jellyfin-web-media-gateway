# 系统设计

## 1. 架构决策摘要

系统采用旁路网关，不修改 Jellyfin Server 核心。Jellyfin 继续负责设备、用户、播放会话和客户端兼容；网关只负责网页媒体解析、会话保护和临时媒体代理。

```mermaid
flowchart LR
    C[Windows / 手机控制台] -->|HTTPS / WebSocket| G[Web Media Gateway]
    G --> S[站点适配器 / yt-dlp]
    S --> O[来源站点]
    G --> P[HLS / DASH Proxy & Remux]
    G --> M[动态 M3U]
    J[Jellyfin Server] -->|读取| M
    J -->|读取媒体分片| P
    T[小米电视 Jellyfin] -->|播放与控制| J
    C -->|Jellyfin Session API| J
```

## 2. 组件

### 2.1 Control Console

- 单页 PWA，由网关托管。
- 负责 URL 输入、设备选择、登录流程、当前内容、播放状态和错误展示。
- 不直接接触媒体 Cookie；控制台只持有网关会话。
- 通过 Jellyfin Session API 和网关 WebSocket 恢复并同步当前播放状态。
- 可注册为浏览器显示端，使用 HTML5 播放器直接呈现网关媒体与字幕。

### 2.1.1 单显示端所有权

每个播放任务只有一个 `active_display`：电视 Jellyfin 客户端或一个浏览器实例。控制台始终显示服务端受控内容；只有成为活动显示端时才实际拉取和解码媒体。

```text
当前显示端 A
  → 保存进度并准备切换
  → 验证显示端 B 在线且可播放
  → 停止 A
  → B 从确认进度开始播放
  → 提交 active_display = B
```

切换过程按“准备、确认、提交”处理。新显示端无法启动时保留旧显示端，避免因网页刷新、设备离线或格式不兼容而中断播放。首个版本不实现多显示端同步播放。

### 2.1.2 与 Jellyfin Web 的边界

Jellyfin Server 自带 Web 客户端保留原样，可直接通过 `http://<server>:8096/web/` 访问，用于媒体库浏览、服务端验证和独立播放。自定义 Control Console 面向“URL → 解析 → 选择或接管显示端 → 控制”流程，是由网关托管的独立 PWA。

不 fork Jellyfin Web，也不把网站会话、解析器或网关逻辑注入其代码。控制台只调用稳定的 Jellyfin API，并可提供“打开 Jellyfin Web”入口，使 Jellyfin 与网关能够分别升级。

### 2.2 Gateway API

- 建议使用 Rust + Axum。
- 负责鉴权、任务生命周期、限流、SSRF 防护和状态事件。
- 使用 SQLite 保存非敏感配置、任务元数据和审计事件。

### 2.3 Session Vault

- 按站点隔离 Cookie 与必要凭据。
- 密钥不与数据库放在同一文件中。
- 仅解析子进程按最小权限读取目标站点会话。

### 2.3.1 Auth Browser Worker

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

### 2.4 Resolver

- 统一接口屏蔽 yt-dlp 与未来站点适配器差异。
- 输出标准 `ResolvedMedia`，包括视频、音频、字幕、请求头、过期时间和 DRM 判断。

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

### 2.5 Media Gateway

- 能直接播放时代理原始媒体，避免处理内容字节。
- 分离音视频时用 FFmpeg remux，不重新编码。
- 对 Jellyfin 暴露短期、签名化的本地 URL。
- 上游 Cookie 和 Authorization 永不下发给客户端。

### 2.6 Jellyfin Adapter

- 生成单一动态 M3U Tuner，例如“网页播放”。
- 调用 Jellyfin API 发现客户端、发送播放命令并读取会话状态。
- 使用专用、最小权限 Jellyfin 用户或 API Key。

## 3. 主要数据流

### 3.1 Direct Stream / Remux

```mermaid
sequenceDiagram
    participant U as 控制端
    participant G as 网关
    participant R as Resolver
    participant J as Jellyfin
    participant T as 小米电视
    U->>G: URL + 目标设备
    G->>R: 解析（携带服务器端站点会话）
    R-->>G: 视频/音频/字幕/过期时间
    G->>G: 校验并生成签名播放 URL
    G->>J: 更新频道并请求目标设备播放
    J->>G: 拉取媒体/字幕
    J-->>T: Direct Play 或 Remux
    T-->>J: 播放状态
    J-->>U: 状态经网关转发
```

### 3.2 浏览器捕获（非 MVP）

浏览器捕获需要 Chromium 渲染、音画捕获和 H.264 实时编码。在当前 Snapdragon 865 Ubuntu chroot 中缺少稳定硬件编码接口，因此只保留技术验证，不作为默认回退。

若未来实现，应作为独立 worker：

```text
Chromium profile
→ isolated display/audio
→ capture
→ encoder
→ low-latency stream
```

它不能绕过 DRM，且 HLS 不适合低延迟网页交互。

## 4. API 草案

```text
POST   /api/v1/jobs                 创建解析任务
GET    /api/v1/jobs/{id}            获取任务状态
DELETE /api/v1/jobs/{id}            停止并清理任务
GET    /api/v1/devices              获取 Jellyfin 显示设备
POST   /api/v1/devices/{id}/play    在目标设备播放
POST   /api/v1/devices/{id}/control 播放/暂停/停止/跳转
GET    /api/v1/sites                获取站点会话状态
POST   /api/v1/sites/{site}/login   启动服务器端登录流程
GET    /live/channels.m3u            Jellyfin M3U Tuner
GET    /stream/{token}/...           临时媒体代理
WS     /api/v1/events                任务与播放状态
GET    /api/v1/now-playing           当前受控内容与活动显示端
POST   /api/v1/display/handoff       将播放接管到指定显示端
```

所有路径仅为设计草案，实施前需完成 OpenAPI 契约和威胁审查。

## 5. 状态与存储

```text
/var/lib/web-media-gateway/
├── gateway.sqlite          # 任务、设备映射、非敏感配置
├── sessions/               # 加密的站点会话
├── cache/                  # 有界元数据/清单缓存
└── runtime/                # 临时 M3U、分片与 sockets
```

- 运行目录可放 tmpfs，重启后允许丢失。
- 会话和数据库必须持久化。
- 大型媒体默认不落盘；如需缓存必须设置容量和过期策略。

## 6. 部署

```text
OpenWrt LAN
├── Ubuntu 手机 10.0.0.116
│   ├── Jellyfin :8096
│   └── Gateway  :本地反向代理入口
└── 小米电视 DHCP 保留地址
```

- 控制台和 API 只允许 LAN/Tailscale 管理网络访问。
- 媒体流优先走局域网地址，不绕 Tailscale。
- Jellyfin 与网关独立启动、独立日志和独立故障域。

## 7. 可观察性

记录：解析阶段、持续时间、站点适配器、媒体格式、Direct/Remux/Transcode、错误码和资源使用。

禁止记录：完整 URL 查询 Secret、Cookie、Authorization、字幕正文和账号密码。

