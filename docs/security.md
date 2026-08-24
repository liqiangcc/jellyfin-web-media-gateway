# 安全设计

## 1. 保护目标

- 网站 Cookie、账号会话和必要 Token。
- Gateway 管理会话、播放任务和 Display Adapter 控制权限。
- Jellyfin Adapter 启用时使用的 API Key 与设备控制权限。
- 手机内网访问能力及外接 SSD 数据。
- 防止网关成为 SSRF 工具、开放代理或未授权媒体中继。
- 防止显示端冒充、Token 重放和跨任务媒体 URL 复用。

## 2. 主要威胁

### SSRF 与 DNS Rebinding

攻击者可能提交指向 localhost、局域网设备、Android 调试端口、云元数据或重绑定域名的 URL。

控制：

- 仅允许 HTTPS，站点适配器显式例外。
- URL 解析、DNS 解析、连接前和每次 redirect 都校验目标。
- 禁止 loopback、private、link-local、multicast、unspecified 和保留地址。
- 连接固定到已验证地址，并校验证书主机名。

### Cookie 泄露

控制：

- Cookie 仅注入到被授权站点及允许的子域。
- redirect 后重新执行站点授权判断。
- 不把 Cookie 写入 M3U、浏览器媒体 URL、Jellyfin URL、错误文本或遥测。
- 会话按站点隔离并静态加密。
- Display Adapter 不允许直接读取 Session Vault。

### 临时媒体能力泄露

浏览器和 Jellyfin 都需要访问 Gateway 暴露的媒体入口，但这些入口不能退化为长期 bearer URL 或开放代理。

控制：

- `/stream` 使用短期签名 Token，绑定用户、`PlaybackSession`、允许方法和资源类型。
- Token 设置明确过期时间；刷新只能由仍有效且已授权的播放任务触发。
- 不允许通过一个任务的 Token 请求其他任务、其他源站或任意 Header。
- 日志、Referer、错误页和前端遥测不得记录完整签名 URL。
- 对异常重放、并发拉流和来源切换记录不含 Secret 的审计事件。

### Web Display 冒充与控制劫持

浏览器既可能是控制端，也可能注册为显示端。攻击者如果伪造 Display Instance，可能接收媒体能力或抢占 `active_display`。

控制：

- Web Display 注册必须绑定已认证 Gateway 会话。
- Display Instance 使用服务器生成的不可预测 ID，不信任客户端自报设备身份。
- 注册、状态上报、播放控制和 handoff 都校验用户、会话与 display 归属。
- WebSocket 校验 Origin，并对敏感写操作使用 CSRF 防护或等价的 same-origin token 机制。
- display 断线后进入短暂 grace period，超过后标记离线，不自动把其他显示端切换到它。
- 浏览器仅获得当前任务所需的短期媒体能力，不能读取站点凭据。

### Display Handoff 竞态

并发 handoff、重复请求或旧 adapter 延迟回报可能导致两个显示端同时认为自己是活动端。

控制：

- `active_display` 只由 Gateway 的 Playback Coordinator 提交。
- 每次 handoff 使用单调递增 generation / revision 或等价 compare-and-swap 机制。
- adapter 回报必须携带 session 与 handoff revision；旧 revision 不得覆盖新状态。
- B 未确认可播放前不得静默停止 A。
- handoff 超时后返回稳定错误，不允许未确认的 adapter 自动晋升为 active。

### 交互登录通道

服务端浏览器相当于持有真实网站账号，远程画面与输入通道必须视为高权限管理入口。

控制：

- 只允许已重新认证的管理员创建登录会话，并使用短期、一次性连接令牌。
- 校验 Origin，启用 CSRF 防护；远程通道不得直接暴露到公网。
- 每个站点使用独立非 root worker、profile 目录和网络允许列表。
- 禁止控制设备下载 profile、读取 Cookie 数据库、打开本地文件或访问内网地址。
- 登录表单、键盘输入、二维码画面和远程帧不写日志、不录屏。
- 进程退出后清理临时显示、共享内存与一次性令牌；持久 profile 进入 Session Vault 加密保存。

### 开放代理与盗链

控制：

- `/stream` 不提供任意目标、任意 Header 或通用 CONNECT。
- 限制并发、字节数、持续时间和目标主机。
- 所有 adapter 必须使用 Gateway 生成的任务绑定媒体 URL。
- 如果 Jellyfin Adapter 需要较长拉流时间，使用可刷新能力而不是无限期 Token。

### Jellyfin Adapter 权限扩大

Jellyfin 是可选 adapter。其 API Key 不应因为方便而获得整个系统或服务器的超额权限。

控制：

- 使用专用、最小权限 Jellyfin 用户或 API Key。
- Jellyfin Key 只由 `JellyfinDisplayAdapter` 读取，不暴露给浏览器和 Resolver。
- Jellyfin 客户端身份必须映射为 Gateway 内的授权 `DisplayInstance`。
- Jellyfin Adapter 被禁用或故障时，相关凭据不可被其他 adapter 接管使用。
- Kodi/ADB 等额外控制接口不属于 MVP，默认关闭。

### 命令注入

yt-dlp 和 FFmpeg 必须使用参数数组调用，禁止拼接 Shell 命令。输入 URL、文件名、字幕语言和标题不得成为可执行参数前缀。

## 3. DRM 与合规边界

- 检测到 DRM 时返回 `DRM_UNSUPPORTED`。
- 不提供 DRM 解密、授权绕过或受保护画面捕获。
- 用户必须有权访问和播放目标内容。
- 站点适配器应遵守适用条款、请求频率和版权要求。
- Web Display 与 Jellyfin Display 的存在不改变上述边界。

## 4. 网络边界

- 默认只监听 loopback，由受信任反向代理暴露到 LAN。
- 不直接暴露公网。
- 远程管理使用 Tailscale；本地媒体流使用 LAN。
- Host、Origin、Content-Type 和请求大小全部校验。
- Adapter 对外连接采用明确 allowlist/capability；不得让 adapter 绕过核心 SSRF 策略。

## 5. 日志与隐私

允许记录 URL 的规范化 host、不可逆任务 ID、adapter 类型、display 匿名 ID、handoff revision 和稳定错误码；默认不记录完整 path/query。

日志自动清除或禁止记录：

- Cookie
- Authorization
- API Key
- 临时媒体签名
- 登录表单内容
- 完整浏览器 profile
- 完整字幕
- 远程登录画面

## 6. 子进程隔离

- Resolver 与 FFmpeg 以非 root 用户运行。
- 限制 CPU、内存、进程数、打开文件数和执行时间。
- 临时目录独立，完成后删除。
- 不把 Android Root、ADB socket 或整个 Ubuntu 文件系统暴露给解析 worker。
- Auth Browser Worker、Resolver、媒体处理与 Display Adapter 尽量使用不同权限边界，避免一个站点解析漏洞直接获得控制其他显示端的能力。

## 7. Adapter 安全不变量

所有 Display Adapter 必须满足：

1. 不读取 Session Vault 原始凭据。
2. 不自行决定全局 `active_display`。
3. 不绕过 Gateway 的媒体 Token、SSRF 和授权策略。
4. 不把 adapter 私有凭据写入统一事件流或客户端响应。
5. 失败时返回结构化错误，不通过“临时放宽安全校验”实现兼容。
