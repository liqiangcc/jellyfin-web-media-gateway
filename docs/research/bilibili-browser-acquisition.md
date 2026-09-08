# Bilibili browser-assisted acquisition：预研与推进决策

日期：2026-09-08。Repository baseline：`c754aa9ff598ce74ffc0d765bd12d730c4f5d4e7`。本文是有边界的技术预研，不是实站验证验收，也不扩展生产权限。

## 结论与证据

预研决策：**CONDITIONAL PASS**，允许推进独立离线探针 #165。真实浏览器取源到 Gateway/TV 产品闭环：**BLOCKED**，尚无该链路证据。

## #165 Review 后的执行结论（2026-09-08）

#165 已在 `3511e70c73686d90ec65ca530a5f48be07023586` 合并并完成 Final Acceptance。其 hosted artifact（run `34211420463`、bundle artifact `10049995177`）的入口仍是明确的 **offline-only synthetic probe**：它不接收实站 selector，也不发起 Bilibili 请求。该结果证明了观察 DTO、隔离 fixture 和下载后独立消费者的实验边界，不能直接作为 #166 的 live entry。

因此 #166 继续保持 `status:draft`，不发布一个无法执行的 handoff。新增 #169 [Bilibili Browser Live Probe](../tasks/169-bilibili-browser-live-probe/task.md) 负责交付显式 live-selector 入口、插件拥有的导航解析、fail-closed 浏览器 broker 和可消费 artifact；只有 #169 Final Acceptance 后，Coordinator 才能冻结 #166 的 artifact/runbook/tx-node admission 并发布实站验证。这个调整是任务契约治理，不改变 #67 的 generic-ytdlp FAIL，也不把任何 live 结果写成 PASS。

| 观察/来源 | 已知事实 | 不能推导的结论 |
| --- | --- | --- |
| [#67 最新 Review](https://github.com/liqiangcc/jellyfin-web-media-gateway/issues/67#issuecomment-5577995984)，runtime `80fb081b129f8f664124b84ddcc9698039e2cfd1` | ordinary Linux direct 匿名前检两次 2xx；实际 brokered generic-ytdlp：`UNSUPPORTED_FORMAT`、`FALLBACK_WEBPAGE`、`RESPONSE_STATUS`、4xx、4 requests、0 streams | 不能断言根因是 DASH、登录、DRM 或风控；也不能推广为所有浏览器/插件路线不可能 |
| 2026-09-08 08:15:11.938 UTC，Chrome MCP 只读观察已打开的 `BV14V411W7r5` **part 2** | `videoPresent=true`，`sourceScheme=blob`，`readyState=4`，`duration=7138`，`mediaError=null`；页面播放信息 DASH video 候选 4、audio 候选 3 | 非 clean anonymous 实验；未记录网络/profile provenance，未验证持续帧进度、音频、独立取流、codec、有效期；part 2 不等于 #67 冻结样本 |
| main `site-adapter-api/src/lib.rs` | 现实现 `StreamProtocol` 仅 HttpFile/Hls；ResolvedStream 未完整表达 role/codec/AV pairing；SiteAdapter::resolve 无现成 browser observation capability 参数 | 文档设想支持 DASH 不等于实现已经支持 |
| main `gateway-core/src/browser_chromium.rs` | 当前 Fetch interception 先 authorize_url 再 continueRequest，启动参数含 no-proxy-server；通用生命周期已有 #33/#75 基础 | 不能仅凭拦截回调证明连接地址 pinning、SW/worker/redirect/WebSocket 等所有出站均被强制约束；这是待验证覆盖面，不是已证实漏洞 |
| #162 / PR #164，Candidate `d7539ff0f2c466e252c007e4b5dc9f34a312de73`，[Actions](https://github.com/liqiangcc/jellyfin-web-media-gateway/actions/runs/34197819966) | 通用 Control → Display 命令应用已有 hosted Evidence；tx-node 使用的演示源是通用 MP4 | 通用 MP4、单张 VNC 图、B 站原生网页播放均不能当 B 站 Gateway E2E PASS |

MCP 观察只输出上述 scheme、计数和播放元素状态；不保留原始 playinfo、DOM、HAR、Cookie、签名 CDN URL。上述既有页面观察是探索性线索，不是任何 Task 的 exact-Candidate required Evidence。

## 分层设计与未实现的缺口

拟研究的组合路径：

```text
Control 用户 URL → Registry → plugins/bilibili 解释内容身份
→ plugin-owned versioned SourceLocator（BVID + part；不是 CDN URL）
→ 受 R008 约束的通用 Browser runtime 有界观察
→ Bilibili 插件解释候选媒体（server-only ephemeral 数据）
→ 通用媒体能力选择/必要的 AV delivery
→ Gateway 同源 capability → TV Web Display
```

Gateway 保持 PlaybackSession authority；Core 不添加 B 站 DOM/API/URL 业务分支。Browser Worker 不是远程播放输出端。Native Panel 不是本轮依赖；其退出不能中断已开始媒体播放。

需要逐项证明：

1. 干净匿名浏览器能获得正确 part 的候选信息，且无需借用现有登录 profile。
2. 出站受强制 EgressPolicy 约束，不能仅靠 JS/CDP 观察当安全边界。
3. 插件解释出的媒体可由**独立服务端消费者**读取；浏览器退出后仍可取到有限字节；请求无需 Cookie/Authorization/profile 私带。站点签发的短期 URL 只在服务端内存中存在，日志/Control/Display 均不可见。
4. 媒体 shape（muxed 或 AV-separated）、codec/container、Range 行为、有效期线索可用脱敏数据描述。候选条数不等于可用质量档。
5. 实际 shape 决定通用交付方案；之后才接入真实 SiteAdapter、Registry 和 SourceSession。当前探针不注册生产插件。

## 方案比较

| 方案 | 判断 | 下一项证据 |
| --- | --- | --- |
| 保留 generic-ytdlp | #67 保留 BLOCK；没有外部或契约变化不重复相同实验 | 同一 Issue 记录可定位的新原因后再决定下一 Attempt |
| Browser 辅助插件取源 | 首选预研分支，尚无产品 PASS | #165 离线边界/探针 → #166 clean anonymous 实站可移植性 |
| 原站浏览器/VNC画面送到 TV | 不满足当前 Gateway 媒体交付目标 | VNC 只辅助观察，不做产品成功路径 |
| 分离 AV streamcopy/remux 成 HLS/fMP4 | 候选；通常无需重编码，但要证明 codec/container/timestamp/seek 兼容 | 真实 shape 后选择；不得预先保证低资源、可 seek 或 TV 兼容 |
| Gateway 提供 DASH + Display MSE 播放器 | 候选；增加 manifest/MSE/兼容测试面 | 与 remux 比较支持范围、资源和恢复语义后冻结最小方案 |

Browser 页面 `blob:` URL 属于浏览器对象资源，不能当跨设备媒体 URL；有 DASH 数组也不独立证明全部播放机制。FFmpeg 的 streamcopy 定义只证明可跳过解码重编码，不证明这份内容可直接 remux。

## 可执行任务与发布条件

| 阶段 | 交付与 Owner | 发布/完成条件 |
| --- | --- | --- |
| #165 [离线探针](../tasks/165-bilibili-browser-probe-prep/task.md) | Luna high Worker：有界实验探针、synthetic fixtures、安全负例、hosted artifact、live runbook | 无 #67 硬依赖；独立 GitHub Publication Gate 后可领取。只离线目标，禁止实站/产品启用 |
| #166 [实站可移植性](../tasks/166-bilibili-browser-source-real/task.md) | 后续 Luna high Worker：tx-node clean anonymous 双会话、独立消费者、小流量取证 | #165 接受后，Coordinator 冻结 exact artifact/runbook/执行用户/命令并重新发布；保持 draft 到此为止 |
| 通用媒体交付与 Bilibili SiteAdapter 集成 | Coordinator 根据 #166 证据再创建最小 combined Task，Worker 实现 | #166 C1/C2 PASS；先完成 contracts/API 缺口评审。依据边界拆通用 AV 能力与站点映射，不按环境拆任务；若 muxed 已可用则无需 remux Task |
| #68 真实产品闭环 | 复用现有 Issue，正式修订 source route 和硬依赖后由 Worker 执行 | 接受实际取源/媒体能力/插件集成；冻结样本、Candidate、artifact 和拓扑，再走 Publication Gate。当前旧 generic-ytdlp Contract 不得直接消费新路线 |
| #7 / #9 / #22 | 后续独立物理 TV / 手机资源 / Core Gate | 桌面浏览器功能证据不替代实体证据；手机部署仍暂缓 |

Coordinator 每个阶段读取 Issue history、task.md、Candidate 和 Evidence，评论 Review 并更新状态；实现接受与实站 Claim PASS 分开。#165 的完成不等于 #166 成功；#166 的负面实验可完整交付，但不能解锁后续实现/产品 PASS。

#27 保留 Native Panel/resource umbrella draft；本次仅明确提升服务端匿名取源预研优先级，不恢复 Native Panel/Auth/手机资源路线。#67 旧 Contract 和历史不改写。#68 不被复制，也不提前发布。

## 实站与最终验收约束

#166 使用全新临时匿名 profile、受限无特权进程和独立消费者，不接管当前浏览器/播放服务。CDP 仅 loopback/限定 SSH；账号/私有 profile 不可访问；runtime 无 root/sudo/长期凭据。所有编译在 GitHub-hosted Actions；tx-node 只验证并运行已核验 artifact。

最多两次全新会话，每次导航预算 120 秒；有限资源请求、总下载预算和取消清理由 #165 runbook 自动执行。出现账号/权限/DRM要求停止，不换代理/复制 Cookie/重复重试制造成功。不能证明 Browser 出站边界，禁止开启 live 模式。

#68 将来的验收至少包括真实用户 URL、正确 part、独立 Display 解码帧数及 currentTime 持续增加、音视频同步与适用的可听观察、pause 不回跳、seek/play/stop、自然 ended、Control/Display reconnect、source browser 退出与过期恢复。VNC 截图只能辅助；音频、持续播放和 TV autoplay 必须另有直接证据。既有通用 MP4 手测出现 pause 回跳/ended 状态不一致的线索，应在 #68 发布前用具体 Candidate 重现、归因并复用或另开聚焦修复 Issue，不当已确认根因。

## 外部技术参考（查阅于 2026-09-08）

- [Chrome DevTools Protocol Network](https://chromedevtools.github.io/devtools-protocol/tot/Network/)：网络观察事件及有界响应读取接口；不是网络隔离证明。
- [Playwright Network](https://playwright.dev/docs/network)：Service Worker 可能影响路由拦截可见性，需独立覆盖此层。
- [W3C Media Source Extensions](https://w3c.github.io/media-source/)：MediaSource 与对象 URL 模型。
- [FFmpeg streamcopy](https://ffmpeg.org/ffmpeg.html#Streamcopy)：无解码/重编码的 packet 路径及限制。
- [yt-dlp Bilibili extractor source](https://github.com/yt-dlp/yt-dlp/blob/master/yt_dlp/extractor/bilibili.py)：DASH/durl、请求上下文处理的研究参考。master 可变，不能替代仓库锁定 runtime 的证据。
