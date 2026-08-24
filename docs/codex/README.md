# Codex 任务入口

本目录保存可以直接交给**外部 Codex Worker** 执行的阶段性任务提示词。

长期、不随单次任务变化的仓库规则放在根 `AGENTS.md`；多环境与网页会话模型见 `../development-environments.md`。

## Web-first

项目默认不是“网页负责设计、Codex 负责执行”，而是：

```text
Web Coordinator
→ 判断任务所需 capability
→ Web Worker 能完成则优先 env:web-gpt
→ Web Worker 缺能力时才路由外部 Codex / 真实设备
```

因此本目录属于 **capability fallback / external worker entry**，不是所有执行任务的默认入口。

需要以下能力时通常使用相应外部 Worker：

- 本地真实编译/自动化测试；
- WSL / Windows 本地文件和进程；
- ADB；
- Ubuntu ARM64 原生运行和资源测量；
- Cloud 长时间执行；
- 真实浏览器/电视/Jellyfin Android TV；
- 其他 Web Worker 无法产生有效 Evidence 的环境能力。

网页分析不能替代这些 runtime / device Evidence；反过来，单纯因为任务涉及代码，也不应自动绕过 Web Worker。

## 推荐任务入口

具体执行任务优先使用：

```text
GitHub Issue
+ docs/tasks/<issue>-<slug>/task.md
```

外部 Codex Worker 读取任务契约、确认 Required Capabilities、claim Issue、执行 Scope、提交实现和 Evidence 后转 `status:review` 并停止；由 Web Coordinator 验收和决定下一项。

如果当前阶段尚未拆成更聚焦 Issue，可以使用本目录的阶段性入口。

## 使用方式

有明确 Issue / Task 时优先：

> 读取 `AGENTS.md`、对应 GitHub Issue 和 `docs/tasks/<issue>-<slug>/task.md`，确认当前环境满足 Required Capabilities 后 claim，只执行当前 Scope；提交结果并转为 `status:review` 后停止。

只有本轮明确要求使用阶段性任务入口时：

> 读取 `AGENTS.md`，然后按照 `docs/codex/technical-feasibility.md` 执行当前最高优先级、当前外部环境能够提供有效 Evidence 的 Research 工作；完成本轮后停止。

Codex 必须自己读取仓库中的 canonical 文档和已有 Evidence，不要求用户重复粘贴全部背景。

## 当前阶段入口

- `technical-feasibility.md`：风险驱动技术预研与可行性验证的外部 Worker 入口。

## 规则

- 阶段任务 Prompt 不能重新定义 canonical architecture。
- 如果 Prompt 与 `requirements.md` / `architecture.md` / `implementation-contracts.md` 冲突，以 canonical 文档为准，并修复 Prompt 漂移。
- 已完成的 Research Item 不重复执行，除非已有 Evidence 失效、环境变化或任务明确要求重新验证。
- 新增任务 Prompt 应保持聚焦，避免复制整个仓库设计文档。
- 外部 Worker 不自行扩大 Scope、不自行关闭 Issue、不自动开始下一项。
