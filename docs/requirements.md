# 需求说明

## 1. 背景

现有设备角色：

- Ubuntu 手机：长期在线、低功耗、Root、外接 SSD，可运行 Web Media Gateway 和 Jellyfin Server。
- Windows / 其他手机：通过浏览器选择内容、完成网站登录、控制播放，也可以直接成为显示端。
- 小米电视：通过 Wi-Fi 连接，可使用 Jellyfin Android TV 客户端作为电视显示端。

普通投屏经常丢失字幕，并要求每台电视客户端维护各网站登录状态。本项目希望把登录、解析和媒体流转换集中到服务器，同时不把最终显示方式绑定到某一个客户端。

## 2. 产品目标

1. 控制设备通过浏览器完成操作，无需安装专用控制 App。
2. 网关统一维护网页媒体播放任务、媒体生命周期和当前显示端。
3. 至少支持浏览器直接显示；Jellyfin 作为首批外部 Display Adapter 支持电视播放。
4. 网站登录状态只保存在 Ubuntu 手机。
5. 支持将非 DRM 网页媒体转换为受控、可播放的媒体源。
6. 优先转发或重新封装原始媒体，不默认重新编码。
7. 支持独立字幕，并允许当前显示端选择字幕轨道。
8. 本地流量走局域网，不依赖公网或 Tailscale 中继。
9. 显示端通过统一 Display Adapter 抽象接入，避免核心播放模型依赖 Jellyfin。

## 3. 非目标

- 绕过 DRM、付费授权、区域限制或网站访问控制。
- 把任意交互网页完整转换成低延迟远程桌面。
- 重新实现 Jellyfin 的媒体库、用户和客户端生态。
- 为每个视频网站永久承诺兼容性。
- 将服务作为公网开放代理或多租户 SaaS。
- 首个版本实现同一播放任务的多端同步播放。

## 4. 核心用户故事

### US-01 创建网页媒体播放任务

用户在 Windows 或手机打开控制台，粘贴受支持网页 URL。网关完成解析后创建 `PlaybackSession`，并展示标题、媒体能力、字幕、过期时间与可用显示端。

### US-02 网页直接显示

用户可以把当前控制网页设为活动显示端。浏览器使用网关签发的短期媒体地址播放内容，不需要 Jellyfin，不接触网站 Cookie 或 Authorization。

### US-03 发送到 Jellyfin 显示端

如果启用了 Jellyfin Adapter，用户可以选择在线 Jellyfin 客户端，例如小米电视。网关把当前播放任务映射为 Jellyfin 可播放的媒体源并发起播放。

### US-04 集中登录

需要登录的网站由服务器维护独立浏览器配置或 Cookie；所有显示端均不接触网站凭据。

### US-05 字幕播放

网关发现字幕后，将字幕标准化为显示端可消费的轨道。Web 显示端和 Jellyfin 显示端分别通过各自 adapter 暴露字幕能力。

### US-06 失败可解释

解析或播放失败时，控制台明确区分：不支持的网站、登录失效、DRM、网络失败、媒体过期、代理失败、显示端离线、格式不兼容和 adapter 错误。

### US-07 跨显示端接管

用户可以把正在浏览器播放的内容切换到 Jellyfin 电视端，或从电视切回当前浏览器。新显示端确认可播放后，旧显示端停止，新显示端从确认进度接管。

### US-08 状态恢复

页面刷新后，控制台从 Gateway 恢复当前 `PlaybackSession`、`active_display`、播放进度、字幕轨道和播放方式，而不是依赖某个具体显示端作为全局状态源。

## 5. 功能需求

### FR-01 Control Console

- 响应式 Web/PWA，支持 Windows 和手机浏览器。
- 接收 URL 并展示解析进度与错误。
- 显示当前播放任务、活动显示端、进度、字幕和播放方式。
- 展示所有可用 Display Adapter 与 Display Instance。
- 保存常用站点入口，但不在浏览器本地保存网站密码。
- 当前浏览器可注册为 Web Display。

### FR-02 Playback Session

- Gateway 是网页媒体播放任务的权威状态源。
- 每个任务至少记录：解析结果、生命周期、播放状态、当前位置、字幕选择、`active_display` 和 adapter 能力快照。
- 同一任务任一时刻默认最多一个活动显示端。
- 页面刷新、adapter 重连或 Jellyfin 状态变化不得覆盖 Gateway 已确认的会话状态，必须通过状态协调流程合并。
- 临时任务服务重启后可以丢失；站点会话必须独立持久化。

### FR-03 Display Adapter

Display Adapter 是显示能力的统一边界。至少抽象以下能力：

- 枚举或注册显示实例；
- 探测在线状态与媒体能力；
- prepare / start / pause / stop / seek；
- 获取或上报播放状态；
- 字幕能力；
- handoff 所需的确认与错误信息。

首批实现：

- `WebDisplayAdapter`
- `JellyfinDisplayAdapter`

核心 Resolver、Session Vault 与 Media Gateway 不得依赖 Jellyfin 类型。

### FR-04 Web Display

- 浏览器可使用 HTML5 Player 直接播放网关媒体。
- 浏览器只获得任务绑定、短期签名的媒体 URL。
- 不返回上游 Cookie、Authorization、浏览器 profile 或任意代理凭据。
- 支持浏览器能力范围内的 HLS/MP4、字幕、暂停、恢复和跳转。
- 如果浏览器无法原生消费某种格式，adapter 必须明确返回能力不匹配，而不是隐式要求 Jellyfin。

### FR-05 Jellyfin Display Adapter

- Jellyfin 是可选显示适配器，不是 Gateway 核心依赖。
- 可通过动态 M3U/M3U8、媒体代理和 Jellyfin API 将任务暴露给 Jellyfin。
- 使用 Jellyfin API 获取客户端、播放状态并发送控制指令。
- Jellyfin 用户、设备和内部会话只在该 adapter 边界内处理。
- 电视只需要 Jellyfin 账号，不需要网站账号。
- Jellyfin 不可用时，Web Display 和核心解析能力仍应工作。

### FR-06 Display Handoff

- 切换采用 prepare → confirm → commit 模型。
- 切换前记录已确认播放位置和字幕状态。
- 新显示端必须先通过在线与可播放能力检查。
- 新显示端确认启动失败时，旧显示端继续保持活动。
- 成功启动新显示端后停止旧显示端，并提交新的 `active_display`。
- adapter 必须对位置精度差异做显式说明，不假设所有显示端能逐帧接管。

### FR-07 URL 安全校验

- 默认只允许 HTTPS。
- 解析前后都执行 DNS/IP/redirect SSRF 校验。
- 禁止 loopback、链路本地、私网和云元数据地址；显式站点适配器例外。
- 限制响应体、重定向次数、解析时间和并发数。

### FR-08 媒体解析

- 首选站点适配器或 yt-dlp 提取元数据。
- 支持 HLS、DASH、MP4 及分离音视频轨道。
- 支持短期签名 URL、必要请求头及服务器端 Cookie 注入。
- 输出与显示端无关的标准 `ResolvedMedia`。
- 检测 DRM 或不支持的保护方式并拒绝处理。

### FR-09 字幕

- 发现 SRT、VTT、ASS/SSA 等字幕。
- 记录语言、默认、强制和听障标记。
- 必要时转换容器或字幕格式，不默认烧录字幕。
- adapter 根据自身能力选择直接传递、转换或明确拒绝。

### FR-10 会话管理

- 每个站点使用隔离的服务器端会话。
- 登录失效时要求用户在控制台重新认证。
- Cookie 不写入日志、不返回显示端、不暴露给 Jellyfin 客户端。

### FR-10A 交互登录浏览器

- 控制台能够启动服务端隔离浏览器并显示其远程画面，接受键鼠和触摸输入。
- 支持用户在控制设备上完成验证码、扫码和二次认证，不要求操作 Ubuntu 手机屏幕。
- 登录状态按用户与站点隔离保存在服务端；控制设备不得获得原始 Cookie、localStorage 或浏览器 profile。
- 登录完成或超时后停止浏览器进程，避免长期占用手机内存和 CPU。
- 登录会话使用短期一次性授权，断开后可安全恢复或终止，并记录不含凭据的审计事件。

### FR-11 生命周期

- 播放任务支持创建、刷新、停止和过期清理。
- 短期媒体 URL 在播放期间按需刷新。
- 大型媒体默认不落盘。
- 服务重启后不要求恢复正在播放的临时任务，但保留加密的站点会话。

## 6. 非功能需求

### 性能与功耗

- Direct Stream / Remux 是默认成功路径。
- Web Display 可直接消费源格式时不得为了经过 Jellyfin 而增加额外链路。
- 单用户场景下不因网关产生持续视频重新编码。
- 默认最多一个重型解析任务，防止手机过热。
- 本地播放启动目标：缓存命中 3 秒内，普通解析 10 秒内；站点响应时间不计入硬保证。

### 可用性

- Jellyfin 故障不得影响 Web Display、Resolver 或 Session Vault。
- Gateway 的 Jellyfin Adapter 故障不得影响 Jellyfin 播放本地 NAS 内容。
- 单个 Display Adapter 故障不得破坏其他 adapter 的注册与播放能力。
- 每种失败返回稳定错误码和用户可执行建议。

### 兼容性

- 服务端首要目标：Ubuntu 24.04 ARM64。
- Web Display：当前主流 Chromium 浏览器。
- 首批外部电视显示端：Jellyfin Android TV 官方客户端。
- 后续新增显示端不得要求修改 Resolver 核心契约。

## 7. Core MVP 验收标准

1. 一个公开、非 DRM URL 可以被解析为 `ResolvedMedia` 并创建 `PlaybackSession`。
2. 当前浏览器无需 Jellyfin 即可成为活动显示端并播放支持的媒体。
3. 视频在可直接播放或 remux 的情况下不进行重新编码。
4. 至少支持一个外挂字幕轨道，并可由 Web Display 选择。
5. 网站 Cookie 不出现在日志、浏览器媒体 URL 或显示端请求中。
6. 私网 URL、redirect 到私网和超限资源会被拒绝。
7. 显示端失败不会破坏播放任务元数据和站点会话。

## 8. Jellyfin Adapter 验收标准

1. 启用 Jellyfin Adapter 后，可发现至少一个在线 Jellyfin 客户端。
2. 同一个 `PlaybackSession` 可从 Web Display handoff 到 Jellyfin Android TV。
3. Jellyfin 日志显示 Direct Play 或 Remux，而非默认视频转码。
4. Jellyfin 不可用或 Adapter 被禁用时，Core MVP 仍然成立。
5. 网关停止后，本地 Jellyfin 媒体库仍正常工作。
