# Codex Task — Technical Feasibility Validation

继续 `liqiangcc/jellyfin-web-media-gateway` 的技术预研与可行性验证。

本文件是**外部 Codex Worker 的阶段性 fallback 入口**，不是默认任务调度器。

项目执行优先级：

```text
Web Worker implementation
→ GitHub Actions automated verification
→ Cloud long-running verification
→ WSL / Windows interactive capability
→ Ubuntu ARM64 / Real TV target proof
```

如果已有明确 GitHub Issue + `docs/tasks/<issue>-<slug>/task.md`，必须优先执行该 Task，不使用本文件自行选择其他工作。

## 0. External Worker Routing Gate

没有明确 Issue/Task 时，先判断：

> 当前最高优先级未完成工作是否真的需要当前外部环境提供 Web + Actions/Cloud 无法覆盖的 capability？

如果答案是否定的：

- 不因为已经打开 Codex 会话就抢占该任务；
- 不自行把 Web/Actions 工作改成外部 Codex 工作；
- 停止并把任务留给 Web Coordinator 路由。

典型外部 capability：

```text
WSL: interactive-linux-debug / local process interaction
Windows: adb / android-device-control
Ubuntu ARM64: arm64-runtime / device-metrics / thermal-metrics
Real TV: tv-browser / remote-control / manual-observation
```

Cloud 如果只是普通长时间自动执行，应优先被视为 long-running backend；只有当前 Cloud Worker 已被明确分派对应 Task 时才执行。

## 1. 开始前

先读取并遵守：

1. `AGENTS.md`
2. `docs/README.md`
3. `docs/requirements.md`
4. `docs/architecture.md`
5. `docs/implementation-contracts.md`
6. `docs/technical-feasibility-validation.md`
7. `docs/mvp-plan.md`
8. `docs/security.md`
9. `docs/development-environments.md`
10. 当前 Research Item 直接相关的专题文档 / ADR

先检查 Git 状态、最近提交、已有 Research 状态、Issue/Task 和 Evidence。不要根据旧聊天假设仓库状态。

## 2. 当前目标

如果没有明确 Issue/Task，并且 Routing Gate 确认当前外部环境确实提供所需 capability，再从 `docs/technical-feasibility-validation.md` 的 Research Matrix 中选择：

> 当前最高优先级、尚未完成、前置条件满足、并且需要当前环境 capability 的 Research 工作。

P0 顺序：

```text
R007 Playback concurrency contract closure
→ R001 Media Path
→ R002 TV Browser remote audible playback / autoplay
→ R003 ARM64 resource baseline
→ R008 Egress / Secret baseline
→ Core Feasibility Review
```

注意：

- R007 contract/design/test authoring 应优先 Web；portable automated tests 应优先 Actions。只有需要 interactive debug 时才进入 WSL。
- R001 portable build/test 优先 Web + Actions；长时间稳定性可 Cloud；目标 ARM64 数据另行 target verification。
- R002 最终 audible autoplay / remote UX 必须真实 TV。
- R003 最终资源数据必须 Ubuntu ARM64；脚本/工具实现仍优先 Web。
- R008 portable security suite 优先 Web + Actions；环境相关边界按 Required Capability 路由。
- R004 Jellyfin 可以并行，但不是 Web-only Core blocker。
- R005 Real Site 用于验证 Site Plugin Contract。
- R006 Site Browser Worker / Native Site Panel 非 Core blocker。

如果某项已有有效 PASS Evidence，不重复执行。

## 3. Implementation 与 Verification

不要把“写了代码”和“证明了 claim”混成一个结果。

```text
Implementation
→ Candidate commit

Verification
→ Candidate SHA + Claims + Required Capabilities
→ actual Executor / Target
→ Evidence

Coordinator
→ Gate Decision
```

当前外部 Worker 如果只负责 Verification，不应顺手扩大实现 Scope；如果发现必须修改实现，记录 blocker/follow-up 或按 Task 契约处理。

## 4. 执行方式

对当前被明确分派的 Research/Verification 工作：

1. 明确 Hypothesis / Claims to Verify。
2. 检查实验前定义的 Success Criteria，禁止事后降低标准。
3. 记录 Candidate commit SHA。
4. 确认当前环境正好提供 Required Capabilities。
5. 实际运行需要的代码、测试、媒体链路或设备验证。
6. 收集 Evidence / Metrics。
7. 覆盖必要失败路径。
8. 根据证据给出：`PASS / CONDITIONAL PASS / FAIL / BLOCKED`。
9. 给出 Architecture Decision：`Continue / Change / Defer / Drop`。
10. 更新对应 Research Evidence；如果证据推翻契约，走 `AGENTS.md` 设计变更流程。
11. 提交当前 Scope 的有效修改/Evidence。
12. 转 `status:review` 后停止，不自动开始下一项。

## 5. Evidence 规则

有效结论必须来自实际执行。

Evidence 至少记录：

```text
Role: verification
Orchestrator:
Executor:
Execution host:
Target host/device:
OS / architecture:
Relevant versions:
Network path:
Candidate commit:
Commands / steps:
Metrics / artifacts / raw evidence:
Result:
```

如果 Cloud/Windows/WSL 远程在目标设备执行，必须记录真正的 Execution host / Target。

禁止把以下文字当完成证据：

- 理论上支持
- 应该可以
- 根据文档推测可行
- 看代码没问题
- 预计目标设备支持

无法运行真实实验则 `BLOCKED`，并写清缺失环境、权限、设备、网络或版本。

## 6. 安全与架构边界

任何 Spike 都不得通过以下方式成功：

- Core 直接调用具体站点代码或 yt-dlp；
- Site Plugin 绕过 `SiteAdapterRegistry`；
- 关闭 SSRF / Egress 检查；
- 将任意 private URL 当普通用户媒体来源；
- 把 Cookie / Authorization / bearer token 下发给 Display；
- 直接向浏览器暴露 Vault/Profile；
- 将 Media Gateway 做成 arbitrary open proxy；
- shell 字符串拼接调用 FFmpeg / yt-dlp / Chromium；
- 让 Jellyfin、浏览器 `<video>` 或站点 Chromium 覆盖 Gateway `PlaybackSession` authority。

PoC 只有破坏这些边界才能工作时，记录 FAIL/架构问题。

## 7. R007 特别要求

R007 至少闭合并测试：

### session revision

明确高频 position telemetry 是否推进 command CAS revision，避免正常 pause/seek 因进度上报持续 `REVISION_CONFLICT`。

### re-resolve

明确同一 `SourceLocator` 因媒体 URL 过期重新 resolve 时的 generation/revision 语义，保证旧异步 resolve 不能覆盖新 `ResolvedMedia`。

### handoff transition

明确 target 已启动但 `active_display` 尚未 commit 期间的 callback/generation 语义；旧 Display/旧 transition 回调不得成为 authority。

### minimum tests

至少覆盖：

- duplicate `request_id`；
- stale expected revision；
- stale item callback；
- stale re-resolve result；
- stale display generation；
- overlapping handoff；
- two-Control concurrent mutation。

优先路由：

```text
contract + test authoring → Web Worker
automated concurrency suite → GitHub Actions
large repeated race → Cloud
interactive failure debug → WSL
```

## 8. R001 特别要求

最小链路保持：

```text
Test Source
→ SiteAdapterRegistry
→ SourceLocator
→ SiteAdapter.resolve
→ ResolvedMedia
→ Media Gateway
→ Web Display
```

至少验证：

- HTTP MP4；
- HLS；
- pause/play；
- seek；
- Range；
- relative/redirect URL；
- Display 不获得上游 Secret；
- Jellyfin 关闭时链路仍成立；
- Gateway 不是 arbitrary proxy。

DASH 环境不足可以明确留后续子项，不能无说明宣称覆盖。

## 9. R002 特别要求

最终验证必须区分：

- muted autoplay；
- audible autoplay；
- 从未 user gesture；
- 首次确认键/点击后；
- 播放结束后再次远程播放；
- 页面刷新后；
- 浏览器/电视恢复后。

核心场景：

```text
TV opens /display
→ waits idle
→ Control remotely sends playback
→ audible media starts or gives explicit recoverable action
```

真实 TV Evidence 不得由桌面浏览器、Cloud 或模拟器替代。

## 10. R003 特别要求

最终目标环境必须记录：

- CPU；
- RSS / memory；
- temperature（可读时）；
- bandwidth；
- startup latency；
- sustained duration；
- error/recovery。

区分：

- idle；
- direct proxy；
- remux；
- Chromium baseline。

关键路径覆盖 5/30/60 分钟；更长 soak 可以用 Cloud orchestrate，但指标必须来自真实 ARM64 Target 才能证明 R003。

## 11. R008 特别要求

至少验证：

- loopback/private/link-local/metadata 拒绝；
- public redirect → private 拒绝；
- configured Jellyfin local service 只能访问配置目标；
- media token 跨 session/item 重放失败；
- ResolvedMedia Secret header 被拒绝；
- Display 看不到 Cookie/Authorization；
- plugin cross-site access 被拒绝。

portable security suite 优先 Actions；不能为了验证方便关闭安全边界。

## 12. 文档与提交

完成一个 Research/Verification Task 后至少检查：

- `docs/technical-feasibility-validation.md`
- 对应 Research Evidence / Metrics / Result / Decision
- `docs/mvp-plan.md` Gate 状态（如实际进度变化）

只有证据确实改变架构时才修改：

- `docs/requirements.md`
- `docs/architecture.md`
- `docs/implementation-contracts.md`
- `docs/security.md`
- ADR

提交要求：

- 一个 Task 一个聚焦 commit/PR；
- 不提交 Secret/Cookie/Token/私有账号数据/不必要大文件；
- 不 force push；
- 测试失败就记录真实失败，不伪装成功。

## 13. 完成后报告

最终只报告：

1. 执行的 Issue / Task / Research Item。
2. 当前 Worker 为什么具备所需 capability。
3. Candidate SHA。
4. `PASS / CONDITIONAL PASS / FAIL / BLOCKED`。
5. 真实 Orchestrator / Executor / Target。
6. 最关键 Evidence / Metrics。
7. 实际运行的测试及结果。
8. commit / PR。
9. 是否改变架构或 MVP 范围。

完成后停止；下一任务由 Web Coordinator 决定。
