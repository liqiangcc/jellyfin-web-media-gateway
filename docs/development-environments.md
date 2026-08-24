# 开发环境与多 Agent 协同

## 1. 目标

项目可以从网页 GPT、Windows、WSL、Ubuntu ARM64 手机、云服务器和真实电视等多套环境开发与验证。

协同目标不是让所有环境平级做同一件事，而是：

1. 让网页 GPT 保持项目全局控制面；
2. 能由网页完成的任务优先由独立 Web Worker 执行；
3. 只有网页缺少有效执行能力或 Evidence 能力时，才路由到外部环境；
4. GitHub 始终作为跨会话、跨环境的唯一事实与交接中心；
5. 不允许用错误环境的结果冒充目标环境 Evidence。

默认原则：

> **Web-first，Capability-driven fallback。**
>
> 先判断 Web Worker 能否产生本任务要求的有效结果；能则网页优先执行，不能才路由到 WSL、Windows、Ubuntu ARM64、Cloud 或真实电视。

“网页优先”表示调度优先级最高，不表示网页分析可以替代真实设备证据。

---

## 2. GitHub 是唯一跨环境事实源

GitHub 仓库负责保存跨环境可共享、可审计的状态：

- `main`：已经接受的当前仓库状态；
- canonical docs：当前产品、架构和实现契约；
- GitHub Issue：实时任务状态、owner、claim、branch、PR/commit、blocker 和结果摘要；
- `docs/tasks/<issue>-<slug>/task.md`：版本化、尽量稳定的执行契约；
- branch / PR：进行中的实现与 Review；
- commit：可追踪交接点；
- `docs/research/`：需要长期保存的实验结论和 Evidence 索引。

不要通过以下方式交接：

- 复制包含未提交修改的整个工作目录；
- 只依赖聊天记录描述未提交状态；
- 让两个 Agent 同时修改同一文件后再机械拼接；
- 把手机部署目录、WSL clone 或云端工作目录当作比 GitHub 更新的事实源。

canonical 产品与架构权威层级仍以 `docs/README.md` 为准；本文只定义执行和协同方式。

---

## 3. 网页 GPT 的两种会话

网页 GPT 不是单一角色。项目明确区分两种网页会话：

```text
Web GPT
├── Web Coordinator Session
│   └── 项目级、长生命周期、全局视角
│
└── Web Worker Session
    └── 任务级、短生命周期、单一 Scope
```

两者都可以调用 GitHub、修改仓库；区别不是“能不能执行”，而是状态范围、生命周期和决策权限。

### 3.1 Web Coordinator Session

Web Coordinator 是项目主 Agent 和控制面。

主要职责：

1. 读取 GitHub、Research Matrix、Issue、PR 和 canonical docs 的全局状态；
2. 决定当前最高优先级工作；
3. 判断任务能否由 Web Worker 直接产生有效结果；
4. 拆分 Issue / Task，定义 Scope、Success Criteria、Evidence Requirements；
5. 把缺少网页能力的任务路由到外部 Worker；
6. Review Web Worker / Codex / 设备实验提交；
7. 根据 Evidence 决定 accept / revise / blocked；
8. 更新 Issue、Gate、canonical docs，并选择下一任务。

Coordinator 可以执行协调本身需要的写操作，例如：

- 创建/修改 Issue；
- 写 `task.md`；
- 修改标签和状态；
- Review PR；
- 根据已经接受的 Evidence 更新 canonical 文档；
- 用户明确要求的极小型协调性文档修正。

但一个已经被定义为独立执行 Task 的工作，默认应交给单独的 Worker Session，而不是让 Coordinator 长期陷入局部实现上下文。

Coordinator 不因为能修改代码就自动承担所有实现任务；它最重要的资产是持续保持项目全局视角。

### 3.2 Web Worker Session

Web Worker 是 `env:web-gpt` 的实际执行者，也是**默认最高优先级 Worker**。

典型生命周期：

```text
读取 AGENTS.md
→ 读取 Issue / task.md
→ claim Task
→ 只执行当前 Scope
→ 修改代码/文档
→ 进行当前环境能够真实完成的检查
→ commit / PR
→ 提交 Evidence / 未验证范围
→ Issue → status:review
→ 停止
```

Web Worker：

- 可以写代码；
- 可以修改文档；
- 可以分析仓库；
- 可以创建提交；
- 可以执行 GitHub 能力范围内的任务；
- 不能把没有真实运行过的编译、测试、设备行为写成 PASS；
- 不自行扩大 Scope；
- 不自行决定并开始下一个任务；
- 完成后返回 Coordinator Review。

`env:web-gpt` 标签专指 **Web Worker 执行资格**，不是 Coordinator 的实时任务身份。

### 3.3 为什么要分成两个网页会话

如果一个网页会话长期同时承担：

```text
全局设计
→ 实现 Task A
→ 实现 Task B
→ 设备分析
→ Review
→ 再决定路线
```

它会逐渐被局部实现上下文占满，降低 Gate、依赖和跨任务一致性的可见性。

因此采用：

```text
Web Coordinator
    ↓ dispatch
Web Worker A / Web Worker B / External Worker
    ↓ result
GitHub
    ↓ review
Web Coordinator
```

---

## 4. Web-first 调度原则

每个新任务先回答：

> **Web Worker 是否能产生这个任务要求的有效结果和 Evidence？**

如果答案是“能”，优先使用 Web Worker，不因为它是“执行任务”就自动交给 Codex。

推荐决策树：

```text
任务出现
  ↓
Web Worker 能否完整完成并产生有效 Evidence？
  ├── Yes → env:web-gpt（优先）
  │
  └── No
       ↓
     缺少什么 capability？
       ├── 编译/本地自动化测试 → WSL / Cloud
       ├── ADB / Android host 操作 → Windows
       ├── ARM64/温度/目标手机运行 → Ubuntu ARM64
       ├── 长时间独立执行 → Cloud（前提是目标 Evidence 不依赖设备）
       └── TV autoplay/遥控/Jellyfin Android TV → 真实电视
```

网页不适合的典型原因不是“这是代码任务”，而是：

- 必须真实运行 `cargo test` / 编译器 / FFmpeg / Chromium 等本地进程；
- 必须访问本地文件、ADB 或局域网设备；
- 必须测量 ARM64 CPU、RSS、温度、功耗、网络；
- 必须在目标电视浏览器观察 autoplay / 遥控器行为；
- 需要长时间持续运行，且网页环境无法提供对应执行保证。

---

## 5. Capability 与环境

环境名称说明“在哪里执行”；任务真正需要声明的是“必须具备什么能力”。

任务可以在 `task.md` 中列出：

```text
Required capabilities:
- github-read-write
- rust-build
- arm64-runtime
- adb
- device-metrics
- lan-access
- tv-browser
- jellyfin-tv
- long-running
```

环境标签仍用于 GitHub 队列调度：

```text
env:web-gpt
env:windows
env:wsl
env:ubuntu-arm64
env:cloud
env:manual-tv
```

Coordinator 根据 Required Capabilities 计算 Eligible Environments，而不是把“某类任务永远属于某个环境”写死。

### 5.1 Web Worker

优先用于：

- 需求与架构分析；
- canonical 文档 / ADR；
- GitHub Issue / PR；
- 仓库 Review；
- 轻量代码修改；
- 契约设计；
- 根据已有真实 Evidence 更新结论；
- 任何不依赖缺失本地运行能力、且网页工具可以真实完成的任务。

不能单独证明：

- 本地代码实际编译通过；
- 本地自动化测试真实通过；
- ARM64 兼容性和资源数据；
- 真实电视 autoplay；
- ADB / Tailscale / Jellyfin / FFmpeg / Chromium 的实际运行状态。

### 5.2 WSL Worker

常用能力：

- Rust workspace 开发；
- `cargo build/test/fmt/clippy`；
- contract / concurrency / security 自动化测试；
- x86_64 本地 PoC。

WSL 结果不能冒充 ARM64 兼容性、手机资源或 TV Evidence。

### 5.3 Windows Worker

常用能力：

- ADB；
- Android / Termux / Magisk 状态；
- 手机重启和部署协调；
- Windows 与真实手机共同参与的实验。

Windows 不是默认主编码环境，也不能替代真实电视。

### 5.4 Ubuntu ARM64 Worker

常用能力：

- 目标 ARM64 原生构建和运行；
- Media Gateway / FFmpeg / Chromium / Jellyfin 兼容性；
- CPU、RSS、温度、吞吐；
- 5/30/60 分钟稳定性；
- 手机 Ubuntu/chroot 特有约束。

### 5.5 Cloud Worker

常用能力：

- 长时间构建；
- 静态分析；
- 自动化测试；
- 不依赖家庭设备的独立复现；
- 经授权通过 Tailscale 在目标机器执行任务。

如果 Cloud Worker 通过 Tailscale 在手机执行命令：

```text
Executor = cloud-codex
Target = ubuntu-arm64-phone
```

Evidence 必须按实际 target 标注，不能写成“云服务器证明了 ARM64 手机行为”。

### 5.6 真实电视 / Manual Worker

以下最终结论只能由目标电视或明确接受的等价设备提供：

- R002 audible autoplay；
- 遥控器焦点；
- 首次初始化确认；
- 等待、刷新、休眠、重启恢复；
- Jellyfin Android TV 实际播放和 handoff。

桌面浏览器、模拟器、云浏览器只能做预检，不能替代最终 Gate。

---

## 6. Issue 与 task.md 的状态所有权

必须避免同一动态状态在多个地方重复保存。

### 6.1 GitHub Issue = 实时状态 authority

以下信息只以 Issue 为实时事实源：

```text
status
assignee / active owner
claimed environment
claimed at
active branch
PR / commit
current blocker
review state
result summary
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

### 6.2 task.md = 版本化执行契约

`task.md` 保存尽量稳定的信息：

```text
Task / Research ID
Goal
Why / Context
Preferred executor
Eligible environments
Required capabilities
Base commit
Preconditions
In Scope
Out of Scope
Architecture Invariants
Files Expected to Change
Execution Steps
Commands / Tests
Success Criteria
Evidence Requirements
Failure / Blocked Rules
Deliverables
```

**不再在 `task.md` 中重复保存：**

- status；
- claimed environment；
- claimed by；
- claimed at；
- active branch；
- 最终实时结果状态。

Worker 不需要为了 claim、block、review 或 done 去修改 `task.md`。

如果任务完成后需要长期保存正式研究结论，写入 `docs/research/`；一般工程结果由 Issue + PR/commit 保存。

如果执行契约本身发生实质变化，由 Coordinator 修改 `task.md` 并重新 Review；执行 Worker 不静默改变 Success Criteria。

---

## 7. Claim 与单任务所有权

一个具体 Issue / Task 任一时刻只能有一个 active owner。

多个 `env:*` 只表示多个环境都有资格执行，不表示允许重复执行。

领取协议：

1. Worker 查询 `status:ready + env:<current-environment>`；
2. 按 Gate / priority / dependency 选择最高优先级任务；
3. 再确认 Issue 仍没有 active owner；
4. 设置 assignee / claim 信息，并改成 `status:in-progress`；
5. 再开始写入性工作。

GitHub 无事务式 claim 时，以最先成功写入 owner + `status:in-progress` 的 Worker 为准；其他 Worker 看到状态变化立即退出。

Worker 完成后：

```text
commit / PR
→ Issue result summary
→ status:review
→ 停止
```

只有 Coordinator 负责：

```text
review
→ done
```

或：

```text
review
→ ready / blocked
```

Worker 不自动领取下一项。

### 7.1 Research Item 与 Task

大型 Research Item 可以拆成多个独立 Task 并行产生 Evidence，但每个 Task 仍只能有一个 active owner。

Research Gate 的最终 PASS / CONDITIONAL PASS / FAIL 由 Coordinator 根据已接受的多个 Evidence 汇总判定，不由单个子任务擅自宣布整个 Gate 完成。

---

## 8. Git 与分支规则

推荐分支：

```text
docs/<topic>
research/r007-<topic>
research/r001-<topic>
feat/<topic>
fix/<topic>
```

规则：

- 代码、Research、跨文件架构修改优先独立分支与 PR；
- 用户明确要求的单 owner 小型文档修改可以直接提交 `main`；
- 不在多个环境同时直接修改 `main`；
- 不 force push，不重写其他环境已引用的提交；
- 一个 Task 一个聚焦提交或一组高度相关提交；
- 冲突必须理解后解决，不得强制覆盖另一环境修改；
- 手机、WSL、Windows、Cloud 维护独立 clone，不同步 `.git` 或整个工作目录。

开始写入前检查：

1. GitHub `main` 是否有更新；
2. 同一 Task 是否已有 owner；
3. 是否已有 PR/branch；
4. 当前环境是否满足 Required Capabilities 和 Evidence 要求。

---

## 9. Web Coordinator → Worker 任务包

需要跨会话或跨环境执行时，推荐：

```text
GitHub Issue
└── docs/tasks/<issue>-<slug>/task.md
```

职责：

```text
Issue
= 实时状态与协作入口

task.md
= 稳定执行契约

Worker
= Scope 内执行和 Evidence

Web Coordinator
= 调度、Review、验收和下一任务决策
```

Issue 进入 `status:ready` 前，`task.md` 应已提交并记录 base commit。

新 Worker 会话不依赖旧聊天，只需：

> 读取 `AGENTS.md` 和对应 Issue / `task.md`，claim 后只执行 Scope，提交结果并转为 `status:review` 后停止。

Web Worker 同样遵守该规则；它不是因为运行在网页里就可以跳过 Task 边界。

---

## 10. Evidence 环境标注

任何 Research 或设备结果至少记录：

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

关键原则：

- Web Worker 的源码/文档分析不是 runtime PASS Evidence；
- WSL x86_64 不能证明 ARM64 资源或兼容性；
- Cloud 不能证明手机温度、家庭 Wi-Fi 或 TV 行为；
- 手机上的浏览器不能自动证明目标电视浏览器；
- 模拟器不能替代 R002 最终真实设备 Gate；
- 远程控制某设备时，Executor 与 Target 必须分别记录。

---

## 11. Tailscale 与远程访问

Tailscale 是管理路径，不是默认媒体数据路径。

- Cloud / Windows / 其他 Worker 可经授权访问手机 Ubuntu；
- 家庭媒体流优先 LAN，不无意义绕行 Tailscale；
- 不直接暴露 ADB、Gateway 管理 API、Browser Worker 调试端口到公网；
- 不提交 Tailscale auth key、SSH key、GitHub token、Cookie、站点 profile；
- 临时端口转发、一次性 token 和远程会话任务结束后清理；
- 远程可达不代表可以扩大当前 Task 的操作范围。

---

## 12. 冲突与失败

### 远程出现新提交

停止覆盖式提交，获取最新状态并重新检查当前 Scope。

### 两个 Worker 重复领取

较晚或尚未形成提交的一方停止。比较 Evidence 与 diff 后只保留一个 active owner，不机械合并两份实现。

### 当前环境能力不足

不要降低 Success Criteria。记录缺失 capability / device / permission / network / version，设置 `status:blocked` 或释放回 `status:ready` 供合适环境领取。

### 实验推翻设计

保留真实 Evidence，交回 Coordinator，按 `AGENTS.md` 的设计变更流程更新 canonical 文档；不要为了 Demo 成功绕过架构和安全边界。

---

## 13. 推荐日常流程

```text
Web Coordinator Session
  → 读取 GitHub 全局状态
  → 选择最高优先级 Gate / Task
  → 定义 Scope / Success Criteria / Evidence
  → 判断 Web Worker 能否完成

能：
  → 新 Web Worker Session（优先）
  → claim / execute / commit / review state

不能：
  → 根据 Required Capabilities
  → WSL / Windows / Ubuntu ARM64 / Cloud / TV Worker
  → claim / execute / evidence / review state

所有 Worker
  → 完成当前 Task 后停止

Web Coordinator Session
  → Review diff + Evidence
  → accept / revise / blocked
  → 更新 Gate / canonical docs
  → 决定下一项
```

最终协作模型不是“多个环境平级竞争任务”，而是：

```text
                    GitHub
                 Single Source
                      │
                      ▼
              Web Coordinator
              Project Control Plane
                      │
          ┌───────────┴───────────┐
          │                       │
   Web Worker（优先）      External Workers
                          WSL / Windows /
                          ARM64 / Cloud / TV
          │                       │
          └───────────┬───────────┘
                      ▼
               Commit / PR / Evidence
                      │
                      ▼
               Web Coordinator Review
                      │
                      ▼
                    main / Gate
```

这使网页既保持最高执行优先级，又通过 Coordinator / Worker 会话分离避免全局上下文被单个实现任务吞噬。