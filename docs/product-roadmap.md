# Product Delivery Roadmap

当前具体 Task sequencing authority，更新于 2026-09-08。用户决定暂缓手机部署，优先交付普通 Linux 上真实 Web 播放。本文不覆盖 requirements/architecture/contracts/security，不保存实时 owner；Issue 是状态 authority。

## 当前可交付里程碑

```text
当前优先预研：#165 离线 Browser 取源探针
  → #166 clean anonymous 实站媒体可移植性（依赖冻结后发布）
    → 按真实 shape 评审/实现最小通用媒体能力与 Bilibili 插件
      → 正式修订并发布 #68 Control → Gateway → Web Display

保留旧路径：#146 → #67 generic-ytdlp（BLOCK，待具体外部/契约变化）
已接受的通用 Web/Display 基础可复用；不是 B 站产品 PASS。
```

当前证据、替代路线、任务边界和下一阶段条件见 [Bilibili Browser 预研](research/bilibili-browser-acquisition.md)。此前 generic-ytdlp 路线的工作拆分见 [推进计划](non-phone-web-playback-plan.md)，其顺序已由本次研究分支更新。恢复整个路线的 Coordinator 入口见 [non-phone-delivery handoff](tasks/handoffs/non-phone-delivery.md)。

## 已接受基础

- #2 R007：Playback revision、请求幂等与 stale race；#3 R001：MP4/HLS Media Gateway；#14 R008：安全基线。
- #44/#45/#47/#49：SourceSession、Web Display、Control、hosted Web 产品组合与重连。
- #66/#73/#79/#83/#85/#95/#97/#99/#101/#103/#105/#107/#109/#111/#114：generic-ytdlp 提取、锁定运行时、sandbox/broker/Secret 和有界兼容修复。
- #28 账号基础、#33/#75 Browser runtime、#71 navigation 基础可复用；有基础不等于当前要扩展这些功能。

## #146：执行准备

这是独立低权限环境/runbook 交付 Task，不复制 #67 的真实解析 Claim。Codex Cloud 经既有 SSH tx-node 编排；hosted Actions 承担可移植验证；live 准备/检查如实记录 external-codex/ssh。无 live B 站请求、无手机/Runner 恢复、无生产启用。

完成后只返回环境 authority 和可重现命令；不代表 B 站可达或可播放。

## #67：真实来源兼容性

保留同一 Issue 与全部历史。旧 R17 在手机上因 4xx 阻塞、未执行解析，旧 R16 FAIL 保留。R18 改变 target/Evidence 路由，R19 强制消费 GitHub-hosted 预编译产物，R20 要求验证编译路径/运行资产与独立消费者可执行性，不改写历史结果。

硬发布依赖：#146 Final Acceptance。正式发布时绑定其实际 host/runbook Evidence；保持精确 runtime Candidate，真实匿名 direct/no-proxy 前检与解析在新 Attempt 中独立运行。样本不可用则 BLOCKED，不从浏览器缓存/账号/抓包 URL 构造 PASS。换样本或改 runtime 必须先正式修订。

在 #68 现有 generic-ytdlp Contract 下，只有 #67 返回受支持 muxed http-file/HLS 并 Final Accepted PASS 才能发布 #68。这个条件不阻止独立 #165/#166 探索不同取源路径。4xx 本身不是 DASH 根因证据；新路线不能改写 #67 的结论，也不能未经修订直接用于 #68。

## Browser 取源研究分支

#165 是离线探针 combined Task，无 #67 硬依赖；portable build/test 在 GitHub-hosted Actions。#166 是独立实站 verification Task，依赖 #165 被接受的边界、artifact/runbook 和普通 Linux 准入，不依赖手机或 Native Panel。具体 task.md 是执行契约，Issue 是发布/owner authority。

#166 只有完整独立消费者 Claim PASS 后才允许据实拆分媒体能力/站点集成任务；不是现在承诺 remux 必需或可行。新 schema/API 改动先按设计变更流程评审。当前只提升服务端匿名取源研究优先级，不恢复 #27 Native Panel/Auth/phone capacity。

## #68：首个真实 Web 产品闭环

现有尚未发布 Contract 的硬依赖仍是 #67 Final Acceptance PASS + 有效 #49 authority。若采用 Browser 路线，Coordinator 必须先根据 #166 接受证据与媒体/插件实现正式替换 #68 Contract 的 source route、hard dependencies、shape、artifacts、Jobs 与 Freshness；完成 GitHub 读回/队列校验前保持 draft。复用产品 URL 输入、Registry、Session、媒体 capability、Display、Control 命令与重连，不允许 seed/store/ResolvedMedia 注入作为成功路径。

Source shape、#67 runtime、#68 Candidate 和 live browser/Gateway 拓扑在发布前冻结。无需物理手机或 TV，但不能因此宣称 TV audible autoplay 或生产就绪。默认生产 generic-ytdlp DisabledRunner 保留，测试组合显式启用。

## 暂缓路线

| 路线 | Issue | 重新进入条件 |
| --- | --- | --- |
| 手机运维/网络 | #142/#131/#113 | 用户恢复手机方向并重新满足其原 management gates |
| 物理 TV | #7 | 真设备与可达测试实例可用，独立安排 |
| 手机资源 | #9 | 手机方向恢复且代表性功能稳定 |
| Jellyfin TV | #16 | 可选显示需求与设备可用 |
| 连续内容 | #72 | #68 稳定后重新冻结实际 multipart 样本 |
| 登录/Native Panel | #26/#27 | #68 稳定且有明确产品需求 |
| 完整 Core feasibility | #22 | 所有原 required P0 Evidence 齐全；本阶段不降低 Gate |

## 遗留成果

#23/#36/#65 与 PR #37 是已处置历史，不重启旧 B 站 API authority。#128 已通过 PR #133/#134 接受；旧未合并 planning PR #129 应按 superseded 关闭，不回灌过时契约。

## 完成命名

#146 accepted = 普通 Linux 执行条件已证明。
#67 PASS = 真实 B 站解析兼容性已证明。
#68 accepted = 普通 Linux 上真实 Web 播放/控制闭环已证明。
#7/#9/#22 仍分别拥有物理 TV、手机资源、完整 Core Gate；本轮结果不替代它们。
