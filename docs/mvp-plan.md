# MVP 实施计划

## 技术选型建议

- 网关：Rust、Tokio、Axum。
- 元数据：SQLite。
- 媒体解析：yt-dlp JSON 输出，后续增加站点适配器接口。
- 媒体处理：FFmpeg，仅 remux/manifest proxy 为默认路径。
- 控制台与 Web Display：轻量 TypeScript PWA；首版可先使用服务端静态 HTML + 原生 media element。
- 显示端抽象：Gateway 内部 `DisplayAdapter` trait。
- Jellyfin 集成：作为可选 Adapter，通过 REST API、WebSocket session events、M3U Live TV 接入。
- Web 入口：`/` 智能角色选择，`/display` 与 `/control` 作为确定性入口。

## Phase 0A：Web Display 与入口可行性验证（Core 阻塞项）

目标：证明网页媒体解析结果可以不经过 Jellyfin，直接由浏览器稳定消费，同时验证电视端只打开一个 Gateway URL 就能进入可投屏状态。

- 使用公开合法 HLS、MP4 测试源。
- 建立最小 Gateway 静态代理或签名 URL 演示。
- 实现 `/` 角色选择页，提供 Display / Control 两种选择。
- MVP 默认 5 秒无操作后进入 `/display?profile=tv`。
- 实现 `/display` 确定性入口和 TV-oriented immersive layout。
- 实现 `/control` 确定性入口；仅打开 Control 不自动注册显示端。
- 在 Windows / 手机 Chromium 中验证播放、暂停、跳转和结束事件。
- 使用 1280×720、1920×1080、3840×2160 viewport 验证 Display 布局。
- 验证 viewport 级沉浸播放不依赖 Fullscreen API 成功。
- 验证用户手势后可在支持的浏览器中请求 Fullscreen；失败不阻塞播放。
- 验证至少一个外挂字幕轨道。
- 记录 HLS/MP4 浏览器兼容边界和是否需要轻量 HLS JS 层。
- 验证浏览器请求中没有上游 Cookie、Authorization 或敏感源 URL 参数。

退出条件：

1. 浏览器无需 Jellyfin 即可稳定播放至少一种流媒体格式和一种字幕格式。
2. 访问 `/` 可选择角色，无操作 5 秒后进入 TV Display。
3. `/display` 和 `/control` 可直接进入对应角色，不依赖倒计时。
4. TV Display 在 Fullscreen 不可用时仍可铺满 viewport 正常播放。

## Phase 0B：Jellyfin Display Adapter 可行性验证（并行、非 Core 阻塞）

目标：验证官方 Jellyfin Android TV 客户端能无修改播放 Gateway 提供的动态媒体入口。

- 部署 Jellyfin Server ARM64。
- 使用公开合法 HLS 测试源建立 M3U Tuner。
- 验证电视 Direct Play、字幕、暂停和恢复。
- 记录 Jellyfin API 的设备发现与远程播放行为。
- 验证从一个已知播放位置启动的精度。

退出条件：电视只安装官方客户端即可稳定播放测试频道。

如果此实验失败，Core MVP 仍继续；失败结果用于重新设计 `JellyfinDisplayAdapter`，而不是阻塞 Resolver、PlaybackSession 或 Web Display。

## Phase 1：核心媒体网关与 PlaybackSession

- 建立与显示端无关的 `ResolvedMedia` 契约。
- 建立 `PlaybackSession` 状态模型。
- 封装 yt-dlp JSON 调用。
- 实现 HLS/MP4 代理和短期签名 URL。
- 实现任务过期、刷新和清理。
- 实现基础 `WebDisplayAdapter`。
- 实现 Unified Entry Router；倒计时配置留在 UI/配置层，不进入播放核心。
- 实现 `DisplayProfile` 基础模型：`tv | desktop | mobile | auto`。
- 实现 Display 空闲页的设备名称、在线状态和控制入口；二维码可作为同阶段增强项。
- 拒绝 DRM、私网目标和不受支持格式。
- 记录 Direct / Remux / Unsupported 决策。

退出条件：公开非 DRM URL 能通过控制 API 创建播放任务，并在 Web Display 播放，且默认无视频重新编码；根入口、Display 与 Control 三种路由行为稳定可复现。

## Phase 2：Display Adapter 与跨端接管

- 定义 `DisplayAdapter` trait 和 capability model。
- 实现 Web Display 注册、在线状态和能力探测。
- Control 中增加“在本机播放”，显式注册当前浏览器为 DisplayInstance。
- 实现 `prepare → confirm → commit` handoff 状态机。
- 接入 `JellyfinDisplayAdapter`：设备发现、动态 M3U、播放命令和状态转换。
- 统一播放、暂停、停止、跳转控制。
- 页面刷新后从 Gateway 恢复 `PlaybackSession` 与 `active_display`。
- adapter 失败不得覆盖 Gateway 已确认状态。

退出条件：同一个播放任务可以在 Web Display 与至少一个 Jellyfin 客户端之间双向接管；任一 adapter 离线不会让另一个 adapter 失效。

## Phase 3：字幕与服务器端登录

- 完善字幕发现、语言标记和格式转换。
- Web/Jellyfin adapter 分别声明字幕能力。
- 站点会话隔离与加密存储。
- 受控登录流程和过期提示。
- Cookie/Token 日志脱敏测试。
- 登录浏览器按需启动，登录完成后关闭并保留加密 profile。

退出条件：显示端不持有网站账号，仍能播放一项需要服务器会话的授权内容。

## Phase 4：稳定性与低功耗

- system service 与健康检查。
- CPU、内存、温度和并发限制。
- 故障恢复、短期 URL 刷新和缓存上限。
- Adapter 独立健康状态和熔断。
- ARM64 手机连续运行测试。
- Jellyfin 停止时验证 Web Display 不受影响。
- 空闲 `/` 与 `/display` 不进行高频轮询、不持续解码媒体。
- 支持的浏览器上验证 Screen Wake Lock 获取、释放和恢复；失败不影响播放器主流程。

退出条件：Gateway 空闲不产生持续解析/编码负载；单个 adapter 失败不影响核心媒体解析与其他显示端；失败不影响本地 Jellyfin 媒体。

## 独立探索：浏览器捕获

此项不阻塞 MVP，也不是 `WebDisplayAdapter` 的默认实现：

- 验证 Chromium 在 chroot 中的画面/音频捕获。
- 测量 720p30 软件编码温度、功耗和延迟。
- 验证低延迟协议而非默认 HLS 是否有必要。
- 验证 DRM 安全画面失败时能明确拒绝。

只有测量结果可接受且存在“无法解析媒体但允许捕获”的合法场景时才进入正式路线图。

## 自动化测试策略

Web Display 同时作为参考 Display Adapter 和主要自动化测试入口。

推荐 Playwright 拓扑：

```text
Browser A → /control
Browser B → /display?profile=tv
Browser C → /display?profile=desktop
```

优先自动化：

- `/` 点击 Control 后不再触发超时 Display 跳转；
- `/` 无操作到期后进入 TV Display；
- `/display`、`/control` 不依赖倒计时即可稳定进入；
- Control 页面不会因为打开而自动出现在 Display 列表；
- Browser B 注册为 Display 后，A 创建 Session 并发送播放；
- B 上报播放状态，A 可以 pause / seek / stop；
- B→C handoff 后旧显示端停止、新显示端从确认位置继续；
- 断线、重连、媒体 URL 过期和 adapter timeout；
- Fullscreen API 被拒绝时视频仍在 viewport 正常显示；
- TV profile 在 720p / 1080p / 4K viewport 下字幕和控件不溢出。

只有 Fullscreen 用户手势、Wake Lock、真实电视浏览器差异和 Jellyfin Android TV 行为需要保留真实设备测试。

## 测试矩阵

| 维度 | 最低覆盖 |
|---|---|
| 服务端架构 | ARM64 Ubuntu 24.04 |
| Web 路由 | `/`、`/display`、`/control` |
| Web Display | Windows Chromium、手机 Chromium、TV-style viewport |
| Display Profile | tv、desktop、mobile；720p/1080p/4K viewport |
| Jellyfin Adapter | Jellyfin Android TV |
| 媒体 | HLS、DASH、MP4、分离音视频 |
| 字幕 | SRT、VTT、ASS |
| Handoff | Web→Web、Web→Jellyfin、Jellyfin→Web |
| 浏览器能力 | Fullscreen allow/deny、Wake Lock available/unavailable |
| 网络 | 同一 5 GHz Wi-Fi、短暂断网、URL 过期 |
| 安全 | SSRF、redirect、Cookie 泄露、命令注入、Token 重放、显示端冒充、角色路由不越权 |
| 故障域 | Jellyfin down、Web Display disconnect、adapter timeout |
| 资源 | 空闲入口、空闲 Display、解析峰值、Remux、并发拒绝 |

## 首个里程碑交付物

Core 首个里程碑：

1. 一个 `ResolvedMedia` 契约和最小 Resolver 演示。
2. 一个创建 `PlaybackSession` 的 API。
3. 一个 `/` 智能入口和 `/display`、`/control` 两个确定性入口。
4. 一个无需 Jellyfin 即可播放公开 HLS/MP4 的最小 Web Display。
5. 一个 TV profile 的沉浸式大屏页面，Fullscreen 失败时仍可正常显示。
6. 一个显示任务状态、`active_display` 和错误的最小控制页。
7. 一组 Playwright E2E：入口路由、Web Display 注册、播放控制和 Web→Web handoff。
8. 一份手机端 CPU/温度/启动延迟记录。
9. 明确列出支持与拒绝的媒体类型。

Jellyfin 并行交付物：

1. 可复现的 Jellyfin + 动态 M3U 演示。
2. Jellyfin 客户端发现与远程播放行为记录。
3. 一份 `JellyfinDisplayAdapter` 能力/限制清单。
