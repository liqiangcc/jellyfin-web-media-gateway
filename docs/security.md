# 安全设计

## 1. MVP 信任模型

首个 MVP 面向可信家庭 LAN / 单用户使用。

当前阶段明确不实现：

- Gateway 用户账号；
- RBAC；
- 家庭成员权限体系；
- `/control` 登录页；
- 管理员二次认证。

这不代表 Gateway 可以无边界暴露。MVP 仍要求：

- 默认不直接暴露公网；
- same-origin、Origin/CSRF 防护；
- SSRF 与开放代理防护；
- Display 注册和媒体能力使用不可预测的短期 token / session id；
- 来源站点 Cookie、Token、profile 只保存在服务器。

未来如果需要多用户或不可信网络访问，Gateway Identity 必须独立设计，不能复用 `SiteAccount` 作为 Gateway 用户身份。

## 2. 保护目标

- 来源网站 Cookie、账号会话和必要 Token。
- Session Vault 中的 Chromium profile、localStorage 与站点会话材料。
- 播放任务、Display Instance 和临时媒体能力。
- Jellyfin Adapter 启用时使用的 API Key 与设备控制权限。
- 手机内网访问能力及外接 SSD 数据。
- 防止网关成为 SSRF 工具、开放代理或未授权媒体中继。
- 防止显示端冒充、Token 重放和跨任务媒体 URL 复用。

## 3. 主要威胁

### SSRF 与 DNS Rebinding

攻击者可能提交指向 localhost、局域网设备、Android 调试端口、云元数据或重绑定域名的 URL。

控制：

- 默认只允许 HTTPS，站点适配器显式例外。
- URL 解析、DNS 解析、连接前和每次 redirect 都校验目标。
- 禁止 loopback、private、link-local、multicast、unspecified 和保留地址。
- 连接固定到已验证地址，并校验证书主机名。

### SiteSession / Cookie 泄露

控制：

- Cookie 仅注入到被授权站点及允许的子域。
- redirect 后重新执行站点授权判断。
- 不把 Cookie 写入 M3U、浏览器媒体 URL、Jellyfin URL、错误文本或遥测。
- 会话按站点隔离并静态加密。
- Display Adapter 不允许直接读取 Session Vault。
- Control 的 `/control/sites` 只能显示脱敏账号元数据，不返回 Cookie、localStorage、Token 或 profile 文件。

### 网站密码与交互登录数据泄露

Gateway 不设计为密码管理器。

控制：

- 不持久化网站密码。
- 登录表单内容、验证码、键盘输入、二维码画面和远程帧不写日志、不录屏。
- Auth Browser Worker 仅在需要时启动，完成/超时后关闭。
- 持久化的是网站完成认证后的必要会话材料，而不是用户输入的密码。

### 站点重新登录误删旧会话

如果“重新登录”一开始就覆盖或删除旧会话，新登录失败会造成不必要的账号失效。

控制：

```text
保留旧 SiteSession
→ 创建新临时登录会话
→ 验证新会话
→ 成功后原子替换 active session
→ 清理旧会话
```

新登录失败或取消时，尽可能保持旧会话。

“退出登录”与“重新登录”语义分开；退出登录属于显式破坏性操作，需要确认。

### SiteAccount 与 Gateway Identity 混淆

MVP 没有 Gateway 身份体系。来源网站账号不能被用来判断谁有权控制 Gateway。

控制：

- `SiteAccount` 只代表来源网站会话。
- Bilibili / YouTube 等账号元数据不产生 Gateway 权限。
- PageRole、DisplayProfile、User-Agent、站点账号标签也都不是身份凭据。
- 未来 Gateway Identity 必须使用独立模型和凭据。

### 临时媒体能力泄露

浏览器和 Jellyfin 都需要访问 Gateway 暴露的媒体入口，但这些入口不能退化为长期 bearer URL 或开放代理。

控制：

- `/stream` 使用短期签名 Token，绑定 `PlaybackSession`、允许方法和资源类型。
- Token 设置明确过期时间；刷新只能由仍有效的播放任务触发。
- 不允许通过一个任务的 Token 请求其他任务、其他源站或任意 Header。
- 日志、Referer、错误页和前端遥测不得记录完整签名 URL。
- 对异常重放、并发拉流和来源切换记录不含 Secret 的审计事件。

### Web Display 冒充与控制劫持

MVP 是可信 LAN，但仍不能让任意页面自报一个 display id 后获取其他任务媒体。

控制：

- Display Instance ID 由服务器生成且不可预测。
- display 注册与后续 WebSocket/媒体能力绑定到同一浏览器会话或短期 display token。
- `/control` 页面仅打开时不自动注册 Display。
- `/display` 访问本身不直接得到任意历史媒体 Token；媒体能力只针对当前任务签发。
- WebSocket 校验 Origin。
- display 断线后进入 grace period，超过后标记离线。

### 入口角色混淆

`/` 提供 Display / Control 选择并在超时后默认进入 TV Display。这是 UX 路由，不是安全认证。

控制：

- `preferred_role`、`DisplayProfile`、viewport、User-Agent 和 localStorage 均视为不可信 UI 输入。
- 根入口自动跳转只能指向 Gateway 自身固定路径。
- Control 与 Display 使用不同前端职责，Control 不因为切换 role 就获得 Session Vault 原始内容。

### Display Handoff 竞态

并发 handoff、重复请求或旧 adapter 延迟回报可能导致两个显示端同时认为自己是活动端。

控制：

- `active_display` 只由 Gateway 的 Playback Coordinator 提交。
- 每次 handoff 使用单调递增 generation / revision 或等价 CAS。
- adapter 回报携带 session 与 revision；旧 revision 不得覆盖新状态。
- B 未确认可播放前不得静默停止 A。

### 交互登录通道

服务端浏览器持有真实来源网站账号上下文，远程画面与输入通道属于敏感管理通道，即使 MVP 不实现 Gateway 登录也必须限制暴露范围。

控制：

- 只从可信 LAN 的 Control 流程启动，不直接暴露远程浏览器端口。
- 每次登录使用短期、一次性连接 token。
- 校验 Origin；远程通道不得直接暴露公网。
- 每个站点使用独立非 root worker、profile 目录和网络允许列表。
- 禁止控制设备下载 profile、读取 Cookie 数据库、打开服务器本地文件或访问任意内网地址。
- 进程退出后清理临时显示、共享内存与一次性 token；持久 profile 进入 Session Vault 加密保存。

### 开放代理与盗链

控制：

- `/stream` 不提供任意目标、任意 Header 或通用 CONNECT。
- 限制并发、字节数、持续时间和目标主机。
- 所有 adapter 必须使用 Gateway 生成的任务绑定媒体 URL。
- 如果 Jellyfin Adapter 需要较长拉流时间，使用可刷新能力而不是无限期 Token。

### Jellyfin Adapter 权限扩大

Jellyfin 是可选 adapter。

控制：

- 使用专用、最小权限 Jellyfin 用户或 API Key。
- Jellyfin Key 只由 `JellyfinDisplayAdapter` 读取，不暴露给浏览器和 Resolver。
- Jellyfin Adapter 被禁用或故障时，相关凭据不可被其他 adapter 接管使用。
- Kodi/ADB 等额外控制接口不属于 MVP，默认关闭。

### 命令注入

yt-dlp 和 FFmpeg 必须使用参数数组调用，禁止拼接 Shell 命令。输入 URL、文件名、字幕语言和标题不得成为可执行参数前缀。

## 4. DRM 与合规边界

- 检测到 DRM 时返回 `DRM_UNSUPPORTED`。
- 不提供 DRM 解密、授权绕过或受保护画面捕获。
- 用户必须有权访问和播放目标内容。
- 站点适配器应遵守适用条款、请求频率和版权要求。
- Web Display 与 Jellyfin Display 的存在不改变上述边界。

## 5. 网络边界

- 默认只监听 loopback，由受信任反向代理暴露到 LAN。
- 不直接暴露公网。
- 远程管理如未来需要，可使用 Tailscale 并重新评审 Gateway Identity 需求。
- 本地媒体流使用 LAN。
- Host、Origin、Content-Type 和请求大小全部校验。
- Adapter 对外连接不得绕过核心 SSRF 策略。
- `/` 的自动角色跳转只能指向 same-origin 固定路径。

## 6. 日志与隐私

允许记录：规范化站点 host、不可逆任务 ID、adapter 类型、display 匿名 ID、handoff revision、站点账号状态变化和稳定错误码。

禁止记录或必须清除：

- Cookie
- Authorization
- API Key
- 临时媒体签名
- 网站密码
- 验证码输入
- 登录表单内容
- 完整浏览器 profile
- 完整字幕
- 远程登录画面

账号标签若写入日志必须脱敏或使用内部不可逆 account id。

## 7. 子进程隔离

- Resolver 与 FFmpeg 以非 root 用户运行。
- 限制 CPU、内存、进程数、打开文件数和执行时间。
- 临时目录独立，完成后删除。
- 不把 Android Root、ADB socket 或整个 Ubuntu 文件系统暴露给解析 worker。
- Auth Browser Worker、Resolver、媒体处理与 Display Adapter 尽量使用不同权限边界。

## 8. 安全不变量

1. SiteAccount 不等于 Gateway 用户身份。
2. Gateway 不保存来源网站密码。
3. Display Adapter 不读取 Session Vault 原始凭据。
4. Display Adapter 不自行决定全局 `active_display`。
5. 媒体 Token 不允许变成任意目标代理能力。
6. 重新登录成功前不无必要地销毁旧 SiteSession。
7. 退出登录明确清理对应站点会话材料。
8. PageRole、DisplayProfile、分辨率和站点账号标签都不是可信身份信号。
9. Gateway 不直接暴露公网是当前 MVP 信任模型的一部分；一旦改变该条件必须重新设计认证边界。
