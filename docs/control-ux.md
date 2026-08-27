# Control UX 与站点账号管理

## 1. UX 目标

Control 不是后台管理页，而是用户的媒体遥控器和异常恢复入口。

设计原则：

- 手机优先。
- 正在播放时优先显示当前内容和控制；空闲时才突出新 URL。
- 默认显示端存在时，常见路径尽量压缩为“粘贴 URL → 播放”。
- 登录、handoff、下一集、URL 刷新等可恢复动作完成后自动继续原意图。
- 用户不需要理解 Resolver、Adapter、revision、plugin 等内部术语。
- Control 只显示 Gateway snapshot，不保存第二套播放状态。

## 2. 页面结构

```text
/control
├── Now Playing / Continue Watching
├── Universal Remote
├── Play New Content
├── Display Selector
└── Native Site Panel（按需）

/control/sites
└── Site Account Management
```

Native Site Panel 是同一体验的一部分，但不是播放 authority。

## 3. Control 状态

### Idle

显示：

- 继续观看；
- 最近内容；
- URL 输入；
- 默认播放位置。

### Resolving

用户看到：

```text
正在识别网页…
正在准备媒体…
正在准备字幕…
```

解析失败与显示端失败必须分开说明。

### Ready / Playing

高频操作一屏完成：

```text
标题 / 剧集
进度
-10s  播放/暂停  +10s
上一集 / 下一集
字幕
当前显示端
自动下一集
停止
```

### Transition

用于：

- handoff；
- next item；
- media refresh；
- 登录后 retry。

文案示例：

```text
正在切换到客厅电视…
正在准备下一集…
登录成功，正在继续刚才的视频…
```

### ActionRequired

只在无法自动恢复时打断用户：

- 需要来源站点登录；
- DRM；
- display unsupported；
- 长时间网络错误；
- 验证码/扫码。

## 4. 播放新内容

推荐：

```text
粘贴 URL
→ Gateway 识别来源
→ 创建 SourceLocator
→ resolve
→ 默认显示端播放
```

Control 不知道 Bilibili/YouTube URL 结构，也不直接调用 yt-dlp。

有默认显示端时不每次弹设备选择，只显示：

```text
将播放到：客厅电视 ▾
```

## 5. 显示端切换

用户操作语言是“在哪里看”。

```text
播放位置
✓ 客厅电视
  当前手机
  其他显示端
```

handoff 期间：

```text
正在切换到当前手机…
```

只有服务端 commit 后才显示成功。

失败：

```text
当前手机无法播放此格式
客厅电视仍在继续播放
```

## 6. 上一集 / 下一集

Control 不修改 URL 猜集号。

```text
PlaybackContext.next
→ SourceLocator
→ Resolution Service
→ new PlaybackItem
→ 当前 Display 继续播放
```

自动下一集 MVP：

```text
autoplay = off | next
```

下一集需要登录时：

```text
当前集结束
→ resolve next
→ SITE_AUTH_REQUIRED
→ Control 提示登录
→ 登录成功
→ 自动继续下一集
```

## 7. 来源站点登录

### 7.1 播放驱动登录

不因为 URL 属于某站点就强制提前登录。

```text
resolve with current SiteSession
→ success：继续
→ SITE_AUTH_REQUIRED：登录该站点后继续
```

Gateway 保存 PendingIntent；登录成功后自动 retry 原 `SourceLocator` 和目标显示端。

### 7.2 站点账号管理

`/control/sites` 提供：

```text
Bilibili
● 已登录
账号：li***ng
最近验证：今天
[重新登录] [退出登录]
```

状态至少允许：

```text
unknown
checking
valid
expired
login_required
error
```

### 7.3 重新登录

```text
保留旧 SiteSession
→ 新临时登录
→ Site Plugin 验证成功
→ 原子替换
→ 清理旧会话
```

失败/取消尽量保留旧会话。

### 7.4 不保存密码

Gateway 不保存：

- 网站密码；
- 验证码输入；
- 二维码画面。

只保存认证后的必要会话材料。

## 8. Native Site Panel

用户体验可以统一：

```text
Now Playing
Universal Remote
▼ Bilibili 更多控制
   Native Site Panel
```

Native Site Panel 可以提供：

- 搜索；
- 选集；
- 收藏/历史；
- 清晰度/弹幕/站点设置。

但内部边界必须是：

```text
Site Browser Worker
→ BrowserEvent
→ Site Plugin interprets
→ SourceLocator / Site State
```

而不是 Browser Worker 自己理解 Bilibili。

用户在原生面板选择第 8 集：

```text
Browser event
→ Bilibili Plugin
→ SourceLocator
→ SelectSource command
→ resolve
→ PlaybackItemTransition
```

站点播放器自身的 pause/seek 不影响电视播放。

## 9. 清晰度 / 字幕 / 弹幕

能力分三类：

1. **通用 Playback command**：play/pause/seek/next/handoff。
2. **可标准化 Preference**：清晰度、音轨、字幕，只有 Site Plugin 能稳定映射时才提升。
3. **Site-only**：收藏、评论、页面布局、站点弹幕开关等。

站点页面关闭弹幕不能被显示成“电视弹幕已关闭”。远端弹幕需要未来独立 Gateway 能力。

## 10. 刷新与多控制端

页面刷新/手机锁屏后：

```text
GET current session snapshot
→ subscribe events
→ restore ControlView
```

多个 Control 同时打开时：

- mutation 发送 `request_id`；
- 可携带 `expected_session_revision`；
- 冲突后刷新服务端最新 snapshot。

不通过“最后打开页面的人”决定控制权。

## 11. 自动化测试

最低场景：

1. Idle → Resolving → Playing。
2. 默认显示端快速播放。
3. `/control` 不自动注册 Display。
4. handoff 失败旧 Display 继续。
5. Control refresh/reconnect 恢复。
6. `SITE_AUTH_REQUIRED` → 登录 → 自动 retry。
7. `/control/sites` 登录/重新登录/退出。
8. NextItem 使用 SourceLocator 而不是前端拼 URL。
9. 旧 item callback 不改变新 item UI。
10. Native Panel crash 不影响 Universal Remote。
11. Browser event 必须经 fake Site Plugin 才能变成 SourceLocator。

## 12. MVP 后置项

首批 Core MVP 不要求：

- 完整 Native Site Panel；
- 站点多账号；
- Gateway 用户/RBAC；
- 多显示端同步；
- 运行时插件市场。
