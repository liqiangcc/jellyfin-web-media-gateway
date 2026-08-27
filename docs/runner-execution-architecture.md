# GitHub Actions Runner 执行架构

## 1. 目的

本文件定义 Web Media Gateway 的 Worker / 自动执行 / Verification 后端分工。

核心目标不是“给每个环境都装 Runner”，也不是让某个聊天客户端承担所有执行，而是：

- **Codex-first** 完成普通仓库实现、修复、重构、测试和 CI authoring；
- **GitHub Actions** 作为统一自动执行总线；
- GitHub-hosted Runner 承担 portable verification；
- Ubuntu ARM64 self-hosted Runner 只承担 phone-specific target proof；
- Web Coordinator 负责 publication / Review / Gate；
- 真实电视保留 Manual Verification。

默认原则：

> **Codex-first for repository work; Actions is the execution bus; GitHub-hosted first; target self-hosted only for target proof.**

```text
Web Coordinator
      ↓
Codex Worker (default repository executor)
      ↓
Candidate commit / PR
      ↓
GitHub Actions
      │
      ├── GitHub-hosted x64 runner
      │     portable / fast verification
      │
      ├── GitHub-hosted ARM64 runner
      │     portable ARM64 verification
      │
      └── Ubuntu ARM64 self-hosted runner
            phone-specific target runtime / metrics / compatibility proof
      ↓
Run / Job / Artifact / Metrics
      ↓
Codex Worker / Web Coordinator Review
```

Web Worker 仍可处理 GitHub-only 轻量 Task 或 Coordinator 明确指定的执行，但不再是普通代码实现的默认首选。

Cloud **不部署 self-hosted Runner**。真实电视等无法合理 Runner 化的最终物理交互继续走 Manual Verification。

本文是 `development-environments.md` 的执行后端细化；安全不变量同时受 `security.md` 约束。

---

## 2. Worker / Orchestrator / Runner / Target 必须分开

```text
Worker / Agent
= claim Issue、修改仓库、组织当前 Task

Orchestrator
= 发起/解释自动化执行

GitHub Actions
= 自动执行与验证调度平面

Runner
= 真正运行 job 的 execution host

Target
= claim 实际需要证明的对象
```

普通 Codex-first 例子：

```text
Worker          = cloud-codex
Orchestrator    = codex-cloud
Execution plane = github-actions
Runner          = github-hosted-x64
Target          = runner itself
```

通用 ARM64：

```text
Worker          = cloud-codex
Orchestrator    = codex-cloud
Execution plane = github-actions
Runner          = github-hosted-arm64
Target          = generic Linux ARM64 environment
```

目标手机：

```text
Worker          = cloud-codex
Orchestrator    = codex-cloud
Execution plane = github-actions
Runner          = ubuntu-arm64-self-hosted
Target          = ubuntu-arm64-phone
```

Runner 不 claim Issue，也不成为 Task owner。Issue owner 是 Codex Worker、Web Worker、Manual verifier 或 Task 明确指定的外部 Worker。

---

## 3. Runner 模型

### Tier 1 — GitHub-hosted Runner

默认、最先使用。

```text
x64 portable verification
+
ARM64 portable verification
```

适合：

- `cargo build`；
- `cargo test`；
- `cargo fmt --check`；
- `cargo clippy`；
- contract / concurrency / security suite；
- portable integration test；
- x64 / generic ARM64 compile + test；
- regression / matrix / repeated test；
- artifact 生成。

优势：

- 无自建机器维护成本；
- 环境相对干净；
- 与 commit / PR 自然绑定；
- x64 和 ARM64 都可以用于通用验证；
- 不把 Codex workspace 的偶然本地状态误当最终 Evidence。

限制：

- generic ARM64 runner 不等于目标 Ubuntu 手机；
- 不代表家庭 LAN；
- 不代表目标 FFmpeg/Chromium/Jellyfin 安装组合；
- 不代表手机温度、真实 RSS/吞吐或 chroot 特性；
- 不代表真实电视。

默认规则：

> 只要 Claim 不依赖目标手机/电视的具体环境，就先用 GitHub-hosted Runner。

### Tier 2 — Ubuntu ARM64 Target Self-hosted Runner

Ubuntu ARM64 手机 Runner 是高价值、受限的 **Target Proof** 后端。

基础 labels：

```text
self-hosted
linux
ARM64
ubuntu-arm64
target-device
```

根据设备实际能力再增加：

```text
device-metrics
ffmpeg-runtime
chromium-runtime
jellyfin-runtime
lan-target
```

它只用于 Claim 本身依赖目标设备真实性的任务，例如：

- Gateway 在目标 Ubuntu/chroot 环境是否可运行；
- 目标设备上的 FFmpeg / Chromium / Jellyfin 兼容性；
- CPU / RSS / temperature / throughput；
- Direct Proxy / Remux 的目标机表现；
- 设备特有网络、文件系统、进程限制；
- 5/30/60 分钟目标稳定性；
- 明确要求的 target deployment verification。

**通用 ARM64 compile/test 优先 GitHub-hosted ARM64 Runner，不占用手机 Runner。**

原则：

> **Generic ARM64 proof stays hosted; phone runner is reserved for phone-specific proof.**

---

## 4. Codex Cloud 的定位

Codex Cloud 是默认的 **generic repository Worker / Orchestrator**，但不是 Runner。

适合：

- 普通 repository implementation / fix / refactor；
- 创建/维护 tests、workflows、PR；
- 处理已有 candidate PR 的 rebase/integration；
- 调度并读取 GitHub Actions；
- 需要 coding workspace 的持续迭代。

因此：

```text
Codex Cloud
= default generic repository Worker

Codex Cloud
!= default verification backend
!= self-hosted runner
!= target phone
```

Cloud 本身资源有限不是问题，因为重 build/test/benchmark 默认由 GitHub-hosted Runner 执行，而不是在 Cloud shell 中承担最终 Verification。

如果 Claim 需要 Cloud-specific 网络/长期交互 state，Task 可以明确把 Cloud host 本身作为 execution host；但必须记录真实 execution plane，不能和 Actions Evidence 混写。

---

## 5. 长时间 / 大量重复验证

不要因为 Worker 是 Codex Cloud，就把 long-running workload 留在 Cloud。

优先：

1. GitHub-hosted Runner；
2. 使用 matrix / shard / repeated jobs 拆分大量 race、benchmark、regression；
3. 每个 job 保存明确 artifact / summary，最终聚合；
4. 如果 Claim 必须连续运行超过 hosted job 能承载的窗口，再按真实 Target/Execution Plane 评审。

例如：

```text
10000x concurrency race
→ 20 hosted jobs × 500 repetitions
→ aggregate result
```

如果研究问题要求“同一个进程连续运行 N 小时”，不能用分片伪装连续 soak。

目标设备连续 soak 如果本身就是 R003 等 target Claim，则使用 Ubuntu ARM64 Target Runner。

---

## 6. 默认 Verification Runner 路由

Coordinator / Codex Worker 对 Verification Claim 按以下顺序路由：

```text
Claim
 ↓
是否依赖目标手机/TV本身？
 ├── Yes
 │    ├── phone runtime / thermal / metrics / target software
 │    │      → Ubuntu ARM64 Target Runner
 │    └── TV / remote / physical UX
 │           → Manual TV Verification
 │
 └── No
      ↓
是否只要求 generic ARM64？
      ├── Yes → GitHub-hosted ARM64 Runner
      └── No  → GitHub-hosted x64 Runner
```

大量重复：

```text
→ GitHub-hosted matrix/sharding first
```

需要特定交互能力时改变 **Worker environment**，不是随意改变最终 Evidence Authority：

```text
generic repository coding
→ Codex Cloud

local Linux-specific interactive debug
→ Codex / WSL

Windows / ADB
→ Codex / Windows

Ubuntu ARM64 target recovery / interactive diagnosis
→ Codex on target

physical TV
→ Manual verifier
```

因此：

```text
Codex = default repository problem-solving Worker
Runner = automatic execution backend
Target = evidence object
```

---

## 7. Codex Worker 的默认开发闭环

普通开发：

```text
Codex Worker
→ read Issue / comments / task.md / current main
→ reuse existing Candidate/PR when present
→ modify code / tests
→ candidate commit / PR
→ Actions on GitHub-hosted x64/ARM64 runner
→ read status / logs / artifact
→ fix / rerun
→ [EXECUTION REPORT]
→ Coordinator Review
```

目标验证：

```text
Codex Worker
→ candidate commit
→ trusted Actions target workflow
→ Ubuntu ARM64 Target Runner
→ target Evidence / metrics
→ Worker report
→ Coordinator Review
```

Web Worker 可以执行 GitHub-only 轻量 Task，但不是默认代码路径。

如果上一 Worker session 卡死但 GitHub 已存在 Candidate/PR/Evidence，新 Worker 必须先复用/评估这些 durable 结果，不要从聊天或空分支重新开始。

---

## 8. Workflow 分层

第一个 Rust workspace / 实际测试落地后，优先建立可复用 workflow。

```text
portable-ci
├── x64 fmt / clippy / unit / contract
├── x64 portable integration
└── generic ARM64 build / test

stress-verification
├── repeated race (sharded)
├── benchmark matrix
├── failure injection
└── bounded soak

target-arm64-verification
├── deploy test instance
├── smoke
├── media path
├── target process compatibility
└── metrics capture
```

Research Item 尽量复用这些能力，并通过 inputs / test selector / candidate SHA 选择测试范围。

Workflow 不应把站点 Secret、Vault、账号资料作为普通 CI 输入。

---

## 9. Candidate SHA 是验证对象

Verification 必须明确验证哪个 commit：

```text
Candidate commit: <sha>
```

Actions run、artifact、metrics 和 Research Evidence 都必须能回指 Candidate SHA。

实现发生变化后，之前 Evidence 不自动证明新 commit。

Worker 切换同样不改变这个原则：旧 Candidate Evidence 可以保留，但 rebase/fix 产生的新 Candidate 必须重新跑 required Verification。

---

## 10. Ubuntu ARM64 Self-hosted Runner 安全边界

Self-hosted Runner 能执行仓库 workflow 中的代码，因此属于部署安全边界。

### 10.1 Runner 身份与权限

Ubuntu ARM64 Runner 必须：

- 使用专用低权限 OS 用户；
- 默认无 root / sudo；
- 不使用 Gateway 生产服务账号；
- Runner workspace 与 Gateway Vault 分离；
- 只能访问当前验证真正需要的文件、设备和端口；
- 不持有 SSH 私钥、Tailscale auth key、长期 GitHub PAT、站点 Cookie 等长期 Secret；
- job 结束后清理临时 workspace / runtime data。

默认禁止读取：

```text
/var/lib/web-media-gateway/vault/
真实 browser profile
来源站点 Cookie/token
Jellyfin API Key
宿主 root credential
ADB privileged socket
```

### 10.2 受信 Workflow 才能进入 Target Runner

Target Runner 不允许任意分支或不可信 PR 自动获得代码执行权。

至少：

- 只验证 Coordinator/Worker 已明确标记的 candidate SHA；
- target workflow 与普通 PR CI 分离；
- workflow 定义来自受信仓库状态；
- 未受信输入不能直接拼接为 shell；
- 需要时使用 manual dispatch / approval gate；
- fork/untrusted change 不直接命中 ARM64 Runner。

原则：

> **PR can request target proof; it must not automatically inherit target-device shell authority.**

### 10.3 并发与资源

目标手机资源有限：

- Runner 默认单并发或严格限并发；
- 高 CPU / Chromium / FFmpeg job 必须有 timeout；
- target soak 与正式服务避免互相污染；
- 资源实验记录其他高负载进程；
- Runner 空闲时不应保持额外重型进程。

---

## 11. Runner 与生产服务隔离

```text
Runner control plane
!=
Gateway production/runtime plane
```

推荐：

```text
/home/gateway-runner/...
    runner binary / work

/var/lib/web-media-gateway/...
    production state / vault / runtime
```

验证优先启动独立 test instance / test ports，不直接覆盖正在使用的正式实例。

只有明确 deployment verification Task 才允许 stop/start 正式服务。

---

## 12. Runner 可用性与降级

```text
GitHub-hosted x64 unavailable
→ retry / BLOCKED

GitHub-hosted ARM64 unavailable
→ generic ARM64 verification BLOCKED / 等价 hosted ARM64 backend
→ 不能直接把 phone target Evidence 当普通 CI 替代

Ubuntu ARM64 Target Runner unavailable
→ 不能用 hosted ARM64 冒充 phone target
→ target verification = BLOCKED
```

如果必须临时使用 Ubuntu ARM64 Codex 交互执行同样验证，应记录真实 execution plane，不能伪装成 Actions Evidence。

Codex Cloud 不可用时可以按 capability 路由 Web/WSL/Windows，但不能因为 Worker 切换而降低 required Verification。

---

## 13. Evidence Contract

Runner 产生的 Evidence 至少包含：

```text
Role: verification
Worker:
Orchestrator:
Execution plane: github-actions
Runner class: github-hosted-x64 | github-hosted-arm64 | ubuntu-arm64-self-hosted
Runner labels / image:
Execution host:
Target:
OS / architecture:
Relevant versions:
Candidate commit:
Workflow / run / job:
Commands / test selector:
Duration / repetitions:
Metrics / artifact:
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

目标环境结果还应记录：

- 温度初始值；
- 是否充电；
- 其他高负载进程；
- 网络路径；
- 浏览器/FFmpeg/Jellyfin 版本。

---

## 14. Research 推荐映射

### R007

```text
contract / test authoring
→ Codex Cloud Worker

automated concurrency suite
→ GitHub-hosted x64

generic ARM64 regression (if useful)
→ GitHub-hosted ARM64

large repeated race
→ GitHub-hosted sharded matrix

interactive race debugging
→ WSL when needed
```

### R001

```text
implementation / proxy / tests / PR integration
→ Codex Cloud Worker

portable MP4/HLS integration
→ GitHub-hosted

generic ARM64 compatibility
→ GitHub-hosted ARM64

target media-path / target FFmpeg / device metrics
→ Ubuntu ARM64 Target Runner when the Claim belongs to a target Task
```

### R003

```text
metrics scripts / harness
→ Codex Cloud Worker

portable harness checks
→ GitHub-hosted

generic ARM64 harness checks
→ GitHub-hosted ARM64

CPU / RSS / temperature / target throughput
→ Ubuntu ARM64 Target Runner
```

R002 probe implementation defaults Codex Cloud；最终 TV autoplay/遥控体验仍保留 Manual TV Gate。

---

## 15. 实施顺序

基础执行能力按以下逻辑形成：

```text
1. Contract / first runnable code
2. GitHub-hosted x64 CI
3. GitHub-hosted ARM64 portable verification
4. repeated/stress workflow 的 matrix/sharding
5. Ubuntu ARM64 Target Runner
6. target verification / metrics artifact 标准化
7. 根据真实缺口再决定是否需要其他执行后端
```

**Cloud Runner 不在计划内。** Codex Cloud 是 Worker，不是 Runner。

---

## 16. 完成定义

成熟后的默认闭环：

```text
大多数代码和测试编写
→ Codex Cloud Worker

普通 x64 runtime verification
→ GitHub-hosted x64

通用 ARM64 verification
→ GitHub-hosted ARM64

大量重复 verification
→ GitHub-hosted matrix/sharding

目标手机 proof
→ Ubuntu ARM64 Target Runner

最终物理 TV UX
→ Manual

Task publication / recovery / Review / final Gate
→ Web Coordinator
```

最终目标：

> **让 Codex 专注仓库实现，让 GitHub Actions 提供可审计真实执行，让 Web Coordinator 管理生命周期；只有“设备本身”是证据对象时才占用自建 Target Runner。**