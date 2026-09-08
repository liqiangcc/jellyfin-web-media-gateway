# Delivery Priority

当前用户目标（2026-09-08）：**暂不部署手机，先完成普通 Linux 上真实 B 站 Web 播放闭环。** Live status/owner 属于 GitHub Issue。

## 当前顺序

1. #146 NON-PHONE-EXECUTION-PREP：交付低权限 tx-node 执行、锁定 runtime、浏览器受控访问与可复现 runbook；无真实 B 站实验。
2. #67 GENERIC-YTDLP-BILIBILI-REAL：在 #146 Final Acceptance 后发布修订契约，证明受控真实解析。
3. #68 BILIBILI-WEB-E2E：#67 Final Acceptance PASS 后冻结 source shape/Candidate 并发布，完成产品播放与 Control 闭环。

#147 CI-NAVIGATION-WORKFLOW-REPAIR 无硬业务依赖，可独立执行；它修复已观察到的无 job workflow failure，不扩展为通用 CI 平台。

## 暂缓

- #142/#131/#113：手机管理面、Runner、手机站点复测；不做恢复或采样循环。
- #9：手机资源/容量；#7/#16：真实 TV/Jellyfin；均不阻塞当前功能阶段。
- #72/#26/#27：等首播稳定，再根据用户能力需求选导航、登录或 Native Panel。
- #22：保留原完整 P0 Evidence 门槛，不能由桌面 Web 测试替代。

## 调度规则

Review 当前主线结果优先；前置 PASS 后立即填齐并发布下一层 Task。FAIL/BLOCKED 只产生真实 Evidence 要求的最小修复，不按主机/Runner 复制业务 Task。保持约 1–2 active、0–2 ready、1–3 下一层 draft；Worker 每次 Attempt 报告后 STOP，由 Coordinator Review/发布下一项。

详细计划见 [non-phone-web-playback-plan.md](non-phone-web-playback-plan.md)，当前能力与验收映射见 [product-roadmap.md](product-roadmap.md)。
