# Task — BILIBILI-BROWSER-PROBE-PREP

## Metadata

```text
GitHub Issue: #165
Parent Goal / Research Item: #68 / R005 public source with R006 runtime and R008 boundary
Task ID: BILIBILI-BROWSER-PROBE-PREP
Task kind: combined
Base commit: c754aa9ff598ce74ffc0d765bd12d730c4f5d4e7
Candidate commit: n/a; live Candidate is owned by Issue/Attempt
Session bootstrap prompt: docs/tasks/165-bilibili-browser-probe-prep/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test
Hard publication dependencies: none
```

## Goal / Context

交付隔离的 **offline browser acquisition probe**，用 synthetic 页面证明安全观察、站点解释和独立服务端读取链路，并产出可被后续 #166 核验/消费的 hosted artifact 与有界 live runbook。#67 的 generic-ytdlp 失败不是本任务依赖。读取 `docs/research/bilibili-browser-acquisition.md`、`docs/site-plugin-architecture.md`、`docs/security.md`、Browser/ResolvedMedia contracts 与全部 AGENTS 启动文档。

## Task Decomposition / Routing

Verification mode: inline for portable synthetic claims；separate-task #166 owns live-site compatibility. #165 完成不依赖 live PASS。普通代码/CI 路由 Codex Cloud；Execution plane github-actions，github-hosted-x64。无 target/physical proof。Job 不 claim Issue。

## In Scope / Implementation Requirements

1. 在 `experiments/bilibili-browser-probe/` 实现可取消的受限实验入口；站点字段解释放 `plugins/bilibili/` 的实验模块并明确非生产 adapter。复用 Browser/R008 能力，缺少的观察 DTO 仅限实验，不能静默扩展生产 API。
2. 输入是部署者冻结的 synthetic selector 或后续 runbook 的公开内容 selector；不接收 Control 提供的任意 CDP endpoint、headers、profile、upstream URL、Egress scope。内容定位独立于短期媒体 URL；part 正确性有测试。
3. 通用 runtime 负责观察/生命周期，插件负责页面字段语义。限制允许的观察字段、事件数、响应大小、资源数、超时。禁止任意页面脚本从 UI 注入、完整 DOM/playinfo/HAR dump。
4. 真实 Chromium synthetic tests 证明 probe 的有效出站隔离：顶层和子资源、redirect、DNS 结果/连接地址一致性、worker/SW、WebSocket、QUIC 或其它直连出口。每个出口必须经过受控连接或被强制禁用；仅 CDP authorize + continue 不算证明。允许受限测试网络内精确 fixture endpoint，不放宽生产私网策略。记录 enforcement 设计与负例；缺少强制边界则报告 BLOCKED，不提供可运行 live 模式。
5. 全新匿名临时 profile；无 Cookie/Authorization/Vault/profile 转移。解析出的短期 URL 只保留服务端有界内存；导出仅 schema version、part 匹配布尔、候选计数、role、codec/container、HTTP status class、Range 支持、非敏感 header 名称/allowlist decision、有效期线索类别、预算/清理结果。未知字段明确 unknown。日志错误路径同样脱敏。
6. 独立消费者通过 R008 读取 fixture 媒体（包括 browser exit 后）；负例覆盖 blob-only、AV-separated、muxed、错 part、缺失/畸形字段、过期/403、只浏览器上下文可用、Secret header、重定向私网、超量/超时/取消。
7. 产出 GitHub-hosted 构建 artifact manifest：exact source SHA、runtime/browser 依赖版本、文件 digest/size、平台、入口。独立 job 下载校验后在无编译路径运行；禁止只验证 build workspace 的副本。若 browser 需目标预装，显式冻结版本/路径及 admission 检查。
8. 编写 #166 live runbook（本 Task 不执行）：低权限普通 Linux；最多 2 个 clean sessions、每个导航 120s、最多 200 网络请求且总响应预算 32 MiB/session、metadata 上限 1 MiB；后续独立媒体读取总共最多 8 次请求、单次最多 1 MiB、累计最多 4 MiB（按先触及上限停止）。所有响应实际字节计费，redirect 计请求，超限自动停止；不调用站点视频 play 触发整段预载。artifact identity、用户、命令在 #166 发布时冻结；cleanup 可重入，不触碰现有服务。

## Out of Scope / Invariants

无 live Bilibili 请求、生产 adapter 注册、真实登录、媒体抓屏、DRM/权限绕过、remux/播放器实现、Playback/Control 功能修改。Browser 是通用 runtime；Core 不理解站点。Gateway authority、R008、Secret/Origin/CSRF 不变。若需要变更 canonical contract/安全前提，保留 Evidence 并交 Coordinator 先完成设计流程。

## Files Expected to Change

`experiments/bilibili-browser-probe/`、`plugins/bilibili/` 实验解释器、`.github/workflows/bilibili-browser-probe.yml`、`docs/research/bilibili-browser-probe-runbook.md`。必要的 Cargo/workspace wiring 可改；复用生产库而非改其语义。越出此边界先报告具体设计缺口。

## Verification Plan / Success Criteria

| Job | Claims / PASS 条件 | Plane / Runner / Target | Selector | Evidence |
| --- | --- | --- | --- | --- |
| J1 | C1：synthetic muxed/AV/part/错误数据解析、schema 限额及泄漏负例全部通过 | github-actions / hosted x64 / synthetic | 新 workflow job `probe-contract` | exact Candidate run/job、脱敏测试结果 |
| J2 | C2：真实 Chromium 下所有声明网络出口被 broker/隔离策略限制或禁用，取消/崩溃/profile cleanup 负例通过 | github-actions / hosted x64 / isolated Chromium fixtures | `probe-containment` | enforcement matrix + denial/cleanup evidence；不可用不是 PASS |
| J3 | C3：独立 job 校验下载 artifact，在无编译环境运行 synthetic 独立消费者，browser exit 后仍能读取；预算自动停止 | github-actions / hosted x64 / artifact consumer | `probe-artifact-consumer` | artifact ID/digest/manifest、字节/请求计数 |
| JI1 | 集成面 compile/test/安全回归通过 | github-actions / hosted x64 / workspace | `probe-integration`：cargo fmt --all -- --check；cargo test --workspace；cargo clippy --workspace --all-targets -- -D warnings | run/job |

所有 job 必须 checkout/核验同一 Candidate SHA；workflow 名称/selector 在本 Task 冻结，Worker 编写实际命令并留 runbook。J1–J3 每 job timeout ≤15min，JI1 ≤35min，浏览器单例、子进程/内存/时间限制与 finally cleanup。无 need for WSL/phone/TV。Task success：C1–C3、JI1 全 PASS、reviewable Candidate/PR、artifact 和 runbook 齐全；不要求 #166 PASS。

Failure：行为/边界断言失败是 FAIL；缺少执行能力或需要未批准设计变更为 BLOCKED，保留 Candidate/证据，在同一 Issue 迭代。不能删负例降低标准。

## Freshness / Integration Contract

Freshness policy: dependency-aware

Semantic authorities: #14 R008、#33/#75 Browser contracts；docs/security.md、implementation-contracts.md、site-plugin-architecture.md；本实验契约。
Semantic freshness domains: Browser lifecycle/event API、R008 DNS/connection/redirect/Secret policy、实验观察 schema。
Integration surfaces: Cargo.toml/Cargo.lock、复用 crate API、hosted browser/workflow image。
Task-owned surfaces: 上述实验目录/解释器、新 workflow、runbook；不拥有生产 SiteAdapter/Playback 语义。
Authority/domain → Claim mapping: Browser lifecycle → C2/C3；R008/Secret → C1/C2/C3；实验 schema/站点解释 → C1/C3。
Integration verification: JI1 = bilibili-browser-probe / probe-integration on frozen Integration Candidate。
Unrelated-main policy: existing exact-Candidate semantic Evidence remains valid，不因 main 前进强制 rebase/full rerun。
Integration-overlap policy: 保留 semantic Evidence，合入 Coordinator 冻结 Integration Base，运行 JI1；语义冲突升级。
Semantic-authority-change policy: reconcile accepted authority，重跑映射 Claims，影响无法界定时扩大验证并解释。
Strict-main reason: n/a。

## Evidence / Completion Protocol

Evidence 必须包含 Task/Claim/Attempt、Worker/Orchestrator、Execution Plane、Runner/Target、OS/architecture、版本、Network path、Base/Candidate SHA、workflow/run/job、artifact ID/digest、命令、预算/实际用量、脱敏结果。Result 仅 PASS / CONDITIONAL PASS / FAIL / BLOCKED。实现结果、验证结果、Coordinator Gate 分开。

遵守 `docs/tasks/issue-lifecycle-protocol.md`：读 live Issue/history → 确认 ready/env/no owner → claim/Attempt → in-progress → durable Candidate/PR/Evidence → EXECUTION REPORT 或 BLOCKER REPORT → review 或 blocked → 释放 owner → STOP。Worker 不 done/close、不自动领取下一项；已有 Candidate/PR 优先续用。

Coordinator 读取契约、history、Candidate/PR、required Evidence，评论 COORDINATOR REVIEW（ACCEPT/REVISE/BLOCK/SPLIT/NOT_PLANNED）。全部成功条件满足后先 FINAL ACCEPTANCE，再 done/close。Contract/routing 改动先 draft，再修订与完整 Publication Gate。

## Worker / Build Constraints

Worker: cloud-codex，模型 `gpt-5.6-luna`、reasoning high；可用时启用 Fast，实际 session 配置写入报告，不宣称不可配置的模式已开启。使用用户已授权且环境实际提供的权限；编排权限不下放 runtime。资源有限，所有编译（含 test binary）必须 GitHub-hosted Actions，禁止本地/tx-node 编译或临时安装工具链。禁止部署手机、修改生产服务或复用个人浏览器 profile。
