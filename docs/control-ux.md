# Control UX 与站点账号管理

## 1. 目标

Control 的定位不是管理后台，也不是某个播放器的附属控制条，而是 `PlaybackSession` 的遥控器、显示端调度器和异常恢复入口。

设计原则：

- 以用户意图为中心，不暴露 Resolver、Adapter、revision 等内部术语。
- 手机优先，绝大多数高频操作在一屏内完成。
- 正在播放时优先显示“当前内容和控制”；空闲时才突出“播放新 URL”。
- 显示端切换、站点登录、媒体刷新等异常处理完成后自动恢复用户原始意图。
- Control 只消费 Gateway 的权威状态；刷新、重连和多控制端并发时不维护自己的“真状态”。
- MVP 假设可信 LAN、单用户使用，不引入 Gateway 用户/角色认证体系；本文件中的“登录”默认指来源网站登录。
- 架构上分离控制域，体验上统一为一个 `/control`；Control 本身不成为新的状态 authority。

## 2. 信息架构

`/control` 首页只保留四类一等信息：

```text
Control
├── 正在播放 / 继续观看
├── 播放新内容
├── 显示端
└── 站点账号
```

其中“站点账号”是辅助管理入口，不应成为普通播放流程的必经步骤。

推荐确定性入口：

```text
/control          主控制页
/control/sites    站点账号管理
```

首个版本不需要复杂多级导航。

在具体内容播放期间，主控制页内部可以组合三层体验：

```text
Now Playing
Universal Remote
Native Site Panel（按需展开）
```

## 3. Control UX 状态机

Control 主要呈现 5 种用户可理解状态：

```text
Idle
  ↓
Resolving
  ↓
Ready / Playing
  ├──→ Transition
  └──→ ActionRequired
```

### 3.1 Idle

当前没有活动播放时，主动作是：

- 继续观看；
- 最近内容；
- 粘贴新 URL；
- 显示默认目标显示端。

URL 输入框只在空闲状态成为视觉主角。

### 3.2 Resolving

用户粘贴 URL 后，Control 显示“正在准备内容”，并把解析和显示端启动区分开。

至少能够区分：

```text
正在识别网页
正在获取媒体
正在准备字幕
正在准备显示端
```

技术细节可以进入诊断信息，但不应成为主界面语言。

### 3.3 Ready / Playing

解析完成后，主界面围绕当前内容：

```text
标题 / 剧集
进度
播放 / 暂停
-10s / +10s
上一集 / 下一集（存在时）
字幕
当前显示端
自动下一集
停止
```

倍速、画质、音轨、详细媒体格式等低频设置放入二级操作；其中站点专有设置可以由 Native Site Panel 提供兼容性兜底。

### 3.4 Transition

以下行为属于过渡状态：

- 切换显示端；
- 播放下一集；
- 媒体 URL 刷新；
- 登录完成后重新解析；
- display 断线后的恢复。

Control 应描述用户意图，例如：

```text
正在切换到客厅电视…
正在准备下一集…
登录成功，正在继续刚才的视频…
```

而不是显示 `prepare/confirm/commit` 等内部状态名。

### 3.5 ActionRequired

只有 Gateway 无法自动恢复时才要求用户介入，例如：

- `SITE_AUTH_REQUIRED`：需要登录来源站点；
- `DRM_UNSUPPORTED`；
- `DISPLAY_UNSUPPORTED`；
- 网络长时间不可恢复；
- 站点要求验证码或扫码。

用户完成动作后，Gateway 应继续原始操作，不要求重新粘贴 URL 或重新选择显示端。

## 4. 播放新内容的自然流程

默认目标是把常见操作压缩到：

```text
粘贴 URL
→ 解析
→ 在默认显示端播放
```

如果已有默认显示端，Control 不应每次强制弹出设备选择。

可在主动作附近显示：

```text
将播放到：客厅电视 ▾
```

用户需要时再修改。

解析成功但显示端不兼容，与解析失败必须分开呈现：

```text
解析失败：需要登录 Bilibili
```

和：

```text
媒体已准备，但客厅电视不支持该格式
```

属于不同故障域。

## 5. 显示端切换

用户操作语言是“在哪里看”，而不是“使用哪个 Adapter”。

示例：

```text
播放位置
✓ 客厅电视
  卧室电视
  当前手机
```

选择新显示端后：

```text
正在切换到当前手机…
```

成功后再更新 `active_display`；失败时明确提示：

```text
当前手机无法播放此格式
客厅电视仍在继续播放
```

Control 不应在目标显示端真正确认前乐观地把 UI 改成“切换成功”。

## 6. 来源网站登录：播放驱动的按需认证

### 6.1 默认不强制提前登录

同一个站点可能存在公开内容和需要登录的内容，因此 Control 不应仅凭 URL 属于某站点就要求登录。

推荐流程：

```text
输入 URL
→ Resolver 使用现有 SiteSession 尝试解析
→ 成功：继续播放
→ SITE_AUTH_REQUIRED：提示“登录该站点后继续”
```

### 6.2 登录是可恢复动作

当播放触发登录时，Gateway 保存当前用户意图，例如：

```text
PendingIntent
├── source_url
├── requested_display
├── playback_action = play
└── related_context
```

用户点击：

```text
登录 Bilibili 后继续
```

流程：

```text
启动 Site Browser Worker（Auth Mode）
→ 用户扫码 / 输入账号 / 完成验证码
→ 新 SiteSession 验证成功
→ 写入 Session Vault
→ 关闭交互浏览器或切回 Native Control Mode
→ 自动 retry 原 URL
→ 继续原显示端播放
```

用户不应重新粘贴 URL。

### 6.3 不保存网站密码

Gateway 不设计为密码管理器。

持久化内容仅限经网站认证后产生的必要会话材料，例如：

- Chromium profile；
- Cookie；
- localStorage；
- 必要 Token；
- 不敏感账号显示元数据。

账号密码、验证码输入和二维码画面不得写入数据库或日志。

## 7. 站点账号管理

按需登录之外，Control 必须提供主动管理能力，方便用户查看、更新或清除服务器上的站点会话。

推荐入口：

```text
/control/sites
```

示例：

```text
站点账号

Bilibili
  ● 已登录
  账号：li***ng
  最近验证：今天 16:00
  [重新登录] [退出登录]

YouTube
  ○ 未登录
  [登录]
```

### 7.1 SiteAccount 模型

不要把实现写死成 `bilibili_cookie` 一类字段。

概念模型：

```text
SiteAccount
├── id
├── site_id
├── label
├── state
├── profile_ref
├── account_metadata
├── last_validated_at
└── invalid_reason
```

MVP 约束：

> 每个站点最多一个活动账号。

但存储模型保留 `SiteAccount.id`，以后可以扩展一个站点多个账号，而不重构 Session Vault。

### 7.2 登录状态

不要只有“已登录 / 未登录”两个状态。

内部至少允许：

```text
unknown
checking
valid
expired
login_required
error
```

Control 翻译成人类可理解文案：

```text
登录状态未知
正在验证
已登录
需要重新登录
未登录
站点暂时不可用
```

### 7.3 登录

主动点击“登录”与播放过程中触发登录使用同一个 Site Browser Worker（Auth Mode）和 Session Vault 流程。

区别只是：主动管理没有 `PendingIntent`，登录成功后返回站点账号页面。

### 7.4 重新登录

“重新登录”不是立即删除旧会话。

推荐流程：

```text
保留当前 SiteSession
→ 启动新的临时登录 profile / session
→ 新会话验证成功
→ 原子替换活动 SiteSession
→ 清理旧会话
```

如果新登录失败或用户取消，尽可能保留原有仍可能有效的会话。

### 7.5 退出登录

退出登录是破坏性操作：

```text
删除 / 失效活动 SiteSession
→ 清理 Cookie / localStorage / profile / token
→ state = login_required
```

Control 应进行一次明确确认，并说明之后再次播放该站点受限内容需要重新登录。

## 8. 连续内容、上一集与下一集

“下一集”不应由 Control 修改 URL 或猜测集号，而应来自 Resolver / 站点适配器提供的播放上下文。

推荐领域模型：

```text
PlaybackContext
├── current_item
├── previous_item
├── next_item
├── queue
└── autoplay_policy
```

`next_item` 保存稳定的来源 locator，而不是提前保存可能很快过期的 HLS URL。

切换下一集：

```text
当前 item ended 或用户点击下一集
→ Gateway 获取 next_item source locator
→ Resolver 重新解析
→ 保持 active_display
→ start next item
```

这是 `PlaybackItemTransition`，不是 Display Handoff。

### 8.1 自动下一集

MVP 可以只支持：

```text
autoplay = off | next
```

接近结尾时可以预取下一集元数据，但短期媒体地址应在真正切换时刷新/重新解析。

如果下一集需要重新登录：

```text
当前集结束
→ resolve next
→ SITE_AUTH_REQUIRED
→ PlaybackSession = ActionRequired
→ Control 显示“下一集需要重新登录”
→ 登录成功
→ 自动继续下一集
```

Display 不应无解释地停在黑屏。

## 9. 刷新、锁屏与恢复

用户重新打开 `/control` 时优先恢复正在播放内容，而不是回到 URL 输入首页。

```text
/control 打开 / WebSocket 重连
→ 获取 now-playing / 当前 PlaybackSession
→ 恢复标题、进度、active_display、字幕和 transition 状态
```

手机锁屏、切后台或页面刷新不应破坏电视播放。

## 10. 多控制端

MVP 虽然是单用户，但手机和 Windows 仍可能同时打开 Control。

原则：

- Gateway 是状态 authority；
- 所有 mutation 由 Gateway 串行化或使用 revision/CAS；
- Control 收到冲突后刷新最新状态；
- 不通过“谁最后打开页面”决定控制权。

完整 Gateway 身份、角色和多用户权限体系暂不属于 MVP。

## 11. 自动化测试场景

Control UX 应优先通过 Playwright 自动化：

1. Idle 粘贴公开 URL → Resolving → Playing。
2. 默认显示端存在时，一次播放动作直接发送到该显示端。
3. 打开 `/control` 不自动注册 Web Display。
4. 点击“在本机播放”后才创建 Web Display 并 handoff。
5. 目标 display handoff 失败时旧 display 继续播放。
6. URL 返回 `SITE_AUTH_REQUIRED` → 登录成功 → 自动 retry 原 URL。
7. 主动进入 `/control/sites` 登录站点，不影响当前播放。
8. 重新登录失败时保留旧会话。
9. 退出登录后站点状态变为“需要登录”。
10. 当前集结束 → 自动解析下一集 → 在同一 active display 继续。
11. 下一集要求认证 → ActionRequired → 登录 → 自动继续。
12. 手机 Control 刷新/重连后恢复当前播放状态。
13. Native Site Panel 展开/收起不影响当前 PlaybackSession。
14. Native Site Panel 失败时 Universal Remote 仍然可用。
15. Native Site Panel 选择新的 source locator 时，通过 Gateway 发生 Item Transition，而不是让站点播放器直接接管 Display。

## 12. 设计边界

首个 MVP 明确不做：

- Gateway 用户账号、RBAC 或家庭成员权限体系；
- 站点多账号同时激活；
- 密码托管；
- 跨用户播放队列；
- 多显示端同步播放；
- 在 Phase 0A 就实现完整 Native Site Panel 远程浏览体验。

这些能力如果以后出现真实需求，应在不破坏 `SiteAccount`、`PlaybackContext`、`PlaybackSession` 和 `DisplayAdapter` 边界的前提下独立扩展。

## 13. 统一体验与 Native Site Panel

### 13.1 架构分离，体验统一

Control 页面可以把 Gateway 遥控器和来源站点原生能力组合在一起，但内部状态所有权保持分离。

```text
/control
├── Now Playing
├── Universal Remote
└── Native Site Panel

底层：
Playback Domain
Source / Site Browser Domain
Site Session Domain
Display Domain
```

Control Experience Layer 只聚合这些状态，不创建自己的业务真状态。

### 13.2 Universal Remote

Universal Remote 只承载已经稳定、跨站有共同语义的能力：

- 播放 / 暂停；
- seek；
- stop；
- 上一集 / 下一集；
- 字幕；
- 自动下一集；
- 显示端切换。

这些操作通过 `PlaybackIntent` 进入 Gateway，而不是模拟站点网页播放器操作。

### 13.3 Native Site Panel

Native Site Panel 负责兼容性和站点专有体验，例如：

- 搜索；
- 选集；
- 收藏；
- 历史；
- 清晰度；
- 弹幕；
- 站点专有设置。

它默认可以折叠，从而保持手机遥控器主界面简洁。

### 13.4 不依赖普通 iframe

来源站点可能使用 CSP、`X-Frame-Options`、SameSite 和 HttpOnly 等机制，普通 iframe 不能作为通用架构基础。

因此 Native Site Panel 对应服务器端 `Site Browser Worker`：

```text
/control
  ↕ 远程画面 + 输入
Site Browser Worker
  ↕
来源站点
```

原始 Cookie、localStorage 和 Chromium profile 不离开 Ubuntu 服务器。

### 13.5 Auth Browser Worker 泛化

原来的 Auth Browser Worker 在概念上泛化为：

```text
Site Browser Worker
├── Auth Mode
└── Native Control Mode
```

MVP 先完成 Auth Mode；Native Control Mode 后续复用相同 profile、Session Vault 和安全边界。

### 13.6 原生站点选择内容的流程

用户在 Bilibili 原生区域选择第 8 集时：

```text
Native Site Panel
→ SourceContext / source locator 改变
→ SiteIntent::SelectSource
→ Resolver
→ PlaybackContext / ResolvedMedia 更新
→ Playback Item Transition
→ 当前 active_display 继续播放
```

站点 Chromium 的内部播放器不是电视播放的全局状态源。

### 13.7 原生功能同步规则

原生能力分为三类：

1. **Source-changing**：选集、选择新视频、搜索结果播放；转换为新的 SourceContext。
2. **可标准化 Playback Preference**：清晰度、音轨、字幕等，在 Adapter 能稳定映射时提升为 Gateway 通用能力。
3. **Site-only**：收藏、评论、页面布局、弹幕开关等默认只影响 Site Browser Domain。

特别是弹幕：在站点原生页面关闭弹幕不能被解释为远端电视弹幕已经关闭。未来如需 Web Display 弹幕，应独立设计 Gateway 弹幕轨道/渲染能力。

### 13.8 逐步提升

站点能力先通过 Native Site Panel 兜底；只有高频、稳定、跨站有共同语义的功能才逐步进入 Universal Remote。

```text
Native Site capability
→ 稳定验证
→ 抽取公共语义
→ Universal Control / PlaybackPreference
```

详细架构见 `docs/control-experience-architecture.md`。
