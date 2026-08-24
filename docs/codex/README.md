# Codex 任务入口

本目录保存可以直接交给**外部交互式/目标环境 Worker** 执行的阶段性任务提示词。

长期规则见根 `AGENTS.md`；完整调度模型见 `../development-environments.md`。

## 默认不是 Codex-first

项目默认执行顺序：

```text
Web Worker implementation
→ GitHub Actions automated verification
→ Cloud long-running verification
→ WSL / Windows interactive debugging when needed
→ Ubuntu ARM64 / Real TV target proof when required
```

因此本目录不是“所有代码任务”的默认入口。

只有出现以下 capability 缺口时通常进入外部 Codex：

- Actions failure 需要交互式 Linux debug；
- 需要本地文件/进程/调试器；
- 需要 ADB / Android host；
- 需要 Ubuntu ARM64 目标运行和资源测量；
- 需要通过 Cloud 之外的特定交互环境处理问题；
- 其他 Web + Actions + Cloud 无法覆盖的执行需求。

真实电视/遥控器等物理 UX 仍属于 manual/target worker，不因为使用 Codex 就自动获得该 Evidence authority。

## GitHub Actions 与 Cloud 不属于这里的“外部 Codex”前置条件

- **GitHub Actions**：默认 automated verification backend；由 commit/PR 触发并提供真实 runner Evidence。
- **Cloud**：默认 long-running backend；适合 soak、重复 race、benchmark matrix、failure injection。

只有自动化/长跑后端不足以完成或定位任务时，才优先启动 WSL / Windows 等交互式 Codex。

## 推荐任务入口

优先使用：

```text
GitHub Issue
+ docs/tasks/<issue>-<slug>/task.md
```

Worker 读取任务契约，确认当前环境正好提供 Task 缺失的 Required Capabilities，claim Issue，只执行 Scope，提交实现/Evidence 后转 `status:review` 并停止。

有明确 Issue / Task 时：

> 读取 `AGENTS.md`、对应 GitHub Issue 和 `docs/tasks/<issue>-<slug>/task.md`；确认当前环境提供 Web/Actions/Cloud 无法提供的 Required Capabilities 后 claim，只执行当前 Scope，记录真实 Executor/Target Evidence，提交后转为 `status:review` 并停止。

## 当前阶段入口

- `technical-feasibility.md`：风险驱动技术预研与可行性验证的外部 Worker 阶段入口。

如果当前阶段尚未拆成更聚焦 Issue，可使用该入口；完成本轮后仍然停止，不自动开始下一项。

## 规则

- Prompt 不能重新定义 canonical architecture。
- Prompt 与 `requirements.md` / `architecture.md` / `implementation-contracts.md` 冲突时，以 canonical 文档为准并修复 Prompt 漂移。
- 已完成 Research Item 不重复执行，除非 Evidence 失效、环境变化或任务明确要求重新验证。
- 外部 Worker 不自行扩大 Scope、不自行关闭 Issue、不自动开始下一项。
- WSL/Cloud/Actions 结果不得冒充 Ubuntu ARM64 或真实电视 Evidence。
