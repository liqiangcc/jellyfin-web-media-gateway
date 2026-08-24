# 需求说明

## 1. 产品目标

Web Media Gateway 面向可信家庭 LAN / 单用户场景，把网页媒体的“来源站点会话、解析、媒体代理、播放状态、显示端”集中到 Ubuntu ARM64 服务器。

系统必须：

1. 允许手机/Windows 通过浏览器控制，无需专用控制 App。
2. 允许浏览器直接成为 Display，不强制依赖 Jellyfin。
3. 将 Jellyfin 作为可选 `DisplayAdapter`，而不是全局播放核心。
4. 让 Gateway 成为 `PlaybackSession` authority。
5. 让 Control 呈现统一体验，但 Playback、Site、SiteSession、Display 各自保持独立状态所有权。
6. 从第一版实现开始使用 Site Plugin Boundary；Core 不直接调用具体站点代码或 yt-dlp。
7. 站点登录状态只保存在服务器，并按站点隔离。
8. 优先 Direct Stream / Remux，不默认视频转码。
9. 支持字幕、跨显示端 handoff、连续内容下一集等通用播放能力。
10. 在普通可信 LAN HTTP 环境也能完成基本控制和播放；HTTPS 提供时再启用 PWA/Wake Lock 等 secure-context 增强能力。

## 2. 非目标

MVP 明确不做：

- Gateway 用户账号、RBAC、家庭成员权限、多租户。
- 公网开放服务或开放代理。
- 绕过 DRM、付费授权、区域限制或网站访问控制。
- 同一播放任务多显示端同步。
- 同一站点多个活动账号。
- 保存来源网站账号密码。
- 重新实现每个视频网站全部原生 UI。
- 运行时第三方插件市场、动态 `.so`、热更新。
- 把站点 Chromium 播放器当作 Gateway 播放状态源。

## 3. Web 入口与显示

### FR-01 Unified Entry

- `/` 是统一入口，提供 Control / Display 选择。
- MVP 默认 5 秒无操作进入 `/display?profile=tv`。
- `/display` 与 `/control` 是确定性入口，不依赖倒计时。
- `PageRole` 与 `DisplayProfile` 分离。
- 仅打开 `/control` 不自动注册 Display。
- 用户显式选择“在本机播放”后才注册当前 Control 浏览器为 Web Display。

### FR-02 Web Display

- Web Display 使用 Gateway 签发的短期媒体 URL。
- TV profile 默认占满 viewport、黑色背景、`object-fit: contain`、大字幕、遥控器友好焦点。
- Fullscreen、Wake Lock、Service Worker 不可用时，基本播放仍必须成功。
- HTTPS 部署时可以启用 installable PWA、Wake Lock 等增强能力。
- Display 不获得来源站点 Cookie、Authorization 或 profile。

## 4. Playback Domain

### FR-03 PlaybackSession Authority

Gateway 是唯一 `PlaybackSession` authority。

每个 Session 至少包含：

```text
session_id
session_revision
current_item
playback_context
position
subtitle_selection
active_display
display_generation
```

### FR-04 PlaybackItem

当前播放项必须独立建模：

```text
PlaybackItem
├── item_id
├── item_revision
├── source_locator
├── resolved_media
└── metadata
```

- 每次切换下一项时 `item_revision` 单调递增。
- Display callback、ended、媒体刷新必须标识 session/item revision。
- 旧 item 的延迟事件不得覆盖新 item 状态。

### FR-05 PlaybackContext

连续内容使用：

```text
previous: SourceLocator?
next: SourceLocator?
queue: SourceLocator[]
autoplay_policy: off | next
```

- Control 不通过修改 URL 或猜集号实现下一集。
- 下一集属于 `PlaybackItemTransition`，默认保持 `active_display`。
- 下一集 resolve 失败必须呈现可解释状态，不能让 Display 无说明黑屏。

### FR-06 Playback Commands

所有播放 mutation 使用统一 command 语义，至少支持：

- Play
- Pause
- Seek
- Stop
- NextItem / PreviousItem
- SetSubtitle
- Handoff

每个 command 必须有 `request_id`；客户端可携带 `expected_session_revision`。冲突返回稳定 `REVISION_CONFLICT`，Control 以服务端 snapshot 为准。

## 5. Display Domain

### FR-07 DisplayAdapter

首批实现：

- `WebDisplayAdapter`
- `JellyfinDisplayAdapter`

Adapter 必须支持或显式拒绝：probe、prepare、start、pause、seek、stop、status、subtitle capability。

### FR-08 Handoff

Handoff 使用 prepare/confirm/commit 语义：

- 目标显示端确认可播放前，不得静默停止源显示端。
- 成功 handoff 后递增 `display_generation`。
- adapter 回调必须携带 generation；旧 generation 不覆盖当前状态。
- Jellyfin 不可用时 Web Display 和 Core 解析仍正常工作。

## 6. Site Plugin Domain

### FR-09 Site Plugin From Day One

第一版实现就必须使用：

```text
Gateway Core
→ SiteAdapterRegistry
→ SiteAdapter Contract
→ concrete plugin
```

- Generic yt-dlp 是 fallback Site Plugin。
- Core 不允许直接调用 yt-dlp。
- 新增具体站点的主要修改范围应位于 `plugins/<site>/`。
- Core 出现站点域名、Cookie key、DOM selector 或 `if site == ...` 业务分支，应默认视为架构越界。

### FR-10 SourceLocator

所有可恢复/可导航内容使用版本化 opaque `SourceLocator`：

```text
site_id
plugin_id
locator_version
opaque_payload
```

- Core 不解析 `opaque_payload`。
- previous/next/queue、PendingIntent、继续观看都引用 locator。
- locator 不得包含 Cookie、Authorization 或账号 Secret。
- 插件无法解释旧版本时返回 `SOURCE_LOCATOR_UNSUPPORTED`。

### FR-11 Resolution Service

Core 的 Resolution Service 只负责：Registry 路由、scoped session capability、调用 adapter、校验 `ResolvedMedia`。

它不得包含具体站点规则，也不得直接调用 yt-dlp。

### FR-12 ResolvedMedia

`ResolvedMedia` 必须与显示端无关，至少支持 HLS、DASH、HTTP 文件、分离音视频和字幕。

- Cookie/Authorization 不得作为普通 header 穿越到 Display。
- 敏感上游访问使用 opaque `upstream_access_ref` / scoped capability。
- `ResolvedMedia` 允许短期过期，因此 `PlaybackItem` 必须同时保留 `SourceLocator`。
- DRM 必须显式返回不支持状态。

## 7. Site Browser / Native Site

### FR-13 Site Browser Worker

Site Browser Worker 是通用 Chromium/Playwright runtime，只负责：

- 生命周期；
- profile attach/materialization；
- 远程画面与输入；
- navigation/browser event；
- timeout、并发、资源限制。

它不得包含具体站点 DOM/API/登录成功判断/下一集规则。

### FR-14 Browser Interpretation

通用 browser event 必须由对应 Site Plugin 解释：

```text
BrowserEvent
→ SiteAdapter.browser_interpret
→ SourceContext / AccountState / NativePanelState
```

### FR-15 Native Site Panel

- Native Site Panel 与 Universal Remote 可以在同一 `/control` 体验里组合。
- 不依赖普通跨站 iframe。
- Panel 崩溃不得停止已经在 Display 播放的媒体。
- 站点原生 pause/seek 不自动等价为远端 Display pause/seek。
- 选集/选择新视频必须输出新的 `SourceLocator`，再经 Resolution Service 产生 Item Transition。
- 清晰度/音轨/字幕只有能稳定映射时才提升为 Gateway 通用 PlaybackPreference。
- 收藏、评论、站点弹幕开关等默认是 Site-only 状态。

## 8. Site Account / Session Vault

### FR-16 Site Account

- MVP 每站点最多一个活动账号。
- SiteAccount 只代表来源网站会话，不代表 Gateway 用户身份。
- 状态至少支持 `unknown/checking/valid/expired/login_required/error` 或语义等价状态。
- `/control/sites` 提供登录、重新登录、退出登录和脱敏状态展示。

### FR-17 按需登录与恢复

- Resolver 首先尝试当前 SiteSession；只有实际需要认证时才返回 `SITE_AUTH_REQUIRED`。
- 播放触发登录时保存 PendingIntent。
- 登录成功后自动 retry 原 `SourceLocator`、目标显示端和播放动作。
- 重新登录必须优先“新会话验证成功后再替换旧会话”。
- Gateway 不保存网站密码、验证码输入或二维码画面。

### FR-18 Scoped Site Access

Site Plugin 不得直接读取 Vault 文件或其他站点 Cookie。

插件只获得 scoped `SiteAccessCapability`/受控 HTTP 能力，限制：

- site/account scope；
- allowed hosts；
- expiry；
- redirect policy。

## 9. Media Gateway

### FR-19 媒体代理

- 优先 Direct Stream。
- 需要兼容时允许 remux。
- 不默认视频重编码。
- 临时媒体 URL 绑定 PlaybackSession/item/resource/method/expiry。
- 大型媒体默认不落盘。
- Secret 不进入 Control、Display、日志或 M3U。

## 10. Network / Security

### FR-20 Central EgressPolicy

所有站点 URL 和 redirect 由中央 EgressPolicy 校验。

- `public_web` 禁止 loopback/private/link-local/metadata/reserved。
- `configured_local_service` 只用于明确配置的内部集成，例如 Jellyfin。
- Site Plugin 不能声明“站点例外”绕过 SSRF。
- Site Browser Worker 默认只获得 public web 或更窄 host allowlist。

### FR-21 MVP Trust Boundary

- MVP 面向可信 LAN / 单用户。
- 默认不直接暴露公网。
- 仍要求 Origin/CSRF、Host/Content-Type/大小校验、短期 token、防开放代理、命令注入防护。
- 一旦部署条件变为不可信网络，必须重新设计 Gateway Identity/Authorization。

## 11. 存储

### FR-22 Session Vault Ownership

持久目录统一：

```text
/var/lib/web-media-gateway/
├── gateway.sqlite
├── vault/
│   ├── accounts/
│   └── browser-profiles/
├── cache/
└── runtime/
```

- `vault/` 是站点会话/profile 唯一安全所有者。
- 其他组件只能通过 ref/capability 使用。
- 结构化 Secret 必须静态加密；profile 使用最小文件权限并禁止通过 Web/API 下载。
- 临时 materialization 放入受限 runtime，并在 worker 结束后清理。

## 12. 验收标准

Core MVP 至少满足：

1. URL 必须先通过 SiteAdapterRegistry 产生 SourceLocator，再 resolve；Core 没有直接 yt-dlp 路径。
2. 公开非 DRM 内容可创建 `PlaybackSession + PlaybackItem` 并在 Web Display 播放。
3. 旧 item revision 和旧 display generation 不会覆盖当前状态。
4. `/`、`/display`、`/control` 行为稳定；HTTP LAN 下无需 Wake Lock/PWA 也能完成播放。
5. Control 能 play/pause/seek/stop，并在刷新/重连后恢复当前 snapshot。
6. handoff 失败时旧显示端保持播放。
7. 私网/redirect SSRF 被中央 EgressPolicy 拒绝。
8. Cookie/Authorization 不出现在 Display URL、Control 或日志中。
9. Site Browser Worker 不包含 concrete site knowledge；Site Plugin contract test 能独立验证站点逻辑。
10. Jellyfin 被关闭时 Web Display 仍工作。
