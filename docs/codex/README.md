# Codex 任务入口

本目录保存可以直接交给**外部交互式/目标环境 Worker** 执行的阶段性任务提示词。

长期规则见根 `AGENTS.md`；完整调度模型见 `../development-environments.md`；自动执行架构见 `../runner-execution-architecture.md`。

## 默认不是 Codex-first

项目默认执行顺序：

```text
Web Worker implementation
→ GitHub Actions
     ├── GitHub-hosted x64 portable verification
     ├── GitHub-hosted ARM64 generic ARM64 verification
     └── Ubuntu ARM64 Target Runner phone-specific proof
→ WSL / Windows / Cloud / Ubuntu Codex only for interactive capability
→ Real TV / Manual physical UX proof
```

因此本目录不是“所有代码/运行任务”的默认入口。

## 什么时候才进入外部 Codex

- Actions failure 需要交互式 Linux debug；
- 需要本地文件/进程/调试器；
- 需要 ADB / Android host；
- Target Runner 无法表达的 Ubuntu ARM64 交互式诊断/恢复；
- Cloud 主机本身是复现对象；
- 需要 Cloud 维持交互 state / Tailscale remote orchestration；
- 其他 Runner/Manual 无法覆盖的明确 capability。

### Cloud 特别说明

Cloud **不部署 GitHub self-hosted Runner**，也不是默认 long-running verification backend。

普通/通用验证优先 GitHub-hosted Runner；大量重复优先 matrix/sharding。

Cloud Codex 只用于 Actions 不适合的交互式或 Cloud-specific 工作。

## 推荐任务入口

优先使用：

```text
GitHub Issue
+ docs/tasks/<issue>-<slug>/task.md
```

有明确 Task 时：

> 读取 `AGENTS.md`、对应 GitHub Issue 和 `docs/tasks/<issue>-<slug>/task.md`；确认 GitHub Actions/Target Runner/Manual 无法提供、而当前环境正好提供缺失 Required Capability 后 claim，只执行当前 Scope，记录真实 Execution Host/Target Evidence，提交后转 `status:review` 并停止。

## 当前阶段入口

- `technical-feasibility.md`：风险驱动技术预研与可行性验证的外部 Worker fallback 入口。

没有明确 Issue/Task 时也必须先经过 routing gate，不因为 Codex 会话已经打开就抢占 Web/Actions 可以完成的工作。

## 规则

- Prompt 不能重新定义 canonical architecture。
- Prompt 与 `requirements.md` / `architecture.md` / `implementation-contracts.md` 冲突时，以 canonical 文档为准并修复 Prompt 漂移。
- 已完成 Research Item 不重复执行，除非 Evidence 失效、环境变化或任务明确要求重新验证。
- 外部 Worker 不自行扩大 Scope、不自行关闭 Issue、不自动开始下一项。
- GitHub-hosted ARM64 不得冒充目标手机；WSL/Cloud 不得冒充目标 ARM64/TV Evidence。
