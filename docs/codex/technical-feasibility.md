# Codex Task — Technical Feasibility Validation

继续 `liqiangcc/jellyfin-web-media-gateway` 的技术预研与可行性验证。

本文件是**外部 Codex Worker fallback 入口**，不是默认任务调度器。

默认执行路径：

```text
Web Worker implementation
→ GitHub Actions
     ├── GitHub-hosted x64 portable verification
     ├── GitHub-hosted ARM64 generic ARM64 verification
     └── Ubuntu ARM64 Target Runner phone-specific proof
→ External Codex only for interactive capability
→ Real TV / Manual physical UX proof
```

Cloud 不部署 Runner。

如果已有 GitHub Issue + `docs/tasks/<issue>-<slug>/task.md`，必须优先执行该 Task。

## 0. External Worker Routing Gate

没有明确 Issue/Task 时，先判断：

> 当前最高优先级工作是否真的需要当前外部环境提供 GitHub Actions / Target Runner / Manual 无法提供的交互 capability？

如果不是：

- 不因为已经打开 Codex 就抢占任务；
- 不把 Web/Actions 工作改成外部 Codex 工作；
- 停止并交回 Web Coordinator。

典型外部 capability：

```text
WSL: interactive-linux-debug / local process interaction
Windows: adb / android-device-control
Cloud: cloud-specific reproduction / persistent interactive state / Tailscale orchestration
Ubuntu ARM64 Codex: target interactive debug / device recovery
Real TV: manual physical observation
```

Cloud **不是普通 long-running 自动验证后端**。

## 1. 开始前

读取：

1. `AGENTS.md`
2. `docs/README.md`
3. `docs/requirements.md`
4. `docs/architecture.md`
5. `docs/implementation-contracts.md`
6. `docs/technical-feasibility-validation.md`
7. `docs/mvp-plan.md`
8. `docs/security.md`
9. `docs/development-environments.md`
10. `docs/runner-execution-architecture.md`
11. 当前 Research Item 相关专题文档 / ADR

检查 Git 状态、最近提交、Issue/Task、Research 状态和 Evidence。

## 2. 当前目标

若没有明确 Task，并且 Routing Gate 确认当前环境确实提供所需交互 capability，再从 Research Matrix 选择当前最高优先级、未完成、前置满足且需要当前环境的工作。

P0：

```text
R007
→ R001
→ R002
→ R003
→ R008
→ Core Feasibility Review
```

路由规则：

- R007 contract/test authoring → Web；portable suite → GitHub-hosted x64；generic ARM64 regression → GitHub-hosted ARM64；大量 repeated race → hosted matrix/sharding；只有交互 debug → WSL。
- R001 implementation → Web；portable MP4/HLS → hosted x64；generic ARM64 → hosted ARM64；phone-specific media path/FFmpeg/resource → Ubuntu ARM64 Target Runner；交互 target debug 才用 ARM64 Codex。
- R002 最终 audible autoplay / remote UX → Real TV Manual。
- R003 harness/scripts → Web；generic ARM64 harness → hosted ARM64；CPU/RSS/temp/target throughput → Ubuntu ARM64 Target Runner。
- R008 portable security suite → hosted x64/ARM64；环境相关边界按 capability 路由。

## 3. Implementation / Verification / Gate

```text
Implementation
→ Candidate SHA

Verification
→ Candidate SHA + Claims + Required Capabilities
→ actual Execution Plane / Runner / Target
→ Evidence

Coordinator
→ Gate Decision
```

外部 Worker 不顺手扩大 Scope。

## 4. 执行方式

对明确分派的 Research / Verification：

1. 明确 Hypothesis / Claims。
2. 检查预先定义 Success Criteria。
3. 记录 Candidate SHA。
4. 确认当前环境正好提供 Required Capability。
5. 实际执行需要的测试/设备操作。
6. 收集 Evidence / Metrics。
7. 覆盖失败路径。
8. 给出 `PASS / CONDITIONAL PASS / FAIL / BLOCKED`。
9. 给出 `Continue / Change / Defer / Drop`。
10. 更新 Evidence；若推翻契约，走设计变更流程。
11. 提交当前 Scope。
12. Issue → `status:review` 后停止。

## 5. Evidence

至少记录：

```text
Role: verification
Orchestrator:
Execution Plane:
Executor / Runner class:
Runner image / labels:
Execution host:
Target:
OS / architecture:
Relevant versions:
Network path:
Candidate commit:
Workflow / run / job:
Commands / steps:
Duration / repetitions / shards:
Metrics / artifacts:
Result:
```

禁止用“理论上”“看代码没问题”代替真实 Evidence。

GitHub-hosted generic ARM64 不能冒充目标手机；Cloud/WSL 不能冒充手机温度/真实 TV。

## 6. 安全边界

任何 Spike 都不得：

- Core 直接调用具体站点或 yt-dlp；
- 绕过 SiteAdapterRegistry；
- 关闭 SSRF / Egress；
- 将任意 private URL 当普通来源；
- 把 Cookie/Authorization/bearer token 下发 Display；
- 暴露 Vault/Profile；
- 做 arbitrary open proxy；
- shell 拼接 FFmpeg/yt-dlp/Chromium；
- 让 Jellyfin/浏览器/站点 Chromium 覆盖 Gateway `PlaybackSession` authority。

Target Runner 还必须遵守 `runner-execution-architecture.md` / `security.md`：低权限、受信 Candidate、Vault 隔离、独立 test runtime。

## 7. R007 最低要求

闭合并验证：

- session revision 与高频 position telemetry；
- re-resolve stale result；
- handoff candidate generation / transition；
- duplicate request_id；
- stale expected revision；
- stale item callback；
- stale display generation；
- overlapping handoff；
- two-Control mutation。

推荐：

```text
Web authoring
→ hosted x64 suite
→ hosted matrix/sharding for repeated race
→ WSL only for interactive diagnosis
```

## 8. R001 最低要求

链路保持：

```text
Test Source
→ SiteAdapterRegistry
→ SourceLocator
→ SiteAdapter.resolve
→ ResolvedMedia
→ Media Gateway
→ Web Display
```

至少验证 MP4/HLS、pause/play、seek、Range/segment、relative/redirect、Secret boundary、Jellyfin-off、non-open-proxy。

通用 ARM64 compile/test 优先 GitHub-hosted ARM64；目标手机特性才进 Target Runner。

## 9. R002 最低要求

最终真实 TV 区分：

- muted autoplay；
- audible autoplay；
- 无 user gesture；
- 一次初始化后；
- 播放结束后；
- refresh/restart/sleep recovery。

桌面/云/模拟器不能替代最终 TV Gate。

## 10. R003 最低要求

最终目标手机记录：

- CPU；
- RSS；
- temperature；
- bandwidth；
- startup latency；
- sustained duration；
- error/recovery。

场景至少：idle、direct proxy、remux、Chromium baseline（适用时）。

这些数据必须来自 Ubuntu ARM64 Target Runner 或等价明确 target execution，不得用 hosted ARM64 冒充。

## 11. R008 最低要求

验证：

- loopback/private/link-local/metadata 拒绝；
- public redirect → private 拒绝；
- configured local service 只访问配置目标；
- media token 跨 session/item 重放失败；
- ResolvedMedia Secret header 被拒绝；
- Display 看不到 Cookie/Authorization；
- plugin cross-site access 被拒绝。

## 12. 完成报告

只报告：

1. Research/Task。
2. PASS / CONDITIONAL PASS / FAIL / BLOCKED。
3. Candidate SHA。
4. 实际 Execution Plane / Runner / Target。
5. 关键 Evidence / Metrics。
6. 架构/MVP 是否改变。
7. 测试结果。
8. commit/PR。
9. 未验证范围。

完成当前 Scope 后停止，不自动开始下一项。
