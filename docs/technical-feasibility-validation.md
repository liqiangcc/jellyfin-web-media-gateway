# 技术预研与可行性验证

## 1. 目的

本文件定义 Web Media Gateway 在进入完整业务实现前的风险驱动技术预研、最小 PoC 和真实环境验证方法。

当前仓库已经通过 `requirements.md`、`architecture.md` 和 `implementation-contracts.md` 定义了系统目标、架构边界与最小可编码契约。下一阶段的主要问题不再是继续扩展设计，而是回答：

> 这些架构假设在真实 Ubuntu ARM64 手机、真实浏览器、真实电视、真实媒体协议、真实 Jellyfin 客户端和真实来源站点上是否成立？

本阶段采用：

```text
Requirements / Architecture
        ↓
Implementation Contracts
        ↓
Technical Feasibility Research
        ↓
Risk Spike / PoC
        ↓
Evidence
        ↓
PASS / CONDITIONAL PASS / FAIL
        ↓
Continue / Change / Defer / Drop
        ↓
正式实现
```

核心原则：

- 不用更多设计推理代替真实验证。
- 只对“结果未知且结果可能改变架构、MVP 范围、核心体验或硬件方案”的问题做正式 Spike。
- 已知的软件工程问题优先使用 contract test、concurrency test、failure injection 解决，不单独做大型可行性研究。
- 技术实验可以使用最小临时代码，但不得为了让 Demo 成功而绕过已经确定的 Site Plugin、Secret、Egress、Playback authority 等边界。
- 实验失败是有效结果。失败后优先缩小范围、替换 Adapter、降级可选能力，不允许为了保住某个外部集成反向污染 Stable Core。
- 尚未执行的实验不得标记为通过。

## 2. 与规范文档的关系

本文件属于“验证与证据”层，不重新定义核心契约。

解释顺序：

```text
requirements.md             WHAT
architecture.md             CURRENT ARCHITECTURE
implementation-contracts.md CODING CONTRACTS
technical-feasibility-validation.md EVIDENCE / FEASIBILITY GATES
mvp-plan.md                 WHEN
```

如果实验结果推翻当前假设，必须执行：

```text
Evidence
  ↓
确认失败原因与影响范围
  ↓
提出最小设计调整
  ↓
requirements.md（如产品目标/非目标变化）
  ↓
architecture.md
  ↓
implementation-contracts.md
  ↓
mvp-plan.md
  ↓
security.md
  ↓
必要时新增 ADR
```

研究文档本身不能静默覆盖 canonical architecture。

## 3. 什么问题需要正式技术预研

满足以下任一条件时，应建立 Research Item：

1. 依赖真实浏览器、电视、Jellyfin、FFmpeg、yt-dlp、来源站点或 ARM64 硬件行为，不能只从 API/文档推出结果。
2. 失败会导致 Core 不可行。
3. 失败会迫使修改 `SourceLocator`、`ResolvedMedia`、`PlaybackSession`、`DisplayAdapter` 等核心契约。
4. 失败会让核心用户体验显著退化，例如电视每次播放都必须手动操作。
5. 资源成本可能违背“低功耗、长期在线 Gateway”产品目标。
6. 存在多个实现路线，且必须用数据决定，而不是凭偏好选型。

以下通常不需要大型 Spike：

- Rust 是否能提供 HTTP API；
- Axum/Tokio 基础用法；
- CAS、request id、revision 的一般实现方式；
- 常规 SQLite CRUD；
- 已有明确安全模型的基础输入校验。

这些问题应直接进入最小实现和自动化测试。

## 4. Research Item 统一模板

每项技术预研必须记录以下内容。

### 4.1 Metadata

```text
Research ID:
Title:
Priority: P0 | P1 | P2
Status: planned | in-progress | blocked | passed | conditional-pass | failed | deferred
Core Blocking: yes | no | partial
Related Requirements:
Related Contracts:
Related Components:
```

### 4.2 Hypothesis

使用一句可证伪的陈述描述当前假设。

错误示例：

> 研究一下 Jellyfin。

正确示例：

> Gateway 生成的临时非 DRM VOD 媒体入口可以由 Jellyfin Server 交给 Jellyfin Android TV 稳定播放，并能在可接受误差内从指定 position 启动。

### 4.3 Why It Matters

必须说明：

- 假设失败影响哪个用户场景；
- 是否阻塞 Web-only Core；
- 是否只影响一个 Adapter/Plugin；
- 是否可能改变硬件方案；
- 是否可能改变产品交互。

### 4.4 Minimal Experiment

实验只实现证明假设所需的最小路径。

不得为了“更真实”提前开发完整 Control、完整账号管理、完整 Native Site Panel 或完整站点插件。

### 4.5 Environment

每次真实验证至少记录适用项：

```text
Hardware:
OS:
Kernel:
Architecture:
Browser / WebView:
TV model / browser:
Jellyfin Server:
Jellyfin Android TV:
FFmpeg:
yt-dlp:
Rust toolchain:
Network:
Media source type:
```

外部软件升级可能改变结论，因此必须保存版本。

### 4.6 Success Criteria

成功标准必须在实验前确定。

禁止实验结束后根据结果重新定义“成功”。

### 4.7 Metrics

根据研究项选择：

- success rate；
- startup latency；
- seek latency；
- seek error；
- handoff latency；
- CPU；
- RSS / peak memory；
- temperature；
- bandwidth；
- media bitrate；
- dropped frames；
- reconnect/recovery time；
- error code / browser rejection reason。

### 4.8 Result

最终只能使用明确状态：

```text
PASS
CONDITIONAL PASS
FAIL
BLOCKED
DEFERRED
```

禁止使用“看起来没问题”“理论上可行”“应该支持”“基本能够”代替结论。

### 4.9 Evidence

证据可以包括：

- 实验代码；
- 可复现命令；
- 自动化测试；
- 脱敏日志；
- HTTP trace 摘要；
- 浏览器错误；
- Jellyfin Session 状态；
- benchmark CSV/Markdown；
- CPU/RAM/温度数据；
- 人工设备验证步骤与观察结果。

不得提交 Cookie、Authorization、API Key、临时媒体签名、登录输入、验证码或完整敏感 URL。

### 4.10 Decision

每个研究项必须落到：

```text
Continue
Change
Defer
Drop
```

并写清是否需要修改正式文档或创建 ADR。

## 5. Research Matrix

| ID | Priority | Research Question | Failure Impact | Core Blocking |
|---|---:|---|---|---|
| R001 | P0 | Media Gateway 能否稳定代理/提供 HLS、MP4，并满足 seek、Range、Secret boundary | Core 媒体链路失败 | Yes |
| R002 | P0 | TV Web Display 是否能低交互接收手机远程触发的带声音播放 | 核心电视体验可能失效 | Yes |
| R003 | P0 | Ubuntu ARM64 手机在 Idle、Direct Proxy、Remux 下是否满足低功耗长期运行目标 | 硬件方案或媒体策略需要改变 | Yes |
| R004 | P1 | Jellyfin Android TV 是否适合作为临时网页 VOD 的 DisplayAdapter | Jellyfin Adapter 路线需要改变 | No |
| R005 | P1 | 真实来源站点能否自然映射到 SourceLocator / ResolvedMedia / navigation 契约 | Site Plugin Contract 可能需调整 | Partial |
| R006 | P2 | Site Browser Worker + Native Site Panel 在 ARM64 上的资源与交互成本是否值得 | 可选原生站点控制需要降级 | No |
| R007 | P1 | 当前 playback revision / media refresh / handoff generation 语义能否抵抗真实异步竞态 | 并发契约需要在编码前修正 | Yes（contract） |
| R008 | P1 | Media proxy、Plugin、Browser Worker 是否能在不打洞的情况下满足 Egress/Secret 安全边界 | 安全架构需要调整 | Yes（security） |

R001、R002、R003 构成 Web-only Core 的 P0 Feasibility Gate。

## 6. R001 — Media Path Proof

### 6.1 Hypothesis

以下链路能够在不依赖 Jellyfin、不暴露上游 Secret 的情况下稳定工作：

```text
Test Source
→ SiteAdapterRegistry
→ SourceLocator
→ SiteAdapter.resolve
→ ResolvedMedia
→ Media Gateway
→ Web Display
```

### 6.2 最小实验范围

第一阶段只使用公开合法、非 DRM 测试源，优先 `generic-direct`。

#### HTTP MP4

验证：

- 普通 GET；
- byte Range；
- duration；
- seek；
- 浏览器中断后重新请求；
- Content-Type / Content-Length / Accept-Ranges 转发语义。

#### HLS

验证：

- master manifest；
- variant playlist；
- relative segment URL；
- query 参数；
- redirect；
- segment request；
- seek；
- 短期 Gateway media capability。

#### DASH

DASH 不阻塞第一条 MP4/HLS PoC，但研究必须保留：

- MPD；
- separate video/audio；
- segment URL；
- 浏览器消费方式；
- 是否需要 remux。

### 6.3 Secret Boundary

浏览器网络请求不得出现：

- Cookie；
- Authorization；
- 站点 bearer token；
- Vault 内容；
- Jellyfin API Key。

Display 只能获得 Gateway 生成、任务绑定、短期有效的媒体能力。

### 6.4 Failure Cases

至少验证：

- 上游 404/403；
- redirect；
- Range 不支持；
- segment 中断；
- Gateway token 过期；
- Display 重复请求；
- 无效 token；
- 跨 session/item token 重放。

### 6.5 Success Criteria

R001 PASS 至少满足：

1. MP4 或 HLS 中至少一种可以稳定连续播放，另一种有明确结果与后续计划。
2. pause/play/seek 行为可用。
3. Range/segment 请求没有被 Gateway 破坏。
4. Display 不获得上游 Secret。
5. `/stream` 不能退化为 arbitrary open proxy。
6. Core 没有站点 special case。
7. Jellyfin 完全关闭时路径仍成立。
8. 长时间播放没有持续增长的缓存/内存泄漏迹象。

如果 HLS/MP4 基础链路都不能成立，Core MVP 为 NO-GO，必须先解决 Media Gateway 模型。

## 7. R002 — TV Browser Remote Playback / Autoplay

### 7.1 为什么是 P0

核心体验假设是：电视打开 Gateway Display 后，可以保持等待；手机之后创建播放任务，电视直接播放。

浏览器对带声音的脚本自动播放通常存在 user activation / autoplay policy 限制，因此不能把“远程 `video.play()` 一定成功”当作既定事实。

### 7.2 Hypothesis

目标电视浏览器存在一种可接受的低交互流程，使 `/display` 初始化后，后续播放主要由手机 Control 驱动。

### 7.3 必测场景

#### Case A：从未交互

```text
打开 /display
→ 不点击/不按键
→ 等待 1~5 分钟
→ 手机发送播放任务
→ 尝试 audible video.play()
```

记录：

- 成功/失败；
- Promise rejection；
- `NotAllowedError` 或平台等价错误；
- 是否只能 muted autoplay。

#### Case B：一次初始化交互

如果 Case A 失败：

```text
Display 显示“按确认键启用远程播放”
→ 用户按一次确认键
→ 手机再次远程触发带声音播放
```

验证后续是否不需要重复操作。

#### Case C：播放结束后的长期等待

至少测试：

- 10 分钟后重新播放；
- 30 分钟后重新播放；
- 条件允许时更长空闲时间。

#### Case D：页面刷新

确认 activation/permission 是否丢失。

#### Case E：浏览器进程或电视重启

确认首次初始化是否需要重新执行。

#### Case F：息屏/休眠恢复

记录浏览器和网络连接行为。

### 7.4 同时验证

- viewport immersive 在 Fullscreen 被拒绝时仍正常；
- remote command arrival；
- video element 生命周期；
- visibility change；
- WebSocket 重连；
- muted → audible 转换行为；
- 浏览器错误能否回报 Gateway/Control。

### 7.5 Success Criteria

允许两种 PASS：

#### PASS

电视打开 `/display` 后无需额外交互，可以稳定接收后续远程带声音播放。

#### CONDITIONAL PASS

首次打开或浏览器重启后只需要一次明确初始化操作，随后多个播放任务不需要再次操作电视。

以下结果视为重大产品风险：

> 每次开始一个新播放任务都必须在电视端再次手动确认。

若发生，必须评审 TV 浏览器是否仍适合作为主 Display 路径，或是否需要设备专用模式/其他 Adapter。

## 8. R003 — ARM64 Resource Baseline

### 8.1 Hypothesis

Ubuntu ARM64 手机能够作为低功耗、长期在线 Gateway，在主要媒体路径下保持可接受的 CPU、内存和温度。

### 8.2 测试场景

#### Idle

Gateway 启动但无播放任务。

记录：

- CPU；
- RSS；
- load；
- temperature；
- 是否存在高频轮询/异常 wakeup。

#### Direct HTTP/HLS Proxy

至少验证一个典型 1080p 媒体。

记录：

- startup latency；
- CPU；
- RSS；
- temperature；
- network throughput；
- 30/60 分钟稳定性。

#### 4K Direct Proxy

如果测试网络、电视和媒体源支持，记录是否仍是网络转发主导，而不是 CPU 瓶颈。

#### FFmpeg Remux

使用 separate audio/video 或其他真实 remux 场景。

记录：

- CPU；
- RSS；
- temperature；
- startup latency；
- sustained stability。

#### Software Transcode Boundary

只做边界测量，不把视频转码提升为 MVP 默认能力。

目的：用数据确认“Direct / Remux 优先、Transcode 非默认”的资源依据。

#### Chromium Baseline

至少记录一个 Site Browser Worker 空闲/加载页面时的基础 CPU/RSS，为 R006 提供基线。

### 8.3 采样时间

关键路径至少覆盖：

```text
5 min
30 min
60 min
```

如果设备存在热降频，还要记录冷启动与稳定温度后的差异。

### 8.4 Success Criteria

不预设未经测量的绝对 CPU/温度阈值，但结论必须回答：

1. Idle 是否足够轻量，可以长期在线。
2. 1080p Direct Proxy 是否能连续运行而不持续热失控/资源增长。
3. Remux 是否可作为可用的兼容路径。
4. 哪些媒体路径必须明确标记 Unsupported 而不是 Transcode。
5. 是否需要限制并发、码率或浏览器 worker 数量。

R003 结果如果显示常规 Direct/Remux 已不符合低功耗目标，则必须评审硬件或媒体架构，不能把问题推迟到“后续性能优化”。

## 9. R004 — Jellyfin Display Adapter PoC

### 9.1 定位

Jellyfin 是可选 Display Adapter。R004 失败不得自动判定 Core NO-GO。

### 9.2 Hypothesis

Gateway 可以把临时非 DRM VOD PlaybackItem 暴露为 Jellyfin Server/Android TV 可可靠消费的媒体入口，并能支持基本远程播放控制和可接受的 position handoff。

### 9.3 最小拓扑

```text
Gateway
→ 临时媒体入口 / HLS / M3U（具体方式以实验为准）
→ Jellyfin Server
→ Jellyfin Android TV
```

### 9.4 必须验证

- 客户端/Session 发现；
- 远程 start；
- pause；
- resume；
- seek；
- stop；
- position 状态；
- 字幕；
- Direct Play / Remux 决策；
- 从指定 position 开始，例如 `00:18:24`。

记录：

```text
Gateway expected position
Jellyfin reported position
实际画面位置
```

### 9.5 Handoff PoC

```text
Web Display at P
→ Gateway snapshot P
→ Jellyfin probe / prepare
→ Jellyfin start(P)
→ 确认已播放
→ Web stop
→ commit active_display
```

记录：

- handoff latency；
- position error；
- start accepted 但客户端未真正播放的情况；
- failure rollback。

### 9.6 Failure Cases

- Jellyfin Server down；
- Android TV offline；
- client online 但不能播放当前媒体；
- M3U/动态媒体入口 stale；
- Gateway media token 过期；
- start command 成功返回但真实播放器失败；
- Jellyfin 状态回报延迟或不一致。

### 9.7 Decision

- PASS：继续正式实现 `JellyfinDisplayAdapter`。
- CONDITIONAL PASS：保留 Adapter，但明确能力/精度限制。
- FAIL：改变或推迟 Jellyfin Adapter 路线，Web Display Core 继续。

除非实验证明通用契约本身错误，否则禁止为了 Jellyfin 修改 `PlaybackSession` authority 或向 Core 加 Jellyfin special case。

## 10. R005 — Real Site Resolution PoC

### 10.1 Hypothesis

一个真实重点来源站点可以通过 Site Plugin 自然映射到现有通用契约，而不要求 Core 理解站点 URL、Cookie、DOM、私有 API 或下一集规则。

第一站点建议选择实际高频目标站点；`generic-ytdlp` 可以作为对照，但不能代替真实站点边界验证。

### 10.2 第一阶段：无需登录公开内容

验证：

```text
Input URL
→ SiteAdapterRegistry.recognize
→ SourceLocator
→ SiteAdapter.resolve
→ ResolvedMedia
```

检查：

- title/duration；
- video/audio；
- subtitle；
- media expiry；
- navigation；
- error mapping。

### 10.3 连续内容

插件必须输出：

```text
previous: SourceLocator?
next: SourceLocator?
```

Core 不允许理解 BV/EP/Season/playlist 等具体站点结构。

### 10.4 URL Expiry / Re-resolve

验证：

```text
ResolvedMedia expires
→ SourceLocator 保持内容身份
→ fresh resolve
→ new ResolvedMedia
→ 当前 PlaybackItem 安全刷新
```

该实验必须同时检查 R007 中 `item_revision/media refresh generation` 契约，确保旧异步 resolve 不会覆盖新媒体结果。

### 10.5 登录内容

只在公开内容链路通过后进入。

正确结果应是：

```text
Site Plugin.resolve
→ SITE_AUTH_REQUIRED
```

而不是 Core 根据具体 URL 推断登录。

随后验证：

```text
PendingIntent
→ Site Auth
→ SiteSession validated
→ retry same SourceLocator
```

### 10.6 Success Criteria

- 新站点的 concrete knowledge 主要停留在 `plugins/<site>/`。
- Core 不新增站点条件分支。
- SourceLocator 能支撑 navigation、retry、URL refresh。
- Secret 不进入 locator/public headers/Display。
- 插件错误能稳定映射到标准错误。

## 11. R006 — Site Browser Worker Feasibility

### 11.1 定位

R006 是 P2，可选功能验证，不阻塞 Core MVP。

### 11.2 Hypothesis

Ubuntu ARM64 上按需运行 Chromium，可以承担 Auth Mode 和可选 Native Control Mode，同时资源成本与交互复杂度不会破坏低功耗 Gateway 的主要价值。

### 11.3 Chromium Lifecycle

记录：

- cold start latency；
- warm start latency；
- RSS / peak memory；
- CPU；
- temperature；
- profile attach/materialization；
- shutdown cleanup；
- restart 后登录状态恢复。

### 11.4 Remote Frame / Input

至少探索：

- screenshot/frame stream；
- DevTools screencast；
- WebRTC 或其他低延迟方案。

不要在验证前预先固定协议。

验证输入：

- mouse；
- touch；
- keyboard；
- scroll；
- focus；
- viewport resize。

### 11.5 Browser Semantic Boundary

Browser Worker 仍然只能知道通用浏览器操作。

禁止出现：

- Bilibili/YouTube selector；
- ep_id / BV / playlist 参数；
- 具体站点登录成功判定；
- 站点下一集规则。

必须保持：

```text
BrowserEvent / BrowserSnapshot
→ Site Plugin.browser_interpret
→ SourceLocator / AccountState / NativePanelState
```

### 11.6 Browser Capability Contract Research

需要回答一个未来实现问题：

> Site Plugin 持有具体 selector/站点语义，而 Browser Worker 不持有站点知识时，插件如何高效、安全地要求 Browser Worker 查询/操作 DOM？

比较至少以下方案：

1. 完整 DOM snapshot；
2. 通用 `BrowserCapability`（query/text/attribute/click/navigate/evaluate 等受控操作）；
3. 受限 CDP operation contract。

评价维度：

- 数据量；
- 安全边界；
- 可测试性；
- 插件隔离；
- DOM 变化适应性；
- ARM64 资源成本。

R006 可以得出 DEFER/DROP，不得因此判定 Core 失败。

## 12. R007 — Playback Concurrency Contract Validation

这不是大型外部技术 PoC，而是编码前必须闭合的契约验证。

### 12.1 `session_revision` 与高频 position telemetry

需要明确：

> Display 每秒上报 position 是否推进 command CAS 使用的 `session_revision`？

如果每次 position 更新都推进 revision，Control 的 `Pause/Seek/Handoff(expected_session_revision)` 可能出现大量无意义 `REVISION_CONFLICT`。

优先比较：

#### Option A

position telemetry 不推进 command CAS revision，只更新可观察 position/telemetry sequence。

#### Option B

拆分：

```text
control_revision
telemetry_revision
```

第一版应优先选择简单、可证明且不会制造冲突风暴的模型。

必须写并发测试：

```text
高频 position callback
+ Control Pause/Seek
→ 不产生不可接受的冲突率
```

### 12.2 Re-resolve 与 `item_revision`

场景：

```text
同一 SourceLocator
→ ResolvedMedia A expires
→ async resolve B
→ async stale result A' 晚到
```

必须定义旧结果如何被拒绝。

可选：

- 每次当前 item 的 media refresh 都增加 `item_revision`；
- 或增加独立 `media_generation`。

第一版优先减少 revision 维度，但必须能证明 stale resolve 不覆盖 current media。

### 12.3 Handoff Candidate Generation

场景：

```text
target start 成功
但 active_display 尚未 commit
```

必须定义 target callback 使用哪个 generation，以及 transition 期间 callback 是否只是 candidate state。

可使用：

```text
transition_id
candidate_generation
from_generation
```

或等价 reservation 机制。

测试至少覆盖：

- target start 后 commit 前 callback；
- source 的旧 generation callback；
- handoff timeout；
- 双 handoff 并发；
- commit 后旧 callback 到达。

R007 退出条件不是“选了字段名”，而是竞态测试能够证明旧异步事件不能覆盖新 authority。

## 13. R008 — Security Feasibility / Boundary Proof

安全需求不是“后续加固”，技术 Spike 从第一天就不能通过打洞获得成功结果。

### 13.1 EgressPolicy

必须自动验证：

- `127.0.0.1` 拒绝；
- loopback hostname 拒绝；
- RFC1918 private address 拒绝；
- link-local 拒绝；
- metadata/reserved target 拒绝；
- public URL redirect 到 private 被拒绝；
- DNS/redirect 每跳重新校验。

### 13.2 Configured Local Service

Jellyfin 等 LAN 集成只能通过管理员显式配置进入：

```text
configured_local_service
```

用户输入 URL、Site Plugin 或 Browser Worker 不得自行声明私网例外。

### 13.3 Secret Boundary

至少验证：

- Plugin 不直接读取 Vault 文件；
- cross-site session capability 被拒绝；
- `ResolvedMedia.public_headers` 拒绝 Cookie/Authorization；
- Display 不获得上游 Secret；
- media token 跨 session/item 重放失败；
- logs/error response 不包含 Secret。

### 13.4 Process Invocation

FFmpeg、yt-dlp、Chromium 全部使用 argv/structured API，禁止 shell 字符串拼接。

技术 PoC 不允许用 shell 拼接临时绕过此约束。

## 14. Go / No-Go Gates

### 14.1 Web-only Core GO

在把项目视为“核心媒体网关技术可行”之前，至少需要：

```text
R001 PASS
+
R002 PASS or acceptable CONDITIONAL PASS
+
R003 PASS or acceptable CONDITIONAL PASS
+
R007 contract closed
+
R008 baseline security proof
```

含义：

1. 基础媒体路径成立；
2. 电视常驻 Display 的交互成本可接受；
3. ARM64 资源符合低功耗目标；
4. Playback concurrency contract 已闭合；
5. PoC 没有通过安全打洞获得成功。

### 14.2 Jellyfin 不属于 Core GO Gate

R004 失败：

```text
JellyfinDisplayAdapter = Change / Defer / Drop
Core Web Gateway = 仍可 GO
```

### 14.3 Native Site Panel 不属于 Core GO Gate

R006 失败：

```text
Native Site Panel = Defer / Drop / Limited Mode
Core Web Gateway = 仍可 GO
```

### 14.4 Real Site 属于 Plugin Contract Gate

R005 不要求在最早的公开媒体 PoC 前完成，但在宣布 Site Plugin Boundary 已被真实站点验证之前必须完成。

如果真实站点迫使 Core 理解 concrete site knowledge，应优先调整 SiteAdapter Contract，而不是接受 Core special case。

## 15. 业务闭环验证

技术可行不等于产品有价值。

最终至少验证下面的最小用户闭环：

```text
电视打开 Gateway Display
        ↓
手机打开 Control
        ↓
输入合法媒体来源
        ↓
Gateway recognize / resolve
        ↓
电视开始播放
        ↓
手机 pause / seek
        ↓
字幕正常
```

记录：

- 首次使用电视需要多少次操作；
- 日常播放电视需要多少次操作；
- 手机从输入到开始播放的步骤数；
- start latency；
- 失败时用户是否能理解下一步；
- 是否比普通投屏减少账号/字幕/设备管理摩擦。

如果技术上成功但每次播放需要复杂电视操作，必须记录为产品条件失败，不能只按播放器 API 成功判定 PASS。

## 16. 实验代码组织

研究代码应与正式产品边界清楚分离。

建议：

```text
research/
├── media-path/
├── tv-browser-autoplay/
├── arm64-resource/
├── jellyfin-display/
├── real-site/
└── site-browser-worker/
```

或使用等价的 workspace experimental crates。

规则：

- Spike 可以丢弃。
- 确认进入产品的代码再迁入正式 crate。
- 不因为 PoC 已经能跑就把临时代码直接视为 production implementation。
- 测试 fixture 与实验数据必须脱敏。

## 17. 结果记录

后续可以在 `docs/research/` 下为具体实验建立独立结果文件，但本文件是研究框架和 Gate 的 canonical 定义。

建议结果索引：

```text
docs/research/
├── README.md
├── R001-media-path.md
├── R002-tv-browser-autoplay.md
├── R003-arm64-resource.md
├── R004-jellyfin-display.md
├── R005-real-site.md
├── R006-site-browser-worker.md
├── R007-playback-concurrency.md
└── R008-security-boundary.md
```

研究总表至少维护：

| ID | Status | Result | Evidence | Architecture Impact |
|---|---|---|---|---|
| R001 | planned | - | - | Core blocker |
| R002 | planned | - | - | TV UX blocker |
| R003 | planned | - | - | Hardware/Core blocker |
| R004 | planned | - | - | Adapter only |
| R005 | planned | - | - | Plugin contract |
| R006 | planned | - | - | Optional feature |
| R007 | planned | - | - | Core contract |
| R008 | planned | - | - | Security gate |

## 18. 执行顺序

推荐顺序：

```text
Contract Freeze
      ↓
R007 Playback contract closure
      ↓
R001 Media Path
      ↓
R002 TV Browser Autoplay
      ↓
R003 ARM64 Resource Baseline
      ↓
R008 Security baseline throughout P0
      ↓
Core Feasibility Review
      ↓
R004 Jellyfin Adapter
      ↓
R005 Real Site
      ↓
R006 Site Browser Worker
```

R004 可以在设备条件允许时与 P0 并行，但不得阻塞 R001~R003。

## 19. 本阶段完成定义

Technical Feasibility Validation 阶段完成时，仓库不应该只写“理论上可行”，而应该能够给出类似：

```text
R001 Media Path: PASS
Evidence: ...

R002 TV Remote Playback: CONDITIONAL PASS
Condition: 浏览器重启后需要一次遥控器确认
Evidence: ...

R003 ARM64 Resource: PASS WITH LIMITS
Limit: Direct/Remux 可用，software transcode 不进入默认能力
Evidence: ...

R004 Jellyfin Display: CONDITIONAL PASS
Limit: handoff position 存在已记录误差
Evidence: ...

R005 Real Site: PASS
Maintenance risk: retained at plugin boundary
Evidence: ...

R006 Native Site Panel: DEFER
Reason: ARM64 Chromium remote interaction cost too high
Evidence: ...
```

只有在 P0 Gate 有真实证据后，才应把“Web-only Core MVP 技术可行”视为已经验证的事实。