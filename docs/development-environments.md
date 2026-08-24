# 开发环境与多 Agent 协同

## 1. 目标

本项目从网页 GPT、GitHub Actions、Cloud、WSL、Windows、Ubuntu ARM64 手机和真实电视等环境开发与验证。

协同模型不把任务永久绑定到某个环境，而是先定义工作与证据，再选择成本最低、能力足够的执行后端。

默认原则：

> **Web-first，Automation-second，Cloud-for-duration，Local-for-debug，Target-for-proof，Manual-for-UX。**

解释：

1. **Web-first**：能由 Web Worker 完成的设计、实现、Review、GitHub 修改优先留在 Web。
2. **Automation-second**：需要真实 build/test/lint 等自动化 Evidence 时，优先使用 GitHub Actions，而不是立即启动交互式 Codex。
3. **Cloud-for-duration**：需要长时间、无人值守、重复执行或稳定实验环境时，Cloud 优先。
4. **Local-for-debug**：需要快速交互式本地调试时，再使用 WSL / Windows。
5. **Target-for-proof**：只有目标架构、资源、网络或设备行为本身是待证明对象时，才进入 Ubuntu ARM64 等目标环境。
6. **Manual-for-UX**：真实电视、遥控器和最终 UX 只在确实需要物理设备结论时使用。

“优先”代表调度成本和效率，不代表低成本环境可以冒充更高证据要求的目标环境。

---

## 2. 先分离角色、工作、执行环境和 Target

多环境流程必须区分以下维度。

### 2.1 Agent Role

```text
Web Coordinator
= 长生命周期、项目级控制面

Worker
= 短生命周期、单一 Scope 执行者
```

Worker 又可以承担两种逻辑工作：

```text
Implementation Worker
= 产生候选实现

Verification Worker
= 对确定的候选实现产生验证 Evidence
```

同一个实际环境可以承担两种角色，但两种结果必须区分。

### 2.2 Orchestrator / Executor / Target

不要把“谁发起任务”和“代码真正在哪里运行”混在一起。

例如 Web Worker 读取 GitHub Actions 结果：

```text
Orchestrator = web-gpt
Executor     = github-actions
Execution host = github-hosted ubuntu x86_64 runner
Target       = runner itself
```

Cloud Codex 通过 Tailscale 在手机执行命令：

```text
Orchestrator = cloud-codex
Executor     = remote shell on phone
Execution host = ubuntu-arm64-phone
Target       = ubuntu-arm64-phone
```

Evidence 必须描述真实 Executor / Execution host / Target，不能只写发起者。

---

## 3. GitHub 是唯一跨环境事实源

GitHub 负责跨会话、跨环境的可共享状态：

- `main`：已经接受的仓库状态；
- canonical docs：需求、架构、实现契约；
- Issue：实时 task 状态、owner、claim、branch、PR/commit、blocker、result summary；
- `docs/tasks/<issue>-<slug>/task.md`：版本化执行契约；
- branch / PR：候选实现和 Review；
- commit SHA：确定的实现/验证对象；
- GitHub Actions run / job / artifact：自动化验证 Evidence；
- `docs/research/`：需要长期保存的研究结论与 Evidence 索引。

不要通过以下方式交接：

- 复制带未提交修改的工作目录；
- 只依赖聊天记录描述状态；
- 多环境同时修改相同文件后机械拼接；
- 把手机、WSL 或 Cloud 工作目录当作比 GitHub 更新的事实源。

canonical 产品与架构权威层级仍以 `docs/README.md` 为准。

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

两者都能调用 GitHub、修改仓库。区别是生命周期和决策权限，不是“能不能执行”。

### 4.1 Web Coordinator

负责：

1. 读取 canonical docs、Research Matrix、Issue、PR、CI/Evidence 全局状态；
2. 决定当前最高优先级 Goal / Gate；
3. 把 Goal 拆成 Implementation / Verification 工作；
4. 定义 Scope、Success Criteria、Claims to Verify、Evidence Requirements；
5. 判断 Web Worker 是否可以直接完成；
6. 为缺失 capability 选择 Actions、Cloud、Local、Target 或 Manual 后端；
7. Review 候选实现和 Evidence；
8. 决定 `done / ready / blocked`、Research Gate 和下一任务。

Coordinator 可以执行协调性 GitHub 修改，但已经形成独立 Task 的实现工作默认交给新的 Worker Session，避免长期会话被局部上下文占满。

### 4.2 Web Worker

Web Worker 是默认最高优先级的 Implementation Worker，同时可以作为 GitHub Actions / Cloud 的 Orchestrator。

可以：

- 需求、架构与 contract 工作；
- 修改 Rust/前端/测试代码；
- 修改文档；
- 创建 commit / PR；
- Review repository / PR / Actions logs；
- 基于 Actions 真实执行结果报告 build/test status；
- 基于已有目标设备 Evidence 更新结果和 canonical docs。

不能：

- 把没有实际执行的命令写成 PASS；
- 把 GitHub-hosted x86 runner 冒充 ARM64 手机；
- 把桌面/云浏览器冒充真实电视；
- 把静态分析冒充 runtime Evidence。

因此：

```text
Web Worker 自身无本地 runtime
≠
Web Worker 无法完成需要 runtime verification 的工作
```

Web Worker 可以通过受控执行后端获得真实 runtime Evidence。

---

## 5. 开发与验证逻辑分离

### 5.1 Implementation Task

目标是产生确定的候选实现。

典型输入/输出：

```text
Input:
- Goal / Contract
- Base commit
- Scope

Output:
- Candidate commit / PR
- Developer checks
- Known limitations
```

Implementation Task 不因为代码“看起来正确”就自动证明 runtime claim。

### 5.2 Verification Task

目标是证明某个确定候选 commit 满足特定 claim。

典型输入：

```text
Candidate commit: <sha>
Claims to verify:
Required capabilities:
Verification environment:
Success / failure criteria:
Evidence requirements:
```

典型输出：

```text
Executor / Target
Commands / steps
Run / artifact / logs
Metrics
PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

### 5.3 不强制每次拆两个 Issue

逻辑角色分离，不代表所有工作必须流程膨胀。

- 小文档、静态 contract、低风险修改：一个 Task 内 `implementation + review` 即可。
- 普通代码：一个 Implementation Task + 自动 GitHub Actions verification 即可。
- 并发、安全、媒体链路：建议明确列出 Verification Claims；必要时拆独立 Verification Task。
- ARM64、真实电视、Jellyfin、资源/稳定性 Research：候选实现与 target verification 必须可独立追踪。

最重要的是保持：

```text
Implementation Result
!= Verification Result
!= Coordinator Gate Decision
```

---

## 6. 默认调度算法

Coordinator 对每个 Task 使用以下顺序：

```text
1. Web Worker 能否直接完成 Implementation？
   ├── Yes → Web Worker
   └── No  → 找缺失 capability

2. Verification 能否由 GitHub Actions 自动产生有效 Evidence？
   ├── Yes → Actions
   └── No

3. 是否主要需要 long-running / repeatable / unattended？
   ├── Yes → Cloud
   └── No

4. 是否主要需要 interactive local debugging？
   ├── Linux dev/debug → WSL
   └── ADB/Android host → Windows

5. Claim 是否依赖目标环境本身？
   ├── ARM64 / thermal / target runtime → Ubuntu ARM64
   └── TV/browser/remote/Jellyfin TV → Real TV
```

因此，不要先问：

> “这个任务属于哪个环境？”

优先问：

> “为什么 Web + automated verification 不能完成？缺的到底是哪项 capability 或 Evidence authority？”

---

## 7. Capability Vocabulary

`task.md` 使用 capability 描述要求，环境只是 capability provider。

推荐 capability：

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
failure-injection
benchmark
long-running
artifact-capture

interactive-linux-debug
windows-host
adb
android-device-control
lan-access

arm64-runtime
device-metrics
thermal-metrics
ffmpeg-runtime
chromium-runtime
jellyfin-server-runtime

tv-browser
remote-control
jellyfin-tv
manual-observation
```

Capability 表示完成某项工作所需能力，不自动等于最终 Evidence authority。

例如：

```text
rust-test
```

可以由 GitHub Actions、Cloud 或 WSL 提供；如果 claim 是“ARM64 上测试通过”，还必须同时要求：

```text
arm64-runtime
```

---

## 8. Environment Capability Profiles

### 8.1 总览

| Environment | 调度定位 | 最大优势 | 核心限制 | 主要权威 Evidence |
|---|---|---|---|---|
| Web Worker | P1 默认 Implementation | 上下文/GitHub/代码修改成本最低 | 自身无本地 runtime | repo/diff/contract/static analysis |
| GitHub Actions | P2 默认 automated verification | 标准、可重复、与 commit 绑定 | runner 不等于目标设备；时长/交互有限 | runner 上真实 build/test |
| Cloud Worker | P3 long-running | 长时间、稳定、无人值守、可保持状态 | 非家庭/手机/TV 默认环境 | cloud runtime 或明确 remote target |
| WSL Worker | interactive local debug | 快速 Linux 交互调试 | 通常 x86_64，非目标手机 | WSL Linux runtime |
| Windows Worker | device-management debug | ADB/Android host/恢复 | 非 Gateway 最终 runtime | Windows/ADB/Android host state |
| Ubuntu ARM64 | target proof | 最终 ARM64/资源/兼容性 | 慢、资源有限、设备宝贵 | ARM64 target/runtime/resource |
| Real TV / Manual | final UX proof | 真实 TV/autoplay/remote UX | 人工、低自动化、高成本 | 目标电视行为 |

### 8.2 Web Worker

**Strengths**

- 默认上下文最完整；
- GitHub 读写、代码、文档、Issue、PR、Review 连续完成；
- 无 clone 同步和本地环境漂移成本；
- 可以读取 Actions run/job/log/artifact，把自动化验证闭环留在 Web；
- 与 Coordinator 交接成本最低。

**Weaknesses**

- 自身不能运行本地任意进程；
- 不直接持有 ARM64/ADB/TV 等物理能力；
- 不适合把长时间持续运行寄托在网页会话本身。

**Best suited for**

- 绝大多数 Implementation；
- contract/architecture；
- 测试代码编写；
- CI 定义；
- PR/Issue/Review；
- Actions result triage；
- Evidence synthesis。

**Authoritative for**

- GitHub repository state；
- diff/contract/static consistency；
- Web 工具真实执行过的 GitHub 操作。

**Not authoritative for**

- 未执行的 build/test；
- 目标硬件/TV runtime。

### 8.3 GitHub Actions

GitHub Actions 是默认 **Automated Verification Plane**，不是 Web Worker 的“模拟能力”。

**Strengths**

- 每次运行绑定明确 commit；
- build/test/fmt/clippy/security suite 可重复；
- 日志、状态和 artifact 可审计；
- Web Worker 可以直接读取结果并迭代修复；
- 适合 PR gate 和 regression。

**Weaknesses**

- GitHub-hosted runner 的 CPU/架构/网络不等于目标环境；
- 不适合强交互 debug；
- 超长 soak/持续状态实验不一定经济或方便；
- 不能替代真实电视、家庭 LAN、手机热环境。

**Best suited for**

```text
build
test
fmt
clippy
contract/concurrency/security suites
short/medium integration tests
artifact generation
PR regression
```

**Evidence rule**

应记录：

```text
Orchestrator: web-gpt (if applicable)
Executor: github-actions
Runner OS / arch
Workflow / run / job
Commit SHA
Commands / tests
Result
Artifacts
```

仓库当前尚未建立 `.github/workflows/`；第一个 Rust workspace/可运行测试落地时再建立真实 CI，不为了流程文档提前创建空 workflow。

### 8.4 Cloud Worker

Cloud 是 **Primary Long-running Execution Environment**。

**Strengths**

- 长时间无人值守；
- 稳定网络与计算资源；
- 可做持续 benchmark / soak / repeated race / failure injection；
- 适合 6h/24h 级实验；
- 可通过 Tailscale 作为 orchestrator 访问明确授权 target。

**Weaknesses**

- 默认不是家庭 LAN、手机热环境或 TV；
- remote execution 容易混淆 Executor 与 Target；
- 交互式快速改代码体验通常不如 WSL。

**Best suited for**

```text
long-running regression
1000x/10000x race tests
soak test
memory-leak observation
large build/test matrix
continuous failure injection
repeatable benchmark
```

**Authoritative for**

- Cloud host 上真实运行行为；
- 或明确命令实际执行在 remote target 时的 target Evidence。

### 8.5 WSL Worker

WSL 是 **Primary Interactive Linux Debug Environment**，不是所有编译测试的默认入口。

**Strengths**

- 反馈快；
- shell/debugger/local files/processes 交互方便；
- Rust workspace、failure debugging、快速实验适合。

**Weaknesses**

- 通常 x86_64；
- Windows/WSL 网络与目标环境可能不同；
- 不代表 ARM64、手机热环境、TV。

**Use when**

- Actions failure 需要反复加日志和现场调试；
- 本地进程、文件、交互式工具是核心需要；
- Web + Actions 不足以高效定位问题。

### 8.6 Windows Worker

Windows 是 **Device Management / Android Host Environment**。

主要能力：

- ADB；
- Android/Termux/Magisk 状态；
- 手机重启、恢复、部署协调；
- Windows 与真实手机共同参与的实验。

不作为普通 Linux/Rust 主开发环境，也不替代 Ubuntu ARM64 或真实 TV Evidence。

### 8.7 Ubuntu ARM64 Worker

Ubuntu ARM64 手机是 **Target Runtime Authority**。

主要能力：

- ARM64 原生 build/run；
- Gateway/FFmpeg/Chromium/Jellyfin 目标兼容性；
- CPU/RSS/temperature/throughput；
- 5/30/60 min 及更长目标稳定性；
- chroot/设备特有问题。

原则：

> 尽量把普通开发、测试工具编写和可移植验证留在 Web/Actions/Cloud/WSL，只把需要目标真实性的最后 Proof 放到手机。

### 8.8 Real TV / Manual Worker

这是 **Final Physical UX Authority**。

必须用于最终证明：

- R002 audible autoplay；
- 遥控器焦点；
- 首次确认；
- 长时间 idle、refresh、sleep、reboot 恢复；
- Jellyfin Android TV 实际播放与 handoff。

桌面浏览器、模拟器、云浏览器只能预检，不能替代最终 TV Gate。

---

## 9. 典型路由示例

### R007 Contract + Concurrency

```text
Contract / test design
→ Web Worker

Candidate code / tests
→ Web Worker

cargo test / concurrency suite
→ GitHub Actions first

CI failure needs interactive debugging
→ WSL

large repeated race / soak
→ Cloud

Coordinator reviews accepted Evidence
→ R007 gate decision
```

### R003 ARM64 Resource

```text
metrics design / scripts / code
→ Web Worker

portable test + lint
→ GitHub Actions

long-running harness dry-run
→ Cloud (when useful)

final CPU/RSS/temp/thermal claim
→ Ubuntu ARM64
```

### R002 TV Autoplay

```text
Display code / test page
→ Web Worker

automated browser checks
→ Actions/Cloud if useful

final audible autoplay / remote UX
→ Real TV / Manual
```

---

## 10. Issue 与 task.md 状态所有权

### GitHub Issue = 实时状态 authority

只在 Issue 维护：

```text
status
assignee / active owner
claimed environment
claimed at
active branch
candidate commit / PR
verification status
blocker
review state
result summary
```

状态：

```text
status:draft
status:ready
status:in-progress
status:blocked
status:review
status:done
```

### task.md = 稳定执行契约

保存：

```text
Task kind
Goal / Context
Base commit
Candidate commit (for verification task, if known)
Claims to verify
Preferred execution path
Eligible environments
Required capabilities
Preconditions
In Scope / Out of Scope
Architecture Invariants
Implementation requirements
Verification plan
Success Criteria
Evidence Requirements
Failure / Blocked Rules
Deliverables
```

不重复维护实时 owner/status/claim/branch/result。

---

## 11. Claim、分支与并行

一个具体 Task 任一时刻只有一个 active owner；大型 Research Item 可以拆多个 Task 并行。

领取：

1. 查询 `status:ready + env:<current-environment>`；
2. 确认 Required Capabilities 匹配；
3. 确认无 active owner；
4. 设置 assignee / claim + `status:in-progress`；
5. 再开始写入。

推荐分支：

```text
docs/<topic>
research/r007-<topic>
research/r001-<topic>
feat/<topic>
fix/<topic>
```

规则：

- 代码、Research、跨文件架构修改优先 branch/PR；
- 小型单 owner 文档修改可在明确授权时直接 main；
- 不 force push，不覆盖其他 Worker 历史；
- 一个聚焦 Task 一个清晰 commit/PR；
- Verification 必须标识所验证的 candidate SHA；
- 发生冲突时以 GitHub 和 canonical docs 为准。

---

## 12. Evidence Contract

所有 runtime / Research Evidence 至少记录：

```text
Role: implementation | verification
Orchestrator:
Executor:
Execution host:
Target host/device:
OS / architecture:
Relevant versions:
Network path:
Candidate / base commit:
Workflow / run / job (if Actions):
Commands / steps:
Metrics / artifact / raw evidence:
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

严格限制：

- Web 静态分析不是 runtime PASS；
- Actions x86 runner 不能证明 ARM64；
- Cloud host 不能证明手机温度或家庭 LAN；
- 模拟器不能证明目标 TV；
- remote orchestration 不改变真实 Execution host / Target。

---

## 13. Tailscale 与远程执行

Tailscale 是管理/执行通道，不是默认媒体数据路径。

- Cloud/其他 Worker 可以在当前 Task 授权范围内通过 Tailscale 访问目标设备；
- Evidence 必须记录命令实际运行位置；
- 家庭媒体流仍优先 LAN；
- 不直接公网暴露 ADB、Gateway 管理 API、Browser Worker 画面或 debug port；
- 不把 Tailscale auth key、SSH key、GitHub token、Cookie/profile 写入仓库或 Issue；
- 临时端口、token、远程会话按任务结束清理。

---

## 14. 推荐日常闭环

```text
Web Coordinator
→ 定义 Goal / Claims / Success Criteria
→ 拆 Implementation / Verification（逻辑上）

Implementation
→ Web Worker first
→ Candidate commit / PR

Verification
→ GitHub Actions first
→ Cloud if long-running
→ WSL/Windows if interactive capability needed
→ ARM64/TV only for target proof

Evidence
→ GitHub
→ Web Coordinator Review
→ accept / revise / blocked
→ main / Research Gate update
```

最终目标不是让每个环境都有工作，而是：

> **尽可能让工作留在 Web；尽可能让验证自动化；只有明确的 capability / Evidence 缺口才下沉到更昂贵、更具体的环境。**
