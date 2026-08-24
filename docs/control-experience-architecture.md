# Control 统一体验架构

## 1. 定位

核心原则：

> Control 是统一体验层，不是统一业务实现层。

用户只看到一个 `/control`，内部仍然由独立领域提供状态：

```text
/control
   ↓
Control Experience Layer
├── Now Playing
├── Universal Remote
└── Native Site Panel
       ↓
────────────────────────────
Playback Domain
Site Domain
Site Session Domain
Display Domain
```

Control 只聚合状态并发送 command，不持久化第二份业务真状态。

## 2. ControlView

概念只读 ViewModel：

```text
ControlView
├── now_playing
├── playback_controls
├── playback_context
├── active_display
├── site
├── site_account_state
├── native_site_panel
└── action_required
```

权威来源：

- position/current item → PlaybackSession。
- active display → Playback Coordinator / Display Domain。
- site account → Site Session Domain。
- native page runtime → Site Browser Worker。
- 页面中的站点语义 → Site Plugin。

UI 聚合状态不等于 UI 拥有状态。

## 3. Universal Remote

只承载跨站稳定语义：

- play / pause；
- seek；
- stop；
- previous / next；
- subtitle；
- autoplay；
- handoff。

这些动作统一转成 Playback command：

```text
POST /api/v1/sessions/{id}/commands
```

Control 不通过模拟站点播放器按钮来实现远端 pause/seek。

## 4. Native Site Panel

Native Site Panel 是兼容性兜底：

- 搜索；
- 选集；
- 收藏/历史；
- 清晰度/弹幕/站点专有设置；
- 其他尚未形成跨站通用契约的能力。

它默认可折叠；失败时 Universal Remote 仍可使用。

普通 iframe 不能作为通用实现基础。实际站点会话由服务器端 Site Browser Worker 持有。

## 5. Site Browser Worker 与 Site Plugin 分离

这是 Control 体验最重要的内部边界之一。

### Site Browser Worker 负责 runtime

- Chromium/Playwright 生命周期；
- profile attach/materialization；
- 远程画面与输入；
- 通用 navigation/browser event；
- timeout/资源限制。

### Site Plugin 负责站点语义

- DOM/API 规则；
- 登录成功判定；
- 当前内容识别；
- 选集/下一集；
- 站点专有 capability。

链路必须是：

```text
Site Browser Worker
→ BrowserEvent / BrowserSnapshot
→ Site Plugin.browser_interpret
→ SourceContext / SourceLocator / AccountState
→ Site command / Playback transition
```

禁止把 `BilibiliSelector`、`ep_id`、YouTube 参数等放进通用 Browser Worker。

## 6. Native 操作分类

### 6.1 Source-changing

例如选第 8 集、打开另一个视频、搜索结果进入播放。

```text
Browser Event
→ Site Plugin
→ SourceLocator
→ Site SelectSource command
→ Resolution Service
→ new PlaybackItem
→ keep active_display
```

### 6.2 PlaybackPreference-mappable

清晰度、音轨、字幕等，如果多个站点能稳定映射，可以形成通用 `PlaybackPreference`。

在形成稳定契约前，可以留在 Native Site Panel；但 UI 必须明确它是否会影响远端 Display。

### 6.3 Site-only

收藏、评论、页面布局、站点弹幕开关等默认只属于 Site Domain。

特别是站点页面“关闭弹幕”不能被解释为电视弹幕已关闭。

## 7. Control 用户状态

Control 主要只有：

```text
Idle
Resolving
Ready / Playing
Transition
ActionRequired
```

- Idle：继续观看/最近内容/新 URL。
- Resolving：正在识别/解析/准备媒体。
- Playing：Now Playing + Universal Remote。
- Transition：handoff、next item、URL refresh、登录后 retry。
- ActionRequired：登录、DRM、不支持、长期网络错误等。

用户文案描述意图，不暴露 prepare/revision/plugin 等内部术语。

## 8. 并发与恢复

Control 不维护本地“真状态”。

所有 mutation 使用：

```text
request_id
expected_session_revision?
```

服务端返回新的 `session_revision`。

手机锁屏、页面刷新、WebSocket 重连后：

```text
GET current session snapshot
→ subscribe events
→ 重建 ControlView
```

不会把电视播放重置到初始页。

## 9. 故障隔离

```text
Site Browser Worker crash
→ 已解析媒体继续播放
→ Universal Remote 仍可 pause/seek/stop
→ Native Panel 显示暂不可用
```

```text
Display offline
→ Site Browser/搜索仍可用
→ Control 显示无可用显示端
```

```text
SiteSession expired
→ SITE_AUTH_REQUIRED
→ Auth Mode
→ 登录成功
→ retry PendingIntent
```

## 10. 自动化测试

最低覆盖：

1. Playing 时 Native Panel 展开/关闭不影响 PlaybackSession。
2. Browser Worker crash 后 Universal Remote 仍工作。
3. Browser event 经 fake Site Plugin 产生 SourceLocator，再切 item。
4. Site native pause 不改变远端 Display。
5. Control refresh/reconnect 恢复 snapshot。
6. 两个 Control 同时 mutation 时 revision conflict 可恢复。
7. stale item/display callback 不影响当前 ControlView。

## 11. 不变量

- Control 是 View + Command 层。
- Site Browser Worker 是通用 runtime，不是站点适配器。
- Site Plugin 承担 concrete site knowledge。
- Native Site Panel 不是全局播放器。
- 站点功能只有经过明确语义映射后才进入 Universal Remote。
- Playback / Site / SiteSession / Display 可以独立失败和恢复。
