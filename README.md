# Jellyfin Web Media Gateway

把 Ubuntu 手机变成低功耗内容服务器：Windows 或其他手机负责选择和控制，小米电视只安装 Jellyfin 客户端并负责显示。

本项目不修改 Jellyfin 核心。它在 Jellyfin 前增加一个网页媒体网关，将受支持网页中的视频、音频和字幕解析为 Jellyfin 可播放的 HLS/M3U 媒体源，并集中保管网站登录会话。

## 目标体验

```text
Windows / 其他手机
  打开控制台、登录网站、选择内容
                 │
                 ▼
Ubuntu 手机
  Web 控制台 + 媒体解析网关 + Jellyfin
                 │
                 ▼
小米电视
  官方 Jellyfin Android TV 客户端，仅负责显示
```

用户在控制台粘贴网页地址并选择电视。网关优先提取原始 HLS/DASH/MP4 与字幕，避免重新编码；Jellyfin 将其作为动态 Live TV 频道呈现给电视。

## 核心原则

- Jellyfin 上游可持续升级，不维护大型私有分支。
- 网站账号、Cookie 和解析逻辑只存在于服务器。
- 电视不保存视频网站账号，也不安装每个网站的客户端。
- 优先 Direct Stream / Remux，保持低功耗和原始画质。
- DRM、验证码和无法合法解析的内容明确拒绝，不尝试绕过。
- 浏览器画面捕获仅作为独立实验，不进入首个 MVP。

## 文档

- [需求说明](docs/requirements.md)
- [系统设计](docs/architecture.md)
- [安全设计](docs/security.md)
- [MVP 实施计划](docs/mvp-plan.md)
- [ADR-0001：使用旁路网关而非修改 Jellyfin 核心](docs/adr/0001-sidecar-gateway.md)

## 当前状态

设计阶段，尚未提供可运行版本。

