# Implementation Contracts

本文件定义开始写 Core 代码前必须稳定的最小可编码契约。它不要求最终 Rust 类型、HTTP JSON 字段逐字一致，但实现不得破坏这里定义的状态所有权、版本语义和安全边界。

## 1. 依赖方向

```text
Control / API
    ↓
Playback / Display / Resolution Core
    ↓
SiteAdapterRegistry / SiteAdapter Contract
    ↓
Concrete Site Plugins
```

`gateway-core` 不允许直接依赖 Bilibili、YouTube、yt-dlp 等具体实现。

## 2. SourceLocator

`SourceLocator` 是“如何重新定位用户选择的内容”的稳定契约，不是短期 CDN/HLS URL。

概念模型：

```text
SourceLocator
├── site_id
├── plugin_id
├── locator_version
└── opaque_payload
```

规则：

- `site_id`：规范化站点身份，例如 `bilibili`、`youtube`、`generic`。
- `plugin_id`：负责解释该 locator 的插件。
- `locator_version`：插件定义的 locator schema 版本。
- `opaque_payload`：只允许对应插件解释；Core 不读取其中字段。
- 用户最初输入的 URL 只是 ingress input，必须先经过 `SiteAdapterRegistry.recognize()` 变成 `SourceLocator`，再进入后续流程。
- `PlaybackContext.previous/next/queue`、继续观看、登录后 retry 都引用 `SourceLocator`，不能保存过期 CDN URL 作为内容身份。
- 插件升级后如果不能解释旧 `locator_version`，必须返回明确 `SOURCE_LOCATOR_UNSUPPORTED`；不得由 Core 猜测迁移规则。

`SourceDescriptor` 可以在 locator 外携带通用编排元数据：

```text
SourceDescriptor
├── locator: SourceLocator
├── account_ref?       # MVP 通常为空；以后多账号时使用
└── display_metadata?  # 只用于展示，不参与定位
```

内容身份与账号身份分离；不要把 Cookie 或账号 Secret 写进 locator。

## 3. SiteAdapter Contract

第一版实现就必须通过 `SiteAdapterRegistry` 调用 SiteAdapter；“先 Core 直接调用 yt-dlp、以后再抽象”不允许。

概念能力：

```text
SiteAdapter
├── manifest()
├── recognize(input)
├── resolve(locator, access)
├── navigation(locator, access)
├── account_probe(account_ref, access)
├── browser_interpret(event, access)
└── capabilities()
```

### 3.1 recognize

输入可以是 URL 或其他未来支持的 source input。

输出：

```text
RecognizeResult
├── matched
├── site_id
├── plugin_id
├── priority
└── locator?
```

多个插件匹配时由 Registry 使用显式 priority/confidence 规则决策，不能依赖注册顺序。

### 3.2 resolve

输入 `SourceLocator + SiteAccessCapability`，输出标准 `ResolvedMedia`。

插件不得返回 Cookie、Authorization bearer token、浏览器 profile 文件等 Secret。

### 3.3 navigation

输出：

```text
NavigationContext
├── previous: SourceLocator?
├── next: SourceLocator?
├── collection_id?
└── current_index?
```

Core 只知道有“上一项/下一项”，不知道插件如何推导。

### 3.4 browser_interpret

`Site Browser Worker` 只产生通用浏览器事件/快照；具体插件负责把它解释成站点语义。

```text
BrowserEvent / BrowserSnapshot
        ↓
SiteAdapter.browser_interpret
        ↓
SourceContext / AccountState / NativePanelState
```

禁止在通用 Browser Worker 中出现站点 DOM selector、Cookie 名称、BV/EP/Season 等 concrete site knowledge。

## 4. SiteAdapterRegistry

Core 只面向 Registry。

职责：

- 插件注册与 `plugin_id/site_id` 唯一性校验；
- recognize 路由和冲突检测；
- capability / version / health 查询；
- fallback 选择；
- timeout / cancellation 包装；
- 输出契约校验。

`generic-ytdlp` 必须作为 fallback Site Plugin 注册；Core 中不允许存在“如果没有插件就直接调用 yt-dlp”的特殊路径。

MVP 使用 Rust workspace + trait 编译期插件：

```text
gateway-core/
site-adapter-api/
plugins/
├── generic-direct/
├── generic-ytdlp/
└── ...
```

运行时 IPC 插件不属于 MVP。

## 5. SiteAccessCapability

Site Plugin 不直接读取 Session Vault 文件，也不通过 `vault.get_raw_cookie()` 获得完整凭据。

Core/受限基础设施向插件传递 scoped capability：

```text
SiteAccessCapability
├── site_id
├── account_ref?
├── allowed_hosts
├── expiry
└── capability_id
```

插件需要带会话访问上游时，通过受控 HTTP/Browser 能力执行：

```text
ScopedSiteHttpClient.request(capability, request)
```

或等价机制。

基础设施负责：

- 注入对应站点 Cookie/Authorization；
- 校验目标 host；
- redirect 后重新校验；
- 屏蔽其他站点 Session；
- 记录不含 Secret 的审计信息。

未来切换为进程插件时，`SiteAccessCapability` 可以通过 IPC 传递，仍不需要把原始 Cookie 发给插件进程。

## 6. ResolvedMedia

`ResolvedMedia` 是 Site Domain 到 Playback/Media Domain 的稳定输出。

概念模型：

```text
ResolvedMedia
├── metadata
│   ├── title
│   ├── duration?
│   ├── poster?
│   └── source_site
├── streams[]
├── subtitles[]
├── expires_at?
└── protection
```

`ResolvedStream`：

```text
ResolvedStream
├── id
├── kind                 # video/audio/muxed
├── protocol             # hls/dash/http-file/...
├── url                  # 上游资源 URL，可短期
├── codecs?
├── width/height/bitrate?
├── language?
├── public_headers?      # 仅允许非 Secret header
└── upstream_access_ref? # 敏感凭据的 opaque handle
```

规则：

- `Cookie`、`Authorization`、站点 bearer token 不进入 `public_headers`。
- 敏感上游认证通过 `upstream_access_ref` / scoped capability 由 Media Gateway 注入。
- `ResolvedMedia` 可以短期失效，所以 `PlaybackItem` 必须同时保留 `SourceLocator` 以便重新 resolve。
- DRM/保护状态至少明确：`clear | drm_unsupported | unsupported`。
- 插件输出必须经过 Core schema validation 后才能进入 Playback。

## 7. PlaybackItem

一个 `PlaybackSession` 可以连续播放多个 item；“下一集”不是创建新的 display session。

概念模型：

```text
PlaybackItem
├── item_id
├── item_revision
├── source_locator
├── resolved_media
├── metadata
└── created_at
```

`item_revision` 在同一 `PlaybackSession` 中每次切换 current item 时单调递增。

Display callback、媒体刷新和 ended 事件必须携带：

```text
session_id
item_id
item_revision
```

旧 item 的延迟回报不能覆盖新 item 状态。

## 8. PlaybackSession / PlaybackContext

```text
PlaybackSession
├── session_id
├── session_revision
├── state
├── current_item: PlaybackItem
├── playback_context
├── position
├── subtitle_selection
├── active_display
└── display_generation
```

`PlaybackContext`：

```text
PlaybackContext
├── previous: SourceLocator?
├── next: SourceLocator?
├── queue: SourceLocator[]
└── autoplay_policy         # MVP: off | next
```

规则：

- `session_revision`：任何会改变 session 全局可见状态的 mutation 成功后递增。
- 切换下一集：resolve 新 locator → 创建新 `PlaybackItem` → CAS 提交为 current item → active display 保持不变 → start new item。
- handoff 不改变 current item；它只改变 `active_display/display_generation`。
- `position` 属于 current item。切 item 后默认重新从 0 或 plugin/context 指定位置开始。
- 首个 MVP 不要求服务重启后恢复 active session；但运行期间必须能防止旧 callback 覆盖新状态。

## 9. Playback Command Envelope

避免同时维护 `/intent`、`/handoff`、`/control` 多套 mutation 语义。

生命周期使用资源 API：

```text
POST   /api/v1/sessions
GET    /api/v1/sessions/{id}
DELETE /api/v1/sessions/{id}
```

所有播放 mutation 统一使用 command endpoint：

```text
POST /api/v1/sessions/{id}/commands
```

请求：

```text
CommandEnvelope
├── request_id
├── expected_session_revision?
└── command
    ├── Play
    ├── Pause
    ├── Seek(position)
    ├── Stop
    ├── NextItem
    ├── PreviousItem
    ├── SetSubtitle(track)
    └── Handoff(display_id)
```

返回：

```text
CommandResult
├── request_id
├── status
├── session_revision
└── session_snapshot?
```

规则：

- `request_id` 用于幂等/重复请求识别。
- 客户端掌握 revision 时应带 `expected_session_revision`；冲突返回稳定 `REVISION_CONFLICT` 并附最新 revision。
- Control 不乐观宣布 handoff 成功；以服务端提交后的 snapshot 为准。

Site Browser / Site Account 操作使用独立：

```text
POST /api/v1/sites/{site_id}/commands
```

不要把 SiteIntent 混入 PlaybackSession command。

## 10. DisplayAdapter Contract

```text
DisplayAdapter
├── list_or_register_displays()
├── probe(display, media)
├── prepare(session, item, display)
├── start(session, item, display, position, generation)
├── pause(...)
├── seek(...)
├── stop(...)
└── status(...)
```

`DisplayInstance`：

```text
DisplayInstance
├── id
├── adapter_type
├── label
├── capabilities
├── online
└── adapter_metadata
```

关键规则：

- `active_display` 只有 Playback Coordinator 能提交。
- 每次 handoff 成功后 `display_generation` 递增。
- adapter callback 必须携带 generation；旧 generation 不覆盖当前状态。
- adapter 不能直接读取 Session Vault。
- adapter 只消费 Gateway 签发的媒体能力。

## 11. Site Browser Worker Contract

Site Browser Worker 是通用 runtime，不是 Site Adapter。

负责：

- Chromium/Playwright 生命周期；
- profile materialization / attach；
- viewport / frame / input 通道；
- navigation、URL、title、必要 DOM snapshot/event 的通用采集；
- timeout、并发、资源限制。

不负责：

- 判断“这是 Bilibili 第 8 集”；
- Bilibili/YouTube selector；
- 计算下一集；
- 判断具体站点登录成功规则；
- 修改 PlaybackSession。

这些全部由对应 Site Plugin 完成。

## 12. Session Vault 与持久化

`Session Vault` 是逻辑安全边界，不再用 `sessions/` 与 `browser-profiles/` 表达两个互不相关的所有者。

推荐目录：

```text
/var/lib/web-media-gateway/
├── gateway.sqlite
├── vault/
│   ├── accounts/
│   └── browser-profiles/
├── cache/
└── runtime/
```

规则：

- `vault/` 归 Session Vault 管理；其他组件只能通过引用/capability 使用。
- browser profile 不允许通过 Control/Plugin API 下载。
- Vault 文件使用最小文件权限；结构化 Secret（Cookie/token/export）必须加密静态存储。
- Chromium 需要活动 profile 时，由 Vault 按最小权限提供受限 materialization/attach；生命周期结束后清理临时副本。
- 是否使用整盘/目录加密由部署实现决定，但文档不再把“profile 一定是单个加密文件”作为不真实前提。

## 13. EgressPolicy

SSRF/内网访问例外属于 Core 安全能力，不属于插件决定权。

中央策略至少区分：

```text
public_web
configured_local_service
```

- `public_web`：禁止 loopback/private/link-local/metadata/reserved，redirect 每跳重检。
- `configured_local_service`：只供明确配置的内部集成（例如 Jellyfin），目标地址由 Gateway 配置，不接受任意用户 URL。
- Site Plugin 与 Site Browser Worker 默认只能获得 `public_web` 或更窄的站点 host allowlist。
- 插件不得声明“我是 Bilibili，所以关闭 SSRF 检查”。

## 14. Web Secure Context 策略

MVP 的基线 Web/Display 功能必须能够在可信 LAN 的普通 HTTP 下工作：

```text
http://10.0.0.116/
```

因此以下能力不得成为 Core 验收前置条件：

- Service Worker / 可安装 PWA；
- Screen Wake Lock；
- 其他只在 secure context 可用的增强 API。

如果部署提供 LAN HTTPS，则启用这些增强能力。文档中的 “PWA” 表述解释为“响应式 Web App；在 secure context 下可进一步安装/增强”。

长期推荐 HTTPS，但首个媒体路径 PoC 不因证书体系阻塞。

## 15. Contract / Architecture Tests

开始实现后 CI 至少增加：

1. SiteAdapter conformance tests。
2. ResolvedMedia schema tests。
3. PlaybackSession revision/item_revision 竞态测试。
4. Display generation 旧回调拒绝测试。
5. scoped SiteAccessCapability 不跨站测试。
6. EgressPolicy 私网/redirect 测试。
7. Core concrete-site-knowledge architecture test：禁止站点域名、Cookie key、DOM selector、`if site == ...` 业务分支进入稳定 Core。

新增一个站点插件的理想 diff 应主要位于 `plugins/<site>/`；若必须修改 PlaybackCoordinator、DisplayAdapter 或 Control 核心业务分支，需要架构评审。
