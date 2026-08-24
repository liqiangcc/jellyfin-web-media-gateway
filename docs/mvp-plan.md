# MVP 实施计划

本计划只描述实施顺序。核心边界以 `architecture.md` 和 `implementation-contracts.md` 为准。

## 0. 开工前 Gate：Contract Freeze

在写业务代码前先落下最小类型/测试骨架：

- `SourceLocator`
- `SiteAdapter` / `SiteAdapterRegistry`
- `ResolvedMedia`
- `PlaybackItem` / `PlaybackSession`
- `DisplayAdapter`
- command envelope + revision
- `SiteAccessCapability`
- `EgressPolicy`

退出条件：

1. Core 只通过 Registry 访问 Site Adapter。
2. `generic-direct` / `generic-ytdlp` 作为插件骨架存在。
3. 至少有一组 conformance test 能验证一个 fake SiteAdapter。
4. 没有 Core 直接调用 yt-dlp 的代码路径。

## Phase 0A-1：Media Path Proof

目标：只证明最底层链路成立，不先做完整 Control UX。

```text
Test Source
→ SiteAdapterRegistry
→ SourceLocator
→ SiteAdapter.resolve
→ ResolvedMedia
→ Media Gateway
→ Web Display
```

实施：

- 建立 Rust workspace：`gateway-core`、`site-adapter-api`、`plugins/generic-direct`，可选 `plugins/generic-ytdlp`。
- 使用公开合法 HLS/MP4 测试源。
- 实现最小 `ResolvedMedia` schema。
- 实现短期媒体 URL / proxy。
- Web Display 只需要最小 `<video>`/HLS 播放能力。
- 验证上游 Secret 不出现在浏览器请求中。

退出条件：

- 至少一种公开 HLS/MP4 能稳定播放。
- Core 没有站点 special case。
- Display 不需要 Jellyfin。

## Phase 0A-2：PlaybackSession + Control Shell

目标：证明 Gateway 自己持有播放状态并能被 Control 操作。

实施：

- `PlaybackSession` / `PlaybackItem` / `session_revision` / `item_revision`。
- `/control` 与 `/display`。
- Web Display 注册与 `display_generation`。
- `POST /api/v1/sessions/{id}/commands`。
- play / pause / seek / stop。
- Control 基础状态：Idle → Resolving → Playing。
- WebSocket/事件恢复。
- 对旧 item revision / display generation 做拒绝测试。

退出条件：

- Control 能创建任务并遥控 Web Display。
- 刷新 Control 后能从 Gateway snapshot 恢复正在播放状态。
- 旧 callback 不会覆盖新 item/display 状态。

## Phase 0A-3：入口、字幕与 Display UX

目标：完成第一个真正可用的 Web-only Core MVP。

实施：

- `/` 智能入口，默认 5 秒进入 TV Display。
- TV profile：viewport 沉浸布局、字幕、遥控器焦点。
- 720p / 1080p / 4K viewport 测试。
- Fullscreen allow/deny degradation。
- HTTP LAN 基线测试；Wake Lock / Service Worker 仅在 HTTPS secure context 中做增强测试。
- 至少一个外挂字幕轨道。

退出条件：

- 普通 LAN HTTP 下可从 Control 到 TV-style Web Display 完成端到端播放。
- secure-context API 不可用不会导致播放失败。

## Phase 0B：Jellyfin Display Adapter PoC（并行）

目标：验证 Jellyfin 是可行外部 Display Adapter，但不阻塞 Core。

- ARM64 Jellyfin Server。
- 动态 M3U / 媒体代理。
- Android TV Direct Play/Remux。
- 设备发现、远程播放、暂停、恢复。
- position/handoff 精度记录。

退出条件：

- Jellyfin 官方客户端能播放 Gateway 媒体入口。
- 失败不会改变 Web-only Core MVP 路线。

## Phase 1：Plugin Boundary Hardening

目标：确保站点扩展不会重新污染 Core。

实施：

- `generic-ytdlp` 作为 fallback plugin，不允许 Core fallback。
- SiteAdapter conformance tests。
- Registry 冲突/priority/health。
- `SourceLocator` 版本/旧版本错误处理。
- `SiteAccessCapability` / `ScopedSiteHttpClient`。
- `ResolvedMedia.upstream_access_ref`。
- architecture test：扫描 Core concrete site knowledge。
- EgressPolicy public_web / configured_local_service。

退出条件：

- 新增一个 fake/new site plugin 不需要修改 PlaybackCoordinator/DisplayAdapter/Control 的站点分支。
- 插件不能直接读取 Vault 或绕过 EgressPolicy。

## Phase 2：Handoff 与连续内容

- Web→Web handoff。
- 接入可用的 `JellyfinDisplayAdapter`。
- prepare/confirm/commit + generation。
- `PlaybackContext` previous/next/queue。
- `NextItem` / `PreviousItem`。
- `autoplay = off | next`。
- 下一集 resolve 期间保持当前 Display ownership。
- 多控制端 revision conflict 测试。

退出条件：

- 同一 item 可跨 Display 双向 handoff。
- 同一 Session 可切换下一 item，旧 item callback 不覆盖新 item。

## Phase 3：Site Auth / Account Management

- `SiteAccount` / `SiteSessionRef`。
- Session Vault `vault/accounts` / `vault/browser-profiles`。
- Site Browser Worker Auth Mode。
- Browser Worker 只产生通用 browser event；具体登录成功判断由 Site Plugin 完成。
- `SITE_AUTH_REQUIRED` + PendingIntent。
- `/control/sites`。
- 登录 / 重新登录（验证后替换）/ 退出。
- 密码、验证码、二维码不落盘/不记录。
- 下一集认证恢复。

退出条件：

- 一项需要服务器会话的合法内容可以登录后自动继续原播放意图。
- 重新登录失败不无必要破坏旧会话。

## Phase 3B：Native Site Panel（非 Core 阻塞）

- Site Browser Worker Native Control Mode。
- Control 中可折叠 Native Site Panel。
- browser event → Site Plugin.browser_interpret → SourceLocator。
- 选集/选择视频 → Playback Item Transition。
- Native panel crash isolation。
- 评估远程画面协议与 ARM64 资源成本。

退出条件：

- Site Browser Worker 崩溃时已解析媒体仍播放，Universal Remote 仍可用。
- Browser Worker 本身没有具体站点 DOM/API 逻辑。

## Phase 4：稳定性与低功耗

- system service / health check。
- CPU、内存、温度、并发和资源上限。
- plugin/adapter timeout 与 circuit breaker。
- URL 过期刷新。
- 空闲状态无高频轮询/持续 Chromium。
- ARM64 长时间运行。
- Vault/runtime 清理和崩溃恢复。

## 运行时进程插件：后续触发条件

不属于 MVP。满足以下任一真实条件后再设计 IPC：

- 站点插件独立更新频率显著高于 Gateway；
- Playwright/Node/Python 依赖需要隔离；
- 插件崩溃影响 Core 稳定性；
- 需要独立 CPU/内存/网络沙箱。

优先进程插件，不优先 Rust `.so`。

## 自动化测试主矩阵

| 领域 | 最低测试 |
|---|---|
| Site Contract | recognize、resolve、navigation、error、secret boundary |
| SourceLocator | opaque、version mismatch、retry |
| Playback | session/item revision、NextItem、stale callback |
| Display | registration、generation、handoff rollback |
| Control | Idle/Resolving/Playing、reconnect、revision conflict |
| Media | HLS/MP4、subtitle、URL expiry |
| Egress | private IP、redirect、configured Jellyfin local service |
| Vault | cross-site denial、relogin replace、runtime cleanup |
| Web | HTTP baseline、Fullscreen degradation、HTTPS enhancement |
| Failure | plugin timeout、display disconnect、Jellyfin down、browser worker crash |

## 首个可检查里程碑

第一批代码提交只要求：

1. workspace / crate 边界；
2. `implementation-contracts.md` 对应的最小 Rust 类型；
3. fake/generic SiteAdapter；
4. media path proof；
5. 最小 Web Display；
6. Contract + architecture tests。

不要在第一批实现中同时加入 Bilibili 登录、Native Site Panel、Jellyfin 完整 handoff 和插件 IPC。
