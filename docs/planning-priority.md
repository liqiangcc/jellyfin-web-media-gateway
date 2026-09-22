# Delivery Priority

当前用户目标（2026-09-22）：**本机 x86_64 VM 部署 Gateway，iPhone 经 Tailscale tailnet 访问 Control/Display；随后恢复 #68 完成真实 B 站 Web 播放闭环。** Live status/owner 属于 GitHub Issue。

## 当前顺序

1. 本机部署：GitHub-hosted Actions 构建精确 Candidate `gateway-server` artifact → 摘要/provenance 校验 → 本机绑定 tailnet 地址运行 → iPhone 访问 `/control`/`/display` 与直链播放链路验证。
2. #68 BILIBILI-WEB-E2E 恢复：续用 PR #278 修复已知编译/取消清理问题 → hosted required Jobs PASS → 一次性实站 J3 → 同一 Candidate 本机部署验收。本机由 Coordinator 正式命名为其 approved ordinary-Linux execution host 时须记录隔离边界。
3. #246 authorized Bilibili target：六项外部前置满足后可考虑在本机低权限边界执行（本机具备 Chrome/MCP）；仍 `status:blocked`。

#67 generic-ytdlp 保留 BLOCK 历史，不改写；浏览器取源路线（#237/#240/#243–#271）已接受。#147 等历史 CI 修复已关闭。

## 暂缓

- #142/#131/#113：手机管理面、Runner、手机站点复测；不做恢复或采样循环。
- #9：手机资源/容量；#7/#16：真实 TV/Jellyfin；均不阻塞当前功能阶段。iPhone tailnet 客户端访问不替代 TV/手机设备 Evidence。
- #72/#26/#27：等首播稳定，再根据用户能力需求选导航、登录或 Native Panel。
- #22：保留原完整 P0 Evidence 门槛，不能由桌面 Web 测试替代。

## 调度规则

Review 当前主线结果优先；前置 PASS 后立即填齐并发布下一层 Task。FAIL/BLOCKED 只产生真实 Evidence 要求的最小修复，不按主机/Runner 复制业务 Task。保持约 1–2 active、0–2 ready、1–3 下一层 draft；Worker 每次 Attempt 报告后 STOP，由 Coordinator Review/发布下一项。

详细计划见 [non-phone-web-playback-plan.md](non-phone-web-playback-plan.md)，当前能力与验收映射见 [product-roadmap.md](product-roadmap.md)。
