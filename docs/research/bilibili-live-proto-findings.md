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
| `x/web-interface/view` | **FAIL**（本机匿名）/ **PASS**（ECS 匿名 + 本机登录） | 本机 412 且页内 fetch 也挂；但 ecs-node 匿名直接 200 全量数据——**实为 IP 画像风控不是鉴权门**，登录 cookie 只是恰好改变信誉画像 |
| `playurl` 清晰度 | 匿名封顶 480p | **登录后 1080p 解锁**（qn=116 请求 → quality=80，avc1+hev1 双编码 ladder；测试号非大会员，4K/60fps 未验证） |
| `search/type`/`search/all/v2` 视频搜索 | **FAIL**（raw）/ **FAIL**（cookie+wbi）/ PASS（浏览器） | 登录态 cookie + wbi 签名仍返回假 200 HTML——**搜索是唯一的浏览器硬依赖** |
| `www.bilibili.com/video/*` HTML | **FAIL**（raw）/ PASS（浏览器） | raw 412；真实 Chrome 完整加载且 `__INITIAL_STATE__` 含全量 videoData（bvid/aid/title/desc/duration/owner/stat/pages/subtitle/ugc_season + 40 条 related 推荐） |
| `x/v2/dm/web/seg.so` 弹幕 | **PASS** | protobuf 200（早期 404 是路径笔误 `x2`）；`x/v1/dm/list.so` XML 也 200——弹幕裸 API 匿名可用 |
| `x/player/online/total` | PASS | 参考性元数据可用 |
| `s.search.bilibili.com/main/suggest` | PASS | 真 JSON 自动补全，但只有关键词无 BV 结果 |

**结论**（登录态补测后修正）：`view` 的匿名 412 实为**鉴权门**——带 SESSDATA cookie 纯 API 即通，且 `playurl` 解锁 1080p（avc1+hev1）。QR 登录流程已在原型实现（`generate`→`poll`→cookie 持久化，页面自动续期）。

正确的三层切法：

- **播放管道（匿名公开视频）**：纯 API 即可——#68 MVP 闭环可以显著简化；
- **登录增强（元数据/合集/1080p）**：`view` + 高清晰度 `playurl` 都是纯 API——**登录态不需要浏览器**；
- **发现管道（搜索）**：唯一剩余的浏览器硬依赖——cookie+wbi 签名也过不了搜索的软风控，只能页面 DOM 提取（原型 `browser.js` 已验证）；
- **弹幕**：裸 API 即可（`x/v2/dm/web/seg.so` protobuf 或 `x/v1/dm/list.so` XML），匿名可用。

浏览器取源的角色进一步收窄：**只为搜索服务**。元数据/合集这类"发现"其实归登录 API 管；搜索是唯一只能在浏览器里做的事。

## 控制面能力对照（预研 → 契约映射）

- **选集/下一集**：BV 内分 P 由 pagelist 覆盖 → `SiteAdapter::navigation` + `NavigationDirection` 已够用；跨 BV 合集需 `view`（FAIL）→ 归登录态/浏览器增强。
- **清晰度**：`accept_quality` 阶梯 + `qn` 钳制 → 契约侧 = 重新 resolve（新 `media_generation`）+ 客户端 position resume；匿名下展示档位须按实际返回的视频列表过滤，不能只信 `accept_quality`。
- **字幕**：`x/player/v2` subtitle slot 匿名可达 → `SubtitleTrackView` 契约已有载体；需有字幕样本再验 URL 格式。
- **倍速/音量/seek/全屏/PiP**：纯 HTMLMediaElement 客户端能力，与站点无关，控制面板直接实现，不进契约。
- **弹幕**：裸 API 可用且已实现——`x/v1/dm/list.so` XML 全量池（本样本 7245 条），服务端解析为 `{t,mode,color,text}` JSON，display 层按 `currentTime` 车道渲染滚动弹幕（WAAPI 动画，pause/play 联动）。截图证据在会话内。渲染质量/密度控制是 display 细节，不进 MVP。
- **收藏夹**：已实现——`x/v3/fav/folder/created/list-all` + `x/v3/fav/resource/list` 纯 API（登录态），控制面板可浏览文件夹并点播。实测收藏 "凡人修仙传" 可取回。
- **番剧（bangumi）**：**独立媒体类型**——收藏到番剧 BV 时 `x/player/playurl` 返回 -404；`view`（登录态）经 `redirect_url` 给出 `bangumi/play/ep<id>`；正解是 `pgc/player/web/playurl?ep_id=`。已实现：BV→bangumi 自动跳转 + `/bangumi/play/epN` 直接识别。**大会员内容非会员只给预览**（`is_preview:1`，3 分钟流 vs 正片 19.5min）——`is_preview`/`has_paid` 必须成为 ResolvedMedia 的一等字段，控制端要区分"预览"和"正片"。免费/限免集数（badge `free`/`限免`）应能拿到全片。
- **直播**：已实现且浏览器实测——`live.bilibili.com/<room>` 识别 → `xlive getRoomPlayInfo` → 多候选探测（部分房间 avc 档 404/超时，hevc/fmp4 存活，需**逐候选探活**而非盲选）→ playlist 代理 + 段重写（含 `#EXT-X-MAP:URI=` init 段）→ hls.js 播放实测 `readyState=4` 720p。直播 URL 分钟级过期 → `/livepl` 每次上游失败时经 locator 重新 resolve（refresh-via-locator 在直播形态下的实现）。`LIVE_OFFLINE` 是一等错误。直播列表 `getList` 被风控（-352）——发现直播房间需走浏览器或已知房间号。
- **发现面（推荐/热门/相关/排行）**：**全部匿名纯 API 可达**——`popular`（40 条）、`archive/related`（40 条）、`index/top/feed/rcmd`（30 条，登录后个性化）、`ranking/v2`（100 条）。已实现 `/api/discover?kind=` 和控制面板"热门/推荐"按钮。**发现层最终定型：只有搜索需要浏览器**，其余发现面（热门、推荐 feed、相关推荐、收藏夹）全是纯 API。
- **搜索**：已实现且端到端验证——原型新增 `browser.js`（CDP 驱动既有 Chrome，镜像 `BrowserObservationHandoff`：导航 → 等待卡片渲染 → 提取 `{bvid,title,duration,cover}` → 关 tab）。`/api/search` → 控制面板结果列表 → 点击播放实测通过（BML 搜索 → 点选 → 播放 2h38m 视频 + 弹幕）。封面经 `/img` 有界代理（`*.hdslb.com` allowlist + 服务端 Referer），不开放泛代理。这是浏览器取源在发现层的首次实证。

## 延迟预算实测（2026-09-22，本机 loopback 到网关，上游为 bilibili CDN）

| 环节 | 耗时 | 说明 |
|---|---|---|
| `/api/play` BV→durl | ~0.9s | recognize + pagelist + playurl 两次上游往返 + session 创建 |
| `/api/play` BV→dash remux | ~2.3–4.1s | resolve + ffmpeg 启动 + 首段产出才发布会话（publish-after-ready 的代价） |
| `/api/play` live 房间 | ~1.8s | getRoomPlayInfo + 逐候选探活（死变体要等 5s 超时） |
| `/stream` 首字节 | ~1.2s | 上游 bilivideo fetch + Range 透传 |
| `/api/favorites` / `/api/discover` | ~0.5s | 纯 API 单次往返 |
| `/api/search`（浏览器） | ~2.6s | Chrome 开 tab + 导航 + 卡片渲染等待 + DOM 提取 |

**结论**：durl 路径从点选到可播约 2s（resolve + 首字节）；dash remux 多 ~1.5–3s（ffmpeg 冷启动 + 首段缓冲）。display 轮询间隔是叠加项——轮询越慢感知延迟越高，正式实现可用 SSE/长轮询消掉。直播的候选探活是必要开销（盲选死链会挂整次播放），但可并行探测压缩到 ~0.5s。

## 出口 IP 画像对比（2026-09-22，本机 VM vs ecs-node 阿里云）

延迟大头是上游 RTT：本机→网关 ~3ms，网关→`api.bilibili.com` 本机 250–550ms/次、ECS 105–130ms/次，网关→bilivideo CDN 波动大（80–1090ms）。resolve 的 ~0.9s 即两次串行 API 往返之和。

| 端点 | 本机 VM 匿名 | ecs-node 匿名 | 判定 |
|---|---|---|---|
| `playurl`/`pagelist`/`nav`/`dm`/`popular`/`rcmd`/`ranking` | PASS | PASS | 匿名稳定 |
| `view` | FAIL 412 | **PASS 200 真数据** | IP 画像风控，非鉴权门 |
| `search` | FAIL 假 200 | FAIL 假 200 | 端点级风控，与 IP 无关 |
| `getRoomPlayInfo`（直播） | PASS | 未测 | 预计 PASS |

**修正结论**：`view` 的本机 412 不是"必须登录"——ECS 匿名直连就通。风控粒度是 **IP 画像 × 端点**：本机 IP 被标记拦 `view`，ECS IP 未被标记。登录 cookie 在本机能过是因为凭证改变了请求信誉，并非 `view` 本身需要鉴权。

**架构推论**：部署环境的风控画像必须在部署后实测（同一份代码在两个 IP 下结论相反）；每条通道保持可降级；登录态的价值在本机是"解锁 `view`/1080p"，在 ECS 上 `view` 本就匿名可达、登录仅剩清晰度/收藏夹/个性化价值。

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

## Youku 预研（2026-03，ECS + headless Chrome CDP）

### 可达性矩阵

| 路径 | 本机 VM | ecs-node（阿里云） |
|---|---|---|
| 播放页 HTML | ❌ rgv587 滑块 punish（连真 Chrome 都拦） | ✅ 200 真页面 |
| `__PAGE_CONF__` 元数据 | — | ✅ title/videoId/seconds/isDRM 可读 |
| 播放取流 | — | ✅ m3u8 + 签名段 URL |
| 弹幕 | 未探 | 未探（不同协议） |

**IP 画像差异极端**：B 站 VM≈ECS，优酷 VM 全拦 ECS 全通——阿里系风控对阿里自家 IP 放行。

### 取流链路（ECS 实测）

```
v.youku.com/v_show/id_<vid>.html
  → headless Chrome 加载页面
  → 播放器自动调 un-acs.youku.com/h5/mtop.youku.play.ups.appinfo.get/1.1/
      appKey=24679788, sign=md5(_m_h5_tk&t&appKey&data)
      data.steal_params={ccode:"0502",utid=cna,ckey:<JS生成>,...}
  → data.stream[] = [{stream_type:flvhd|mp4hd|mp4hd2|mp4hd3, m3u8_url, w,h,size}]
  → m3u8 内段 URL 带 vkey/expire=18000/ups_client_netip 签名（IP 绑定）
```

### 架构结论

- **浏览器管道是主力方案**：mtop sign + steal_params.ckey 是 JS 动态生成，纯 API 重放易碎——优酷上 BrowserWorker 不是降级，是主路径
- **ECS 上浏览器取流可行**：headless Chrome 加载页面 → CDP 拦截 ups 响应 → 提取 m3u8 → 段代理（同直播 HLS 代理模式，需重写段 URL 过网关）
- **段 URL IP 绑定**（`ups_client_netip`）——必须从 ECS 出网拉段，手机端不能直接拉
- **VIP/付费内容**：`pay_info_ext.can_play`/`is_vip` 字段区分；测试片 `can_play:true` 匿名可播
- **元数据免浏览器**：`__PAGE_CONF__` SSR JSON 可直接 curl 解析（ECS 上）

### 待办

- [x] 实现 youku adapter（recognize `id_*.html` + browser-driven resolve）
- [x] 弹幕协议（`mopen.youku.danmu.list` mtop，页内 `lib.mtop.request` 代签，按 mat 分钟桶懒拉）
- [ ] 登录/VIP 链路
- [ ] 搜索（`yksearch` 只给热搜榜，`so.youku.com` 全线 punish——真结果不可达）

## 第三轮：腾讯视频 (2026-09-23)

对比验证第三个站点，证明多站点架构泛化。

### 腾讯视频 vs B站 vs 优酷

| 维度 | B 站 | 优酷 | 腾讯视频 |
|---|---|---|---|
| **播放页** | VM/ECS 全通 | VM 拦/ECS 通（现限流冷却中） | VM/ECS 全通 |
| **取流协议** | `playurl` 纯 GET | mtop 签名 | `vinfo_proxy` POST + **加密响应** |
| **签名** | 无 | mtop sign + ckey | 响应加密（页面 JS 解密） |
| **清晰度** | qn 数字档 | stream[] 数组 | `fl.fi[]` 格式表（480p/720p/1080p/4K） |
| **流格式** | MP4/DASH | HLS m3u8 | HLS m3u8 |
| **CDN** | bilivideo.com | cibntv.net | smtcdns/apdcdn.tc.qq.com |
| **弹幕** | XML API | mtop `danmu.list` | trpc `pbaccess.video.qq.com` |
| **选集** | pagelist | 未探 | `getPage` trpc（CardList 含剧集模块） |

### 实测验证（ECS）

```
✅ v.qq.com 页面加载 → __VINFO_DATA__ 可 evalInPage 提取
✅ vinfo_proxy POST → proxyhttp.vinfo 加密载荷 → 页面解密后
   {"dltype":8,"fl":{"fi":[hd/shd/fhd/uhd]},"vl":{"vi":[{ul:{ui:[4个CDN]}}]}}
✅ CDN apd-vlive.apdcdn.tc.qq.com → m3u8 playlist → 段 200（908KB TS）
⚠️ smtcdns.com CDN 对 ECS TLS 层拒绝（只 apdcdn 通）——需 CDN 镜像降级
```

### 架构含义

**第三种站点形态**：
- B 站 = API-first（无签名，浏览器只在搜索降级）
- 优酷 = Browser-first（mtop 签名 + IP 风控）
- 腾讯 = **Encrypted-response**（API 可发但响应加密，解密在页面 JS 内）

取流路径：`captureResponse(vinfo_proxy)` 不行（响应加密）→ 必须 `evalInPage` 读 `__VINFO_DATA__`（页面自己解密的）。这比优酷还依赖浏览器——但页面不被风控，所以更稳定。

### 待办

- [ ] 实现 tencent adapter（`v.qq.com/x/cover/...` → evalInPage 读 `__VINFO_DATA__`）
- [ ] CDN 镜像降级（smtcdns TLS 拒 → 遍历 ul.ui 找能通的）
- [ ] 弹幕 trpc 协议（`pbaccess.video.qq.com/trpc.danmu.*`）
- [ ] 搜索（v.qq.com/x/search/ 是否 punish 未探）
- [ ] VIP 内容（加密 vinfo 对 VIP 内容可能返回 DRM 流）
