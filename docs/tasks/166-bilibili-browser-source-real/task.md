# Task — BILIBILI-BROWSER-SOURCE-REAL

## Metadata

```text
GitHub Issue: #166
Parent Goal / Research Item: #68 / R005 real source portability
Task ID: BILIBILI-BROWSER-SOURCE-REAL
Task kind: verification
Base commit: c754aa9ff598ce74ffc0d765bd12d730c4f5d4e7
Candidate commit: unresolved upstream artifact; must freeze before publication
Session bootstrap prompt: docs/tasks/166-bilibili-browser-source-real/prompt.md
Preferred worker: cloud-codex with authenticated SSH tx-node
Eligible worker environments: env:cloud after publication gate
Required capabilities: github-read-write, repository-static-analysis, cloud-interactive, interactive-linux-debug, authenticated SSH tx-node
Hard publication dependencies: #165 Final Acceptance with C1-C3 PASS; exact artifact/live runbook and target admission freeze
```

## Goal / Decomposition / Routing

用已接受探针在普通 Linux 证明公开 Bilibili 来源是否可从 clean anonymous 浏览器映射为**独立服务端可读取**的媒体。Verification mode: separate-task，linked implementation #165；live network/target Evidence authority 与离线实现分离，不按 CPU 架构拆任务。

Orchestrator: cloud-codex；Execution plane: external-codex/ssh（既有受信 SSH 的有界交互取证）；Execution host/Target: tx-node isolated low-privilege Linux，不能把 SSH 写成 Actions run。Artifact build authority: GitHub-hosted Actions #165。所有编译禁本地/tx-node。无手机/实体 TV 要求。

## Publication Preconditions — must freeze before ready

本 planning package 不提供可执行 live 授权，保持 draft，直至 Coordinator 在本文件冻结：#165 accepted Candidate、workflow/run/job/artifact ID/digest/manifest、完整 runbook commit/入口/参数、目标 OS/arch/uid/gid/能力约束、browser executable/version、隔离目录/profile/出站 enforcement admission、精确清理命令、所有预算。不得从旧聊天推断或使用当前个人浏览器。#146 只作已接受低权限/SSH准备参考，不自动证明新 browser probe 安全。

拟冻结内容：公开无登录 `BV14V411W7r5`，part 2；仅此 selector。它与 #67 裸 selector 不等价，不做同样本成功比较。更换样本/登录/代理路径必须先 Contract Revision。网络路径使用普通 direct/no-proxy；若目标不能满足路径，BLOCKED，不能借旧 MCP 浏览器隐含配置。

## In Scope / Steps

1. 读取 AGENTS 启动文档、`docs/research/bilibili-browser-acquisition.md`、#165 accepted runbook、当前 Issue/history。本 Task 不修改 probe 或产品代码；发现实现 bug 返回 #165 同一 Issue 的修复流程，由 Coordinator 重开/复核。
2. 核验 artifact provenance、digest、浏览器版本、低权限/隔离/出站 enforcement、预算/清理能力；本阶段失败不得发送 live 请求。
3. 两次独立全新匿名 profile，每次一次冻结页面导航。记录 part 匹配与候选 shape；不得借用账号 Cookie/localStorage/现存 profile。匿名临时浏览器状态不导出给插件/消费者。
4. 独立服务端消费者仅用 R008 与公共 allowlisted headers，读取最小有限媒体数据；浏览器退出后复查。分别记录 role、codec/container（能确定才填）、HTTP/Range/expiry 类别、字节/请求数，不保存 CDN URL/headers 值/媒体体/HAR/DOM。未能确定 codec/有效期不是猜测填值的理由，写 unknown 并限制结论。
5. 最多 2 sessions、每次导航 120s/200 requests/32 MiB（含全部响应）、metadata ≤1 MiB；独立消费者总共 ≤8 requests、≤1 MiB/request、累计 ≤4 MiB。与 #165 accepted runbook 自动限制一致；低于预算也不反复 retry。权限/DRM/登录要求立即停止；达到预算报告有界失败，不扩大请求。
6. 输出两次完整脱敏 Evidence 与 delivery shape 决策输入；清理新建进程/profile/目录。不修改现有 Gateway/VNC/Chrome/生产服务。

## Verification Matrix / Success Criteria

| Job | Claim / PASS 条件 | Plane / Host | Evidence |
| --- | --- | --- | --- |
| J0 | C0：exact artifact、版本、无特权、匿名 profile、enforcement、预算、清理 admission 全满足 | external-codex/ssh / tx-node | provenance + preflight；未 PASS 不运行 J1/J2 |
| J1 | C1：两次干净匿名会话识别正确 part 和至少一组可描述媒体候选 | external-codex/ssh / tx-node | 两份限字段 metadata + counts + 实际网络路径 |
| J2 | C2：独立消费者读取至少一种可交付候选（分离时 video/audio 均需），browser exit 后仍可读，未携带 Secret/超预算 | external-codex/ssh / tx-node | 响应类别/有限字节计数/生命周期/负面结果 |

当前具体 selector/命令由 publication freeze 绑定 #165 runbook；未冻结不得 claim。Implementation requirements: N/A。Files expected to change: 脱敏 Evidence 文档与 publication freeze 后固定的报告位置；不改代码。

Task success：按已冻结步骤和预算完成实验并完整报告各 Claim 的 PASS/FAIL/BLOCKED、provenance/cleanup；负面结果也可由 Coordinator 接受为完整研究交付。**解锁后续取源/媒体实现必须 C0/C1/C2 全 PASS**。部分字段 unknown 只允许相应结论 CONDITIONAL PASS，不扩大到完整 codec/remux/TV 能力。

正常权限下无候选、独立读取不成立或错误 part 是对应 Claim FAIL；目标/依赖 artifact/预检能力不可用为 BLOCKED。不将网站 4xx 自动诊断为登录/DRM/风控。失败仅阻塞这条新路线，不改写 #67；不通过代理轮换/复制凭据重复尝试。

## Out of Scope / Architecture

不实现生产 Bilibili adapter、remux、Display MSE、登录、navigation、手机部署；不发布 #68、不宣称 Gateway/TV E2E。Core 不识别站点；Browser 通用、插件解释、R008 网络 authority、Secret 边界不变。

## Freshness / Integration Contract

Freshness policy: dependency-aware
Semantic authorities: #165 accepted probe/runbook；R008/security；本冻结样本/网络/profile/evidence contract。
Semantic freshness domains: probe artifact/browser/R008/schema、target isolation、sample/part。
Integration surfaces: #165 runbook/provenance manifest；无产品代码集成面。
Task-owned surfaces: 本 Task Evidence 文档、脱敏报告。
Authority/domain → Claim mapping: artifact/browser/R008/isolation → C0/C1/C2；sample/part → C1/C2。
Integration verification: JI1 n/a（纯证据任务）；artifact/target authority 变化重跑 J0 及映射 J1/J2，不用文档 mergeability 代替实站 Evidence。
Unrelated-main policy: existing exact-Candidate Evidence remains valid，不因无关 main 更新重试网站。
Integration-overlap policy: review runbook/provenance 差异；语义变化按对应 Claim 重验。
Semantic-authority-change policy: 先冻结新 authority，再按影响重验；样本/路线变化先 draft/republication。
Strict-main reason: n/a。

## Evidence / Completion Protocol

Evidence 必须包含 Task/Claim/Attempt、Worker/Orchestrator、Execution Plane、Runner/Target、OS/architecture、版本、Network path、Base/Candidate SHA、workflow/run/job、artifact ID/digest、命令、预算/实际用量、脱敏结果。Result 仅 PASS / CONDITIONAL PASS / FAIL / BLOCKED。实现结果、验证结果、Coordinator Gate 分开。

遵守 `docs/tasks/issue-lifecycle-protocol.md`：读 live Issue/history → 确认 ready/env/no owner → claim/Attempt → in-progress → durable Candidate/PR/Evidence → EXECUTION REPORT 或 BLOCKER REPORT → review 或 blocked → 释放 owner → STOP。Worker 不 done/close、不自动领取下一项；已有 Candidate/PR 优先续用。

Coordinator 读取契约、history、Candidate/PR、required Evidence，评论 COORDINATOR REVIEW（ACCEPT/REVISE/BLOCK/SPLIT/NOT_PLANNED）。全部成功条件满足后先 FINAL ACCEPTANCE，再 done/close。Contract/routing 改动先 draft，再修订与完整 Publication Gate。

## Worker / Build Constraints

Worker: cloud-codex，模型 `gpt-5.6-luna`、reasoning high；可用时启用 Fast，实际 session 配置写入报告，不宣称不可配置的模式已开启。使用用户已授权且环境实际提供的权限；编排权限不下放 runtime。资源有限，所有编译（含 test binary）必须 GitHub-hosted Actions，禁止本地/tx-node 编译或临时安装工具链。禁止部署手机、修改生产服务或复用个人浏览器 profile。
