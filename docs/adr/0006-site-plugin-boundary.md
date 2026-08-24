# ADR-0006：具体站点知识必须停留在 Site Plugin Boundary 之外

- 状态：已接受（设计阶段）
- 日期：2026-08-24

## 背景

Gateway 需要支持 Bilibili、YouTube 以及未来其他网页媒体站点。站点相关逻辑变化频率远高于 Playback、Display、Media Gateway 等核心领域。

如果具体站点规则直接进入 Gateway Core，会逐渐出现：

```text
if site == bilibili
if site == youtube
```

以及具体 URL 参数、Cookie 名称、DOM selector、下一集算法、清晰度枚举散落在 Resolver、Control 和 PlaybackCoordinator 中。

这会使每次站点改版都侵入稳定核心，也无法形成真正的插件边界。

## 决策

### 1. Gateway 识别站点，但不理解站点

Core 可以持有：

```text
site_id
SourceLocator
SiteCapabilities
SiteSessionRef
```

因此 Core 可以知道：

```text
site_id = bilibili
```

用于插件路由、Session Vault 隔离、状态展示和可观察性。

但 Core 不得理解 Bilibili 的 BV/EP/Season、Cookie、DOM、私有 API、清晰度编码或下一集规则。

### 2. SiteAdapter Contract 是具体站点知识穿越系统的唯一边界

具体站点实现只能通过标准 SiteAdapter / Site Plugin Contract 向 Core 提供能力。

概念能力包括：

```text
recognize
resolve
navigation
account_probe
login_support
native_panel_support
source_context
capabilities
```

最终方法形状可以演进，但业务含义必须保持站点无关。

### 3. Core 不允许直接依赖具体站点插件

禁止：

```text
PlaybackCoordinator → BilibiliAdapter
Control → BilibiliPanel
ResolverCore → YouTube special case
```

允许：

```text
Core
→ SiteAdapterRegistry
→ SiteAdapter Contract
→ concrete plugin
```

具体站点条件分支如果出现在 Stable Core，应默认视为架构越界并要求评审。

### 4. SourceLocator 是站点与播放核心之间的内容定位契约

Core 保存的是稳定来源 locator，而不是站点短期 CDN/HLS URL。

例如下一集：

```text
PlaybackContext.next_item
→ SourceLocator
→ Site Plugin resolve
→ fresh ResolvedMedia
```

下一集如何得到由插件决定，播放下一集如何在当前 Display 启动由 Playback Domain 决定。

### 5. Generic yt-dlp 也作为插件

Generic yt-dlp 不是 Core 的特殊后门。

它作为一个 fallback Site Plugin 存在，与 Bilibili、YouTube 等插件遵守同一个 Registry 和错误边界。

这样可以避免“具体站点都插件化，但通用解析器仍穿透 Core”的双重架构。

### 6. Native Site Panel 仍通过 Site Plugin Boundary

Control 不直接 import 站点页面组件。

插件声明 `native_panel` 能力，Site Browser Worker 负责站点浏览器运行；插件把用户选择转换成标准 `SourceContext / SourceLocator`。

站点 Chromium 仍然不能成为 Gateway 的播放状态 authority。

### 7. 第一阶段采用编译期插件化

MVP 优先验证架构边界，不追求运行时插件市场。

推荐：

```text
gateway-core
site-adapter-api
plugins/
├── generic-ytdlp
├── bilibili
└── ...
```

使用 Rust trait / workspace 一起编译发布。

成功标准是：新增站点主要新增 `plugins/<site>`，而不是修改 Core 业务代码。

### 8. 第二阶段优先演进为进程插件，而不是 Rust 动态库

当插件数量、独立更新频率或故障隔离需求足够高时，再稳定 Site Plugin Protocol：

```text
Gateway Core
↕ IPC
Site Plugin Process
```

独立进程优先于 Rust `.so`，因为可以获得：

- 崩溃隔离；
- 独立升级；
- 资源限制；
- 网络沙箱；
- 多语言实现；
- Playwright/Chromium 重依赖隔离。

具体 IPC 在契约成熟后再决定。

### 9. 插件协议必须显式版本化

未来进程插件至少需要：

```text
protocol_version
plugin_version
capabilities
```

Gateway 遇到不兼容版本必须拒绝加载，而不是静默运行未知行为。

### 10. 插件必须通过统一 Contract Test

所有插件需要验证：

- recognition 确定性；
- `ResolvedMedia` 契约；
- 导航 locator；
- 标准错误；
- Secret 不穿透边界；
- timeout / cancellation；
- capability 声明一致性。

CI 应逐步增加 architecture test，禁止 Core 出现 concrete site knowledge。

## 结果

优点：

- 站点变化被限制在高变化边缘。
- 新增站点不需要扩散修改 Playback / Display / Control Core。
- Native Site Panel、站点认证和 Resolver 能共享同一站点边界。
- 可以从编译期插件平滑演进到进程插件。
- 插件故障和安全风险未来可以独立隔离。
- Core 的模型和测试更加稳定。

代价：

- 必须认真设计 SiteAdapter Contract，而不能直接调用站点代码。
- 某些看似方便的站点特例不能直接写进 Core。
- 插件与 Core 之间需要标准错误、能力和版本模型。
- 运行时插件化若未来实施，会增加 IPC、生命周期和兼容性管理成本。

## 被拒绝方案

### Core 直接包含所有站点适配代码

初期简单，但会迅速让高变化站点知识污染稳定核心。

### Core 中保留若干站点 special case

会使插件边界失去约束力，最终重新演变为条件分支集合。

### MVP 立即采用 Rust 动态 `.so`

Rust ABI 和插件生命周期复杂度在契约尚未稳定前没有足够收益。

### MVP 立即建立第三方插件市场

当前最需要验证的是边界、能力契约和端到端播放，不是分发未知第三方代码。
