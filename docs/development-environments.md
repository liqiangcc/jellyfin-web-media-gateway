# 开发环境与多 Agent 协同

## 1. 目标

项目可以从网页 GPT、Windows、WSL、Ubuntu ARM64 手机和云服务器等多套环境开发。协同目标不是让所有环境同时做相同工作，而是把任务交给成本最低、证据最有效的环境，同时避免重复开发、覆盖提交和错误的可行性结论。

默认策略：

> 网页 GPT + GitHub MCP 优先；只有必须执行、编译、访问本地资源或验证真实设备时，才使用相应 Codex。

该策略用于节省 Codex 执行成本，不降低测试、Evidence、安全或架构要求。

## 2. 唯一事实与交接中心

GitHub 仓库是跨环境唯一交接中心：

- `main` 表示已经接受的当前仓库状态；
- Issue/任务文档表示待执行工作的目标、边界和验收条件；
- 分支和 PR 表示进行中的修改与审查；
- commit 表示可追踪的交接点；
- `docs/research/`（建立后）保存真实实验 Evidence。

不要使用以下方式在环境间交接：

- 复制包含未提交修改的整个工作目录；
- 仅依赖聊天记录描述未提交状态；
- 让两套 Agent 同时编辑同一文件后再人工拼接；
- 把手机部署目录当作比 GitHub 更新的事实源。

canonical 产品与架构权威层级仍以 `docs/README.md` 为准；本文件只定义开发与协同流程。

### 2.1 网页 GPT/MCP 作为协调控制面

网页 GPT + GitHub MCP 默认承担项目协调者，而不只是文档编辑器：

1. 读取仓库、Issue、PR 和 Research Matrix 的当前状态；
2. 判断哪些工作可以直接由网页能力完成；
3. 对必须真实执行的工作创建 GitHub Issue；
4. 为 Issue 写入可执行的 `task.md`；
5. 标记可执行环境、优先级、Scope、成功标准和 Evidence；
6. 跟踪 Codex 提交/PR，审查测试和环境证据；
7. 验收、合并、关闭 Issue，或根据真实失败重新派发；
8. 只有前一项结束后才选择下一项高优先级任务。

职责分离：

```text
GitHub Issue
  = 状态、讨论、负责人、依赖、PR/commit 链接

docs/tasks/<issue>-<slug>/task.md
  = 目标环境可以直接执行的任务契约

Codex
  = Scope 内的编译、运行、设备操作、测试与 Evidence

网页 GPT + GitHub MCP
  = 全局协调、Review、验收与下一任务决策
```

Issue 与 `task.md` 缺一不可：只有 Issue 容易让执行细节散落在评论中；只有文件则缺少实时 owner、状态和讨论入口。

Issue 进入 `status:ready` 前，`task.md` 必须已经提交到 `main`，Issue 正文应链接其路径并记录 base commit。这样各环境只需拉取仓库和查询标签，不依赖额外聊天上下文。

推荐状态：

```text
draft → ready → in-progress → review → done
                          ↘ blocked
```

### 2.2 环境自助领取队列

Issue 通过标签说明“哪些环境有资格执行”，而不是由协调者逐个发送聊天消息。

环境标签：

```text
env:web-gpt
env:windows
env:wsl
env:ubuntu-arm64
env:cloud
env:manual-tv
```

状态标签：

```text
status:draft
status:ready
status:in-progress
status:blocked
status:review
status:done
```

一个 Issue 可以有多个 `env:*`，表示任一匹配环境都能产生有效结果；它不表示允许多个环境重复执行。除非拆成独立子 Issue，否则始终只有一个 active owner。

每个执行环境（包括网页 GPT/MCP）按以下协议拉取任务：

1. 查询 `status:ready + env:<current-environment>`；
2. 按 Issue priority、Research Gate 和依赖选择最高优先级未领取任务；
3. 重新确认 Issue 仍无 assignee/active claim；
4. 设置 assignee，留言执行环境、base commit 和预计分支，并把状态改为 `status:in-progress`；
5. 拉取最新仓库，读取 `AGENTS.md` 与对应 `task.md`；
6. 只执行任务 Scope；完成后提交 commit/PR，把状态改为 `status:review`；
7. 等待网页 GPT/MCP 验收，不自动领取下一任务。

领取必须先于执行，避免两个环境同时看到 ready 后重复开发。GitHub 不支持事务式领取时，以最先成功设置 assignee + `status:in-progress` 的环境为 owner；另一环境看到状态变化后退出。

无法完成时：

- 环境或外部条件暂时缺失：记录准确 blocker，设置 `status:blocked`；
- 当前环境不适合但其他已标记环境可做：记录已完成工作，清除 assignee，恢复 `status:ready`；
- 需要新的环境类型：交回网页 GPT/MCP 修改 Issue 标签和 `task.md`，执行者不静默改变证据标准。

Codex 遇到 Scope 外问题时，将其记录为 follow-up 或 blocker，不自行扩大当前任务。完成当前 `task.md` 后停止，由协调者决定是否创建下一 Issue。

## 3. 环境职责矩阵

### 3.1 网页 GPT + GitHub MCP（默认入口）

网页 GPT + GitHub MCP 同时具有两种角色：

- 协调者：维护全局任务队列、拆分、路由、Review 和验收；
- `env:web-gpt` 执行者：自行领取并完成网页环境能够真实完成的任务。

它作为执行者时同样遵守 claim、Scope、commit/PR、Evidence 和 `status:review` 规则。协调能力不意味着可以把未运行的编译、测试或设备行为写成 PASS。

适合：

- 需求澄清与方案比较；
- canonical 文档、ADR、Issue 和任务包；
- 仓库导航、轻量代码修改与 Review；
- 根据已有 Evidence 更新文档；
- 把大任务拆成可由某一执行环境独立完成的工作单元。

不能单独证明：

- 代码能够编译或测试通过；
- Ubuntu ARM64 上的 CPU、内存、温度和稳定性；
- 真实电视浏览器的 autoplay/遥控行为；
- ADB、Tailscale、Jellyfin 或外部进程的真实运行状态。

没有真实执行证据时，只能记录设计、假设、计划或 `BLOCKED`，不得标记 Research Item 为 PASS。

### 3.2 Windows 本地 Codex

主要职责：

- ADB 连接、Android/Termux/Magisk 状态检查；
- 手机重启、部署编排和故障恢复；
- Windows 与手机之间的受控文件/命令传递；
- 需要 Windows 主机与真实手机同时参与的实验。

它不自动替代 WSL 的主开发环境，也不能替代真实电视验证。

### 3.3 WSL Codex

推荐作为主要编码与快速反馈环境：

- Rust workspace 开发；
- 编译、格式化、lint 和单元/集成测试；
- contract/concurrency/security 测试；
- 不依赖 ARM64/Android 特性的本地 PoC。

WSL x86_64 的通过结果不等于 Ubuntu ARM64 通过。需要目标架构证据的任务必须继续交给手机 Ubuntu。

### 3.4 手机 Ubuntu ARM64 Codex

主要作为目标部署和真实设备验证环境：

- ARM64 原生构建与运行；
- Media Gateway、FFmpeg、Chromium、Jellyfin 实际兼容性；
- CPU、RSS、温度、网络吞吐和 5/30/60 分钟稳定性；
- Android/Ubuntu chroot 特有故障与资源约束。

日常文档和普通编码优先在网页 GPT/MCP 或 WSL 完成。手机仓库通常拉取已经提交的交接点，再进行部署和 Evidence 采集。

### 3.5 火山云服务器 Codex

适合：

- 长时间构建、静态分析和自动化测试；
- 不依赖家庭局域网硬件的独立复现；
- 通过 Tailscale 在明确授权范围内访问手机 Ubuntu 执行部署/测试。

云服务器结果不能冒充家庭 Wi-Fi、手机热环境或真实电视结果。通过 Tailscale 访问设备时，仍应把命令执行位置和测量对象记录为手机 Ubuntu，而不是笼统写“云端测试”。

### 3.6 真实电视与目标浏览器

以下结论只能由目标设备或明确列出的等价设备提供：

- R002 audible autoplay；
- 遥控器焦点和首次确认交互；
- 长时间等待、刷新、休眠和重启后的 Display 行为；
- Jellyfin Android TV 实际播放与 handoff。

桌面浏览器、模拟器和云浏览器可以提前发现问题，但不能替代最终真实设备 Evidence。

## 4. 任务路由原则

使用最低成本、但能够产生有效结论的环境：

```text
只需分析/文档/Review？
  → 网页 GPT + GitHub MCP

需要编译或自动化测试？
  → WSL Codex（默认）或云 Codex（长任务）

需要 ADB/Android 操作？
  → Windows 本地 Codex

需要 ARM64/功耗/温度/真实手机运行？
  → 手机 Ubuntu Codex

需要真实电视/autoplay/遥控/Jellyfin Android TV？
  → 真实电视实验，Codex 负责部署和记录
```

能够远程访问某设备不等于执行环境发生变化。例如云 Codex 通过 Tailscale 在手机上运行命令，Evidence 必须标记 target 为手机 Ubuntu ARM64。

## 5. 单任务所有权

任一时刻，一个 Research Item、Issue 或明确修改范围只能有一个 active owner。

各环境不需要等待单独通知，可以自行查询匹配队列：

```text
status:ready + env:web-gpt
status:ready + env:windows
status:ready + env:wsl
status:ready + env:ubuntu-arm64
status:ready + env:cloud
status:ready + env:manual-tv
```

但“可领取”不等于“已拥有”。只有成功 claim 并把 Issue 改为 `status:in-progress` 后才能开始写入性工作。

开始前检查：

1. `main` 和远程分支是否有更新；
2. 是否已有同一任务的 Issue、PR 或分支；
3. 其他环境是否正在修改相同文件；
4. 当前环境是否能满足该任务的 Evidence 要求。

建议通过 Issue assignee/状态、PR 或任务分支表达 owner。无法使用这些机制时，至少在交接消息中写清：任务、环境、分支、起始 commit 和修改范围。

任务转交时，原 owner 应先提交可用修改；如果尚未形成可提交状态，应明确列出未提交内容，而不是让新环境猜测。

## 6. Git 与分支规则

推荐分支名：

```text
docs/<topic>
research/r007-<topic>
research/r001-<topic>
feat/<topic>
fix/<topic>
```

协同规则：

- 代码、Research 和跨文件架构修改优先使用独立分支与 PR。
- 单 owner 的小型文档修改可以在用户明确要求时直接提交 `main`，但提交前仍需拉取最新状态并检查 diff。
- 不在多套环境同时直接修改 `main`。
- 不 force push，不重写其他环境已经引用的提交。
- 一个清晰任务一个聚焦 commit；不要把多个 Research Item 混在一起。
- 合并或提交前检查 Secret、大文件、生成物和实验原始数据。
- 发生冲突时以 GitHub 当前提交和 canonical 文档为依据，人工理解后解决；不得用强制覆盖丢弃另一环境修改。

手机、WSL、Windows 和云端各自维护独立 clone。不要使用双向文件同步软件同步 `.git` 或工作目录。

## 7. 网页 GPT/MCP → Codex 任务包

为了减少重复上下文和执行 Token，网页 GPT/MCP 对需要 Codex 的工作创建：

```text
GitHub Issue
└── docs/tasks/<issue>-<slug>/task.md
```

`task.md` 使用仓库模板 `docs/tasks/task.template.md`，至少包含：

```text
Task / Research ID:
Issue:
Goal:
Eligible environments:
Claimed environment:
Claimed by / at:
Base commit:
Files in scope:
Out of scope:
Architecture invariants:
Commands / tests to run:
Success criteria:
Evidence to capture:
Expected deliverables:
Branch / commit convention:
```

Codex 接到任务后仍需读取 `AGENTS.md` 和适用 canonical 文档，但不需要用户重新粘贴完整项目背景，也不应重新扩展已经明确的任务范围。

任务包不能预先宣称实验 PASS；它只能定义 Hypothesis、环境与成功标准。

Issue 负责实时状态、assignee、讨论、依赖以及最终 PR/commit 链接；不要把长篇执行契约只放在 Issue 评论里。`task.md` 负责版本化执行要求，必须与代码一起 Review。

同一个 Issue 只绑定一个 active `task.md` 和一个 active owner。任务需求发生实质变化时由网页 GPT/MCP 更新任务文件和 Issue，而不是由执行 Codex 静默改变目标。

## 8. Codex → GitHub/网页 GPT 交接

Codex 完成执行任务后，至少报告：

1. 实际环境和 target；
2. 起始与结束 commit；
3. 修改文件；
4. 实际运行的命令/测试；
5. PASS/FAIL/BLOCKED 与关键 Evidence；
6. 未验证范围和已知限制；
7. 下一步建议，但不自动扩大任务范围。

提交后由网页 GPT/MCP 继续负责跨文档审查、PR Review、Issue 状态和下一任务拆分。

Codex 不自行关闭 Issue；它把 Issue 状态交给 `review`，由网页 GPT/MCP 根据 `task.md` 的验收条件确认 `done` 或退回修改。

## 9. Evidence 环境标注

任何 Research 结果至少记录：

```text
Executor: web-gpt | windows-codex | wsl-codex | ubuntu-arm64-codex | cloud-codex | manual
Execution host:
Target host/device:
OS / architecture:
Relevant versions:
Network path:
Base commit:
Commands / steps:
Raw evidence location:
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

关键限制：

- 网页 GPT/MCP 的源码或文档分析不是 runtime PASS Evidence。
- WSL x86_64 不能证明 ARM64 资源或兼容性。
- 云服务器不能证明手机温度、家庭 Wi-Fi 或电视行为。
- 手机上的浏览器实验不能自动证明小米电视浏览器行为。
- 模拟器不能替代 R002 最终真实设备 Gate。

## 10. Tailscale 与远程访问

Tailscale 是管理路径，不是默认媒体数据路径。

- 云服务器或其他控制端可通过 Tailscale SSH 访问手机 Ubuntu，但只执行当前任务授权范围内的操作。
- 家庭内媒体流优先使用 LAN，避免无意义绕行中继。
- 不把 ADB、Gateway 管理 API、Browser Worker 远程画面端口或调试端口直接暴露公网。
- 不在仓库、Issue、日志或任务包中记录 Tailscale auth key、SSH 私钥、GitHub Token、Cookie 或站点 profile。
- 临时端口转发、一次性 token 和远程会话在任务结束后应按设计清理。
- 远程可达不代表允许扩大操作范围；破坏性设备操作仍需满足 `AGENTS.md` 和当前任务授权。

## 11. 冲突与失败处理

### 远程已有新提交

停止提交，先获取远程更新并检查影响。能安全快进或基于新提交继续时再操作；不要覆盖远程历史。

### 两个环境重复实现

暂停较晚开始或尚未提交的一方，比较 Evidence 与 diff，选择一个 owner 继续。不要把两份实现机械合并进 Core。

### 当前环境无法验证

完成不依赖目标设备的准备后记录 `BLOCKED`，写清缺失设备/权限/网络/版本以及恢复所需的最小条件，然后交给合适环境。

### 实验推翻设计

保留真实 Evidence，按 `AGENTS.md` 的设计变更流程更新 canonical 文档。不要为了让某一环境的 Demo 成功而绕过架构或安全边界。

## 12. 推荐日常流程

```text
网页 GPT + GitHub MCP
  → 读取项目全局状态
  → 澄清需求、创建 Issue + task.md
  → 处理文档、Review 和轻量修改

需要真实执行时
  → Issue 标记 owner environment / in-progress
  → 路由 task.md 到 WSL / Windows / 手机 Ubuntu / 云 Codex
  → 在独立任务范围内编译、测试或采集 Evidence
  → 提交聚焦 commit / PR，Issue 进入 review

网页 GPT + GitHub MCP
  → 审查 Evidence 与 canonical 一致性
  → 合并或要求修正
  → 选择下一项任务
```

这样可以把 Codex Token 主要用于必须真实执行的工作，同时让所有环境通过 GitHub 获得一致、可审计的项目状态。
