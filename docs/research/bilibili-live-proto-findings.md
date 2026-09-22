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
4. 播放 URL 带签名参数（`deadline`、`upsig` 等）→ `expires_at`/refresh-via-locator 语义是必要的，CDN URL 仍不是内容身份。**实测 `deadline` = 签发后 ~2 小时**（2026-09-22 采样），过期后需经 locator 重新 resolve 取新 URL——这条语义已被实站证实而非推测。
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
| `x/web-interface/view` | **FAIL** | 412 风控；wbi + 真 buvid3 + **浏览器页内 fetch（credentials）**均无效——IP/端点级硬风控 |
| `www.bilibili.com/video/*` HTML | **FAIL**（raw）/ PASS（浏览器） | raw 412；真实 Chrome 完整加载且 `__INITIAL_STATE__` 含全量 videoData（bvid/aid/title/desc/duration/owner/stat/pages/subtitle/ugc_season + 40 条 related 推荐） |
| `x/v2/dm/web/seg.so` 弹幕 | **PASS** | protobuf 200（早期 404 是路径笔误 `x2`）；`x/v1/dm/list.so` XML 也 200——弹幕裸 API 匿名可用 |
| `x/player/online/total` | PASS | 参考性元数据可用 |
| `search/type`/`search/all/v2` 视频搜索 | **FAIL**（raw）/ PASS（浏览器） | 200 但 body 是"出错啦"软风控页（无 JSON）；wbi 无效；真实浏览器 DOM 返回真结果含 BV id |
| `s.search.bilibili.com/main/suggest` | PASS | 真 JSON 自动补全，但只有关键词无 BV 结果 |

**结论**：第一版产品闭环（匿名播放 + BV 内选集 + 封顶清晰度 + 字幕槽）**纯 API 即可覆盖，浏览器取源对基础匿名播放不是硬依赖**。

浏览器对照实验改变了分工边界。**真实 Chrome 无登录即可加载搜索页与视频页**——浏览器取源不是"可要可不要"，而是发现层的必经路径。正确的切法：

- **播放管道（匿名公开视频）**：纯 API 即可——#68 MVP 闭环可以显著简化；
- **发现管道（搜索、合集、rich metadata、related 推荐）**：必须走已接受的 Browser 取源链（#165–#240），提取形态 = **加载页面 → 读 `__INITIAL_STATE__` / DOM**，页内 fetch 也救不了 `view`（IP/端点级硬风控）；
- **弹幕**：裸 API 即可（`x/v2/dm/web/seg.so` protobuf 或 `x/v1/dm/list.so` XML），不是浏览器依赖项。

那笔浏览器投资没有被浪费——用途从"播放必需"重新定位为"发现层必需"。这直接缩小 #68 剩余实现面，同时为搜索功能提供了实证可行性。

## 控制面能力对照（预研 → 契约映射）

- **选集/下一集**：BV 内分 P 由 pagelist 覆盖 → `SiteAdapter::navigation` + `NavigationDirection` 已够用；跨 BV 合集需 `view`（FAIL）→ 归登录态/浏览器增强。
- **清晰度**：`accept_quality` 阶梯 + `qn` 钳制 → 契约侧 = 重新 resolve（新 `media_generation`）+ 客户端 position resume；匿名下展示档位须按实际返回的视频列表过滤，不能只信 `accept_quality`。
- **字幕**：`x/player/v2` subtitle slot 匿名可达 → `SubtitleTrackView` 契约已有载体；需有字幕样本再验 URL 格式。
- **倍速/音量/seek/全屏/PiP**：纯 HTMLMediaElement 客户端能力，与站点无关，控制面板直接实现，不进契约。
- **弹幕**：裸 API 可用且已实现——`x/v1/dm/list.so` XML 全量池（本样本 7245 条），服务端解析为 `{t,mode,color,text}` JSON，display 层按 `currentTime` 车道渲染滚动弹幕（WAAPI 动画，pause/play 联动）。截图证据在会话内。渲染质量/密度控制是 display 细节，不进 MVP。
- **搜索**：必须浏览器路径（页面 DOM/`__INITIAL_STATE__`）；对应 `BrowserObservationHandoff` 的场景扩展——导航到 `search.bilibili.com/all?keyword=X` 提取结果卡片（bvid/title），属于插件导航解析的既有模式。

## 第三轮：真实浏览器播放验证（2026-09-22）

headless Chrome 151（CDP 驱动）实测 `/display` 页，两条投递路径均播放成功：

| 路径 | 结果 | 实测值 |
| --- | --- | --- |
| muxed durl → 同源代理 | PASS | `readyState=4`，正常播放，`duration=7137`（与 cid 一致），640x360 |
| Range seek（中段跳转） | PASS | `currentTime=3600` 跳转后 seeked 恢复播放（`t=3609`），无错误 |
| DASH 分离 A/V → remux → HLS | PASS | hls.js MSE 播放 480p，`readyState=4`；live playlist 模式下 `duration` 随已产出分段增长（首播时 ~470s，持续增长） |

截图证据：file 路径（BML 画面 + bilibili 水印）与 hls-remux 路径（480p 池年画面）均在会话内留存。

**原型期间发现并修复的真实 bug（对 Rust 实现的启示）**：

1. **客户端中断导致进程崩溃**：`<video>` 元素做 Range 探测/暂停时会主动断开连接，未处理的 stream pipeline 错误使 Node 进程整体退出。Rust 侧对应要求：媒体代理流的 client-abort 必须是正常路径而非 panic/error 传播。
2. **半就绪 session 被 Display 抢读**：`createSession` 先于 remux 完成登记会话，Display 轮询拿到 session 时 `media_url` 还是 file 占位路径，播了"只有画面的视频流"。修正为**媒体路径就绪后才发布会话**。这与正式架构的 `publish_prepared_session`（先备好再发布，一次性原子）语义一致——原型重现了这个不变量存在的理由。
3. Display 切换判定不能只比 `session_id`，媒体 URL 变化（如同 session 换投递路径）也要触发 reload。

## 未测/不能推导

- 登录态内容、4K/HEVC、drm（`protection` 未触发非 clear 值）；
- 切 P 导航（prototype `navigation()` 已写出但未实测）、多并发、长期稳定性；
- 有字幕样本的 `subtitle_url` 实际格式（本样本列表为空）；
- 弹幕 protobuf/XML 解析格式细节（端点已通，内容结构未深入）；
- iPhone Safari 真实播放（部署好待人工确认）；
- 本 spike 未实现 revision/CAS/Vault/EgressPolicy——这些是 Rust 层职责，原型不假装验证它们。

## 使用方式

```bash
cd experiments/bilibili-live-proto
PROTO_BIND=100.64.98.39 PROTO_PORT=8899 node src/server.js
# iPhone: http://100.64.98.39:8899/control （提交 BV 链接）
#        http://100.64.98.39:8899/display （播放端）
```
