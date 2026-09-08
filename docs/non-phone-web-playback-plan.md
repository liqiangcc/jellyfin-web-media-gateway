# 暂缓手机部署：真实 Web 播放推进计划

计划日期：2026-09-08。

用户目标：暂不部署手机，先完成普通 Linux 上的真实 B 站 Web 播放与控制闭环。

本文是完整推进方案；具体执行以链接的 Task Contract 为准，实时发布/状态以 GitHub 为准。计划编制时 GitHub main 为 `b17a27ca5d8c2f76cddc4c7cf3fdaa239169593a`；执行前重新检查 main、Issue history、Candidate 和 ownership。本文完成不代表下述 Task 已执行或发布。

## 1. 本轮交付目标

用户从浏览器打开隔离测试实例，选择 Web Display，在 Control 输入一个事先冻结、正常可访问的公开非 DRM B 站视频，完成：

```text
Control URL 输入
→ SiteAdapterRegistry / generic-ytdlp
→ ResolvedMedia
→ SourceSession / PlaybackSession
→ Gateway 同源媒体 capability
→ Web Display 实际播放
→ pause / seek / play / stop
→ Control 与 Display 刷新重连
```

本轮成功名称为“普通 Linux 上真实 B 站 Web 功能闭环已验收”。不据此宣布完整 TV MVP、手机容量、生产就绪或 #22 Core Gate 通过。

暂缓手机部署、手机管理面恢复、手机 Runner 重注册、手机性能实验。保留原产品的未来 ARM64/TV 方向；这次改变实施顺序与功能 Evidence 路由，不删除未来设备验收要求。

## 2. 已有基础与未决事实

| 项目 | 编制计划时的事实 | 对计划的影响 |
| --- | --- | --- |
| #2 / #3 / #14 | 已有 Playback、Media Path、安全基线验收 | 复用，修改相关领域时运行回归 |
| #49 | 已接受 hosted Web 产品组合与重连流程 | #68 复用既有入口及状态所有权 |
| #67 | R17 因冻结页面 4xx BLOCKED，J3 未运行 | 不能判断 #114 实际修复效果 |
| #114 | 网页归一化修复已接受 | 首次重测优先保留 `80fb081b129f8f664124b84ddcc9698039e2cfd1`，新环境兼容性另证 |
| #68 | draft，硬依赖 #67 Final Acceptance PASS | 不提前绕过发布依赖 |
| tx-node | SSH 可达，x86_64 Linux；当前连接为 root；PATH 有 Python/Chrome/setpriv，未找到 Cargo/Rust/FFmpeg | 候选执行主机，尚未成为合格低权限验证环境；PATH 检查不等于软件全盘清查 |
| 站点诊断 | MCP 首页加载完成；冻结视频页标题为“出错啦!”；tx-node 一次无代理请求为 4xx | 不能把首页访问当作样本或解析 PASS |
| MCP 拓扑 | 尚未确认 MCP Chrome 与 tx-node 是否同一实例 | 在任何 E2E Evidence 前明确浏览器与 Gateway 各自位置 |
| CI | main portable-ci 成功；site-navigation-prep 失败且无 job | 定位 workflow 问题，不宣称全 CI 绿色 |

上述 SSH/浏览器观察仅为诊断，不是原 #67 的正式 Attempt，不消费原手机复测的次数。

## 3. 执行角色与环境

| 工作 | Worker / Orchestrator | 实际执行位置 | 产出 |
| --- | --- | --- | --- |
| 文档契约、代码、测试、PR | Codex Cloud | 仓库 workspace | 精确 Candidate / PR |
| portable fmt/clippy/test/浏览器回归 | Codex 发起 Actions | GitHub-hosted x64；按改动需要 generic ARM64 | run/job/artifact |
| 真实站点诊断与解析 | Codex 经 SSH 编排 | 合格后的 tx-node，独立低权限进程 | 显式标记 SSH execution plane 的有限 Evidence |
| 真实 Web 旅程 | Codex + Chrome MCP 或仓库浏览器脚本 | 已确认的浏览器主机 + tx-node 测试 Gateway | 同一 Candidate 的播放/控制/重连事实 |
| Review / 发布 / 关闭 | Coordinator | GitHub | append-only Review、状态、Acceptance |

所有编译/构建强制使用远程 GitHub Actions 的 GitHub-hosted Runner。复用已有离线运行时与 harness，在兼容目标 ABI 的 hosted 构建环境生成二进制和必要测试产物，以精确 Candidate、run/job、manifest/摘要绑定后传至 tx-node。ABI 不匹配时修复远程构建环境，不安装目标机工具链或回退本地编译。目标仅运行无编译步骤的已验证产物；现有会调用 cargo 的 smoke 脚本不能直接用于目标机。首播不依赖 FFmpeg，除非真实媒体形状证明需要 remux。

本轮不默认在 tx-node 安装 self-hosted Runner。通用必需验证继续走 Actions；真实网络步骤经 SSH 执行须在修订后的 task.md 中明确允许，不能伪装成 Actions Evidence。若之后确需自动化桥接，其凭据传输和受信任务边界必须单独明确。

## 4. 阶段 A：修订路线和执行契约

负责人：Coordinator。依赖：用户已明确暂缓手机部署。

工作：

1. 基于最新 main 与完整 Issue history 整理现状，移除路线图中的“#67 Attempt 4 正在执行”等过时调度描述。
2. 更新 `product-roadmap.md`、`planning-priority.md`，明确功能主线无手机前置条件；检查 requirements、architecture、contracts、feasibility、mvp、security、environment/runner 文档的一致性。按 canonical 顺序修改真正受影响的条款，不把暂缓部署写成永久放弃 ARM64。
3. #67 先回 draft，修订为普通 Linux 真实公开网络解析验证。保留原 Issue、全部 Attempt、Candidate 与失败原因；修订 Evidence Authority 后按新范围重新验证，不追认旧手机 Claims。
4. #67 的下一 Attempt 编号以发布时 live history 为准；优先重测已接受 #114 精确 Candidate，记录候选 x86_64 环境支持情况。若必须改代码，先独立最小修复并冻结新 Candidate，不混淆对原修复的验证。
5. #68 保持 draft；明确后续 tx-node + 浏览器路由，精确 source shape、Candidate 等依赖 #67 返回后再填写。
6. 把手机线从本轮调度中移出，按第 9 节处置原 Issue。发布未来 Task 时执行完整 GitHub read-back、ready、queue verification，再输出实际入口。

验收：正式路线不再要求 #142/#131/#113 先通过才能开发普通 Linux 功能；#67 新契约清楚区分功能与手机 Claims；未满足依赖的 Task 不进入 ready。

本轮治理会把路线与 Task Package 持久化到 GitHub；是否正式发布须以 Issue 的独立读回/队列验证记录为准。

## 5. 阶段 B：最小执行环境与站点前检

负责人：Codex，按已发布契约执行。依赖：阶段 A 的环境与探测授权已明确。

采用独立 #146 NON-PHONE-EXECUTION-PREP 交付可复现低权限执行条件，因其运行方式/host 准备具有独立交付物与生命周期。B1 属于 #146；B2 属于 #67 的前检，不在 #146 请求 B 站。真实解析只由原 #67 拥有。

### B1 运行与浏览器边界

- 确认 OS、CPU 架构、可用磁盘/内存、用户和必要可执行文件；不运行性能长测。
- 确认或准备独立非 root 身份、独立 workspace/runtime/cache/test port；验证无生产 Vault/profile/长期凭据访问，无 sudo/capability 继承。
- 通过现有 accepted sandbox 验证 `no_new_privs`、直接 socket 禁止、broker IPC、超时及子进程清理；主机默认 root 连接不成为运行权限依据。
- 验证精确源码与离线运行时 provenance；复用已锁定产物，具体身份在执行契约中冻结，不猜测可用 artifact。
- 确认 MCP 浏览器所在主机、Gateway 可达路径和浏览器版本；使用独立未登录测试上下文，保持 Chromium sandbox 与正常 autoplay 策略。
- 初始服务绑定 loopback。浏览器不在同机时使用明确限定的 SSH 隧道/私有测试访问路径，配置匹配的 Host/Origin。不要把 Gateway/CDP 开到公网。

### B2 冻结样本的有界前检

- 保留现有 `BV14V411W7r5` 作为首选原样本；固定 direct/no-proxy、匿名请求形状和超时。
- 在新契约中预先冻结采样次数、间隔、成功准则。例如最多三次固定观察、两次连续 2xx；不继承为已执行，也不在失败后临时加次数。
- 状态类符合条件仍只代表前检通过；真正源内容、解析 shape 和播放由后续 Jobs 证明。
- 若仍为 4xx、transport error 或不满足规则：停止当前 live Attempt，记录 BLOCKED。不得采用 Cookie、身份/代理轮换、CAPTCHA 绕过或浏览器缓存充当成功。
- 如果旧样本本身失效或不再适合作为公开样本：Coordinator 基于可复核的正常访问事实正式修订样本，并同步 #67/#68。可选择一个合法公开、非 DRM、无需登录的替代样本，但不能无限搜索直到出现成功，也不能声称旧样本修复。
- 浏览器页面可达而普通请求不通时，仅做预先批准的有限差异诊断；这不自动授权把抓包 URL 注入 Gateway 或改用 BrowserWorker 解析架构。

验收：独立低权限环境、精确产物、浏览器拓扑与有限前检均有明确结果。任一必要条件缺失均不能进入解析 Job。

## 6. 阶段 C：完成 #67 真实受控解析

负责人：Codex verification Worker；Coordinator 接受结果。

按修订后的 J0–J4 验证：

| Job | 核心内容 | 必需 Evidence |
| --- | --- | --- |
| J0 | 精确 Candidate、新目标身份和低权限边界 | SHA、OS/arch、身份与隔离结果 |
| J1 | 锁定离线运行时和 sandbox | 来源/摘要校验、缓存结果、隔离检查 |
| J2 | 同一 Attempt 的真实样本可达性 | 固定请求条件、状态类、有限采样结果 |
| J3 | 仓库 smoke → BrokerProcessRunner → R008Broker → 插件 | broker 计数、protocol、stream_count、闭合错误枚举 |
| J4 | 清理与脱敏 | 无残留 worker、无媒体落盘/Secret 泄漏、清理结果 |

保持 Core 不理解 B 站语义、插件不直读 Vault、不绕过 EgressPolicy。保留 accepted raw/JSON body 上限和网页归一化边界，不能为首播随意扩大。

结果分支：

- **PASS**：真实返回当前首播支持的 muxed `http-file` 或 `hls`，至少一个 stream，全部 required Jobs 和安全清理通过。Coordinator 接受后立即准备 #68。
- **CONDITIONAL PASS**：必须已有有效媒体结果，且仅有契约允许的非安全限制。当前 #68 依赖 PASS，不能把条件通过自动视作满足；Coordinator 先明确是否需要契约修订。
- **FAIL**：完整受控路径已执行，但出现有效 `unsupported_stage/fallback_reason`，如仅有 DASH/分离音视频。仅从实际 Evidence 立项最小格式能力修复；不要提前建设转码平台。
- **BLOCKED**：来源、网络、权限、sandbox、broker 或安全证据不足。仅修复实际 blocker；没有新条件时不循环重跑。

每次新代码 Candidate 重跑受影响 required Jobs；旧手机结果和旧 Candidate 结果保留为历史，不冒充新主机证据。

## 7. 阶段 D：完成 #68 用户可见播放

依赖：#67 Final Acceptance PASS、#49 authority 有效、来源 shape 与新 Candidate 已冻结、Publication Gate 完成。

实现限制：复用既有 `/control`、`/display`、`POST /api/v1/sessions`、rendering view 和媒体 capability；只增加真实插件运行时到产品组合所需最小接线。不能通过 fixture seed、直接 store 写入、raw ResolvedMedia 注入构造成功。测试实例的显式运行时启用与默认生产 `DisabledRunner` 分开。

验收矩阵：

| 旅程/失败情况 | 验证要求 |
| --- | --- |
| Display 上线 | 真实注册、heartbeat、Control 可发现/选择 |
| Control 输入 URL | 从产品 API 经 Registry/插件创建 Session/Item |
| 实际媒体播放 | 浏览器仅取 Gateway 同源媒体；metadata/readyState 合理，currentTime 实际前进 |
| pause / seek / play / stop | 浏览器行为与 Gateway snapshot 一致；seek 有实际位置变化 |
| Control 刷新/重连 | 从服务端 snapshot 恢复，不创建第二份状态 |
| Display 刷新/lease 轮换 | 恢复正确 Session/Item，旧 lease/callback 被拒绝 |
| 连续使用 | Session A 停止后同一 Display 播放 Session B，刷新后仍为 B |
| 上游失败/Display 离线 | 返回可解释状态，无半成品 Session/media authority |
| 隐私/边界 | 浏览器、日志和 artifact 无源站 Cookie/Auth/profile、媒体签名 URL 或 capability token 泄漏 |

自动化修改 Playback 时覆盖最低七项：duplicate request_id、stale expected revision、stale item callback、stale re-resolve result、stale display generation、overlapping handoff、two-Control concurrent mutation。

required Actions 覆盖 workspace fmt/clippy/tests、SourceSession、Display、Control/R007、#49、R001/R008、插件运行时及受影响 navigation 回归。真实站点/浏览器 Evidence 与这些验证绑定同一个 Candidate；MCP 观察作为补充，不替代规定的 Actions Jobs。

浏览器 autoplay 如需一次正常用户激活，记录交互前提；不得以关闭策略的 Chrome flags 宣称无人值守远程有声自动播放。

退出：#68 B1–B8 与 required Jobs 被 Coordinator 接受，形成可复现的启动、访问、测试、清理说明。交付仍为隔离测试实例；长期生产启用另有边界。

## 8. CI 与文档维护工作

处理已观察到的 `site-navigation-prep.yml` 失败：先读取 Actions validation/annotation、解析工作流，确认具体原因，再做最小修复。无 job 的失败不能称测试断言失败，也不能仅凭猜测批量修改 workflow。

使用 #147 CI-NAVIGATION-WORKFLOW-REPAIR 这个聚焦 combined Task；不与 B 站运行时修复混为一个提交。验收为 YAML/workflow 校验及相关 Actions job 真正创建、运行且通过，并绑定 Candidate。

该工作不依赖 B 站网络，可利用网络主线被外部条件阻塞的时段完成。这指任务依赖独立，不要求额外多 Agent。

## 9. 现有 Issue 处置计划

| Issue | 本轮处置 | 恢复或推进条件 |
| --- | --- | --- |
| #67 | 同一 Issue 回 draft 修订 target/Evidence，再发布 | 普通 Linux 契约、环境与前检完整 |
| #68 | 保持 draft，准备下一层方案 | #67 PASS 后填全身份/shape 并发布 |
| #113 | 保留手机 reachability 历史，暂不执行 | 用户恢复手机目标后重新做管理面 gate |
| #142 / #131 | 保留 blocked，记录暂缓调度 | 用户恢复手机部署；不做 ping/recovery/re-register 循环 |
| #9 | 保留 blocked，延后手机性能 | 手机方向恢复且功能路径已稳定 |
| #7 | 保持 draft，不作为当前功能前置 | 真实 TV 与可达测试服务就绪时独立安排 |
| #16 | 保持 draft | 明确 Jellyfin/TV 需求与设备可用 |
| #72 | 保持 draft | #68 稳定后再优先评估连续内容 |
| #26 / #27 | 保持 draft | 首播稳定且明确真实登录/Native Panel 需求 |
| #22 | 保持 draft，保留原 P0 门槛 | 所需物理 TV/手机 Evidence 后续补齐，或正式调整长期产品目标 |

“暂缓”是调度决定，不把 blocked 改成 done，不将历史 FAIL/BLOCKED 改成 PASS，不新增非协议 status。正式协调决定要写入相应 Issue；以本轮 GitHub 治理评论及读回为准。

## 10. 顺序、投入与停止规则

执行顺序：

```text
A 路线/契约修订
→ B 普通 Linux 环境 + 有界站点前检
→ C #67 真实受控解析
→ D #68 产品播放闭环
→ 后续再选择 #72 / 真实 TV / 账号能力
```

CI 修复与当前文档整理不依赖实时站点，可在主线等待外部条件时完成。保持约 1–2 active、0–2 ready、1–3 下一层 draft；不要提前发布账号/导航/Native Panel/性能队列。

规划投入估算（工程工作日，非完成承诺）：

| 阶段 | 基础投入 | 估算前提 |
| --- | --- | --- |
| A 契约/路线 | 0.5–1 天 | 无新的产品目标冲突 |
| B 环境/前检 | 0.5–1 天 | 现有主机可隔离运行、产物可复用 |
| C #67 | 0.5–1 天 | 样本正常可达，接受的修复可用 |
| D #68 | 2–4 天 | 返回 muxed http-file/HLS，既有产品接口足够 |
| CI 修复 | 0.25–0.5 天 | 原因是局部 workflow 问题 |

正常路径约 4–8 个工程工作日；站点持续 4xx、产物不可用、格式能力缺口、Review/Actions 等待不计入保证时限。一旦出现这些情况，报告实际 blocker 与最小恢复条件，更新估算，不以重复环境 Task 填充进度。

## 11. 每一阶段的可审阅产出

- A：路线/契约 diff、Issue 处置记录、正式发布的 read-back 与真实 downstream entry。
- B：有限环境/站点检查矩阵；PASS 或具体 BLOCKED 层次；测试实例边界。
- C：Candidate、runtime provenance、J0–J4、脱敏解析结果、Coordinator Review。
- D：同一 Candidate 的 Actions + 实际 Web 旅程证据、已知限制、启动/清理说明、Final Acceptance。

所有报告区分 Implementation Result、Verification Result、Coordinator Decision；真实执行记录 Orchestrator、Execution Plane、Runner/Executor、Target、Candidate、run/job 或 SSH 命令选择器。未执行的验证明确标注未运行，不用规划说明替代 Evidence。

## 12. 编制依据

- [产品路线](product-roadmap.md)、[执行优先级](planning-priority.md)、[发布与任务协议](tasks/README.md)。
- [#67 当前解析任务与历史](https://github.com/liqiangcc/jellyfin-web-media-gateway/issues/67)。
- [#68 首次真实 Web E2E](https://github.com/liqiangcc/jellyfin-web-media-gateway/issues/68)。
- [#49 已接受 hosted 产品基础](https://github.com/liqiangcc/jellyfin-web-media-gateway/issues/49)。
- [#142 手机管理面阻塞](https://github.com/liqiangcc/jellyfin-web-media-gateway/issues/142)。
- [#131 手机 Runner 恢复边界](https://github.com/liqiangcc/jellyfin-web-media-gateway/issues/131)。
- [当前 main navigation workflow 失败](https://github.com/liqiangcc/jellyfin-web-media-gateway/actions/runs/33374985619)。

## 13. 已物化的接手入口

- #146：`docs/tasks/146-non-phone-execution-prep/prompt.md`，第一执行优先级。
- #147：`docs/tasks/147-ci-navigation-workflow-repair/prompt.md`，独立 CI 修复。
- #67 R20：`docs/tasks/67-generic-ytdlp-bilibili-real/prompt.md`，等 #146 验收后发布。
- #68：`docs/tasks/68-bilibili-web-e2e/prompt.md`，等 #67 PASS 后发布。
- 全路线 Coordinator 恢复：`docs/tasks/handoffs/non-phone-delivery.md`。

文件存在不表示 ready；每个 Worker 必须先读回 live Issue、包和可领取队列。
