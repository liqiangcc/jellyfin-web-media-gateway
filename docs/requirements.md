# 需求说明

## 1. 背景

现有设备角色：

- Ubuntu 手机：长期在线、低功耗、Root、外接 SSD，可运行 Web Media Gateway 和 Jellyfin Server。
- Windows / 其他手机：通过浏览器选择内容、完成网站登录、控制播放，也可以直接成为显示端。
- 小米电视：通过 Wi-Fi 连接，可直接打开 Gateway Web Display，也可使用 Jellyfin Android TV 客户端作为电视显示端。

普通投屏经常丢失字幕，并要求每台电视客户端维护各网站登录状态。本项目希望把登录、解析和媒体流转换集中到服务器，同时不把最终显示方式绑定到某一个客户端。

## 2. 产品目标

1. 控制设备通过浏览器完成操作，无需安装专用控制 App。
2. Control 以“播放、继续看、下一集、换显示端、登录后继续”等用户意图为中心，而不是管理后台式 UI。
3. Control 对用户呈现统一体验，但 Playback、Source/Site Browser、Site Session、Display 四个控制域保持独立状态所有权。
4. 网关统一维护网页媒体播放任务、媒体生命周期和当前显示端。
5. 至少支持浏览器直接显示；Jellyfin 作为首批外部 Display Adapter 支持电视播放。
6. 使用一个容易记忆的根 URL 作为统一入口，并允许用户选择显示或控制角色。
7. 根入口无操作时默认进入电视显示模式，降低电视端遥控器操作成本。
8. 来源网站登录状态只保存在 Ubuntu 手机，并按站点隔离。
9. 播放时按需触发站点登录，同时提供独立的站点账号管理入口。
10. 允许 Control 内按需呈现来源站点原生能力，作为兼容性兜底，而不是要求 Gateway 重写所有站点功能。
11. 支持将非 DRM 网页媒体转换为受控、可播放的媒体源。
12. 支持连续内容的上一集、下一集与自动下一集，不依赖前端猜 URL。
13. 优先转发或重新封装原始媒体，不默认重新编码。
14. 支持独立字幕，并允许当前显示端选择字幕轨道。
15. 本地流量走局域网，不依赖公网或 Tailscale 中继。
16. 显示端通过统一 Display Adapter 抽象接入，避免核心播放模型依赖 Jellyfin。

## 3. 非目标

- 绕过 DRM、付费授权、区域限制或网站访问控制。
- 把任意交互网页完整转换成低延迟远程桌面用于内容播放。
- 重新实现 Jellyfin 的媒体库、用户和客户端生态。
- 重新实现每个视频网站的全部原生 UI 与账号功能。
- 为每个视频网站永久承诺兼容性。
- 将服务作为公网开放代理或多租户 SaaS。
- 首个版本实现同一播放任务的多端同步播放。
- 仅根据屏幕分辨率自动决定页面角色。
- MVP 实现 Gateway 用户账号、RBAC、家庭成员权限或多租户身份体系。
- MVP 实现同一站点多个活动账号。
- 保存来源网站账号密码。
- 把站点原生播放器当作 Gateway 全局播放状态源。

## 4. 核心用户故事

### US-01 创建网页媒体播放任务

用户在 Windows 或手机打开 Control，粘贴受支持网页 URL。网关完成解析后创建 `PlaybackSession`，并展示标题、媒体能力、字幕与可用显示端。

如果已有默认显示端，常规路径应尽量压缩为“粘贴 URL → 播放”，不要求每次重新选择设备。

### US-02 网页直接显示

用户可以把浏览器设为活动显示端。浏览器使用网关签发的短期媒体地址播放内容，不需要 Jellyfin，不接触网站 Cookie 或 Authorization。

### US-03 发送到 Jellyfin 显示端

如果启用了 Jellyfin Adapter，用户可以选择在线 Jellyfin 客户端，例如小米电视。网关把当前播放任务映射为 Jellyfin 可播放的媒体源并发起播放。

### US-04 播放过程中按需登录站点

用户粘贴 URL 后，Resolver 先使用已有站点会话尝试解析。如果内容确实要求登录，Control 显示“登录该站点后继续”。用户完成服务器端浏览器登录后，Gateway 自动重试原 URL 和原播放意图，不要求重新粘贴或重新选显示端。

### US-05 主动管理站点账号

用户可以进入 `/control/sites` 查看各站点的登录状态、脱敏账号标签和最近验证时间，并执行登录、重新登录、退出登录/清除会话。

账号管理是辅助入口，不是普通播放的必经步骤。

### US-06 字幕播放

网关发现字幕后，将字幕标准化为显示端可消费的轨道。Web 显示端和 Jellyfin 显示端分别通过各自 adapter 暴露字幕能力。

### US-07 失败可解释

解析或播放失败时，Control 明确区分：不支持的网站、登录失效、DRM、网络失败、媒体过期、代理失败、显示端离线、格式不兼容和 adapter 错误。

完成可恢复动作后，系统优先继续用户原来要做的事情。

### US-08 跨显示端接管

用户可以把正在浏览器播放的内容切换到 Jellyfin 电视端，或从电视切回浏览器。新显示端确认可播放后，旧显示端停止，新显示端从确认进度接管。

### US-09 状态恢复

页面刷新、手机锁屏或 WebSocket 重连后，Control 从 Gateway 恢复当前 `PlaybackSession`、`active_display`、播放进度、字幕轨道和过渡状态，而不是回到空白 URL 输入页。

### US-10 统一入口选择模式

用户在电视、Windows 或手机访问同一个 Gateway 根 URL。入口页提供“显示模式”和“控制模式”；用户可立即选择。如果在短超时内没有操作，页面自动进入电视显示模式并等待播放任务。

专用入口 `/display` 和 `/control` 始终可直接访问，不经过自动倒计时，便于电视书签、kiosk、二维码和自动化测试。

### US-11 电视网页作为常驻显示端

电视浏览器进入 Display 后使用沉浸式布局占满 viewport，空闲时显示设备名称、连接状态和控制入口二维码；收到播放任务后直接切换为播放器。真正的浏览器 Fullscreen 如果要求用户手势，则在首次遥控器/点击交互后进入，而不是把 Fullscreen 失败视为播放失败。

### US-12 上一集、下一集与自动下一集

当 Resolver 能识别连续内容时，Control 直接显示上一集、下一集和自动下一集。下一集由 Gateway 根据 `PlaybackContext` 重新解析来源 locator，不由前端修改 URL 或猜集号。

如果下一集需要重新登录，系统进入可恢复的 `ActionRequired` 状态；登录完成后自动继续下一集。

### US-13 同一 Control 内使用站点原生能力

用户在观看 Bilibili、YouTube 或其他受支持站点内容时，可以在同一个 `/control` 页面展开 Native Site Panel，继续使用站点原生搜索、选集、收藏、历史、清晰度、弹幕或其他站点专有能力。

用户不需要在 Gateway 遥控页与另一个独立站点管理页之间频繁切换。

Native Site Panel 的存在不改变 Gateway 的播放 authority：如果用户在原生面板选择第 8 集，站点浏览域输出新的 `SourceContext`，Gateway 重新 Resolver 并切换当前 Playback Item；原生站点播放器本身不直接接管电视或覆盖 `PlaybackSession`。

## 5. 功能需求

### FR-01 Unified Entry Router

- `/` 是统一的人机入口。
- 入口页至少提供“显示模式”和“控制模式”两个明确选择。
- MVP 默认无操作超时为 5 秒；超时行为进入 Display role，并使用 TV profile。
- 超时时间必须作为配置项保留，不写死在核心状态机。
- `/display` 是确定性 Display 入口，不等待根入口倒计时。
- `/control` 是确定性 Control 入口，不等待根入口倒计时。
- 用户应能从 Display 或 Control 页面显式切换角色。
- 可以记住浏览器的 `preferred_role` 以减少重复选择，但必须提供清除/切换入口。
- 页面路由只决定 UI 与运行模式。MVP 不把路由角色当成身份体系；未来若引入 Gateway Identity，必须独立于 PageRole。
- 自动化测试可以直接使用 `/display`、`/control`，避免依赖倒计时。

### FR-02 Control Console

- 响应式 Web/PWA，手机优先，同时支持 Windows 浏览器。
- Control 主要状态为 `Idle`、`Resolving`、`Ready/Playing`、`Transition`、`ActionRequired` 或语义等价状态。
- 正在播放时优先显示当前内容、进度、播放/暂停、±10 秒、上一集/下一集、字幕、当前显示端、自动下一集和停止。
- 空闲时优先显示继续观看、最近内容和播放新 URL。
- 展示所有可用 Display Instance，但不暴露 Adapter 实现术语给普通用户。
- 当前控制浏览器只有在用户显式选择“在本机播放”或等价操作后才注册为 Web Display；仅打开 `/control` 不应自动制造显示端实例。
- handoff 成功前不得乐观更新 `active_display`；失败时明确说明旧显示端是否仍在播放。
- Control 刷新或重连时优先恢复当前播放，而不是回到初始页。

### FR-02A Control UX 恢复原则

- 登录、媒体 URL 刷新、短暂断线和下一集准备等可恢复动作完成后，Gateway 自动继续原始用户意图。
- 用户可见文案描述“正在切换到客厅电视”“登录成功，正在继续”等用户动作，不要求理解内部状态机。
- 技术诊断细节可以提供二级入口，但不占据主控制界面。

### FR-02B Control Experience Layer

- `/control` 对用户呈现一个统一体验，但内部不得把 Playback、Source/Site Browser、Site Session、Display 四个领域合并为同一个状态对象。
- Control Experience Layer 只聚合领域状态形成 `ControlView` 或语义等价 ViewModel。
- `ControlView` 不作为业务 authority 或第二套持久化状态库。
- UI 操作必须转换为明确 Intent，至少区分 `PlaybackIntent` 与 `SiteIntent`。
- 同一个用户动作不得同时通过模拟站点播放器按钮和调用 Gateway Playback API 修改同一语义状态。

### FR-02C Native Site Panel

- 当前来源站点支持时，Control 可以在同一页面展示可折叠 Native Site Panel。
- Native Site Panel 由服务器端 `Site Browser Worker` 持有实际来源站点页面与登录上下文，不依赖普通跨站 iframe 作为通用实现。
- Control 只能获得受控的远程画面/输入能力，不获得来源站点原始 Cookie、localStorage 或 profile。
- Native Site Panel 可以提供搜索、选集、收藏、历史、清晰度、弹幕和其他站点专有操作。
- Native Site Panel 故障不得停止已经在 Display 上播放的已解析媒体；Universal Remote 仍应可用。
- Display 故障不得阻止 Native Site Panel 继续浏览和选择内容。

### FR-02D SourceContext 与站点原生操作同步

站点原生操作按语义分类：

1. `Source-changing`：选集、选择新视频、搜索结果进入播放等，必须输出新的 `SourceContext` / source locator，再经 Resolver 更新 Playback Item。
2. `PlaybackPreference-mappable`：清晰度、音轨、字幕等只有在站点 Adapter 能稳定映射时，才可以提升为 Gateway 通用 PlaybackPreference。
3. `Site-only`：收藏、评论、页面布局、弹幕开关等默认只影响 Site Browser Domain。

- 站点原生播放器的 pause/seek 不得自动覆盖远端 Display 的 pause/seek。
- 原生页面关闭弹幕不得被解释为远端 Display 弹幕已关闭；未来远端弹幕能力需要单独 Gateway 设计。
- Native Site Panel 的能力应先作为兼容性兜底，只有经过验证的高频共性能力才提升为 Universal Control。

### FR-03 Playback Session

- Gateway 是网页媒体播放任务的权威状态源。
- 每个任务至少记录：解析结果、生命周期、播放状态、当前位置、字幕选择、`active_display` 和 adapter 能力快照。
- 同一任务任一时刻默认最多一个活动显示端。
- 页面刷新、adapter 重连或 Jellyfin 状态变化不得覆盖 Gateway 已确认的会话状态，必须通过状态协调流程合并。
- 手机和 Windows 可同时打开 Control；mutation 由 Gateway 串行化或使用 revision/CAS，前端不维护独立真状态。
- Site Browser Worker 的内部播放器状态不得直接覆盖 PlaybackSession。
- 临时任务服务重启后可以丢失；站点会话必须独立持久化。

### FR-03A Playback Context

对于连续内容，Gateway 使用显示端无关的 `PlaybackContext`：

```text
PlaybackContext
├── current_item
├── previous_item
├── next_item
├── queue
└── autoplay_policy
```

- `previous_item` / `next_item` 保存稳定来源 locator，而不是短期 HLS URL。
- 切换下一集属于 Playback Item Transition，不属于 Display Handoff。
- MVP `autoplay_policy` 至少支持 `off | next`。
- 可在接近结束时预取下一集元数据，但短期媒体 URL 在实际切换时刷新或重新解析。

### FR-03B SourceContext

站点浏览域使用稳定 `SourceContext` 描述当前用户选择的来源内容：

```text
SourceContext
├── site_id
├── source_locator
├── title
├── collection_metadata
├── episode_metadata
└── site_metadata
```

- SourceContext 不包含 Display 私有信息。
- SourceContext 变化必须经过 Resolver 才能成为新的 ResolvedMedia / Playback Item。
- Control 不直接解析站点 DOM 来生成媒体 URL。

### FR-04 Display Adapter

Display Adapter 是显示能力的统一边界。至少抽象：枚举/注册显示实例、能力探测、prepare/start/pause/stop/seek、状态上报、字幕能力和 handoff 确认。

首批实现：

- `WebDisplayAdapter`
- `JellyfinDisplayAdapter`

核心 Resolver、Session Vault 与 Media Gateway 不得依赖 Jellyfin 类型。

### FR-05 Web Display

- 浏览器可使用 HTML5 Player 直接播放网关媒体。
- `/display` 表示专用显示页面；默认采用 TV-oriented immersive layout。
- Display 页面默认占满 viewport，使用黑色背景和 `object-fit: contain`，避免裁切源内容。
- 支持 `DisplayProfile = tv | desktop | mobile | auto` 或等价模型；profile 只影响布局/交互能力。
- TV profile 使用大字号字幕、遥控器/方向键友好焦点、播放后自动隐藏控制层，并在浏览器允许时申请 Screen Wake Lock。
- 真正 Fullscreen 只能在浏览器策略允许时请求；无法进入 Fullscreen 时仍必须保持 viewport 级沉浸播放。
- 空闲 Display 应显示设备名称、连接状态和可选的 `/control` 地址/二维码。
- 浏览器只获得任务绑定、短期签名的媒体 URL，不获得上游 Cookie、Authorization 或浏览器 profile。

### FR-06 Jellyfin Display Adapter

- Jellyfin 是可选显示适配器，不是 Gateway 核心依赖。
- 可通过动态 M3U/M3U8、媒体代理和 Jellyfin API 将任务暴露给 Jellyfin。
- 使用 Jellyfin API 获取客户端、播放状态并发送控制指令。
- Jellyfin 用户、设备和内部会话只在该 adapter 边界内处理。
- Jellyfin 不可用时，Web Display 和核心解析能力仍应工作。

### FR-07 Display Handoff

- 切换采用 prepare → confirm → commit 模型。
- 切换前记录已确认播放位置和字幕状态。
- 新显示端必须先通过在线与可播放能力检查。
- 新显示端确认启动失败时，旧显示端继续保持活动。
- 成功启动新显示端后停止旧显示端，并提交新的 `active_display`。

### FR-08 URL 安全校验

- 默认只允许 HTTPS。
- 解析前后都执行 DNS/IP/redirect SSRF 校验。
- 禁止 loopback、链路本地、私网和云元数据地址；显式站点适配器例外。
- 限制响应体、重定向次数、解析时间和并发数。

### FR-09 媒体解析

- 首选站点适配器或 yt-dlp 提取元数据。
- 支持 HLS、DASH、MP4 及分离音视频轨道。
- 支持短期签名 URL、必要请求头及服务器端 Cookie 注入。
- 输出与显示端无关的标准 `ResolvedMedia`。
- 可识别连续内容时同时提供稳定的上一集/下一集来源 locator 或等价导航上下文。
- 能接收 SourceContext / source locator 作为解析入口。
- 检测 DRM 或不支持的保护方式并拒绝处理。

### FR-10 字幕

- 发现 SRT、VTT、ASS/SSA 等字幕。
- 记录语言、默认、强制和听障标记。
- 必要时转换容器或字幕格式，不默认烧录字幕。
- adapter 根据自身能力选择直接传递、转换或明确拒绝。

### FR-11 Site Account / Site Session

- 每个站点使用隔离的服务器端会话。
- Resolver 应先尝试现有 SiteSession；只有确实需要认证时才返回 `SITE_AUTH_REQUIRED`。
- 播放触发登录时保存 `PendingIntent` 或等价上下文，登录成功后自动 retry 原播放意图。
- Cookie 不写入日志、不返回显示端、不暴露给 Jellyfin 客户端。
- Gateway 不保存来源网站密码、验证码输入或二维码画面。
- MVP 每站点最多一个活动 `SiteAccount`，但数据模型保留独立 account id 以支持未来多账号。

### FR-11A Site Browser Worker

- `Site Browser Worker` 使用服务端隔离 Chromium/Playwright 和站点持久 profile。
- 至少支持 `Auth Mode`；后续支持 `Native Control Mode`。
- Control 可以显示受控远程画面并转发键鼠/触摸输入，但不得下载原始 profile 或会话材料。
- Auth Mode 与 Native Control Mode 复用同一 Session Vault 边界。
- worker 按需启动、限制并发、设置空闲超时和总时长上限。
- Native Control Mode 不得把服务端浏览器内部播放器状态写成 Gateway PlaybackSession 真状态。

### FR-11B 站点账号管理

- `/control/sites` 提供站点账号管理。
- 站点状态至少允许 `unknown | checking | valid | expired | login_required | error` 或语义等价状态。
- UI 展示脱敏账号标签、最近验证时间和失效原因，不展示 Cookie 或 Token。
- 主动“登录”与播放驱动登录复用同一 Site Browser Worker Auth Mode 和 Session Vault 流程。
- “重新登录”优先保留旧会话，新会话验证成功后原子替换；取消/失败时尽可能不破坏旧会话。
- “退出登录”属于破坏性操作，需明确确认，并清理/失效对应站点会话材料。

### FR-12 生命周期

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
- 空闲入口页和 Display 等待页不得持续产生高频轮询或视频解码负载。
- Site Browser Worker 按需运行；Native Site Panel 关闭/空闲后应允许释放 Chromium 资源。

### 可用性

- Jellyfin 故障不得影响 Web Display、Resolver 或 Session Vault。
- 单个 Display Adapter 故障不得破坏其他 adapter 的注册与播放能力。
- Site Browser Worker 故障不得无必要地停止已经在 Display 上播放的媒体。
- Display 故障不得阻止 Site Browser Worker 继续浏览和选择内容。
- 登录完成后应自动恢复原播放意图；用户不应重复输入相同 URL。
- 重新登录失败时不应无必要地删除仍可用的旧 SiteSession。
- 根入口自动跳转失败时必须保留可点击的模式选择。
- 每种失败返回稳定错误码和用户可执行建议。

### MVP 部署信任边界

- MVP 目标环境是可信家庭 LAN / 单用户。
- 暂不实现 Gateway 用户认证与 RBAC，也不得把 `SiteAccount` 当成 Gateway 用户身份。
- Gateway 默认不直接暴露公网。
- same-origin、Origin/CSRF 防护、SSRF、防开放代理和短期媒体能力仍属于 MVP 安全要求。
- Site Browser Worker 的远程画面/输入通道不得直接暴露为通用公网远程桌面。

### 兼容性

- 服务端首要目标：Ubuntu 24.04 ARM64。
- Web Display：当前主流 Chromium 浏览器。
- TV Web Display：布局测试覆盖 1280×720、1920×1080 和 3840×2160 viewport。
- 首批外部电视显示端：Jellyfin Android TV 官方客户端。
- Native Site Panel 不以普通 iframe 能成功加载所有站点为前提。

## 7. Core MVP 验收标准

1. 一个公开、非 DRM URL 可以被解析为 `ResolvedMedia` 并创建 `PlaybackSession`。
2. 浏览器无需 Jellyfin 即可成为活动显示端并播放支持的媒体。
3. 访问 `/` 时能选择 Control 或 Display；无操作 5 秒后进入电视显示模式。
4. `/display` 与 `/control` 可以绕过倒计时直接进入对应角色。
5. 专用 Display 页面在 TV profile 下铺满 viewport；Fullscreen API 不可用时仍能正常播放。
6. 仅打开 `/control` 不会自动注册为新的显示端。
7. Control 在已有播放任务时优先恢复“正在播放”视图。
8. ControlView 不作为第二份 PlaybackSession 状态源。
9. 视频在可直接播放或 remux 的情况下不进行重新编码。
10. 至少支持一个外挂字幕轨道。
11. 私网 URL、redirect 到私网和超限资源会被拒绝。

## 8. Site Auth / Control 验收标准

1. 公开内容不因为站点存在登录能力而被强制要求登录。
2. 需要登录的 URL 返回明确 `SITE_AUTH_REQUIRED`，Control 可启动站点登录。
3. 登录成功后自动重试原 URL，并保持原目标显示端。
4. `/control/sites` 可以查看站点状态并主动登录、重新登录和退出登录。
5. Gateway 不保存网站账号密码，Cookie/Token 不出现在 Control、Display 或日志中。
6. 重新登录失败或取消时尽可能保留旧会话。
7. 如果存在下一集，Control 能触发下一集；自动下一集时保持当前 active display。
8. 下一集需要认证时，完成登录后可自动继续。

## 9. Unified Control / Native Site 验收标准

1. `/control` 可以同时呈现 Universal Remote 和可折叠 Native Site Panel，而不要求跳转到另一个独立产品页面。
2. Native Site Panel 使用 Site Browser Worker，而不是依赖站点允许 iframe。
3. Native Site Panel 展开、关闭或崩溃不改变当前 PlaybackSession 的播放状态。
4. 用户在 Native Site Panel 选择新的剧集/视频后，系统得到新的 SourceContext 并通过 Resolver 执行 Playback Item Transition。
5. Native Site Player 的 pause/seek 不会被错误同步成远端 Display pause/seek。
6. 清晰度等能力只有在 Adapter 明确支持映射时才成为 Gateway PlaybackPreference。
7. Site-only 设置不会伪造远端 Display 状态。
8. Control 刷新后可以重新聚合 Playback、Source、SiteSession、Display 状态。

## 10. Jellyfin Adapter 验收标准

1. 启用 Jellyfin Adapter 后，可发现至少一个在线 Jellyfin 客户端。
2. 同一个 `PlaybackSession` 可从 Web Display handoff 到 Jellyfin Android TV。
3. Jellyfin 日志显示 Direct Play 或 Remux，而非默认视频转码。
4. Jellyfin 不可用或 Adapter 被禁用时，Core MVP 仍然成立。
5. 网关停止后，本地 Jellyfin 媒体库仍正常工作。
