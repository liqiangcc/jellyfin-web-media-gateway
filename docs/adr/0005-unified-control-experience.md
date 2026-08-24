# ADR-0005：Control 统一体验，内部控制域保持分离

- 状态：已接受（设计阶段）
- 日期：2026-08-24

## 背景

Control 既需要提供稳定的跨站播放控制，又希望利用来源站点已经存在的原生能力，例如搜索、选集、清晰度、弹幕、收藏和历史记录。

如果 Gateway 重新实现每个视频网站的所有功能，站点变化会持续侵入核心播放模型；如果把 Gateway 遥控器和站点页面完全分开，用户又需要频繁切换入口，体验割裂。

项目需要同时满足：

- 用户只感知一个 `/control`；
- 高频播放控制保持跨站稳定；
- 低频、站点专有能力可以复用原生站点页面；
- 站点浏览器不能成为 Gateway 播放状态的第二个 authority；
- 任一领域故障不能无必要地扩散到其他领域。

## 决策

### 1. Control 是统一体验层，不是统一业务实现层

Control 负责聚合状态、呈现 UI 和发送 Intent。

它不拥有第二份播放、站点、登录或显示端真状态。

```text
Control Experience Layer
├── Now Playing
├── Universal Remote
└── Native Site Panel
```

底层保持独立：

```text
Playback Domain
Source / Site Browser Domain
Site Session Domain
Display Domain
```

### 2. `/control` 同时呈现 Universal Remote 与 Native Site Panel

用户体验上保持一个页面。

Universal Remote 提供已经稳定抽象的跨站能力：

- play / pause；
- seek；
- stop；
- 上一集 / 下一集；
- 字幕；
- 自动下一集；
- 显示端切换。

Native Site Panel 提供站点专有能力和兼容性兜底：

- 搜索；
- 选集；
- 收藏 / 历史；
- 清晰度、弹幕等尚未形成通用契约的站点能力；
- 其他站点原生操作。

Native Site Panel 可以默认折叠，不破坏手机端遥控器的简洁性。

### 3. Native Site Panel 使用服务端 Site Browser Worker

不把普通跨站 iframe 作为架构依赖。

原因包括 CSP、`X-Frame-Options`、SameSite、HttpOnly 和会话隔离等限制。

实际站点会话继续由 Ubuntu 服务端 Chromium/Playwright 持有：

```text
Control
↕ 受控远程画面 + 输入
Site Browser Worker
↕
来源站点
```

原始 Cookie、localStorage 和 profile 不下发给 Control 浏览器。

### 4. Auth Browser Worker 泛化为 Site Browser Worker

概念上统一为：

```text
Site Browser Worker
├── Auth Mode
└── Native Control Mode
```

MVP 可以先只有 Auth Mode；Native Control Mode 在不改变 Session Vault 边界的情况下增加。

### 5. SourceContext 是站点页面与播放核心之间的主要边界

站点页面主要负责“用户想看什么”。

当用户选择新的内容或剧集时，Site Browser Domain 输出稳定的 `SourceContext` / source locator：

```text
Native Site Panel
→ SourceContext
→ Resolver
→ PlaybackContext / ResolvedMedia
→ PlaybackSession
→ active_display
```

不能让站点 Chromium 内部播放器直接替代 Gateway 的 PlaybackSession。

### 6. 操作通过 Intent 路由

Control 不直接操作领域私有状态。

至少区分：

```text
PlaybackIntent
SiteIntent
```

例如：

- “暂停” → `PlaybackIntent::Pause`；
- “切到客厅电视” → `PlaybackIntent::Handoff`；
- “B 站选择第 8 集” → `SiteIntent::SelectSource` → Resolve → Item Transition。

一个用户动作不得同时通过“模拟站点播放器按钮”和 Gateway Playback API 修改同一个语义状态。

### 7. 站点原生操作按同步语义分类

#### Source-changing

选集、选择新视频、搜索结果进入播放等行为转成新的 SourceContext，再进入 Gateway 播放链路。

#### 可标准化 Playback Preference

清晰度、音轨、字幕等只有在站点 Adapter 能稳定映射时，才能提升为 Gateway `PlaybackPreference`。

#### Site-only

弹幕开关、收藏、页面布局、评论等默认只影响 Site Browser Domain。

例如 Bilibili 原生页面“关闭弹幕”不能被解释为远端电视弹幕已经关闭；如果未来要在 Web Display 支持弹幕，应设计独立 Gateway 弹幕能力。

### 8. 能力逐步提升，不提前统一所有站点

默认策略：

```text
原生站点能力兜底
→ 观察高频稳定用例
→ 抽取共同语义
→ 提升为 Universal Control
```

Gateway 核心契约只接受已经证明稳定、跨站有清晰共同语义的能力。

### 9. 故障域保持分离

- Site Browser Worker 崩溃时，已经解析并在 Display 播放的媒体继续工作；
- Display 离线时，Native Site Panel 仍可以浏览和选择内容；
- SiteSession 过期只进入站点认证恢复流程；
- ControlView 聚合失败不得写坏 PlaybackSession。

## 结果

优点：

- 用户只有一个统一 Control 体验；
- 可以直接利用站点原生能力提高兼容性；
- Gateway 不需要复制整个视频网站；
- 高频能力仍然有稳定的跨站接口；
- 可以逐步把成熟站点能力提升为 Universal Control；
- 状态所有权和故障边界清晰。

代价：

- Control Experience Layer 需要聚合多个领域状态；
- Site Browser Worker 需要低延迟远程画面/输入通道；
- 原生站点操作与远端播放之间必须维护明确同步语义；
- 某些站点原生设置只能影响服务端浏览器，不能自动影响远端 Display。

## 被拒绝方案

### Gateway 重写所有站点功能

维护成本过高，核心会被频繁变化的站点特性污染。

### 原生站点页面和 Gateway 遥控器完全分离

架构简单，但用户需要反复切换页面，无法形成自然的遥控体验。

### 直接 iframe 来源站点

跨站安全策略和 Cookie 边界不可靠，不能作为通用架构基础。

### 让站点原生播放器成为全局播放 authority

会与 `PlaybackSession`、DisplayAdapter 和跨设备 handoff 产生双重状态源。

### 所有原生设置都自动同步到远端 Display

不同功能的语义不同，容易产生“UI 显示已改变但远端没有改变”的错误状态。只有显式映射的能力才能同步。
