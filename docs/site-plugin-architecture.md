# Site Plugin Architecture

## 1. 目标

网页媒体站点是整个系统里变化最快的部分。Bilibili、YouTube 或未来其他站点的 URL 结构、Cookie、页面 DOM、接口返回、选集方式、清晰度能力和登录流程都可能独立变化。

Gateway 的稳定核心不应该吸收这些变化。

核心原则：

> Gateway 可以识别站点，但不理解站点。

也就是说，Core 可以知道：

```text
site_id = bilibili
```

以便完成插件路由、站点会话隔离、能力展示和错误归类；但 Core 不应该知道 Bilibili 的 BV/EP/Season 规则、Cookie 名称、DOM selector、接口参数或下一集算法。

## 2. 分层与变化方向

系统按变化率分成三层：

```text
                Stable Core
────────────────────────────────
PlaybackSession / PlaybackContext
PlaybackCoordinator
DisplayAdapter
MediaGateway
Control Experience
Session Vault boundary

                ↓ contracts

             Site Abstraction
────────────────────────────────
SiteId
SourceDescriptor / SourceLocator
SiteAccountRef / SiteSessionRef
SiteCapabilities
SiteAdapter Contract
SiteError

                ↓ implementations

              Volatile Edge
────────────────────────────────
Bilibili Plugin
YouTube Plugin
Generic yt-dlp Plugin
Site Browser automation
DOM selectors
site-specific APIs / cookies / rules
```

依赖方向只能从具体实现指向抽象契约，Stable Core 不反向依赖任何具体站点实现。

## 3. Core 允许知道什么

Core 可以持有站点身份和稳定的通用描述，例如：

```text
SourceDescriptor
├── site_id
├── source_locator
└── account_ref?          # 可选，仅引用，不包含凭据
```

Core 可以使用 `site_id`：

- 从 `SiteAdapterRegistry` 选择插件；
- 从 Session Vault 选择对应站点会话；
- 在 Control 中显示“当前来源：Bilibili”；
- 判断某站点是否提供 Native Site Panel；
- 记录不含 Secret 的站点级错误和可观察性数据；
- 做插件健康状态、版本和能力查询。

这些属于路由和编排知识，不属于站点实现知识。

## 4. Core 禁止知道什么

以下内容必须留在具体 Site Plugin 内部：

- 具体站点域名匹配细节和 URL 参数语义；
- Bilibili 的 BV、AV、EP、Season 等标识规则；
- YouTube playlist/video 参数规则；
- 站点 Cookie、localStorage key、Token 名称；
- 站点私有 API 请求/响应结构；
- 页面 DOM selector；
- 清晰度编码和站点播放器内部枚举；
- 弹幕、收藏、历史等站点专有实现；
- 上一集/下一集如何从站点页面或接口推导；
- 站点登录成功如何判定；
- 站点风控、重定向和页面跳转的特殊处理。

尤其禁止在 Core 中出现：

```text
if site == "bilibili" { ... }
if site == "youtube" { ... }
```

业务层出现这种分支通常意味着 Site Plugin Boundary 被穿透。

## 5. SiteAdapter Contract

接口应从用例和能力出发，而不是为某个站点现有代码量身定制。

概念能力：

```text
SiteAdapter
├── recognize(source)
├── resolve(source, session_ref)
├── navigation(source, session_ref)
├── account_probe(session_ref)
├── login_support()
├── native_panel_support()
├── source_context(browser_state)
└── capabilities()
```

这些名称只是设计级表达，最终 Rust trait / Plugin Protocol 可以拆分或合并；稳定的是能力边界，而不是方法名。

### 5.1 recognize

回答：

> 这个插件是否能处理这个 source？

输出至少包含：

- match / no-match；
- 规范化 `site_id`；
- 可选的 confidence / priority；
- 标准化 `SourceLocator`。

多个插件同时匹配时由 Registry 使用明确优先级处理，不能依赖注册顺序产生隐式行为。

### 5.2 resolve

把来源 locator 转换成与站点无关的：

```text
ResolvedMedia
```

站点插件可以使用被授权的 Site Session，但输出不得把原始 Cookie、Authorization 或站点私有 Token泄露给 Display。

### 5.3 navigation

如果站点能识别连续内容，插件输出稳定的导航上下文：

```text
NavigationContext
├── previous: SourceLocator?
├── next: SourceLocator?
├── collection_id?
└── current_index?
```

Core 只知道“下一项 locator”，不知道某站点如何计算下一集。

### 5.4 account / login

插件负责识别站点会话是否有效以及站点登录成功条件，但原始会话材料仍由 Session Vault / Site Browser Worker 保管。

插件不应该自己建立第二套凭据数据库。

### 5.5 native panel

插件可以声明：

```text
native_panel = unsupported | supported
```

支持时，Control Experience 通过统一 Site Browser Session 边界显示受控的站点原生交互界面。

Control 不 import `BilibiliPanel` 之类的具体组件。

## 6. SiteAdapterRegistry

Gateway Core 只面向 Registry：

```text
Gateway Core
    ↓
SiteAdapterRegistry
    ├── bilibili
    ├── youtube
    ├── generic-ytdlp
    └── ...
```

推荐 Registry 责任：

- 插件注册；
- `site_id` 唯一性校验；
- source recognition；
- 插件能力和版本查询；
- 健康状态；
- 冲突检测；
- fallback 选择。

Generic yt-dlp 应作为 fallback adapter，而不是在 Core 中直接调用 yt-dlp 并穿透插件边界。

## 7. SourceLocator 是关键分离点

Core 不应该长期保存某站点内部短期媒体 URL。

站点之间共享的是：

```text
SourceLocator
```

它表示“如何重新定位用户选择的内容”，而不是“现在这一刻可以拉流的 CDN URL”。

例如：

```text
PlaybackContext.next_item
    ↓
SourceLocator
    ↓
对应 Site Plugin resolve()
    ↓
新的 ResolvedMedia
```

这样下一集、URL 过期、重新登录后的 retry 都不需要 Core 理解具体站点 URL 语义。

## 8. 与 Control Experience 的边界

Control 只消费通用：

```text
SiteCapabilities
SourceContext
NativePanelDescriptor
```

而不是直接写：

```text
if bilibili → render BilibiliPanel
```

Native Site Panel 中用户选择新内容时：

```text
Site Browser Worker
→ Site Plugin 提取 SourceContext / SourceLocator
→ SiteIntent::SelectSource
→ Resolver / SiteAdapter
→ Playback Item Transition
```

站点 Chromium 内部播放器不会成为 Gateway 的播放 authority。

## 9. 能力逐步提升

插件化不意味着所有站点能力永久停留在插件层。

采用：

```text
站点原生能力
    ↓
多个插件出现稳定共同语义
    ↓
形成标准 capability
    ↓
提升到 Gateway 通用契约
```

例如第一版清晰度可能只存在于 Bilibili Native Site Panel；当多个插件都能稳定表达清晰度时，可以提升为：

```text
PlaybackPreference.quality
```

插件负责把通用 preference 映射成站点具体实现。

## 10. 第一阶段：架构插件化

MVP 不要求运行时动态加载插件。

推荐先使用 Rust workspace 和 trait：

```text
gateway-core/
site-adapter-api/
plugins/
├── generic-ytdlp/
├── bilibili/
└── ...
```

插件与 Gateway 一起编译和发布。

这样先验证：

- SiteAdapter Contract 是否稳定；
- Core 是否真的没有具体站点知识；
- 插件测试是否能独立运行；
- 新增站点是否无需修改 Core 业务代码。

不要为了“插件”这个名字过早引入 Rust 动态 `.so` ABI。

## 11. 第二阶段：进程插件化

当站点插件数量、更新频率、依赖差异或故障隔离需求足够高时，再把 SiteAdapter Contract 稳定成进程协议：

```text
Gateway Core
    ↕ Site Plugin Protocol
Plugin Host / Plugin Process
    ├── bilibili
    ├── youtube
    └── generic-ytdlp
```

协议可使用 stdio JSON-RPC、Unix socket 或其他明确 IPC；具体选型在契约稳定后决定。

相较 Rust 动态库，独立进程优先级更高，因为它允许：

- 插件崩溃不拖垮 Gateway；
- 插件独立升级和回滚；
- 独立 CPU / 内存 / 超时限制；
- 独立网络权限；
- Playwright / Chromium 等重依赖不污染 Core；
- 插件可以使用 Rust、Python、Node 等不同技术栈；
- 高风险站点解析代码获得更清晰的沙箱边界。

## 12. Plugin Protocol 需要稳定的对象

未来进程协议优先围绕数据契约稳定，而不是暴露内部 Rust 类型。

候选对象：

```text
PluginManifest
SiteCapabilities
SourceDescriptor
SourceLocator
ResolvedMedia
NavigationContext
AccountProbeResult
NativePanelDescriptor
SiteError
```

协议版本需要显式：

```text
protocol_version
plugin_version
capability_version
```

Gateway 必须能够拒绝不兼容插件，而不是带着未知契约继续运行。

## 13. 错误模型

插件内部可以拥有丰富站点错误，但穿越边界时应标准化：

```text
SITE_AUTH_REQUIRED
SITE_UNSUPPORTED
SOURCE_NOT_FOUND
SOURCE_EXPIRED
SITE_RATE_LIMITED
SITE_TEMPORARILY_UNAVAILABLE
DRM_UNSUPPORTED
PLUGIN_BUG
PLUGIN_TIMEOUT
```

必要的站点调试细节只能进入脱敏诊断信息，不能把 Cookie、Token、完整私有 URL 或页面内容直接传给 Control。

## 14. 故障隔离

插件失败时：

- 不得破坏当前 `PlaybackSession` 已确认状态；
- 不得使其他站点插件失效；
- 不得清空无关 SiteSession；
- 下一集 resolve 失败时当前 Display 应获得可解释状态，而不是黑屏；
- Native Site Panel 崩溃不应停止已在电视播放的媒体。

未来进程插件应具备：

- 超时；
- restart policy；
- health check；
- circuit breaker；
- resource limit。

## 15. 安全边界

Site Plugin 属于高变化、高风险边缘代码。

原则：

- 插件不能直接读取其他站点 Session；
- 插件只能获得当前调用所需的 scoped session capability；
- 插件不能直接控制 `active_display`；
- 插件不能直接绕过 Media Gateway 给 Display 下发上游 Cookie；
- 插件不能任意访问本机内网；网络访问继续受 Gateway / sandbox 策略约束；
- Site Browser Worker 的 profile 与插件调用按站点隔离。

## 16. 测试策略

### 16.1 Contract Test

所有插件必须通过同一套 SiteAdapter conformance tests：

- manifest / capability 合法；
- recognize 行为确定；
- resolve 输出满足 `ResolvedMedia` 契约；
- Secret 不越界；
- 错误映射稳定；
- timeout / cancel 可处理。

### 16.2 Plugin-specific Test

每个插件维护自己的：

- URL fixtures；
- DOM / API fixture；
- 登录状态 fixture；
- 导航和下一集 fixture；
- 站点能力测试。

### 16.3 Architecture Test

CI 应增加架构约束，例如扫描 Core 目录，禁止出现已注册具体站点域名、Cookie 名称、DOM selector 等 concrete site knowledge。

新增一个站点插件的理想 diff 应主要位于：

```text
plugins/<site>/
```

如果新增站点要求同时修改 PlaybackCoordinator、DisplayAdapter 或 Control 核心业务分支，应进行架构评审。

## 17. 示例：Bilibili

```text
用户粘贴 Bilibili URL
        ↓
SiteAdapterRegistry.recognize
        ↓
site_id = bilibili
        ↓
Bilibili Plugin.resolve
        ↓
ResolvedMedia + NavigationContext
        ↓
Gateway PlaybackSession
        ↓
Web/Jellyfin Display
```

Gateway Core 可以记录：

```text
source.site_id = bilibili
```

但不会出现：

```text
BV / ep_id / season_id
Bilibili Cookie key
Bilibili DOM selector
Bilibili quality enum
```

这些全部属于 Bilibili Plugin。

## 18. 首阶段非目标

首个实现明确不要求：

- 第三方插件市场；
- 在线安装未知插件；
- Rust 动态 `.so` ABI；
- 插件热更新；
- 任意未签名外部插件执行。

先证明边界和契约稳定，再决定是否需要真正的运行时插件生态。
