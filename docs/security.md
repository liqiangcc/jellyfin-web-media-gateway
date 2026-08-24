# 安全设计

## 1. 保护目标

- 网站 Cookie、账号会话和必要 Token。
- Jellyfin API Key 与设备控制权限。
- 手机内网访问能力及外接 SSD 数据。
- 防止网关成为 SSRF 工具、开放代理或未授权媒体中继。

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
- 不把 Cookie 写入 M3U、客户端 URL、错误文本或遥测。
- 会话按站点隔离并静态加密。

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

- `/stream` 使用短期签名 Token，绑定任务和允许方法。
- 限制并发、字节数、持续时间和目标主机。
- 不提供任意 Header、任意代理目标或通用 CONNECT。
- Jellyfin 与控制台身份必须映射到授权用户。

### 设备劫持

控制：

- 设备播放控制要求显式授权。
- 控制台使用独立账号和 CSRF/Origin 防护。
- Jellyfin API Key 使用最小权限并定期轮换。
- Kodi/ADB 等额外控制接口不属于 MVP，默认关闭。

### 命令注入

yt-dlp 和 FFmpeg 必须使用参数数组调用，禁止拼接 Shell 命令。输入 URL、文件名、字幕语言和标题不得成为可执行参数前缀。

## 3. DRM 与合规边界

- 检测到 DRM 时返回 `DRM_UNSUPPORTED`。
- 不提供 DRM 解密、授权绕过或受保护画面捕获。
- 用户必须有权访问和播放目标内容。
- 站点适配器应遵守适用条款、请求频率和版权要求。

## 4. 网络边界

- 默认只监听 loopback，由受信任反向代理暴露到 LAN。
- 不直接暴露公网。
- 远程管理使用 Tailscale；本地媒体流使用 LAN。
- Host、Origin、Content-Type 和请求大小全部校验。

## 5. 日志与隐私

允许记录 URL 的规范化 host 和不可逆任务 ID；默认不记录完整 path/query。日志自动清除：

- Cookie
- Authorization
- API Key
- 临时媒体签名
- 登录表单内容
- 完整字幕和媒体标题（可配置）

## 6. 子进程隔离

- Resolver 与 FFmpeg 以非 root 用户运行。
- 限制 CPU、内存、进程数、打开文件数和执行时间。
- 临时目录独立，完成后删除。
- 不把 Android Root、ADB socket 或整个 Ubuntu 文件系统暴露给解析 worker。

