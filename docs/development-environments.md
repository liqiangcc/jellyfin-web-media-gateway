# 开发环境与多 Agent 协同

## 1. 目标

本项目从网页 GPT、GitHub Actions、Cloud、WSL、Windows、Ubuntu ARM64 手机和真实电视等环境开发与验证。

协同模型不把任务永久绑定到某个环境，而是先定义 Goal、Task、Claim 和 Evidence，再选择成本最低、能力足够的执行方式。

默认原则：

> **Web-first，GitHub-hosted-first，Target-runner-for-proof，Codex-for-interaction，Manual-for-UX。**

解释：

1. **Web-first**：设计、实现、测试代码、Review、GitHub 修改尽可能留在 Web Worker。
2. **GitHub-hosted-first**：真实 build/test/lint 和通用 x64/ARM64 验证优先使用 GitHub-hosted Actions Runner。
3. **Target-runner-for-proof**：只有 claim 依赖目标 Ubuntu 手机本身时，才使用 Ubuntu ARM64 self-hosted Target Runner。
4. **Codex-for-interaction**：WSL、Windows、Cloud、Ubuntu ARM64 Codex 主要用于自动化难以表达的交互式诊断、设备控制或恢复。
5. **Manual-for-UX**：真实 TV、遥控器、最终物理 UX 保持人工验证。

Cloud 资源有限，**不部署 Cloud self-hosted Runner**。普通自动验证优先使用 GitHub 自带 Runner。

---

## 2. 分离 Role、Work、Execution Plane、Runner 与 Target

### 2.1 Agent Role

```text
Web Coordinator
= 长生命周期、项目级控制面

Worker
= 短生命周期、单一 Scope 执行者
```

Worker 可承担：

```text
Implementation Worker
= 产生候选实现

Verification Worker
= 对确定 candidate 产生验证 Evidence
```

### 2.2 Execution 维度

```text
Orchestrator
= 谁发起/解释执行

Execution Plane
= GitHub Actions | external Codex | manual

Runner / Execution Host
= 命令真正运行的位置

Target
= claim 真正要证明的对象
```

例如 Web Worker 使用 GitHub-hosted ARM64：

```text
Orchestrator    = web-gpt
Execution Plane = github-actions
Runner          = github-hosted ARM64
Target          = generic Linux ARM64
```

Web Worker 使用手机 Target Runner：

```text
Orchestrator    = web-gpt
Execution Plane = github-actions
Runner          = ubuntu-arm64-phone
Target          = ubuntu-arm64-phone
```

Cloud Codex 经 Tailscale 在手机执行：

```text
Orchestrator    = cloud-codex
Execution Plane = external-codex
Execution Host  = ubuntu-arm64-phone
Target          = ubuntu-arm64-phone
```

Evidence 必须记录真实执行位置，不能只记录发起者。

---

## 3. GitHub 是跨环境唯一事实源

GitHub 保存：

- `main`：已经接受的仓库状态；
- canonical docs：需求、架构、实现契约；
- Issue：实时 task 状态、owner、claim、branch、PR/commit、blocker、verification status、result summary；
- `docs/tasks/<issue>-<slug>/task.md`：版本化执行契约；
- branch / PR：候选实现；
- commit SHA：确定验证对象；
- GitHub Actions run / job / artifact：自动化 Evidence；
- `docs/research/`：需要长期保存的 Research Evidence。

不要：

- 复制带未提交修改的工作目录跨环境交接；
- 只靠聊天描述状态；
- 多环境同时修改相同文件后机械拼接；
- 把手机、WSL、Cloud 工作目录当成比 GitHub 更新的事实源。

---

## 4. 网页 GPT 的两种会话

```text
Web GPT
├── Web Coordinator Session
│   └── 项目级、长生命周期、全局视角
│
└── Web Worker Session
    └── Task 级、短生命周期、单一 Scope
```

### 4.1 Web Coordinator

负责：

1. 读取 canonical docs、Research Matrix、Issue、PR、CI/Evidence；
2. 决定当前最高优先级 Goal / Gate；
3. 先决定 Task 边界与 Implementation / Verification 是否需要分离；
4. 定义 Scope、Claims、Success Criteria、Evidence Requirements；
5. 优先安排 Web Worker；
6. 将 Verification Claims 映射为 Jobs；
7. 为每个 Job 选择 GitHub-hosted x64 / ARM64、Target Runner 或 Manual；
8. 只有自动化能力不足时才路由外部 Codex；
9. Review candidate / Evidence 并决定 Parent Goal / Gate。

### 4.2 Web Worker

Web Worker 是默认最高优先级 Implementation Worker，同时是 GitHub Actions 的主要 Orchestrator。

可以：

- 需求、架构、contract；
- 修改 Rust/前端/测试代码；
- 修改文档；
- 创建 commit / PR；
- 设计/修改 workflow；
- 读取 Actions status/job/log/artifact；
- 根据真实 Runner Evidence 修复并重新验证；
- 汇总 target Evidence。

因此：

```text
Web Worker 自身没有本地 shell
!=
Web Worker 无法完成 runtime verification
```

只要 execution backend 能提供真实 Evidence，工作仍可以保持在 Web 闭环。

---

## 5. Implementation 与 Verification 逻辑分离

### 5.1 Implementation

```text
Input:
- Goal / Contract
- Base commit
- Scope

Output:
- Candidate commit / PR
- tests / harness
- developer checks
- known limitations
```

### 5.2 Verification

```text
Input:
- Candidate commit SHA
- Claims to verify
- Required capabilities
- Success / failure criteria

Output:
- Jobs
- Execution Plane / Runner / Target
- Run / job / logs / artifact
- metrics
- PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

必须保持：

```text
Implementation Result
!= Verification Result
!= Coordinator Gate Decision
```

逻辑分离不等于必须拆两个 Issue。

---

## 6. Task 拆分与 Job 拆分必须分开

### 6.1 层级

```text
Goal / Research Item
        ↓
Task
        ↓
Claims
        ↓
Verification Jobs
        ↓
Runner / Target
```

含义：

```text
Task
= 做什么 / 证明什么
= 有 Scope、Owner、Success Criteria、Review 生命周期

Job
= 在哪里 / 用什么命令验证某个 Claim 切片
= 没有业务 Owner
= 不 claim Issue
```

### 6.2 默认 combined

普通工程工作优先保持一个 `combined` Task：

```text
Web Worker implementation
→ Candidate SHA
→ standard GitHub Actions Jobs
→ Verification Result
→ Coordinator Review
```

例如 fmt、clippy、unit、contract、portable integration、普通 x64 / generic ARM64 regression，一般不需要单独 Verification Issue。

### 6.3 何时拆独立 Verification Task

满足以下任一条件时优先拆：

- Verification 使用独立 Evidence Authority，例如 Ubuntu ARM64 Target Runner、Real TV；
- Verification 生命周期、Owner 或调度时间与 Implementation 明显不同；
- Target 暂不可用但 Implementation 可以先完成；
- 关键 Research Gate 要独立追踪“实现完成”和“claim 已证明”；
- Verification 成本/风险高，需要独立重试或多轮 target proof；
- Verification 的 PASS / FAIL / BLOCKED 本身是重要交付结果。

此时：

```text
Implementation Task
→ Candidate SHA

Verification Task
→ Candidate SHA + Claims
→ Jobs / Target / Evidence

Parent Goal / Research Gate
→ Coordinator 汇总结果
```

Implementation Task 完成只表示候选实现已接受，不代表 Parent Goal / Gate 已通过。

### 6.4 不按环境拆 Task

禁止：

```text
x64 Task
ARM64 Task
Phone Task
TV Task
```

仅仅因为存在不同环境就拆业务 Task。

正确做法：如果它们服务同一个稳定 Claim 集合，则在一个 Verification Task 中建立多个 Jobs：

```text
Verification Task
├── J1 → GitHub-hosted x64
├── J2 → GitHub-hosted ARM64
├── J3 → Ubuntu ARM64 Target Runner
└── J4 → Manual TV（如 Claim 需要）
```

只有 Claim、Success Criteria、Owner、生命周期或 Evidence Authority 真正不同才拆新的 Task。

---

## 7. 默认调度算法

### 7.1 先决定 Task 边界

```text
Verification 只是标准、快速 CI？
├── Yes → combined Task
└── No
     ↓
是否存在独立 Evidence Authority / lifecycle / owner / gate result？
├── Yes → Implementation Task + Verification Task
└── No  → combined / research Task
```

### 7.2 Implementation 路由

```text
Web Worker 能否完成？
├── Yes → Web Worker
└── No
     ├── interactive Linux debug → WSL Codex
     ├── ADB/Android host → Windows Codex
     ├── Cloud-specific/Tailscale interactive state → Cloud Codex
     └── target interactive debug/recovery → Ubuntu ARM64 Codex
```

### 7.3 Verification 先生成 Jobs，再选 Runner

```text
Claim
↓
Required Capabilities
↓
Job
↓
Claim 是否依赖目标设备本身？
├── Yes
│    ├── Ubuntu phone target → Ubuntu ARM64 self-hosted Runner
│    └── TV physical UX → Manual TV
│
└── No
     ↓
是否要求 generic ARM64？
├── Yes → GitHub-hosted ARM64 Runner
└── No  → GitHub-hosted x64 Runner
```

### 7.4 大量重复 / 长时验证

优先使用 GitHub-hosted：

- matrix；
- sharding；
- repeated jobs；
- bounded soak；
- artifact aggregation。

Cloud 不因为“可以长期运行”就自动获得优先级；它资源有限且不作为 Runner。

如果 claim 要求**同一个进程连续运行**超过 GitHub-hosted job 可承载窗口，不能用分片伪装连续 soak。此时单独评审实际执行环境，并视生命周期决定是否拆独立 Verification Task。

---

## 8. Capability Vocabulary

Task 用 capability 描述要求。

```text
github-read-write
repository-static-analysis
code-authoring
pr-review

automated-build
automated-test
rust-build
rust-test
rust-fmt
rust-clippy
generic-arm64
failure-injection
benchmark
high-repetition
bounded-soak
artifact-capture

interactive-linux-debug
cloud-interactive
windows-host
adb
android-device-control
lan-access

arm64-target-runtime
device-metrics
thermal-metrics
ffmpeg-target-runtime
chromium-target-runtime
jellyfin-target-runtime

tv-browser
remote-control
jellyfin-tv
manual-observation
```

`generic-arm64` 可以由 GitHub-hosted ARM64 提供；`arm64-target-runtime` 必须由目标手机提供。

---

## 9. Environment / Backend Profiles

### 9.1 总览

| Environment / Backend | 定位 | 最大优势 | 核心限制 | 权威 Evidence |
|---|---|---|---|---|
| Web Worker | 默认 Implementation | 上下文/GitHub/修改成本最低 | 自身无任意本地 shell | repo/diff/contract + orchestrated evidence |
| GitHub-hosted x64 | 默认 portable verification | 干净、可重复、无需维护 | 非目标设备 | x64 runner runtime |
| GitHub-hosted ARM64 | 默认 generic ARM64 verification | 无需占用目标手机 | 非目标手机环境 | generic Linux ARM64 runtime |
| Ubuntu ARM64 self-hosted | Target Proof | 目标设备真实性 | 稀缺、安全边界高 | phone-specific runtime/resource |
| WSL | Interactive Linux Debug | 快速反复调试 | 非目标环境 | WSL runtime/diagnosis |
| Windows | Device Management | ADB/Android host | 非 Gateway target | Windows/ADB state |
| Cloud | Optional External Worker | 远程/Tailscale/状态保持 | 资源有限，不做 Runner | cloud-specific/remote execution |
| Real TV / Manual | Final UX Proof | 真实 TV 行为 | 人工、高成本 | TV UX/browser behavior |

### 9.2 Web Worker

优先承担绝大多数 implementation、test authoring、CI authoring、Review 和 Evidence synthesis。

### 9.3 GitHub-hosted x64 / ARM64

GitHub Actions 是默认 automated verification plane。

适合：

```text
build / test / fmt / clippy
contract / concurrency / security
portable integration
generic ARM64 compile/test
matrix / regression / repeated race
artifact generation
```

GitHub-hosted ARM64 只能证明 generic ARM64 软件行为，不能证明目标 Ubuntu 手机的 chroot、温度、网络、FFmpeg/Chromium 安装或设备资源。

### 9.4 Ubuntu ARM64 Target Runner

只用于 target-bound claim：

- phone Ubuntu/chroot runtime；
- FFmpeg / Chromium / Jellyfin target compatibility；
- CPU/RSS/temperature/throughput；
- target media path；
- target stability。

通用 ARM64 测试不要占用手机 Runner。

### 9.5 WSL

只有自动化失败后需要交互式 Linux 调试、加日志、本地进程/文件操作时使用。

### 9.6 Windows

负责 ADB、Android/Termux/Magisk 状态、设备重启/恢复/部署协调。

### 9.7 Cloud

Cloud **不部署 GitHub Runner**。

只在这些情况下使用 Cloud Codex / Cloud shell：

- Cloud 主机本身是复现对象；
- 需要长期保持交互 state；
- 需要经 Tailscale 做明确授权的 remote orchestration；
- Actions/Runner 模型不适合当前交互式操作。

不用于普通 build/test，不作为默认 long-running backend。

### 9.8 Real TV

最终证明 audible autoplay、遥控器、TV 焦点/恢复、Jellyfin Android TV。

---

## 10. Actions / Runner 执行架构

详细规则见：

- `runner-execution-architecture.md`

核心：

```text
GitHub Actions = execution bus

GitHub-hosted x64
→ portable x64 verification

GitHub-hosted ARM64
→ generic ARM64 verification

Ubuntu ARM64 self-hosted
→ phone-specific target proof

Cloud
→ not a runner
```

Self-hosted Target Runner 必须遵守 `security.md`：低权限、Vault/生产 Secret 隔离、受信 candidate、独立 test runtime、限制并发/资源。

---

## 11. 典型路由

### R007

```text
Implementation Task
→ contract / test authoring
→ Web Worker

Verification Task or combined verification
→ J1 portable concurrency suite → GitHub-hosted x64
→ J2 generic ARM64 regression（需要时）→ GitHub-hosted ARM64
→ J3 large repeated race → GitHub-hosted matrix/sharding

interactive failure debug
→ WSL
```

### R001

```text
Implementation
→ Web Worker

Verification Claims
→ portable MP4/HLS → GitHub-hosted x64
→ generic ARM64 compatibility → GitHub-hosted ARM64
→ target media path / FFmpeg / resource → Ubuntu ARM64 Target Runner
```

如果 target media-path 的生命周期与实现明显不同，应拆独立 Verification Task；不要按 x64/ARM64/phone 三个环境机械拆三个 Task。

### R003

```text
Implementation
→ metrics harness / scripts → Web Worker

Verification Task
→ harness checks → GitHub-hosted x64/ARM64 Jobs
→ CPU/RSS/temperature/target throughput → Ubuntu ARM64 Target Job
```

R003 最终 Gate 依赖 Target Evidence，因此 target verification 应可独立追踪。

### R002

```text
Implementation
→ Display implementation → Web Worker

portable automated checks
→ GitHub-hosted Job

independent Verification Task
→ final audible autoplay / remote UX → Real TV / Manual
```

---

## 12. Issue 与 task.md

### Issue = 动态状态 authority

```text
status
active owner / claim
branch
candidate commit / PR
linked implementation / verification task
verification status
blocker
review state
result summary
```

### task.md = 稳定执行契约

至少描述：

```text
Task kind
Parent Goal / Research Item
Goal / Context
Task decomposition decision
Base / candidate commit
Claims to verify
Required capabilities
Verification Job Matrix
Execution Plane
Runner class / image / labels
Target / Trust gate
Scope
Implementation Requirements
Verification Plan
Success Criteria
Evidence Contract
Failure / Blocked Rules
Deliverables
```

不重复维护动态 owner/status/result。

---

## 13. Claim、分支与并行

一个 Task 任一时刻只有一个 active owner；Research Item 可拆多个 Task 并行。

领取前：

1. 查询 ready Task；
2. 确认 Task kind / Scope / Required Capabilities；
3. 确认无 active owner；
4. claim + `status:in-progress`；
5. 再开始写入。

Verification 必须标识 candidate SHA。GitHub Actions Job 不 claim Issue。

---

## 14. Evidence Contract

runtime / Research Evidence 至少记录：

```text
Role
Task / Claim
Job ID
Orchestrator
Execution Plane
Executor / Runner class
Runner image / labels
Execution host
Target
OS / architecture
Relevant versions
Network path
Candidate commit
Workflow / run / job
Commands / test selector
Duration / repetitions / shards
Metrics / artifact / raw evidence
Result
```

严格限制：

- Web 静态分析不是 runtime PASS；
- GitHub-hosted ARM64 不是目标手机 Evidence；
- Cloud host 不是手机温度/家庭 LAN Evidence；
- 模拟器不是目标 TV；
- remote orchestration 不改变实际 Execution Host / Target。

---

## 15. Tailscale 与远程执行

Tailscale 是管理/执行通道，不是默认媒体路径。

Cloud/Windows/其他 Worker 只有当前 Task 明确授权时才能经 Tailscale 访问目标设备。

- 不公网暴露 ADB/Gateway 管理/Browser debug；
- 不把 auth key、SSH key、GitHub token、Cookie/profile 写入仓库或 Issue；
- Evidence 记录命令真正执行位置；
- 临时端口/token/远程会话按任务结束清理。

---

## 16. 推荐日常闭环

```text
Web Coordinator
→ Goal / Claims / Success Criteria
→ 决定 combined 还是 Implementation + Verification 分离

Implementation
→ Web Worker first
→ Candidate commit

Verification Task / inline verification
→ Claims
→ Jobs
→ GitHub-hosted x64 first
→ GitHub-hosted ARM64 when generic ARM64 matters
→ Ubuntu ARM64 Target Runner only when phone-specific proof matters
→ Manual TV only for physical UX

Interactive diagnosis only when needed
→ WSL / Windows / Cloud / Ubuntu Codex

Evidence
→ GitHub
→ Web Coordinator Review
→ Task Result
→ Parent Goal / Research Gate Decision
```

最终目标：

> **先按工作和 Claim 拆 Task，再按 capability 拆 Job；尽可能让工作留在 Web，并让 GitHub 自带计算资源承担通用验证，自建 Runner 只为 GitHub 无法提供的目标设备真实性服务。**
