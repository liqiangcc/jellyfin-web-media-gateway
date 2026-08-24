# MVP 实施计划

## 技术选型建议

- 网关：Rust、Tokio、Axum。
- 元数据：SQLite。
- 媒体解析：yt-dlp JSON 输出，后续增加站点适配器接口。
- 媒体处理：FFmpeg，仅 remux/manifest proxy 为默认路径。
- 控制台：轻量 TypeScript PWA，或首版使用服务端静态 HTML。
- Jellyfin 集成：REST API、WebSocket session events、M3U Live TV。

## Phase 0：可行性验证

目标：不写完整产品，验证官方 Jellyfin Android TV 客户端能无修改播放动态 M3U。

- 部署 Jellyfin Server ARM64。
- 使用公开合法 HLS 测试源建立 M3U Tuner。
- 验证电视 Direct Play、字幕、暂停和恢复。
- 记录 Jellyfin API 的设备发现与远程播放行为。

退出条件：电视只安装官方客户端即可稳定播放测试频道。

## Phase 1：直接媒体网关

- 建立 `ResolvedMedia` 契约。
- 封装 yt-dlp JSON 调用。
- 实现 HLS/MP4 代理和短期签名 URL。
- 实现动态 M3U。
- 实现任务过期和清理。
- 拒绝 DRM、私网目标和不受支持格式。

退出条件：公开非 DRM URL 能从控制 API 转换并在电视播放，且无视频重编码。

## Phase 2：控制台与设备控制

- 设备列表与在线状态。
- URL 输入、解析进度和错误展示。
- 目标电视选择。
- 播放、暂停、停止、跳转。
- 响应式 Windows/手机界面。

退出条件：用户不进入 Jellyfin 管理页即可完成一次端到端播放。

## Phase 3：字幕与服务器端登录

- 字幕发现、语言标记和格式转换。
- 站点会话隔离与加密存储。
- 受控登录流程和过期提示。
- Cookie/Token 日志脱敏测试。

退出条件：电视不持有网站账号，仍能播放一项需要服务器会话的授权内容。

## Phase 4：稳定性与低功耗

- system service 与健康检查。
- CPU、内存、温度和并发限制。
- 故障恢复、短期 URL 刷新和缓存上限。
- ARM64 手机连续运行测试。

退出条件：网关空闲不产生持续解析/编码负载，失败不影响本地 Jellyfin 媒体。

## 独立探索：浏览器捕获

此项不阻塞 MVP：

- 验证 Chromium 在 chroot 中的画面/音频捕获。
- 测量 720p30 软件编码温度、功耗和延迟。
- 验证 Jellyfin HLS 缓冲是否可接受。
- 验证 DRM 安全画面失败时能明确拒绝。

只有测量结果可接受时才进入正式路线图。

## 测试矩阵

| 维度 | 最低覆盖 |
|---|---|
| 架构 | ARM64 Ubuntu 24.04 |
| 客户端 | Jellyfin Android TV |
| 媒体 | HLS、DASH、MP4、分离音视频 |
| 字幕 | SRT、VTT、ASS |
| 网络 | 同一 5 GHz Wi-Fi、短暂断网、URL 过期 |
| 安全 | SSRF、redirect、Cookie 泄露、命令注入、Token 重放 |
| 资源 | 空闲、解析峰值、Remux、并发拒绝 |

## 首个里程碑交付物

1. 可复现的 Jellyfin + 动态 M3U 演示。
2. 一个创建播放任务的 API。
3. 一个显示任务状态的最小控制页。
4. 一份手机端 CPU/温度/启动延迟记录。
5. 明确列出支持与拒绝的媒体类型。

