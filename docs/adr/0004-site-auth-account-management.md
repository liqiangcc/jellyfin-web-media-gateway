# ADR-0004：站点认证按需触发，并提供独立账号管理

- 状态：已接受（设计阶段）
- 日期：2026-08-24

## 背景

Gateway 需要处理两类“登录”概念：

1. Gateway 自身的用户/权限认证；
2. 来源网站的登录会话，例如 Bilibili、YouTube 或其他站点的 Cookie、localStorage 和 Token。

当前 MVP 的使用环境是可信 LAN、单用户，不计划先实现完整 Gateway 用户账号、RBAC 或家庭权限体系。因此当前要解决的核心问题是来源网站认证，以及它如何和播放流程自然结合。

如果要求用户在播放前先进入“账号设置”逐个登录，会增加不必要步骤；如果只允许在粘贴 URL 后临时登录，又缺少主动检查、重新登录、退出和会话维护能力。

## 决策

### 1. MVP 暂不实现 Gateway 身份体系

首个 MVP 假设可信 LAN / 单用户控制场景。

不实现：

- Gateway 用户账号；
- RBAC；
- 家庭成员权限；
- `/control` 登录页；
- 管理员二次认证。

仍保留基础网络和 Web 安全约束，例如只在受信任网络开放、same-origin、CSRF/Origin 校验、短期媒体 Token、SSRF 防护等。

未来如果开放到不可信网络或引入多用户，再单独设计 Gateway Identity，不与 SiteAccount 混用。

### 2. Site Auth 按站点隔离

每个来源站点拥有独立会话边界：

```text
SiteAccount
→ SiteSession
→ Session Vault
```

Bilibili 的会话失效不得影响其他站点。

### 3. 正常播放采用按需登录

Resolver 首先使用已有 SiteSession 尝试解析。

```text
输入 URL
→ resolve with current SiteSession
→ success：继续播放
→ SITE_AUTH_REQUIRED：提示“登录该站点后继续”
```

不能仅因为 URL 属于某站点就强制登录，因为同一站点可能同时存在公开内容和受限内容。

### 4. 登录完成后恢复原始用户意图

播放过程中触发登录时，Gateway 保存 `PendingIntent` 或等价上下文，包括：

- 原始 source locator / URL；
- 目标 display；
- 原播放动作；
- 连续播放上下文。

登录成功后自动 retry，用户不需要重新粘贴 URL、重新选显示端或重新点击播放。

### 5. 同时提供主动站点账号管理

Control 提供独立入口：

```text
/control/sites
```

用户可以查看：

- 站点；
- 登录状态；
- 脱敏账号标签；
- 最近验证时间；
- 失效原因。

并执行：

- 登录；
- 重新登录；
- 退出登录 / 清除会话。

账号管理是辅助能力，不是播放前置步骤。

### 6. 不保存网站密码

Gateway 只保留网站登录完成后产生的必要会话材料：

- Cookie；
- localStorage；
- 必要 Token；
- Chromium profile；
- 不敏感账号元数据。

账号密码、验证码输入、扫码画面不写入持久化存储或日志。

### 7. 重新登录采用验证后替换

重新登录时优先保留旧 SiteSession：

```text
旧会话继续保留
→ 新临时登录会话
→ 验证新会话成功
→ 原子替换活动会话
→ 清理旧会话
```

如果新登录失败或取消，不应无必要地破坏旧会话。

### 8. 退出登录是显式破坏性操作

退出登录会删除或失效服务器端保存的站点会话材料，并将账号状态切换为 `login_required`。

Control 必须要求明确确认。

### 9. MVP 每站点一个活动账号，但模型保留多账号扩展能力

MVP UI 和业务约束：

```text
site_id → 0..1 active SiteAccount
```

但数据模型保留独立 `SiteAccount.id`，不得使用 `bilibili_cookie` 一类站点专用单例字段。

未来需要多个 Bilibili 账号时，可以扩展：

```text
site_id → N SiteAccount
→ 选择 active account
```

而无需重构 Session Vault。

### 10. 站点状态不简化为布尔值

至少允许：

```text
unknown
checking
valid
expired
login_required
error
```

UI 再翻译为“已登录”“需要重新登录”“正在验证”等友好文案。

## 结果

优点：

- 普通播放路径不被账号管理打断。
- 真正需要登录时才要求用户操作。
- 登录成功后自动恢复播放意图。
- 用户仍然可以主动查看和维护服务器上的站点账号。
- 不需要 Gateway 保存网站密码。
- 站点会话彼此隔离。
- MVP 保持单用户简单性，同时给未来多账号留下稳定模型。

代价：

- Gateway 需要保存 PendingIntent 并处理登录后的自动 retry。
- SiteAccount 与 SiteSession 需要明确状态机。
- 重新登录需要临时会话和原子替换逻辑。
- 账号状态验证可能受站点接口变化影响。

## 被拒绝方案

### 播放前强制进入站点账号管理

步骤过多，而且公开视频也会被无意义地要求登录。

### 只在 URL 解析失败时临时登录，不提供管理页面

短期实现简单，但用户无法主动检查、重新登录或清理服务器上的站点会话。

### Gateway 保存网站账号密码

扩大敏感数据范围，没有必要。服务端浏览器登录 + Session Vault 已能满足需求。

### MVP 直接实现站点多账号

增加选择、默认账号和会话切换复杂度，当前没有必要；只预留数据模型。

### 把 Gateway 用户认证和 Site Auth 合并

两者安全域、生命周期和目的不同，会导致模型耦合。Gateway Identity 如果以后需要，应独立设计。
