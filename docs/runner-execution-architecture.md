# GitHub Actions Runner 执行架构

## 1. 目的

本文件定义 Web Media Gateway 的自动执行与验证后端。

核心目标不是“给每个环境都装 Runner”，而是让 Web Worker 尽可能通过 GitHub + GitHub Actions 完成真实 build/test、通用 ARM64 验证和目标设备验证，同时避免浪费 Cloud 与 Ubuntu 手机等稀缺资源。

默认原则：

> **Actions is the execution bus; GitHub-hosted first; target self-hosted only for target proof.**

```text
Web Coordinator
      ↓
Web Worker
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
Web Worker / Coordinator Review
```

Cloud **不部署 self-hosted Runner**。真实电视等无法合理 Runner 化的最终物理交互继续走 Manual Verification。

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

例如：

```text
Orchestrator    = web-gpt
Execution plane = github-actions
Runner          = github-hosted-x64
Target          = runner itself
```

```text
Orchestrator    = web-gpt
Execution plane = github-actions
Runner          = github-hosted-arm64
Target          = generic Linux ARM64 environment
```

```text
Orchestrator    = web-gpt
Execution plane = github-actions
Runner          = ubuntu-arm64-phone
Target          = ubuntu-arm64-phone
```

Runner 不 claim Issue，也不成为 Task owner。Issue owner 仍然是 Web Worker 或明确的外部 Worker；Runner 只是它使用的执行能力。

---

## 3. Runner 模型

### Tier 1 — GitHub-hosted Runner

默认、最先使用。

GitHub-hosted Runner 同时承担：

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
- 资源通常优于当前 Cloud 小机器；
- 环境相对干净；
- 与 commit / PR 自然绑定；
- x64 和 ARM64 都可以用于通用验证；
- 最适合 Web Worker 的快速迭代闭环。

限制：

- generic ARM64 runner 仍不等于目标 Ubuntu 手机；
- 不代表家庭 LAN；
- 不代表目标 FFmpeg/Chromium/Jellyfin 安装组合；
- 不代表手机温度、真实 RSS/吞吐或 chroot 特性；
- 不代表真实电视。

默认规则：

> 只要 claim 不依赖目标手机/电视的具体环境，就先用 GitHub-hosted Runner。

### Tier 2 — Ubuntu ARM64 Target Self-hosted Runner

Ubuntu ARM64 手机 Runner 是高价值、受限的 **Target Proof** 后端。

建议 labels：

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

它只用于 claim 本身依赖目标设备真实性的任务，例如：

- Gateway 在目标 Ubuntu/chroot 环境是否可运行；
- 目标设备上的 FFmpeg / Chromium / Jellyfin 兼容性；
- CPU / RSS / temperature / throughput；
- Direct Proxy / Remux 的目标机表现；
- 设备特有网络、文件系统、进程限制；
- 5/30/60 分钟目标稳定性；
- 明确要求的 target deployment verification。

**通用 ARM64 compile/test 优先 GitHub-hosted ARM64 Runner，不占用手机 Runner。**

禁止把普通可移植单元测试批量丢给手机 Runner。

原则：

> **Generic ARM64 proof stays hosted; phone runner is reserved for phone-specific proof.**

---

## 4. Cloud 的定位

Cloud 不加入 Runner 池。

原因：

- 当前 Cloud 资源有限；
- 普通 x64/ARM64 build/test 使用 GitHub-hosted Runner 更合适；
- 把低资源 Cloud 常驻成 Runner会增加维护和状态污染，却没有形成明显验证优势。

Cloud 只保留为低优先级 **External Worker / Remote Orchestrator**，适用于 GitHub Actions 不适合表达的场景，例如：

- 需要长期保持交互式 shell/state；
- 需要经 Tailscale 做明确授权的远程设备操作；
- 需要人工持续观察而不是一次 Actions job；
- 特定网络复现必须来自该 Cloud 主机。

因此：

```text
Cloud
!= default verification backend
!= self-hosted runner
```

---

## 5. 长时间 / 大量重复验证

不要因为存在 `long-running` 就自动使用 Cloud。

优先：

1. GitHub-hosted Runner；
2. 使用 matrix / shard / repeated jobs 拆分大量 race、benchmark、regression；
3. 每个 job 保存明确 artifact / summary，最终聚合；
4. 如果 claim 必须连续运行超过 hosted job 能承载的窗口，再单独评审执行后端。

例如：

```text
10000x concurrency race
→ 20 hosted jobs × 500 repetitions
→ aggregate result
```

而不是：

```text
→ 小 Cloud 单机硬跑
```

如果研究问题本身要求“同一个进程连续运行 N 小时”，不能用分片结果伪装连续 soak；此时应根据 claim 选择真正足够的环境，并把限制写入 Verification Task。

目标设备连续 soak 如果本身就是 R003/R001 的 target claim，则使用 Ubuntu ARM64 Target Runner。

---

## 6. 默认 Runner 路由

Coordinator / Web Worker 对 Verification Claim 按以下顺序路由：

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

如果任务需要大量重复：

```text
→ GitHub-hosted matrix/sharding first
```

只有自动化失败且需要人工交互式诊断时，才进入：

```text
WSL interactive debug
Windows / ADB debug
Cloud external worker (rare)
Ubuntu ARM64 Codex (target interactive debug)
```

因此：

```text
Runner = automatic execution backend
Codex = interactive problem-solving fallback
```

---

## 7. Web Worker 的默认开发闭环

普通开发：

```text
Web Worker
→ read Issue / task.md
→ modify code / tests
→ candidate commit / PR
→ Actions on GitHub-hosted x64/ARM64 runner
→ read status / logs / artifact
→ fix
→ rerun
→ verification PASS
→ Coordinator Review
```

目标验证：

```text
Web Worker
→ candidate commit
→ Actions target workflow
→ Ubuntu ARM64 Target Runner
→ target Evidence / metrics
→ Web Worker review
```

这样 Web Worker 即使自身没有本地 shell，也可以完成绝大多数 implementation + runtime verification 闭环。

---

## 8. Workflow 分层

第一个 Rust workspace / 实际测试落地后，优先建立可复用 workflow。

建议逻辑能力：

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

Verification 必须明确验证哪个 commit。

```text
Candidate commit: <sha>
```

Actions run、artifact、metrics 和 Research Evidence 都必须能回指 Candidate SHA。

实现发生变化后，之前 Evidence 不自动证明新 commit。

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

---

## 13. Evidence Contract

Runner 产生的 Evidence 至少包含：

```text
Role: verification
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
→ Web Worker

automated concurrency suite
→ GitHub-hosted x64

generic ARM64 regression (if useful)
→ GitHub-hosted ARM64

large repeated race
→ GitHub-hosted sharded matrix

interactive race debugging
→ WSL
```

### R001

```text
implementation / proxy / tests
→ Web Worker

portable MP4/HLS integration
→ GitHub-hosted

generic ARM64 compatibility
→ GitHub-hosted ARM64

target media-path / target FFmpeg / device metrics
→ Ubuntu ARM64 Target Runner
```

### R003

```text
metrics scripts / harness
→ Web Worker

portable harness checks
→ GitHub-hosted

generic ARM64 harness checks
→ GitHub-hosted ARM64

CPU / RSS / temperature / target throughput
→ Ubuntu ARM64 Target Runner
```

R002 最终 TV autoplay/遥控体验仍保留 Manual TV Gate。

---

## 15. 实施顺序

当前仓库尚无可运行 Rust workspace，也尚无 `.github/workflows/` 或 self-hosted Runner。

按以下顺序落地：

```text
1. Contract / first runnable code
2. GitHub-hosted x64 CI
3. GitHub-hosted ARM64 portable verification
4. repeated/stress workflow 的 matrix/sharding
5. Ubuntu ARM64 Target Runner
6. target verification / metrics artifact 标准化
7. 根据真实缺口再决定是否需要其他执行后端
```

**Cloud Runner 不在计划内。**

---

## 16. 完成定义

成熟后的默认闭环：

```text
大多数代码和测试编写
→ Web Worker

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
```

最终目标：

> **让 Web 通过 GitHub Actions 调度尽可能多的真实执行能力；只有“设备本身”是证据对象时才占用自建 Target Runner。**
