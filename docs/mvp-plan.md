# MVP 实施计划

本计划只描述实施顺序。核心边界以 `architecture.md` 和 `implementation-contracts.md` 为准；风险驱动技术预研、实验方法和 Go / No-Go 标准以 `technical-feasibility-validation.md` 为准。

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

## 0.5 技术可行性 Gate：Risk-Driven Research

Contract Freeze 后不直接假设完整架构已经在真实环境成立。按 `technical-feasibility-validation.md` 执行风险驱动验证。

P0 Core Feasibility：

```text
R007 Playback concurrency contract closure
        ↓
R001 Media Path
        ↓
R002 TV Browser remote audible playback / autoplay
        ↓
R003 ARM64 resource baseline
        ↓
R008 Egress / Secret baseline throughout P0
        ↓
Core Feasibility Review
```

说明：

- R001 的主要实现载体就是 Phase 0A-1。
- R002 的主要实现载体就是 Phase 0A-3，但最小浏览器 autoplay spike 可以更早独立执行。
- R003 从 Phase 0A-1 开始采集，不等到 Phase 4 才第一次测资源。
- R007 属于编码前必须闭合的 concurrency contract，不需要大型外部 PoC，但必须有竞态测试。
- R008 不是后置安全加固；所有 Phase 0 spike 从第一天就必须遵守 Egress/Secret boundary。
- R004 Jellyfin 可以并行执行，但失败不阻塞 Web-only Core。
- R005 真实站点用于验证 Site Plugin Contract，不阻塞最早的公开媒体链路。
- R006 Native Site Panel / Browser Worker 是非 Core 阻塞研究。

Web-only Core 可以被标记为技术可行之前，至少要求：

1. R001 PASS。
2. R002 PASS 或产品可接受的 CONDITIONAL PASS。
3. R003 PASS 或有明确限制的 CONDITIONAL PASS。
4. R007 的 revision / re-resolve / handoff generation 竞态契约已闭合。
5. R008 的基础安全验证通过；PoC 没有通过关闭 SSRF 或泄露 Secret 获得成功。

所有结论必须附真实实验/测试证据，不能使用“理论上应该可行”替代。

## Phase 0A-1：Media Path Proof

目标：只证明最底层链路成立，不先做完整 Control UX，同时完成 R001 的主要证据采集并开始 R003 资源基线。

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
- 验证 `/stream` 不能成为 arbitrary open proxy。
- 记录 Idle 与 Direct Proxy 的 CPU、RSS、温度和网络吞吐基础数据。
- 关键 Direct Proxy 路径逐步增加 5/30/60 分钟稳定性记录。

退出条件：

- 至少一种公开 HLS/MP4 能稳定播放；另一种必须有明确验证结果/后续处理，不允许完全未验证却默认为支持。
- pause/play/seek 以及适用的 Range/segment 行为不被 Gateway 破坏。
- Core 没有站点 special case。
- Display 不需要 Jellyfin。
- Display 看不到 Cookie/Authorization 等上游 Secret。
- 基础资源数据已记录，不存在明显持续内存增长或 Direct Proxy 立即违背低功耗目标的证据。

## Phase 0A-2：PlaybackSession + Control Shell

目标：证明 Gateway 自己持有播放状态并能被 Control 操作，同时闭合 R007 的核心并发契约。

实施：

- `PlaybackSession` / `PlaybackItem` / `session_revision` / `item_revision`。
- 明确高频 position telemetry 与 command CAS revision 的关系，避免 position callback 制造无意义 `REVISION_CONFLICT`。
- 明确同一 SourceLocator re-resolve / media refresh 时如何拒绝 stale async result。
- 明确 handoff target start 到 `active_display` commit 之间的 candidate generation / transition 语义。
- `/control` 与 `/display`。
- Web Display 注册与 `display_generation`。
- `POST /api/v1/sessions/{id}/commands`。
- play / pause / seek / stop。
- Control 基础状态：Idle → Resolving → Playing。
- WebSocket/事件恢复。
- 对旧 item revision / display generation 做拒绝测试。
- 增加高频 position + Control mutation、双 handoff、stale resolve、旧 callback 等竞态测试。

退出条件：

- Control 能创建任务并遥控 Web Display。
- 刷新 Control 后能从 Gateway snapshot 恢复正在播放状态。
- 旧 callback 不会覆盖新 item/display 状态。
- position 高频上报不会导致不可接受的 command revision conflict。
- re-resolve 的旧异步结果不能覆盖当前 `ResolvedMedia`。
- handoff transition 中 candidate callback 不会提前成为全局 authority。

## Phase 0A-3：入口、字幕与 Display UX

目标：完成第一个真正可用的 Web-only Core MVP，并完成 R002 TV Browser remote playback 验证。

实施：

- `/` 智能入口，默认 5 秒进入 TV Display。
- TV profile：viewport 沉浸布局、字幕、遥控器焦点。
- 720p / 1080p / 4K viewport 测试。
- Fullscreen allow/deny degradation。
- HTTP LAN 基线测试；Wake Lock / Service Worker 仅在 HTTPS secure context 中做增强测试。
- 至少一个外挂字幕轨道。
- 在真实目标电视浏览器验证：页面从未交互时，手机远程触发 audible `video.play()` 的结果。
- 如果被 autoplay policy 拒绝，验证“首次/浏览器重启后按一次确认键启用远程播放”的降级流程。
- 验证播放结束后 10/30 分钟重新远程播放。
- 验证页面刷新、浏览器重启、条件允许时息屏/休眠恢复后的行为。
- 所有 autoplay / play rejection 必须能被 Display 上报并在 Control 中形成可解释状态。

退出条件：

- 普通 LAN HTTP 下可从 Control 到 TV-style Web Display 完成端到端播放。
- secure-context API 不可用不会导致播放失败。
- Fullscreen 被拒绝时 viewport immersive 仍可用。
- TV audible autoplay 得到真实设备结论：PASS，或首次初始化一次后可长期远程播放的 CONDITIONAL PASS。
- 如果每次新播放都必须操作电视，不能直接宣布 Web TV Display 产品体验通过，必须进入架构/产品评审。

## Phase 0B：Jellyfin Display Adapter PoC（并行）

目标：执行 R004，验证 Jellyfin 是可行外部 Display Adapter，但不阻塞 Core。

- ARM64 Jellyfin Server。
- 动态 M3U / 媒体代理，具体媒体入口形式以 PoC 结果为准。
- Android TV Direct Play/Remux。
- 设备/Session 发现、远程播放、暂停、恢复、seek、stop。
- 从指定 position 启动，例如 `00:18:24`。
- position/handoff 精度记录。
- 验证 start command accepted 但客户端没有真正播放的失败语义。
- Jellyfin Server down / TV offline / media incompatible / token expiry 等失败场景。

退出条件：

- Jellyfin 官方客户端能播放 Gateway 媒体入口，且基本远程控制行为有真实设备证据；或者形成明确 CONDITIONAL PASS / FAIL 结论。
- 失败不会改变 Web-only Core MVP 路线。
- 不为了 Jellyfin 给 Playback Core 增加 Jellyfin special case。

## Phase 1：Plugin Boundary Hardening

目标：确保站点扩展不会重新污染 Core，并执行/吸收 R005 真实站点验证结果。

实施：

- `generic-ytdlp` 作为 fallback plugin，不允许 Core fallback。
- SiteAdapter conformance tests。
- Registry 冲突/priority/health。
- `SourceLocator` 版本/旧版本错误处理。
- `SiteAccessCapability` / `ScopedSiteHttpClient`。
- `ResolvedMedia.upstream_access_ref`。
- architecture test：扫描 Core concrete site knowledge。
- EgressPolicy public_web / configured_local_service。
- 选择至少一个真实重点来源站点做最小 Site Plugin PoC。
- 验证真实 URL recognize → SourceLocator → resolve → ResolvedMedia。
- 验证真实连续内容 previous/next 只通过 SourceLocator 进入 Core。
- 验证短期媒体 URL 过期后同一 SourceLocator re-resolve。
- 公开内容验证通过后，再验证 `SITE_AUTH_REQUIRED` / PendingIntent / retry 边界。

退出条件：

- 新增一个 fake/new site plugin 不需要修改 PlaybackCoordinator/DisplayAdapter/Control 的站点分支。
- 至少一个真实重点站点证明当前 Site Plugin Contract 可以承载真实 source/resolve/navigation 场景，或有证据驱动的契约修正。
- 插件不能直接读取 Vault 或绕过 EgressPolicy。
- 真实站点变化被限制在 Plugin Boundary，而不是进入 Stable Core。

## Phase 2：Handoff 与连续内容

- Web→Web handoff。
- 接入 R004 证明可用的 `JellyfinDisplayAdapter`；如果 R004 FAIL，则不强行接入。
- prepare/confirm/commit + generation。
- `PlaybackContext` previous/next/queue。
- `NextItem` / `PreviousItem`。
- `autoplay = off | next`。
- 下一集 resolve 期间保持当前 Display ownership。
- 多控制端 revision conflict 测试。

退出条件：

- 同一 item 可跨已证明可用的 Display 双向 handoff。
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

本阶段吸收 R006 结论；如果 R006 DEFER/DROP，不为了完整愿景强行进入实现。

- Site Browser Worker Native Control Mode。
- Control 中可折叠 Native Site Panel。
- browser event → Site Plugin.browser_interpret → SourceLocator。
- 选集/选择视频 → Playback Item Transition。
- Native panel crash isolation。
- 评估远程画面协议与 ARM64 资源成本。
- 明确 Plugin 如何通过通用 BrowserCapability/受限 CDP 等机制请求 DOM 操作，而不把 concrete selector 放进 Browser Worker。

退出条件：

- Site Browser Worker 崩溃时已解析媒体仍播放，Universal Remote 仍可用。
- Browser Worker 本身没有具体站点 DOM/API 逻辑。
- ARM64 Chromium/远程画面成本有真实数据支持；如果成本不可接受，明确保持 DEFER/Limited Mode。

## Phase 4：稳定性与低功耗

- system service / health check。
- CPU、内存、温度、并发和资源上限。
- 将 R003 的短期/基线测量扩展为长期稳定性验证。
- plugin/adapter timeout 与 circuit breaker。
- URL 过期刷新。
- 空闲状态无高频轮询/持续 Chromium。
- ARM64 长时间运行。
- Vault/runtime 清理和崩溃恢复。

退出条件至少包括：

- Idle 长期无异常资源占用。
- Direct Proxy 长时间稳定。
- Remux 的支持边界明确。
- Software Transcode 如果不符合资源目标，保持非默认/Unsupported，而不是通过隐藏资源成本满足兼容性。

## 运行时进程插件：后续触发条件

不属于 MVP。满足以下任一真实条件后再设计 IPC：

- 站点插件独立更新频率显著高于 Gateway；
- Playwright/Node/Python 依赖需要隔离；
- 插件崩溃影响 Core 稳定性；
- 需要独立 CPU/内存/网络沙箱。

优先进程插件，不优先 Rust `.so`。

编译期 MVP Plugin Boundary 属于强架构/first-party trust 边界，不把它描述为对恶意插件的 OS 级安全沙箱。

## 自动化测试与研究主矩阵

| 领域 | 最低测试 / 证据 |
|---|---|
| Site Contract | recognize、resolve、navigation、error、secret boundary、真实站点 PoC |
| SourceLocator | opaque、version mismatch、retry、real-site re-resolve |
| Playback | session/item revision、position concurrency、NextItem、stale callback、stale resolve |
| Display | registration、generation、candidate handoff、rollback |
| Control | Idle/Resolving/Playing、reconnect、revision conflict |
| Media | HLS/MP4、Range/segment、subtitle、URL expiry、30/60 分钟稳定性 |
| TV Web | audible autoplay、首次交互降级、Fullscreen degradation、refresh/restart |
| ARM64 | Idle、Direct Proxy、Remux、CPU/RSS/temperature |
| Egress | private IP、redirect、configured Jellyfin local service |
| Vault | cross-site denial、relogin replace、runtime cleanup |
| Web | HTTP baseline、HTTPS enhancement |
| Jellyfin | real Android TV start/pause/seek/position/handoff/failure |
| Failure | plugin timeout、display disconnect、Jellyfin down、browser worker crash |

## 首个可检查里程碑

第一批代码/研究提交只要求：

1. workspace / crate 边界；
2. `implementation-contracts.md` 对应的最小 Rust 类型；
3. fake/generic SiteAdapter；
4. R007 中最关键的 revision/stale callback 契约测试；
5. R001 media path proof；
6. 最小 Web Display；
7. Contract + architecture + baseline security tests；
8. R003 Idle/Direct Proxy 第一批资源数据；
9. `technical-feasibility-validation.md` 对应研究结果开始使用 PASS / CONDITIONAL PASS / FAIL + Evidence 记录。

不要在第一批实现中同时加入 Bilibili 完整登录、Native Site Panel、Jellyfin 完整 handoff 和插件 IPC。

## Technical Feasibility 完成定义

在 P0 研究完成前，仓库只能说“设计上计划支持”，不能说真实设备可行性已经验证。

Technical Feasibility Review 最终至少应形成类似：

```text
R001 Media Path: PASS / CONDITIONAL PASS / FAIL
Evidence: ...

R002 TV Remote Playback: PASS / CONDITIONAL PASS / FAIL
Evidence: ...

R003 ARM64 Resource: PASS / CONDITIONAL PASS / FAIL
Limits: ...
Evidence: ...

R007 Playback Contract: CLOSED / OPEN
Evidence: concurrency tests

R008 Security Boundary: PASS / FAIL
Evidence: SSRF / Secret / replay tests
```

只有 P0 Gate 有真实证据后，才把 Web-only Core MVP 标记为技术可行并进入后续功能扩展。