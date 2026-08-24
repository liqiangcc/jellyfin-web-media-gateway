# ADR-0001：使用旁路网关而非修改 Jellyfin 核心

- 状态：已接受（设计阶段）
- 日期：2026-08-24

## 背景

项目需要在服务器集中处理网页媒体解析、网站登录、字幕与临时媒体代理，并允许解析后的媒体通过不同显示路径播放。其中 Jellyfin 是重要的电视显示路径，但浏览器也可以直接成为显示端。

如果直接修改 Jellyfin Server，会把快速变化的网站逻辑、敏感 Cookie、通用 URL 处理以及 Gateway 自己的播放状态模型引入成熟媒体服务器的核心升级路径；同时还会让 Web Display 被迫依赖 Jellyfin。

## 决策

建立独立 Web Media Gateway。Gateway 与 Jellyfin 通过 `JellyfinDisplayAdapter` 集成，优先使用稳定边界：

- Jellyfin M3U Live TV 输入；
- Jellyfin REST/WebSocket 会话接口；
- Gateway 提供的短期签名媒体 URL。

不 fork Jellyfin Server，不修改 Jellyfin 数据库结构，不要求修改 Android TV 客户端。

Gateway 同时可以通过 `WebDisplayAdapter` 直接向浏览器提供受控媒体；这条路径不经过 Jellyfin。

Jellyfin 的角色限定为一个外部 Display Adapter，不作为 Resolver、Session Vault、Media Gateway 或 Gateway 全局播放状态的强制依赖。播放会话与 Display Adapter 的详细所有权由 ADR-0002 定义。

## 结果

优点：

- Jellyfin 可独立升级和回滚。
- Jellyfin 故障不阻塞 Web Display。
- 网站解析故障不影响 NAS 本地媒体。
- Cookie、SSRF 和开放代理风险可限制在 Gateway 边界。
- 可以独立测试 Web Direct Stream、Jellyfin Direct/Remux 和浏览器捕获实验。
- 新显示端可以在不修改 Jellyfin 的情况下扩展。

代价：

- 需要维护 Display Adapter 抽象和 Jellyfin API/M3U 集成适配。
- Gateway 需要自己维护播放任务与跨显示端状态协调。
- Jellyfin 动态频道体验仍受 Jellyfin Live TV UI 限制。
- 不同显示端的 seek、字幕和状态精度可能不同，需要能力协商。

## 被拒绝方案

### 直接修改 Jellyfin Server

升级冲突、安全边界过大，并把站点适配器生命周期绑定到 Jellyfin 发布周期；同时会让浏览器直接播放也被 Jellyfin 架构约束。

### 把 Jellyfin 作为所有播放任务的全局 authority

这样 Web Display 也必须映射成 Jellyfin 会话，制造无必要依赖，并让 Gateway 很难正确表达非 Jellyfin 显示端。该问题由 ADR-0002 进一步决策。

### 自研完整 Android TV 播放器

重复实现成熟的解码、字幕、设备适配和播放状态能力，维护成本过高。首批电视路径继续优先复用 Jellyfin Android TV。

### 默认浏览器捕获

在手机 ARM64 chroot 上需要持续软件编码，功耗、延迟和稳定性不符合低功耗目标。Web Display 默认直接消费解析后的媒体，而不是服务端录屏。
