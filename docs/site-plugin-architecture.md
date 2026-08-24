# Site Plugin Architecture

## 1. 目标

网页媒体站点是系统里变化最快的部分。Gateway Core 必须隔离 Bilibili、YouTube、yt-dlp 等具体实现变化。

核心原则：

> Gateway 可以识别站点，但不理解站点。

具体数据形状以 `implementation-contracts.md` 为准；本文件只解释插件边界和演进方式。

## 2. 分层

```text
Stable Core
├── Playback
├── Display
├── Media Gateway
├── Control Experience
├── Resolution Service
└── SiteAdapterRegistry
        ↓ contract
Site Abstraction
├── SiteId
├── SourceLocator
├── SiteCapabilities
├── SiteAccessCapability
├── ResolvedMedia
└── SiteError
        ↓ implementation
Volatile Edge
├── generic-direct
├── generic-ytdlp
├── bilibili
├── youtube
└── ...
```

依赖只能指向抽象契约；Core 不 import concrete plugin。

## 3. Core 允许知道什么

Core 可以知道：

- `site_id`；
- `plugin_id`；
- capability/health/version；
- opaque `SourceLocator`；
- 标准 `SiteError`。

用途：路由、会话隔离、UI 标签、可观察性、健康状态。

Core 不得知道：

- BV/AV/EP/Season；
- YouTube 参数语义；
- Cookie/localStorage key；
- 站点私有 API；
- DOM selector；
- 清晰度内部枚举；
- 下一集算法；
- 站点登录成功判定。

`if site == "bilibili"` 等站点业务分支出现在 Stable Core，默认视为架构越界。

## 4. Registry

Core 的唯一入口：

```text
Gateway Core
→ SiteAdapterRegistry
→ SiteAdapter
```

Registry 负责：

- 注册；
- `site_id/plugin_id` 唯一性；
- recognize 路由；
- priority/conflict；
- capability/version/health；
- timeout/cancellation；
- fallback；
- output validation。

Generic yt-dlp 是 fallback plugin，不是 Core fallback。

## 5. SourceLocator

站点与 Core 之间共享版本化 opaque locator：

```text
site_id
plugin_id
locator_version
opaque_payload
```

Core 只保存和传递，不解释 payload。

用途：

- current/previous/next；
- queue；
- PendingIntent；
- 继续观看；
- URL 过期后的重新 resolve。

短期 HLS/CDN URL 不是 SourceLocator。

## 6. SiteAdapter 能力

概念能力：

```text
manifest
recognize
resolve
navigation
account_probe
browser_interpret
capabilities
```

最终 Rust trait 可以调整方法形状，但不能改变边界。

### recognize

把输入 URL/source 转换成插件拥有的 `SourceLocator`。

### resolve

接收 `SourceLocator + SiteAccessCapability`，输出 `ResolvedMedia`。

### navigation

输出 previous/next 等标准 locator。

### browser_interpret

把通用 `BrowserEvent/BrowserSnapshot` 解释成：

- SourceContext/SourceLocator；
- AccountState；
- NativePanelState。

## 7. Site Browser Worker 不是插件

Site Browser Worker 是通用 Chromium runtime：

```text
Chromium lifecycle
profile attach/materialization
frame/input
navigation/browser events
resource limits
```

具体站点语义：

```text
Browser Worker event
→ concrete Site Plugin
→ standard site result
```

这条边界用于防止 Browser Worker 演变成新的站点 special-case 集合。

## 8. Session / Secret Boundary

Plugin 不直接读取 Vault 文件或完整 Cookie jar。

```text
Site Plugin
→ SiteAccessCapability
→ ScopedSiteHttpClient / controlled browser access
→ Session Vault injects Secret
```

Capability 必须限制 site/account/host/expiry。

未来进程插件也优先传 capability，而不是通过 IPC 传完整 Cookie。

## 9. Egress Boundary

Plugin 无权关闭 SSRF。

网络访问由中央 `EgressPolicy` 决定：

```text
public_web
configured_local_service
```

Site Plugin 默认只获得 public web 或更窄站点 allowlist。

Jellyfin 私网访问属于明确配置的 local-service integration，不是 Site Plugin 例外。

## 10. 与 Control / Native Site Panel

Control 只消费：

```text
SiteCapabilities
SourceContext
NativePanelState
```

不 import `BilibiliPanel`。

Native Site Panel 选择新内容：

```text
Browser Event
→ Site Plugin.browser_interpret
→ SourceLocator
→ Site command SelectSource
→ Resolution Service
→ PlaybackItemTransition
```

站点 Chromium 播放器不是 Gateway 播放 authority。

## 11. 能力提升

```text
站点原生能力
→ 多个插件证明存在稳定共同语义
→ 标准 capability / PlaybackPreference
→ Universal Control
```

不要为了 UI 统一而过早把站点私有细节塞进 Core。

## 12. 第一阶段：编译期插件

MVP：

```text
gateway-core/
site-adapter-api/
plugins/
├── generic-direct/
├── generic-ytdlp/
└── ...
```

一起编译发布。

成功标准不是“目录叫 plugins”，而是：

- Core 只依赖 SiteAdapter Contract；
- 新站点主要修改 `plugins/<site>/`；
- contract test 独立运行；
- architecture test 能发现 concrete site knowledge 泄漏。

## 13. 第二阶段：进程插件

只有真实需求出现后才稳定 IPC：

```text
Gateway Core
↕ versioned Site Plugin Protocol
Plugin Process
```

触发条件：独立更新、依赖隔离、崩溃隔离、资源沙箱、多语言实现。

优先独立进程而不是 Rust `.so`。

协议必须显式版本化：

```text
protocol_version
plugin_version
capability_version
```

MVP 不做第三方插件市场、热更新、未知未签名插件执行。

## 14. 标准错误

穿越插件边界时至少标准化：

- `SITE_AUTH_REQUIRED`
- `SITE_UNSUPPORTED`
- `SOURCE_NOT_FOUND`
- `SOURCE_EXPIRED`
- `SOURCE_LOCATOR_UNSUPPORTED`
- `SITE_RATE_LIMITED`
- `SITE_TEMPORARILY_UNAVAILABLE`
- `DRM_UNSUPPORTED`
- `PLUGIN_BUG`
- `PLUGIN_TIMEOUT`

站点调试信息必须脱敏。

## 15. Contract / Architecture Tests

所有插件至少验证：

- recognize 确定性；
- SourceLocator schema/version；
- ResolvedMedia schema；
- navigation；
- Secret 不越界；
- timeout/cancel；
- capability 声明一致性。

CI 对 Stable Core 增加规则：禁止已知站点域名、Cookie key、DOM selector 和站点条件业务分支。
