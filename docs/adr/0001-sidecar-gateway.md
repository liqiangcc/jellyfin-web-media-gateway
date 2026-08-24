# ADR-0001：使用旁路网关而非修改 Jellyfin 核心

- 状态：已接受（设计阶段）
- 日期：2026-08-24

## 背景

目标是让电视只安装 Jellyfin 客户端，同时由服务器集中处理网页媒体解析和网站登录。如果直接修改 Jellyfin Server，会把快速变化的网站逻辑、敏感 Cookie 和通用 URL 处理引入成熟媒体服务器的核心升级路径。

## 决策

建立独立 Web Media Gateway，通过以下稳定边界集成：

- Jellyfin M3U Live TV 输入；
- Jellyfin REST/WebSocket 会话接口；
- 网关提供的短期签名媒体 URL。

不 fork Jellyfin Server，不修改 Jellyfin 数据库结构，不要求修改 Android TV 客户端。

## 结果

优点：

- Jellyfin 可独立升级和回滚。
- 网站解析故障不影响 NAS 本地媒体。
- Cookie、SSRF 和开放代理风险可限制在网关边界。
- 可以独立测试 Direct Stream、Remux 和浏览器捕获实验。

代价：

- 需要维护 Jellyfin API 和 M3U 集成适配。
- 动态频道体验受 Jellyfin Live TV UI 限制。
- 任意网页交互仍不能由原生 Jellyfin 播放器承载。

## 被拒绝方案

### 直接修改 Jellyfin Server

升级冲突、安全边界过大，并把站点适配器生命周期绑定到 Jellyfin 发布周期。

### 自研完整 Android TV 播放器

重复实现成熟的解码、字幕、设备适配和播放状态能力，维护成本过高。

### 默认浏览器捕获

在手机 ARM64 chroot 上需要持续软件编码，功耗、延迟和稳定性不符合低功耗目标。

