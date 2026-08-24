# Codex / Agent Working Rules

本文件定义在本仓库工作的长期 Agent 约束。它不是一次性任务提示词；阶段性任务入口位于 `docs/codex/`。

## 1. 开始任何任务前

先读取并遵守当前 canonical 文档：

1. `docs/README.md` — 文档权威层级
2. `docs/requirements.md` — WHAT
3. `docs/architecture.md` — CURRENT ARCHITECTURE
4. `docs/implementation-contracts.md` — CODING CONTRACTS
5. `docs/technical-feasibility-validation.md` — EVIDENCE / FEASIBILITY GATES
6. `docs/mvp-plan.md` — WHEN
7. `docs/security.md` — SECURITY INVARIANTS
8. 与当前任务直接相关的专题文档 / ADR

不要从旧聊天、README 摘要或已有代码猜测新的架构事实；发生冲突时按 `docs/README.md` 的权威层级处理。

## 2. 当前架构不变量

- Gateway 是 `PlaybackSession` authority。
- Jellyfin 只是可选 `DisplayAdapter`，不能反向定义 Core。
- Control 是统一体验层，不是第二份业务状态库。
- Gateway Core 可以识别 `site_id`，但不能理解具体站点 URL、Cookie、DOM、私有 API、清晰度枚举、下一集算法或登录成功规则。
- 第一版实现即通过 `SiteAdapterRegistry`；`generic-ytdlp` 也是 Site Plugin，Core 不允许直接 fallback 到 yt-dlp。
- `SourceLocator` 是插件拥有的版本化 opaque 内容定位契约；短期 CDN/HLS URL 不是内容身份。
- Site Browser Worker 是通用 Chromium runtime；具体站点语义由 Site Plugin 解释。
- Display Adapter 不读取 Session Vault。
- Site Plugin 不直接读取 Vault，也不能绕过 `EgressPolicy`。
- Native Site Panel 故障不得停止已经开始的 Gateway 播放。
- Jellyfin 故障不得阻塞 Web Display。
- MVP 面向可信 LAN / 单用户；这不等于可以关闭 Origin/CSRF、SSRF、Secret、token 等安全边界。

如果实现需要破坏以上任一不变量，先停止当前实现路径，记录证据并按设计变更流程修改文档；不要在代码里偷偷引入例外。

## 3. 风险验证优先于扩大实现

当前阶段采用 risk-driven feasibility validation。

只把真实实验、测试或可复现证据当作技术可行性结论。禁止把以下表述当作 PASS：

- “理论上支持”
- “应该可行”
- “看起来没问题”
- “预计不会有问题”

研究结果使用：

- `PASS`
- `CONDITIONAL PASS`
- `FAIL`
- `BLOCKED`

尚未执行的实验不得标记为通过。

实验失败是有效结果。优先 `Change / Defer / Drop` 可选能力，而不是为了让 Demo 成功绕过架构或安全边界。

## 4. 当前 P0 Gate

以 `docs/technical-feasibility-validation.md` 与 `docs/mvp-plan.md` 为准。当前优先级是：

```text
R007 Playback concurrency contract closure
→ R001 Media Path
→ R002 TV Browser remote audible playback / autoplay
→ R003 ARM64 resource baseline
→ R008 Egress / Secret baseline
→ Core Feasibility Review
```

- R004 Jellyfin PoC 可以并行，但失败不阻塞 Web-only Core。
- R005 Real Site 用于验证 Site Plugin Contract。
- R006 Site Browser Worker / Native Site Panel 非 Core blocker。

不要因为实现更有趣而跳过更高优先级 Gate。

## 5. 实现与实验规则

- PoC 可以小，但必须真实验证目标风险。
- 不为了 PoC 临时让 Core 直接调用具体站点或 yt-dlp。
- 不为了 PoC 关闭 SSRF、允许任意私网访问、下发 Cookie/Authorization 或建立开放代理。
- FFmpeg、yt-dlp、Chromium 等子进程必须使用 argv/参数数组，禁止 shell 字符串拼接。
- 实验代码与正式代码边界要清楚；未经验证的 PoC 不自动升级为产品实现。
- 新增具体站点的主要 diff 应位于 `plugins/<site>/`；如果需要修改 PlaybackCoordinator/DisplayAdapter/Control 中的站点业务分支，先做架构评审。
- 运行时第三方插件、动态 `.so`、插件市场、完整 Native Site Panel、完整 Jellyfin handoff 都不是首批 Core 实现前置条件。

## 6. 并发与状态规则

实现/修改 Playback 时必须覆盖：

- `request_id` 幂等；
- command revision/CAS；
- stale item callback；
- stale display generation callback；
- re-resolve race；
- handoff transition race；
- 多 Control 并发 mutation。

任何旧异步结果都不得覆盖已确认的新 `PlaybackItem`、`active_display` 或新媒体解析结果。

## 7. 测试要求

修改代码时，至少运行与改动对应的测试。优先维护：

- SiteAdapter conformance tests；
- ResolvedMedia schema / Secret boundary tests；
- SourceLocator version tests；
- Playback revision / stale callback tests；
- Display generation / handoff rollback tests；
- EgressPolicy / SSRF tests；
- Web Display 基线测试；
- 与当前 Research Item 对应的可复现实验。

不能运行的测试必须明确说明原因；不得把“未运行”写成“通过”。

## 8. 设计变更流程

真实证据推翻当前假设时，按以下顺序检查并更新：

```text
Evidence
→ requirements.md（如果产品目标/非目标改变）
→ architecture.md
→ implementation-contracts.md
→ technical-feasibility-validation.md
→ mvp-plan.md
→ security.md
→ 相关专题文档
→ 必要时 ADR
```

不要只修改一个专题文档或代码，让 canonical 文档继续漂移。

## 9. Git 工作方式

- 一个清晰研究/实现单元一个聚焦提交。
- 不把多个不相关 Spike 塞进一个巨大 commit。
- 提交信息说明意图，而不是只写 `update` / `fix`。
- 不重写用户已有历史，不 force push，除非任务明确要求。
- 提交前检查当前 diff 是否包含 Secret、账号信息、私有 Cookie、Token、完整敏感 URL 或实验产生的大文件。

## 10. 阶段任务入口

当前可直接执行的 Codex 任务：

- `docs/codex/technical-feasibility.md`

新会话推荐指令：

> 读取 `AGENTS.md`，然后按照 `docs/codex/technical-feasibility.md` 继续执行下一项。
