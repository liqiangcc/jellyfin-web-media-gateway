# 安全设计

## 1. MVP 信任模型

首个 MVP 面向可信家庭 LAN / 单用户。

明确不实现：

- Gateway 用户账号；
- RBAC；
- 家庭成员权限；
- `/control` 登录页；
- 公网开放服务。

仍必须满足：

- 默认不直接暴露公网；
- Origin/CSRF/Host/Content-Type/大小校验；
- SSRF 与开放代理防护；
- Display/媒体能力使用不可预测短期 token；
- 来源网站 Secret 只保存在服务器安全边界。

一旦部署环境变为不可信网络，必须重新设计 Gateway Identity/Authorization，不能复用 `SiteAccount`。

## 2. 保护目标

- SiteAccount / SiteSession 中的 Cookie、Token、localStorage、profile。
- `vault/` 中的持久会话材料。
- PlaybackSession / PlaybackItem / DisplayInstance。
- 临时媒体签名和上游访问 capability。
- Jellyfin API Key。
- Ubuntu 手机、内网和外接 SSD。
- Site Plugin 与 Site Browser Worker 不得越过各自 scope。
- Ubuntu ARM64 self-hosted Target Runner 不得成为读取生产 Secret 或任意控制宿主的后门。

## 3. Session Vault

推荐：

```text
/var/lib/web-media-gateway/
├── gateway.sqlite
├── vault/
│   ├── accounts/
│   └── browser-profiles/
└── runtime/
```

规则：

- `vault/` 是 Site Session/Profile 唯一安全所有者。
- Control、Display、Site Plugin 不允许直接读取 Vault 文件。
- 结构化 Secret（Cookie/token/export）必须静态加密。
- browser profile 使用最小权限，禁止通过 Web/API 下载。
- Chromium 运行需要 profile 时由 Vault 受限 materialize/attach；临时副本进入 `runtime/` 并在 worker 退出后清理。
- 不把“所有 profile 都是单个加密 blob”作为实现前提；可以使用受限目录/加密文件系统等部署手段。

## 4. Scoped Site Access

Site Plugin 不获得原始 Vault 访问权。

```text
SiteAccessCapability
├── site_id
├── account_ref?
├── allowed_hosts
├── expiry
└── capability_id
```

插件发需要登录态的上游请求时，通过 `ScopedSiteHttpClient` 或等价受控能力；基础设施负责注入 Cookie/Authorization。

必须保证：

- Bilibili plugin 不能读取 YouTube session。
- capability 过期后不能继续访问。
- redirect 后重新检查 host/scope。
- Secret 不写入插件日志/错误/Control。

未来进程插件通过 capability IPC，也不得直接传完整 Cookie jar。

## 5. Central EgressPolicy

所有站点网络访问由 Core 中央策略控制。

### `public_web`

禁止：

- loopback；
- private；
- link-local；
- cloud metadata；
- multicast/unspecified/reserved。

DNS 解析、连接前和每次 redirect 都重新校验。

### `configured_local_service`

只用于 Gateway 明确配置的内部集成，例如 Jellyfin。

- 地址来自管理员/部署配置，不来自任意用户 URL。
- 不允许 Site Plugin 自己声明私网例外。

Site Browser Worker 默认使用 public web 或更窄的站点 allowlist。

匿名 R008 broker 的请求与响应 Secret 权限必须分开处理：

- Caller、Worker、Extractor 提供的 URL userinfo、Cookie、Authorization、proxy-auth、API-token 以及 Basic/Bearer 请求材料，在产生受禁止的网络副作用前拒绝；
- Origin 响应中的 `Set-Cookie`、认证挑战头和 Basic/Bearer/API-token 值仍由共享 Secret classifier 标记为 Secret，但不因此拒绝整个安全的公共响应；
- Secret 响应头在构造 `BrokerResponse`、跨 broker IPC 之前被过滤/包含。状态、受限 body 和非 Secret 公共响应头可以在其他 R008 校验通过时继续；
- 包含的响应头不会写入 Cookie/auth store，不会被匿名请求重放，也不会进入 Worker、SiteAdapter、`ResolvedMedia`、日志或 artifacts；响应头总量、名称和值的既有上限仍适用于被过滤的头，无法安全包含时 fail closed。

这不是给匿名 generic-ytdlp 增加凭据能力；生产 `GenericYtdlpAdapter::default()` 仍保持 disabled。R008 的 DNS、public-IP、address pinning、TLS、逐跳 redirect、body、frame、timeout 和 cancellation authority 不因响应 Secret containment 改变。

## 6. Site Plugin Threat Model

Site Plugin 属于高变化、高风险边缘代码。

安全不变量：

- 不读取其他站点 Session。
- 不直接读取 Vault。
- 不自行控制 `active_display`。
- 不绕过 Media Gateway 给 Display 下发上游 Secret。
- 不绕过 EgressPolicy。
- 不能通过错误输出泄露 Cookie、Token、完整敏感 URL。
- timeout/cancel 后必须停止继续操作。
- 插件输出 `ResolvedMedia` / `SourceLocator` 必须经过 Core schema validation。

MVP 编译期插件仍需要 contract test；未来进程插件再增加：

- crash isolation；
- resource limit；
- protocol version；
- health check；
- restart/circuit breaker。

## 7. Site Browser Worker Threat Model

Site Browser Worker 是通用 Chromium runtime，不理解站点业务。

必须：

- 使用非 root worker；
- 按站点隔离 profile；
- 限制 CPU/内存/进程数/执行时间；
- 远程画面/输入端口不直接暴露公网；
- 使用短期一次性控制 token；
- 不允许 Control 下载 profile/Cookie DB；
- 禁止浏览器打开服务器本地文件或任意内网地址；
- 登录输入、二维码、远程帧不日志、不录屏。

Browser Worker 只提供通用 browser event；具体站点 DOM/API 解释在 Site Plugin 中。

## 8. Site Account / 登录

Gateway 不保存网站密码。

允许持久化：

- Cookie；
- localStorage；
- 必要 Token；
- profile；
- 脱敏账号元数据。

禁止持久化/记录：

- 密码；
- 验证码输入；
- 登录表单内容；
- 二维码画面。

重新登录：

```text
保留旧 SiteSession
→ 创建新临时会话
→ Site Plugin 验证新会话
→ 原子替换 active session
→ 清理旧会话
```

失败/取消时尽可能保留旧会话。

## 9. ResolvedMedia / Media Gateway Secret Boundary

`ResolvedMedia.public_headers` 不允许包含 Cookie/Authorization/bearer token。

敏感认证使用：

```text
upstream_access_ref
```

由 Media Gateway 在上游请求阶段通过 scoped capability 注入。

临时媒体 URL 必须绑定：

- PlaybackSession；
- PlaybackItem；
- resource；
- HTTP method；
- expiry。

禁止提供：

- 任意目标代理；
- 任意 Header；
- CONNECT；
- 无限期 token。

日志/Referer/错误页不得记录完整签名 URL。

## 10. Display Security

- Display ID 由服务器生成、不可预测。
- `/control` 仅打开不自动注册 Display。
- `/display` 访问本身不能获得历史任务媒体能力。
- WebSocket 校验 Origin。
- Display callback 绑定 session/item/display_generation。
- 旧 generation 不得改变当前 active display 状态。
- Display Adapter 不读取 Vault。
- Jellyfin API Key 只由 JellyfinDisplayAdapter 使用。

## 11. Command / Replay Safety

所有 Playback command 携带 `request_id`，可选 `expected_session_revision`。

- 重复 `request_id` 不应重复执行破坏性命令。
- revision 冲突返回 `REVISION_CONFLICT`。
- 旧 item revision / display generation 的异步事件丢弃或仅记录诊断。

## 12. Command Injection / Process Isolation

- yt-dlp、FFmpeg、Chromium 子进程全部使用参数数组，禁止 shell 字符串拼接。
- 输入 URL、标题、字幕语言、文件名不能成为可执行参数前缀。
- Resolver/插件/FFmpeg/Browser Worker 使用最小权限。
- 临时目录独立，完成后清理。
- 不暴露 Android Root、ADB socket 或整个宿主文件系统。

## 13. DRM / 合规

- DRM 返回明确 `DRM_UNSUPPORTED`。
- 不提供 DRM 解密、授权绕过或受保护画面捕获。
- Native Site Panel 远程画面只用于 Control 操作，不作为绕过 DRM 的媒体播放路径。
- 用户必须有权访问和播放目标内容。

## 14. Web Secure Context

基本播放必须支持可信 LAN HTTP；因此：

- Service Worker / installable PWA 不属于 Core 成功前提。
- Screen Wake Lock 不属于 Core 成功前提。
- Fullscreen 被拒绝时仍保持 viewport 沉浸播放。

提供 LAN HTTPS 时再启用 secure-context 增强能力。

长期推荐 HTTPS，但首个 media-path PoC 不因证书体系阻塞。

## 15. 日志与隐私

允许记录：

- site/plugin/adapter id；
- 不可逆任务/account id；
- session/item revision；
- command 类型；
- handoff 阶段；
- worker 生命周期；
- 稳定错误码。

禁止记录：

- Cookie；
- Authorization；
- API Key；
- 临时媒体签名；
- 网站密码/验证码；
- browser profile 内容；
- 远程登录画面；
- 完整敏感 URL query。

## 16. GitHub Actions / Ubuntu ARM64 Target Runner Security

GitHub Actions 是自动执行平面。GitHub-hosted Runner 用于通用 x64/ARM64 验证；项目只计划在目标 Ubuntu 手机部署 self-hosted Runner。

详细执行架构见 `runner-execution-architecture.md`。

### 16.1 Target Runner 最小权限

Ubuntu ARM64 self-hosted Runner 必须：

- 使用专用低权限用户；
- 默认无 root / sudo；
- 不使用 Gateway 正式服务账号；
- Runner work directory 与 `/var/lib/web-media-gateway/` 分离；
- 不持有 Tailscale auth key、SSH 私钥、长期 GitHub PAT、站点 Cookie/profile 等长期 Secret；
- job 完成后清理临时 workspace / runtime data；
- 高 CPU/FFmpeg/Chromium/长跑 job 必须有 timeout 和资源约束。

### 16.2 Target Runner 禁止默认读取生产 Secret

默认不得读取：

```text
/var/lib/web-media-gateway/vault/
真实 browser profile
来源站点 Cookie/token
Jellyfin API Key
宿主 root credential
ADB privileged socket
```

如果某项 Verification 确实需要受控 Secret，必须在 Task Contract 中显式定义 scope、注入方式、生命周期和清理方式；不得因为 Runner 与 Gateway 在同一台机器上就继承生产权限。

### 16.3 不可信变更不能自动获得 Target Shell

Ubuntu ARM64 Target Runner 不允许任意 PR/分支自动执行不可信代码。

至少遵守：

- target job 只验证明确 candidate SHA；
- target workflow 与普通 PR CI 分离；
- 未受信 PR/fork 不直接命中 target runner；
- 必要时使用 manual dispatch / approval gate；
- issue title、branch、PR body、URL 等不可信输入不得直接拼接 shell；
- workflow/script 使用最小 GitHub token 权限。

原则：

> PR 可以请求 target verification，但不能自动继承目标设备 shell authority。

### 16.4 Runner 与生产实例隔离

```text
Runner workspace
!=
Gateway vault/runtime
```

验证优先启动独立 test instance / test ports，不直接覆盖用户正在使用的正式实例。

只有明确 deployment verification Task 才允许 stop/start 正式服务，并必须在 Scope、Evidence 和恢复步骤中写清。

### 16.5 Cloud / Tailscale 是外部管理路径，不是 Runner 路径

Cloud 不部署 Runner。

Cloud/Windows/其他外部 Worker 经 Tailscale 访问目标设备时：

- 只允许当前 Task Scope 所需目标和端口；
- 不因为接入 Tailnet 就获得家庭 LAN 任意扫描/访问权限；
- Evidence 记录真实 Execution host / Target；
- Cloud host 结果不能冒充手机温度、目标 LAN 或真实电视。

## 17. 安全测试最低集

1. private/loopback/metadata URL 拒绝。
2. public URL redirect 到 private 被拒绝。
3. configured Jellyfin local service 只允许配置目标。
4. plugin cross-site session access 被拒绝。
5. plugin timeout/cancel。
6. ResolvedMedia Secret header schema rejection。
7. Media token 跨 session/item 重放失败。
8. 旧 display generation callback 不生效。
9. 重新登录失败保留旧会话。
10. Browser Worker 不能下载 profile/访问本地文件。
11. Target Runner 默认不能读取 Vault/profile/长期 Secret。
12. 未受信 PR/分支不能直接调度 Target Runner。
