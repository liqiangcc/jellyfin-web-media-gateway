# Bilibili live prototype：匿名直链取源与 remux 实测发现

日期：2026-09-22。执行环境：本机 x86_64 VM（`grok-bot-vm-411252337`），Tailscale 地址 `100.64.98.39`。代码：`experiments/bilibili-live-proto/`（一次性 spike，零依赖纯 Node，不进 workspace/CI）。本文是经验性发现记录，不是 Task Evidence，不改变任何 Issue 状态。

## 动机

#68 的核心未知是"匿名公开 B 站内容能否经 Gateway 式同源代理在真实客户端播放"。原路线（#165→#166→#169→#68）为回答这个问题投入了整个浏览器取源管道。本 spike 用最小路径直接测量该问题，并把结果对照 `implementation-contracts.md` 的形状记录，供 `plugins/bilibili` 实现与 #68 契约评审复用。

## 实测结果

| 断言 | 结果 | 证据 |
| --- | --- | --- |
| 匿名访问 `x/player/pagelist` | PASS | `bvid=BV14V411W7r5` 返回 4 个分 P，part2 `cid=418303512`、`duration=7138`（与 #166 观察一致） |
| 匿名 `x/player/playurl?fnval=0`（durl） | PASS | 返回 **muxed MP4**（quality=16/360p，size=380506437）；无需 remux 的最简投递路径存在 |
| 匿名 `x/player/playurl?fnval=16`（DASH） | PASS | video 候选 4、audio 候选 3（首个 video 852x480@30, 397kb/s；audio AAC-LC 48kHz 67kb/s） |
| CDN 直取需 Referer+UA | PASS | `upos-*` 镜像对 `Referer: https://www.bilibili.com` + 桌面 UA 的 Range 请求返回 206 + 真实字节（`ftypisom`）；不带 Referer 的行为未测 |
| 同源代理 + Range 透传 | PASS | `GET /stream/...` Range `bytes=0-4095` → 206；中段 `bytes=100000000-...` → 206（seek 可行） |
| DASH A/V → ffmpeg streamcopy → HLS | PASS | `ffmpeg -c copy` 以 ~5x 速度产出 VOD playlist + TS 段；iPhone Safari 原生兼容 HLS |
| `-hls_playlist_type vod` | FAIL（已修正） | vod 模式下 playlist 直到 EOF 才写出（2h 视频 ≈ 25min 等待）；改用默认 live 模式 + `-hls_list_size 0` 保留全部段，playlist 首段即写、可边下边播 |
| 端到端（/control → session → display） | PASS（本机） | `POST /api/play` → recognize → resolve → session → `/stream` 与 `/hls` 路径均返回真实媒体字节 |

## 对契约层的含义（供实现参考，不是变更决定）

1. `ResolvedMedia` 需表达两种真实 shape：muxed `http_file` 与 video+audio 分离对；#265/#271 的 MediaShapeV1/remux 方向得到实站佐证。
2. 匿名内容存在 muxed durl 路径时，首版投递可以不做 remux——实现顺序上 "http_file proxy" 应优先于 "DASH remux"。
3. `upstream_headers`（Referer/UA）必须属于 ResolvedStream 的服务端注入面，绝不下发客户端——原型按此边界实现且实测可行。
4. 播放 URL 带签名参数（`deadline`、`upsig` 等）→ `expires_at`/refresh-via-locator 语义是必要的，CDN URL 仍不是内容身份。
5. 环境差异本身是证据：tx-node 历史 4xx 更可能是网络/ASN 特征而非 B 站必然拦截；本机（中国大陆直连网络）匿名全通。#68 J3 的 host 准入应记录该差异。

## 第二轮：匿名 API 能力地图（2026-09-22 补测）

逐端点探测匿名可达性（本机 IP，桌面 UA + Referer）：

| 端点 | 结果 | 含义 |
| --- | --- | --- |
| `x/player/pagelist` | PASS | 分 P 选集（part 名/时长/cid）匿名完整可用 |
| `x/player/playurl` fnval=0 | PASS | muxed MP4 durl（360p） |
| `x/player/playurl` fnval=16 | PASS | DASH ≤480p：每个清晰度有 `avc1`+`hev1` 双 codec 变体，audio 三档码率 |
| `playurl` 高 `qn` | PASS（被钳制） | 匿名请求 qn=80/116/127 一律回落 quality=64，但实际 dash 视频列表只到 480p；`accept_quality`/`accept_description` 返回完整阶梯，可作清晰度选项展示 |
| `x/player/v2`（subtitle slot） | PASS | 匿名可达；本样本字幕列表为空，有待字幕样本复验 |
| `x/frontend/finger/spi` | PASS | 匿名 buvid3 可用 |
| `x/web-interface/nav` | PASS | 匿名返回 wbi img/sub key |
| `x/web-interface/view` | **FAIL** | 412 风控；wbi 签名 + 真实 buvid3 cookie 均无效 |
| `www.bilibili.com/video/*` HTML | **FAIL** | 412；`__INITIAL_STATE__` 抓取路径在本机网络不可行 |
| `x2/dm/web/seg.so` 弹幕 | FAIL | 404（端点形态可能已变；未深入） |
| `x/player/online/total` | PASS | 参考性元数据可用 |

**结论**：第一版产品闭环（匿名播放 + BV 内选集 + 封顶清晰度 + 字幕槽）**纯 API 即可覆盖，浏览器取源对基础匿名播放不是硬依赖**。仍需浏览器路径或授权账号的：rich metadata（`view`/合集）、弹幕、登录态/高清晰度。浏览器取源从"必经管道"降级为"增强路径"——这直接缩小 #68 剩余实现面。

## 控制面能力对照（预研 → 契约映射）

- **选集/下一集**：BV 内分 P 由 pagelist 覆盖 → `SiteAdapter::navigation` + `NavigationDirection` 已够用；跨 BV 合集需 `view`（FAIL）→ 归登录态/浏览器增强。
- **清晰度**：`accept_quality` 阶梯 + `qn` 钳制 → 契约侧 = 重新 resolve（新 `media_generation`）+ 客户端 position resume；匿名下展示档位须按实际返回的视频列表过滤，不能只信 `accept_quality`。
- **字幕**：`x/player/v2` subtitle slot 匿名可达 → `SubtitleTrackView` 契约已有载体；需有字幕样本再验 URL 格式。
- **倍速/音量/seek/全屏/PiP**：纯 HTMLMediaElement 客户端能力，与站点无关，控制面板直接实现，不进契约。
- **弹幕**：端点需另验；不进 MVP。

## 未测/不能推导

- 登录态内容、4K/HEVC、drm（`protection` 未触发非 clear 值）；
- URL 实际有效期、切 P 导航（prototype `navigation()` 已写出但未实测）、多并发、长期稳定性；
- 有字幕样本的 `subtitle_url` 实际格式（本样本列表为空）；
- 弹幕正确端点形态；
- iPhone Safari 真实播放（部署好待人工确认）；
- 本 spike 未实现 revision/CAS/Vault/EgressPolicy——这些是 Rust 层职责，原型不假装验证它们。

## 使用方式

```bash
cd experiments/bilibili-live-proto
PROTO_BIND=100.64.98.39 PROTO_PORT=8899 node src/server.js
# iPhone: http://100.64.98.39:8899/control （提交 BV 链接）
#        http://100.64.98.39:8899/display （播放端）
```
