# MVP 实施计划

## 技术选型建议

- 网关：Rust、Tokio、Axum。
- 元数据：SQLite。
- 媒体解析：yt-dlp JSON 输出，后续增加站点适配器接口。
- 媒体处理：FFmpeg，仅 remux/manifest proxy 为默认路径。
- 控制台与 Web Display：轻量 TypeScript PWA；首版可先使用服务端静态 HTML + 原生 media element。
- 显示端抽象：Gateway 内部 `DisplayAdapter` trait。
- Jellyfin 集成：作为可选 Adapter，通过 REST API、WebSocket session events、M3U Live TV 接入。

## Phase 0A：Web Display 可行性验证（Core 阻塞项）

目标：证明网页媒体解析结果可以不经过 Jellyfin，直接由浏览器稳定消费。

- 使用公开合法 HLS、MP4 测试源。
- 建立最小 Gateway 静态代理或签名 URL 演示。
- 在 Windows / 手机 Chromium 中验证播放、暂停、跳转和结束事件。
- 验证至少一个外挂字幕轨道。
- 记录 HLS/MP4 浏览器兼容边界和是否需要轻量 HLS JS 层。
- 验证浏览器请求中没有上游 Cookie、Authorization 或敏感源 URL 参数。

退出条件：浏览器无需 Jellyfin 即可稳定播放至少一种流媒体格式和一种字幕格式。

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
- 拒绝 DRM、私网目标和不受支持格式。
- 记录 Direct / Remux / Unsupported 决策。

退出条件：公开非 DRM URL 能通过控制 API 创建播放任务，并在 Web Display 播放，且默认无视频重新编码。

## Phase 2：Display Adapter 与跨端接管

- 定义 `DisplayAdapter` trait 和 capability model。
- 实现 Web Display 注册、在线状态和能力探测。
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

退出条件：Gateway 空闲不产生持续解析/编码负载；单个 adapter 失败不影响核心媒体解析与其他显示端；失败不影响本地 Jellyfin 媒体。

## 独立探索：浏览器捕获

此项不阻塞 MVP，也不是 `WebDisplayAdapter` 的默认实现：

- 验证 Chromium 在 chroot 中的画面/音频捕获。
- 测量 720p30 软件编码温度、功耗和延迟。
- 验证低延迟协议而非默认 HLS 是否有必要。
- 验证 DRM 安全画面失败时能明确拒绝。

只有测量结果可接受且存在“无法解析媒体但允许捕获”的合法场景时才进入正式路线图。

## 测试矩阵

| 维度 | 最低覆盖 |
|---|---|
| 服务端架构 | ARM64 Ubuntu 24.04 |
| Web Display | Windows Chromium、手机 Chromium |
| Jellyfin Adapter | Jellyfin Android TV |
| 媒体 | HLS、DASH、MP4、分离音视频 |
| 字幕 | SRT、VTT、ASS |
| Handoff | Web→Web、Web→Jellyfin、Jellyfin→Web |
| 网络 | 同一 5 GHz Wi-Fi、短暂断网、URL 过期 |
| 安全 | SSRF、redirect、Cookie 泄露、命令注入、Token 重放、显示端冒充 |
| 故障域 | Jellyfin down、Web Display disconnect、adapter timeout |
| 资源 | 空闲、解析峰值、Remux、并发拒绝 |

## 首个里程碑交付物

Core 首个里程碑：

1. 一个 `ResolvedMedia` 契约和最小 Resolver 演示。
2. 一个创建 `PlaybackSession` 的 API。
3. 一个无需 Jellyfin 即可播放公开 HLS/MP4 的最小 Web Display。
4. 一个显示任务状态、`active_display` 和错误的最小控制页。
5. 一份手机端 CPU/温度/启动延迟记录。
6. 明确列出支持与拒绝的媒体类型。

Jellyfin 并行交付物：

1. 可复现的 Jellyfin + 动态 M3U 演示。
2. Jellyfin 客户端发现与远程播放行为记录。
3. 一份 `JellyfinDisplayAdapter` 能力/限制清单。
