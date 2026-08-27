# ADR-0002：Gateway 持有播放会话并使用 Display Adapter

- 状态：已接受（设计阶段）
- 日期：2026-08-24

## 背景

最初设计主要围绕“小米电视 + Jellyfin Android TV”展开，因此容易把 Jellyfin Session 当成整个系统的播放状态来源。

随着 Control PWA 同时承担 Web Player 能力，出现了不经过 Jellyfin 的合法显示路径：浏览器可以直接消费 Gateway 解析和代理后的 HLS/MP4/字幕。若仍让 Jellyfin 作为全局播放 authority，Web Display 就必须人为映射成 Jellyfin 会话，导致核心 Resolver、播放状态和 handoff 被一个可选客户端生态绑定。

项目需要一个能同时容纳 Web、Jellyfin 以及未来其他显示方式的稳定边界。

## 决策

### 1. Gateway 是 PlaybackSession authority

每个网页媒体播放任务由 Gateway 创建并维护 `PlaybackSession`。至少包含：

- `ResolvedMedia`；
- 生命周期与临时媒体能力；
- 播放状态；
- 已确认播放位置；
- 字幕选择；
- 当前 `active_display`；
- adapter 能力快照。

Adapter 的本地播放会话可以作为观测和控制来源，但不能直接覆盖 Gateway 的全局状态。

### 2. 显示端通过 DisplayAdapter 抽象接入

核心播放流程只依赖标准显示能力，不依赖 Jellyfin 类型。

概念接口包括：

```text
DisplayAdapter
├── list_or_register_displays
├── probe
├── prepare
├── start
├── pause
├── seek
├── stop
├── status
└── subtitle_capabilities
```

首批实现：

- `WebDisplayAdapter`
- `JellyfinDisplayAdapter`

未来如果增加 DLNA、Chromecast、mpv、Kodi 或其他显示端，应优先新增 adapter，而不是修改 Resolver 或把特殊分支写入 PlaybackSession 核心。

### 3. active_display 使用统一 DisplayInstance

`active_display` 不使用“电视/网页”布尔值，而引用一个统一显示实例：

```text
DisplayInstance
├── id
├── adapter_type
├── capabilities
└── adapter_metadata
```

Jellyfin device ID、浏览器 connection ID 等只存在于 adapter metadata 中。

### 4. Handoff 由 Gateway 协调

跨显示端切换采用：

```text
snapshot
→ probe / prepare target
→ target confirms playable
→ start target from confirmed position
→ stop source
→ commit active_display
```

必须满足：

- 目标未确认可播放前，不因 handoff 请求静默停止当前显示端；
- 只有 Gateway 可以提交新的 `active_display`；
- 并发 handoff 使用 revision / generation 或等价 CAS 机制拒绝旧状态覆盖；
- adapter 必须显式报告 seek、字幕、容器等能力差异。

### 5. Web Display 不经过 Jellyfin

浏览器直接使用 Gateway 签发的任务绑定媒体 URL。Jellyfin 未运行、被禁用或故障时，Web Display 仍应工作。

### 6. Jellyfin 只拥有其 adapter 内部状态

`JellyfinDisplayAdapter` 可以使用 Jellyfin REST/WebSocket、Live TV M3U 和 Jellyfin 内部 Session，但必须把这些状态转换成统一 adapter 状态再交给 Playback Coordinator。

Jellyfin Session 不是 Gateway `PlaybackSession` 的数据库替代品。

## 结果

优点：

- Core MVP 可以在没有 Jellyfin 的情况下成立。
- Web 和 Jellyfin 共享同一个 Resolver、Session Vault、Media Gateway 和任务生命周期。
- 新显示端可以通过 adapter 扩展。
- Jellyfin 故障域不会扩散到核心播放逻辑。
- handoff、状态恢复和安全授权拥有单一权威边界。

代价：

- Gateway 必须实现自己的 Playback Coordinator 和状态机。
- Adapter 状态与 Gateway 状态可能短暂不一致，需要 revision、超时和 reconciliation。
- 不同显示端能力差异必须显式建模。
- Jellyfin 已有的部分播放会话能力无法直接作为全局实现复用。

## 被拒绝方案

### Jellyfin 作为全局播放 authority

会让 Web Display 无意义地依赖 Jellyfin，并使未来其他显示端也必须伪装成 Jellyfin 会话。

### 在核心代码里直接区分 `if web` / `if jellyfin`

初期代码更少，但显示端增加后会把协议、状态和错误处理散落到 PlaybackSession、Resolver 和 API 层，形成长期耦合。

### 默认多显示端同步播放

会带来双倍或多倍媒体流量、进度竞争、时钟同步和额外解码负载，不符合首个版本的低功耗与简单性目标。首版保持单 `active_display`。
