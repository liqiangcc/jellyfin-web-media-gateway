# Product Delivery Roadmap

当前具体 Task sequencing authority，更新于 2026-09-08。用户决定暂缓手机部署，优先交付普通 Linux 上真实 Web 播放。本文不覆盖 requirements/architecture/contracts/security，不保存实时 owner；Issue 是状态 authority。

## 当前可交付里程碑

```text
#146 普通 Linux 执行准备
  → #67 真实 B 站受控解析
    → #68 Control → Gateway → Web Display 播放/控制/重连

#147 已知 navigation workflow 修复（独立 portable CI）
```

完整工作拆分、风险、工作日估算和 Job/验收矩阵见 [推进计划](non-phone-web-playback-plan.md)。恢复整个路线的 Coordinator 入口见 [non-phone-delivery handoff](tasks/handoffs/non-phone-delivery.md)。

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

只有完整路径返回当前支持的 muxed http-file/HLS 并 Final Accepted PASS，才能发布 #68。DASH/分离音视频等真实缺口只触发最小格式能力评审。

## #68：首个真实 Web 产品闭环

硬发布依赖：#67 Final Acceptance PASS + 有效 #49 authority。复用产品 URL 输入、Registry、Session、媒体 capability、Display、Control 命令与重连，不允许 seed/store/ResolvedMedia 注入作为成功路径。

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
