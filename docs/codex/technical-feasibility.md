# Codex Task — Technical Feasibility Validation

继续 `liqiangcc/jellyfin-web-media-gateway` 的技术预研与可行性验证。

本任务必须实际读取和修改当前仓库，不要仅给建议或生成一份聊天报告。

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
9. 当前 Research Item 直接相关的专题文档 / ADR

先检查当前 Git 状态、最近提交、已有 Research 状态和 Evidence。不要根据旧聊天假设仓库状态。

## 2. 当前目标

从 `docs/technical-feasibility-validation.md` 的 Research Matrix 中选择：

> 当前最高优先级、尚未完成、且前置条件已经满足的 Research Item。

当前 P0 顺序原则：

```text
R007 Playback concurrency contract closure
→ R001 Media Path
→ R002 TV Browser remote audible playback / autoplay
→ R003 ARM64 resource baseline
→ R008 Egress / Secret baseline
→ Core Feasibility Review
```

说明：

- 如果某项已经有有效 PASS Evidence，不重复执行。
- 如果某项已是 CONDITIONAL PASS，先判断其条件是否已经满足/需要补证据，再决定是否继续。
- 如果真实设备、外部依赖或测试环境缺失，记录 `BLOCKED` 和准确缺口；不得伪造实验结果。
- R004 Jellyfin 可以并行，但不是 Web-only Core blocker。
- R005 Real Site 用于验证 Site Plugin Contract。
- R006 Site Browser Worker / Native Site Panel 非 Core blocker。

除非当前最高优先级项目被真实阻塞，否则不要提前进入后续 Research Item。

## 3. 执行方式

对选中的 Research Item：

1. 明确当前 Hypothesis。
2. 检查实验开始前定义的 Success Criteria，禁止实验后降低标准来制造 PASS。
3. 设计/复用最小可复现实验。
4. 实际运行代码、测试、浏览器、媒体链路或真实设备验证（按该 Research Item 需要）。
5. 收集 Evidence 和 Metrics。
6. 覆盖必要失败路径，不只验证 happy path。
7. 根据证据给出：
   - `PASS`
   - `CONDITIONAL PASS`
   - `FAIL`
   - `BLOCKED`
8. 给出 Architecture Decision：
   - `Continue`
   - `Change`
   - `Defer`
   - `Drop`
9. 更新对应研究文档/矩阵/证据。
10. 如果证据推翻现有契约，按 `AGENTS.md` 的设计变更流程同步 canonical 文档；不要只改 PoC。
11. 运行与改动对应的测试。
12. 提交本 Research Item 的所有有效修改。

## 4. 证据规则

有效结论必须来自实际执行结果。

禁止将以下文字当作完成证据：

- 理论上支持
- 应该可以
- 根据文档推测可行
- 看代码没问题
- 预计目标设备支持

文档/源码分析可以帮助设计实验，但不能代替需要真实运行时验证的外部行为。

如果无法运行真实实验：

```text
Status = BLOCKED
```

并记录：

- 缺失什么环境；
- 为什么现有环境无法替代；
- 已完成哪些不依赖该环境的准备工作；
- 恢复执行所需的最小输入/设备/配置。

不要为了避免 BLOCKED 而把模拟结果写成真实设备 PASS。

## 5. 安全与架构边界

任何 Spike 都不得通过以下方式获得成功：

- Core 直接调用具体站点代码或 yt-dlp；
- Site Plugin 绕过 `SiteAdapterRegistry`；
- 关闭 SSRF / Egress 检查；
- 将任意 private network URL 当普通用户媒体来源；
- 把 Cookie / Authorization / bearer token 下发给 Display；
- 直接向浏览器暴露 Vault/Profile；
- 将 Media Gateway 做成 arbitrary open proxy；
- shell 字符串拼接调用 FFmpeg / yt-dlp / Chromium；
- 让 Jellyfin、浏览器 `<video>` 或站点 Chromium 覆盖 Gateway `PlaybackSession` authority。

PoC 如果只有破坏这些边界才能工作，应记录 FAIL/架构问题，而不是提交绕过方案。

## 6. R007 特别要求

如果当前项是 R007，至少闭合并测试：

### session revision

明确高频 position telemetry 是否推进 command CAS revision，避免正常 pause/seek 因进度上报持续 `REVISION_CONFLICT`。

### re-resolve

明确同一 `SourceLocator` 因媒体 URL 过期重新 resolve 时的 generation/revision 语义，保证旧异步 resolve 不能覆盖新 `ResolvedMedia`。

### handoff transition

明确 target 已启动但 `active_display` 尚未 commit 期间的 callback/generation 语义；旧 Display/旧 transition 的回调不得成为 authority。

### minimum tests

至少覆盖：

- duplicate `request_id`；
- stale expected revision；
- stale item callback；
- stale re-resolve result；
- stale display generation；
- overlapping handoff；
- two-Control concurrent mutation。

如果这要求修改 `implementation-contracts.md`，先把契约收敛，再实现测试骨架。

## 7. R001 特别要求

如果当前项是 R001，最小链路必须保持：

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
- relative/redirect URL（适用时）；
- Display 不获得上游 Secret；
- Jellyfin 关闭时链路仍成立；
- Gateway 不是 arbitrary proxy。

DASH 如果本轮环境不足可以作为明确后续子项，但不能无说明地宣称已覆盖。

## 8. R002 特别要求

如果当前项是 R002，必须区分：

- muted autoplay；
- audible autoplay；
- 从未 user gesture；
- 首次确认键/点击后；
- 播放结束后再次远程播放；
- 页面刷新后；
- 浏览器/电视恢复后。

核心用户场景是：

```text
TV opens /display
→ waits idle
→ Control remotely sends playback
→ audible media starts or gives explicit recoverable action
```

如果目标电视要求一次用户初始化，可以给 CONDITIONAL PASS；如果每次远程播放都要求操作电视，应记录严重 UX 限制而不是弱化问题。

## 9. R003 特别要求

如果当前项是 R003，必须在目标 Ubuntu ARM64 环境记录可比较的：

- CPU；
- RSS / memory；
- temperature（设备可读取时）；
- bandwidth；
- startup latency；
- sustained duration；
- error/recovery。

场景至少区分：

- idle；
- direct proxy；
- remux；
- Chromium 基础成本（如果环境已有）。

长期链路不要只跑数秒；按研究文档要求执行短、中、长时间样本。软件转码仅用于边界测量，不得因此变成 MVP 默认路径。

## 10. R008 特别要求

R008 不是“最后补安全”。从 R001 开始所有实验已经必须遵守这些边界。

最终至少验证：

- loopback/private/link-local/metadata 拒绝；
- public redirect → private 拒绝；
- configured Jellyfin local service 只能访问配置目标；
- media token 跨 session/item 重放失败；
- ResolvedMedia Secret header 被拒绝；
- Display 看不到 Cookie/Authorization；
- plugin cross-site access 被拒绝。

## 11. 文档更新

完成一个 Research Item 后，至少检查并按需要更新：

- `docs/technical-feasibility-validation.md`
- 对应 Research 状态 / Evidence / Metrics / Result / Architecture Decision
- `docs/mvp-plan.md` 的 Gate 状态（如实际进度发生变化）

只有证据确实改变架构时才修改：

- `docs/requirements.md`
- `docs/architecture.md`
- `docs/implementation-contracts.md`
- `docs/security.md`
- ADR

不要为了“看起来有产出”无意义改动 canonical 文档。

## 12. 提交要求

- 一个 Research Item 使用聚焦 commit。
- 提交前检查 diff 不含 Secret、Cookie、Token、私有账号数据或不应进入仓库的大型实验文件。
- 不 force push，不重写已有历史。
- 如果测试失败，不提交把失败伪装成成功的结果；应修复或把真实失败写入 Evidence。

建议 commit message：

```text
research: close playback concurrency contracts
research: validate baseline media path
research: validate tv browser remote playback
research: record arm64 resource baseline
research: validate egress and secret boundaries
```

## 13. 完成后报告

最终回复保持简洁，只报告：

1. 本轮执行的 Research Item。
2. `PASS / CONDITIONAL PASS / FAIL / BLOCKED`。
3. 最关键 Evidence / Metrics。
4. 是否改变架构或 MVP 范围。
5. 实际运行的测试及结果。
6. commit SHA。
7. 下一个最高优先级 Research Item。

不要提前执行下一项；等待下一轮继续。
