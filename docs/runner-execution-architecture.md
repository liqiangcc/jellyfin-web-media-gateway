# GitHub Actions Runner 执行架构

## 1. 目的

本文件定义 Web Media Gateway 的自动执行与验证后端。

核心目标不是“给每个环境都装一个 Runner”，而是让 Web Worker 尽可能通过 GitHub + GitHub Actions 完成实现后的真实执行、验证、长时间实验和目标设备验证，同时严格控制 Cloud 与 Ubuntu ARM64 等稀缺资源的使用范围。

默认原则：

> **Actions is the execution bus; runners are capability providers.**

```text
Web Coordinator
      ↓
Web Worker
      ↓
Candidate commit / PR
      ↓
GitHub Actions
      │
      ├── GitHub-hosted runner
      │     fast / portable verification
      │
      ├── Cloud self-hosted runner
      │     long-running / repeated / unattended
      │
      └── Ubuntu ARM64 self-hosted runner
            target runtime / resource / compatibility proof
      ↓
Run / Job / Artifact / Metrics
      ↓
Web Worker / Coordinator Review
```

真实电视等无法合理 Runner 化的最终物理交互继续走 Manual Verification。

本文是 `development-environments.md` 的执行后端细化；安全不变量同时受 `security.md` 约束。

---

## 2. Runner 不是 Agent

必须区分：

```text
Agent / Orchestrator
= 决定做什么、修改什么、如何解释结果

GitHub Actions
= 自动执行与验证调度平面

Runner
= 真正运行 job 的 execution host

Target
= claim 实际需要证明的对象
```

典型例子：

```text
Orchestrator   = web-gpt
Execution plane = github-actions
Runner         = github-hosted-x64
Target         = runner itself
```

```text
Orchestrator   = web-gpt
Execution plane = github-actions
Runner         = cloud-x64
Target         = cloud-x64
```

```text
Orchestrator   = web-gpt
Execution plane = github-actions
Runner         = ubuntu-arm64-phone
Target         = ubuntu-arm64-phone
```

GitHub Actions Runner 不 claim Issue，也不成为 Task owner。Issue owner 仍然是 Web Worker 或外部 Worker；Runner 只是它使用的执行能力。

---

## 3. 三层 Runner 模型

### Tier 1 — GitHub-hosted Runner

默认、最先使用。

适合：

- `cargo build`；
- `cargo test`；
- `cargo fmt --check`；
- `cargo clippy`；
- contract / concurrency / security suite；
- portable integration test；
- 短中时长 regression；
- artifact 生成。

优势：

- 环境相对干净；
- 无设备维护成本；
- 与 commit / PR 自然绑定；
- 最适合 Web Worker 的快速迭代闭环。

限制：

- 不代表 ARM64 手机；
- 不代表家庭 LAN；
- 不代表目标 FFmpeg/Chromium 组合；
- 不代表手机温度/资源；
- 不代表真实电视。

默认规则：

> 只要 claim 不依赖特定目标环境，先尝试 GitHub-hosted Runner。

### Tier 2 — Cloud Self-hosted Runner

Cloud Runner 是长期自动执行资源，不是默认开发 Agent。

建议 capability labels：

```text
self-hosted
linux
x64
cloud
long-running
```

适合：

- 6h / 24h soak；
- 大量 repeated concurrency race；
- benchmark matrix；
- failure injection；
- 大型 regression matrix；
- 持续内存增长观察；
- 长时间 media-path synthetic test；
- GitHub-hosted Runner 不适合承载的稳定长跑工作。

Cloud 资源有限，因此不得因为 job “也能在 Cloud 跑”就默认使用 Cloud。

Cloud 调度条件至少满足一个：

```text
requires: long-running
or
requires: high-repetition
or
requires: persistent-experiment
or
GitHub-hosted execution is insufficient for the defined claim
```

普通 build/test/fmt/clippy 继续使用 Tier 1。

### Tier 3 — Ubuntu ARM64 Target Runner

Ubuntu ARM64 手机 Runner 是高价值、受限的 Target Proof 后端。

建议 capability labels：

```text
self-hosted
linux
arm64
ubuntu-arm64
target-device
```

根据设备真实能力再增加：

```text
device-metrics
ffmpeg-runtime
chromium-runtime
jellyfin-runtime
lan-target
```

它只用于 claim 本身依赖目标设备真实性的任务，例如：

- ARM64 原生 build/run；
- Gateway 在目标 Ubuntu/chroot 环境是否可运行；
- FFmpeg / Chromium / Jellyfin 目标兼容性；
- CPU / RSS / temperature / throughput；
- Direct Proxy / Remux 的目标机稳定性；
- 设备特有网络、文件系统、进程限制；
- 5/30/60 min 目标稳定性；
- 后续明确要求的更长 target soak。

禁止把普通可移植单元测试批量丢给手机 Runner。

原则：

> **Portable verification stays off target; target runner is reserved for target-bound proof.**

---

## 4. 默认 Runner 路由

Coordinator / Web Worker 对每个 Verification Claim 按以下顺序路由：

```text
Claim
 ↓
是否依赖特定目标环境？
 ├── Yes
 │    ├── ARM64 / phone runtime / device metrics
 │    │      → Ubuntu ARM64 Target Runner
 │    └── TV / remote / physical UX
 │           → Manual TV Verification
 │
 └── No
      ↓
是否需要长时间 / 大量重复 / 持续状态？
      ├── Yes → Cloud Self-hosted Runner
      └── No  → GitHub-hosted Runner
```

只有自动化失败后需要人工交互式诊断时，才进入：

```text
WSL interactive debug
Windows / ADB debug
External Codex Worker
```

因此：

```text
Runner = execution backend
Codex = interactive problem-solving fallback
```

不能把两者混为同一层。

---

## 5. Web Worker 的默认开发闭环

普通开发尽可能保持：

```text
Web Worker
→ read Issue / task.md
→ modify code / tests
→ candidate commit / PR
→ Actions on GitHub-hosted runner
→ read status / logs / artifact
→ fix
→ rerun
→ verification PASS
→ Coordinator Review
```

长跑场景：

```text
Web Worker
→ candidate commit
→ Actions workflow
→ Cloud Runner
→ long-running Evidence / artifact
→ Web Worker review
```

目标验证：

```text
Web Worker
→ candidate commit
→ Actions target workflow
→ Ubuntu ARM64 Runner
→ target Evidence / metrics
→ Web Worker review
```

这意味着 Web Worker 即使自身没有本地 shell，也可以完成大量具有真实 runtime Evidence 的工程任务。

---

## 6. Workflow 分层

第一个 Rust workspace / 实际测试落地后，优先建立可复用 workflow，而不是为每个 Research Item 写一套重复脚本。

建议逻辑能力：

```text
portable-ci
├── fmt
├── clippy
├── unit / contract
└── portable integration

long-running-verification
├── repeated race
├── soak
├── benchmark
└── failure injection

target-arm64-verification
├── build / deploy
├── smoke
├── media path
├── process compatibility
└── metrics capture
```

Research Item 尽量复用这些能力，并通过 inputs / test selector / candidate SHA 选择测试范围。

Workflow 不应把站点 Secret、Vault、账号资料作为普通 CI 输入。

---

## 7. Candidate SHA 是验证对象

Verification 必须明确验证哪个 commit。

```text
Candidate commit: <sha>
```

Actions run、artifact、metrics 和 Research Evidence 都必须能回指 Candidate SHA。

禁止：

```text
“测试昨天那个版本通过了”
```

正确：

```text
Candidate: abc123
Workflow: target-arm64-verification
Runner: ubuntu-arm64-phone
Result: PASS
```

如果实现发生变化，之前的 runtime Evidence 不自动证明新 commit。

---

## 8. Self-hosted Runner 安全边界

Self-hosted Runner 能执行仓库 workflow 中的代码，因此属于部署安全边界，不只是 CI 工具。

### 8.1 Runner 身份与权限

Cloud / Ubuntu ARM64 Runner 必须：

- 使用专用低权限 OS 用户；
- 默认无 root / sudo；
- 不使用 Gateway 生产服务账号运行；
- Runner workspace 与 Gateway Vault 分离；
- 只能访问当前验证真正需要的文件、设备和端口；
- 不持有 SSH 私钥、Tailscale auth key、GitHub PAT、站点 Cookie 等长期 Secret；
- 任务结束后清理临时 workspace / runtime data。

Ubuntu ARM64 Runner 尤其禁止读取：

```text
/var/lib/web-media-gateway/vault/
真实 browser profile
来源站点 Cookie/token
Jellyfin API Key（除非某个受控 verification 明确需要并使用专门短期 secret）
宿主 root credential
ADB privileged socket
```

### 8.2 受信 Workflow 才能进入 Target Runner

Target Runner 不允许任意分支或不可信 PR 自动获得代码执行权。

Target-bound workflow 必须采用受控触发，例如：

- 只验证 Coordinator/Worker 已明确标记的 candidate SHA；
- workflow 定义来自受信仓库状态；
- 未受信输入不能直接拼接为 shell；
- 需要时使用 manual dispatch / approval gate；
- 不允许 fork/untrusted change 直接命中 ARM64 Runner；
- 高风险 target job 与普通 PR CI 分离。

原则：

> **PR can request target proof; it must not automatically inherit target-device shell authority.**

### 8.3 网络权限

Runner 网络能力按最小权限配置。

Cloud Runner：

- 默认 public internet；
- 只有任务明确要求时经 Tailscale 访问目标设备；
- 不因为加入 Tailnet 就获得任意家庭 LAN 扫描权限。

Ubuntu ARM64 Runner：

- 只允许测试需要的 LAN/public targets；
- Runner 本身不得成为绕过 Gateway `EgressPolicy` 的任意代理；
- 测试不允许读取生产 Vault 后自行向站点发请求。

### 8.4 并发与资源

目标手机资源有限：

- ARM64 Runner 默认单并发或严格限并发；
- 高 CPU / Chromium / FFmpeg job 必须有 timeout；
- target soak 与真实服务运行避免互相污染；
- 资源实验要记录是否有其他重负载 job 同时运行；
- Runner 空闲时不应保持高频轮询或额外重型进程。

---

## 9. Runner 与生产服务隔离

目标机既可能运行 Gateway，又可能作为 Runner，因此必须明确两套生命周期：

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

Runner 只通过受控部署脚本或临时测试实例操作被测 binary。

验证默认优先启动独立 test instance / test ports，不直接覆盖正在使用的正式实例。

只有明确的 deployment verification Task 才允许 stop/start 正式服务，并必须在 Task Scope 和 Evidence 中记录。

---

## 10. Runner 可用性与降级

Runner 不可用不能自动改变 Success Criteria。

```text
GitHub-hosted unavailable
→ retry / BLOCKED

Cloud Runner unavailable
→ 如果 claim 不需要 long-running，退回 GitHub-hosted
→ 如果 claim 需要 long-running，BLOCKED 或显式换等价 backend

ARM64 Runner unavailable
→ 不能用 Cloud x64 冒充
→ target verification = BLOCKED
```

如果必须临时使用 Ubuntu ARM64 Codex 交互执行同样的验证，应记录：

```text
Execution plane = external-codex/manual
Executor / Target = ubuntu-arm64-phone
```

结论可以等价，但不能伪装成 Actions Evidence。

---

## 11. Evidence Contract

Runner 产生的 Evidence 至少包含：

```text
Role: verification
Orchestrator:
Execution plane: github-actions
Runner class: github-hosted | cloud-self-hosted | ubuntu-arm64-self-hosted
Runner labels:
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

目标环境结果还应记录环境噪声，例如：

- 温度初始值；
- 是否充电；
- 其他高负载进程；
- 网络路径；
- 浏览器/FFmpeg/Jellyfin 版本。

---

## 12. R007 / R001 / R003 的推荐映射

### R007

```text
contract / test authoring
→ Web Worker

automated concurrency suite
→ GitHub-hosted Runner

large repeated race
→ Cloud Runner

interactive race debugging
→ WSL
```

### R001

```text
implementation / test source / proxy logic
→ Web Worker

portable MP4/HLS integration
→ GitHub-hosted Runner（能力允许时）

long-running synthetic stability
→ Cloud Runner

target media-path / target FFmpeg / target resource sample
→ Ubuntu ARM64 Runner
```

### R003

```text
metrics scripts / harness
→ Web Worker

portable harness checks
→ GitHub-hosted Runner

long-running harness dry-run
→ Cloud Runner

CPU / RSS / temperature / target throughput
→ Ubuntu ARM64 Runner
```

R002 的最终 TV autoplay/遥控体验仍保留 Manual TV Gate。

---

## 13. 实施顺序

当前仓库尚无可运行 Rust workspace，也尚无 `.github/workflows/`。

因此 Runner 架构按以下顺序落地：

```text
1. Contract / first runnable code
2. GitHub-hosted portable CI
3. Cloud self-hosted Runner + long-running workflow
4. Ubuntu ARM64 self-hosted Runner + target workflow
5. Target metrics/artifact 标准化
6. 根据真实使用再考虑更多 Runner / runner group / ephemeral strategy
```

不要为了流程完整性提前创建空 workflow 或让手机 Runner 空跑。

---

## 14. 完成定义

Runner 执行架构成熟后，应达到：

```text
大多数代码和测试编写
→ Web Worker

普通 runtime verification
→ GitHub-hosted Actions

长时间自动实验
→ Cloud Runner

目标 ARM64 proof
→ ARM64 Runner

最终物理 TV UX
→ Manual
```

最终目标不是“减少使用其他环境”，而是：

> **把环境变成 Web 可以按需调度的受控能力，让上下文、实现和验证尽可能留在 Web 闭环中。**
