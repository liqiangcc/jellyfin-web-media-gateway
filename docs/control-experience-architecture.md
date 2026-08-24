# Control 统一体验架构

## 1. 设计目标

Control 对用户呈现为一个完整、连续的控制体验，但内部不把所有能力揉成一个业务模块。

核心原则：

> Control 是统一体验层，不是统一业务实现层。

用户看到的是一个页面、一个当前内容、一个遥控器和一个站点上下文；内部仍保留独立控制域、独立状态所有权和独立故障边界。

```text
/control
   ↓
Control Experience Layer
├── Now Playing
├── Universal Remote
└── Native Site Panel
       ↓
────────────────────────────────
内部领域保持分离
────────────────────────────────
Playback Domain
Source / Site Browser Domain
Site Session Domain
Display Domain
```

## 2. 为什么需要统一体验

如果所有站点能力都由 Gateway 重新实现，会快速遇到兼容性和维护成本问题。例如某个视频网站可能同时提供：

- 搜索；
- 选集；
- 清晰度；
- 弹幕；
- 收藏；
- 历史记录；
- 稍后再看；
- 会员专属能力；
- 站点频繁变化的播放器设置。

这些功能不应该全部进入 Gateway 的跨站核心契约。

另一方面，如果把 Gateway 遥控器和站点原生页面设计成两个完全分开的产品入口，用户会频繁在“遥控器”和“站点页面”之间切换，体验割裂。

因此采用：

```text
体验统一
+
控制域分离
```

## 3. Control Experience Layer

`Control Experience Layer` 负责组合各领域状态形成一个用户可理解的 ViewModel，但自身不成为业务状态 authority。

概念 ViewModel：

```text
ControlView
├── now_playing
│   ├── title
│   ├── episode
│   ├── position
│   └── state
├── playback_controls
├── active_display
├── playback_context
│   ├── previous
│   ├── next
│   └── autoplay
├── site
├── site_account_state
├── native_site_panel
└── action_required
```

规则：

- `position` 的权威状态来自 `PlaybackSession`；
- `active_display` 的权威状态来自 Playback Coordinator / Display Domain；
- 站点登录状态来自 `SiteSession`；
- 站点原生页面状态来自 `Site Browser Worker`；
- Control 只聚合和呈现，不建立第二套“真状态”。

因此：

```text
UI 聚合状态
≠
UI 拥有状态
```

## 4. 四个独立控制域

### 4.1 Playback Domain

负责跨站、跨显示端都成立的播放语义：

- play / pause；
- seek；
- stop；
- 当前进度；
- PlaybackContext；
- 上一集 / 下一集；
- 自动下一集；
- 字幕选择；
- 媒体生命周期。

权威对象：

```text
PlaybackSession
PlaybackContext
Playback Coordinator
```

站点原生播放器不能直接覆盖这些全局状态。

### 4.2 Source / Site Browser Domain

负责“用户想看什么”以及站点专有浏览能力：

- 搜索；
- 浏览首页 / 分类 / 历史；
- 打开番剧或视频详情；
- 选集；
- 收藏；
- 站点原生设置；
- 其他 Gateway 尚未抽象的站点能力。

权威对象：

```text
Site Browser Worker
SourceContext
```

其主要输出不是最终播放状态，而是稳定的来源上下文，例如：

```text
SourceContext
├── site_id
├── source_locator
├── title
├── collection / episode metadata
└── site_metadata
```

### 4.3 Site Session Domain

负责来源网站登录：

```text
SiteAccount
→ SiteSession
→ Session Vault
```

包括：

- 登录；
- 重新登录；
- 会话有效性；
- Cookie / localStorage / Token；
- 浏览器 profile。

它不负责播放进度，也不负责 Display。

### 4.4 Display Domain

负责“在哪里看”：

- Web Display；
- Jellyfin Display；
- 未来其他 DisplayAdapter；
- capability probe；
- start / stop；
- handoff；
- online / offline。

权威对象：

```text
DisplayInstance
DisplayAdapter
active_display
```

## 5. Control 页面组合

建议 `/control` 在体验上呈现三层：

```text
1. Now Playing
   当前内容 / 进度 / 当前显示端

2. Universal Remote
   播放暂停 / seek
   上一集 / 下一集
   字幕
   自动下一集
   handoff

3. Native Site Panel
   搜索 / 选集 / 收藏
   清晰度 / 弹幕 / 站点设置
   其他站点原生能力
```

Native Site Panel 默认可以折叠；当前来源没有原生面板时不显示。

手机端示意：

```text
┌────────────────────────────┐
│ 某番剧 · 第 7 集           │
│ 客厅电视 ●                 │
│                            │
│ -10s    暂停    +10s       │
│ 上一集          下一集     │
│ 字幕           切换显示端   │
├────────────────────────────┤
│ ▼ Bilibili 更多控制        │
│                            │
│ Native Site Panel          │
│ 搜索 / 选集 / 清晰度        │
│ 弹幕 / 收藏 / 历史          │
└────────────────────────────┘
```

用户感知为一个遥控器，而不是两个系统。

## 6. Native Site Panel 不是普通 iframe

不能把“内嵌原生页面”定义为直接 `<iframe src="站点">`。

原因包括：

- 站点可能通过 CSP `frame-ancestors` 禁止嵌入；
- 可能存在 `X-Frame-Options`；
- 跨站 Cookie / SameSite 行为不稳定；
- HttpOnly Cookie 无法由 Control 复制；
- 站点登录状态必须继续保留在 Ubuntu 服务端；
- iframe 无法提供可靠的跨站状态和输入控制边界。

因此 Native Site Panel 使用服务器端 `Site Browser Worker`：

```text
/control
   ↕ 远程画面 + 输入
Site Browser Worker
   └── Chromium / Playwright
          ↕
       来源站点
```

它可以复用现有站点 profile 和 `SiteSession`，但原始 Cookie、localStorage 和浏览器 profile 不下发到 Control 浏览器。

实现协议可以在 PoC 后选择，例如低延迟 WebRTC、远程画面流或其他受控通道；架构要求是“服务端浏览器是实际站点会话持有者”，不是特定传输协议。

## 7. Auth Browser Worker 泛化为 Site Browser Worker

原先 `Auth Browser Worker` 只在登录时启动。

新的长期模型是：

```text
Site Browser Worker
├── Auth Mode
│   └── 登录 / 验证码 / 扫码
└── Native Control Mode
    └── 搜索 / 选集 / 站点原生操作
```

MVP 可以继续只实现 Auth Mode；Native Control Mode 作为紧随其后的增量能力。

两种模式共享：

- 相同站点 profile；
- Session Vault；
- 站点网络限制；
- 输入/画面安全约束；
- 生命周期和资源限制。

## 8. Intent 层

统一体验不能让 UI 直接调用各领域私有实现。

推荐使用明确的 Intent：

```text
Control Intent
├── PlaybackIntent
│   ├── Play
│   ├── Pause
│   ├── Seek
│   ├── NextItem
│   └── Handoff(display_id)
│
└── SiteIntent
    ├── OpenSource(locator)
    ├── SelectSource(locator)
    ├── Login(site)
    └── NativeAction(...)
```

例如用户在 Native Site Panel 中选择第 8 集：

```text
用户选第 8 集
→ Site Browser Worker 当前 SourceContext 改变
→ SiteIntent::SelectSource(locator)
→ Resolver
→ 更新 PlaybackContext / ResolvedMedia
→ Playback Item Transition
→ 当前 active_display 继续播放
```

而不是让站点 Chromium 自己变成电视播放的状态 authority。

## 9. 站点原生功能与 Gateway 播放的同步分类

原生页面上的操作并不都能自动影响远端 Display，因此必须显式分类。

### 9.1 Source-changing

例如：

- 选择第 8 集；
- 打开另一个视频；
- 搜索后选择结果。

这些行为可以转换成新的 `SourceContext`，再由 Resolver 进入 Gateway 播放链路。

```text
Native Site
→ SourceContext
→ Resolve
→ PlaybackSession
```

### 9.2 可标准化 Playback Preference

例如部分站点的：

- 清晰度；
- 音轨；
- 字幕语言。

如果站点 Adapter 能稳定映射，可以提升为：

```text
PlaybackPreference
├── quality
├── audio
└── subtitle
```

然后由 Resolver / Media Gateway 选择对应媒体。

如果无法稳定映射，Native Panel 可以继续展示原生操作，但 UI 必须明确它只影响站点浏览器，而不保证影响当前电视播放。

### 9.3 Site-only UI Preference

例如：

- 页面布局；
- 站点弹幕开关；
- 收藏按钮；
- 评论区显示；
- 站点主题。

这些默认只属于 Site Browser Domain。

特别是弹幕：如果远端 Web Display 未来要显示弹幕，需要单独设计 `DanmakuTrack` 或等价 Gateway 能力；不能假设在 Bilibili 原生页面关闭弹幕会自动关闭电视上的弹幕。

## 10. 能力逐步提升原则

Native Site Panel 是兼容性兜底，不意味着所有能力永远停留在原生页面。

演进路线：

```text
阶段 1
站点原生页面兜底

   ↓ 高频能力稳定后

阶段 2
抽取跨站共性

   ↓

阶段 3
形成 Universal Control 能力
```

例如清晰度：

```text
最初
Bilibili Native Panel 中选择

后续
PlaybackPreference.quality
→ Bilibili Adapter
→ YouTube Adapter
→ 其他 Adapter
```

只有被证明稳定、跨站有共同语义的能力才进入 Gateway 核心契约。

## 11. 状态一致性原则

统一体验最容易出现的问题是多个状态源互相覆盖。

必须保证：

1. Site Browser Worker 不直接覆盖 `PlaybackSession.position`。
2. Native Site Player 的暂停状态不等于远端 Display 已暂停。
3. Site Page 选集只有经过 Gateway 接受新的 SourceContext 后才改变全局 current item。
4. Display 状态只能经 DisplayAdapter 汇报到 Playback Coordinator。
5. ControlView 只能聚合，不写入第二套状态数据库。
6. 同一用户动作只产生一个明确 Intent，避免同时模拟站点按钮和调用 Gateway Playback API。

## 12. 故障隔离

Native Site Panel 故障不得破坏已经存在的播放：

```text
Bilibili Site Browser 崩溃
→ 当前已解析媒体继续在客厅电视播放
→ Universal Remote 仍可 pause / seek / stop
→ Native Site Panel 显示“站点控制暂不可用”
```

反过来：

```text
Display 离线
→ Native Site Panel 仍可搜索和选内容
→ Control 明确显示“当前没有可用显示端”
```

Session 过期：

```text
SiteSession expired
→ Native Panel / Resolver 进入 SITE_AUTH_REQUIRED
→ 登录完成后恢复 PendingIntent
```

## 13. 自动化测试

Web Control 可以用 Playwright 覆盖体验层组合；Site Browser Worker 需要额外的 worker fake 或受控测试站点。

最低场景：

1. PlaybackSession 正在播放时，Native Panel 展开/收起不影响播放。
2. Native Panel 崩溃时 Universal Remote 仍能 pause / seek。
3. Native Panel 选择新的 source locator 后，Gateway 创建 item transition，Display 不变。
4. Site Page 的原生 pause 不得被误认为远端 Display pause。
5. Native quality 可映射时触发 PlaybackPreference；不可映射时不伪造远端状态。
6. 站点登录完成后 Native Panel 和 Resolver 使用同一新 SiteSession。
7. Control 刷新后重新聚合 Playback、Display、SiteSession 和 Site Browser 状态。

## 14. 设计不变量

- 架构分离，体验统一。
- Control 是 View + Intent 层，不是业务状态 authority。
- Native Site Panel 是兼容性兜底，不是全局播放器。
- Gateway 不重新实现所有视频网站能力。
- 站点原生能力只有经过显式映射后才能成为 Universal Control 能力。
- Playback / Source / Site Session / Display 四个领域可以独立失败和恢复。
