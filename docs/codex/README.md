# Codex 任务入口

本目录保存可以直接交给 Codex 执行的阶段性任务提示词。

长期、不随单次任务变化的仓库规则放在根 `AGENTS.md`；本目录只描述“当前阶段要完成什么”。

默认由网页 GPT + GitHub MCP 准备和审查任务；只有必须编译、执行、访问本地/真实设备或采集 Evidence 时才调用相应 Codex。环境分工和交接规则见 `../development-environments.md`。

具体执行任务优先使用 GitHub Issue + `docs/tasks/<issue>-<slug>/task.md` 派发。Codex 读取任务契约、执行 Scope 内工作、提交实现和 Evidence 后停止，由网页 GPT/MCP 验收并决定下一项。

## 使用方式

新开 Codex 会话时优先使用短指令：

> 读取 `AGENTS.md`，然后按照 `docs/codex/technical-feasibility.md` 继续执行下一项。

Codex 必须自己读取仓库中的 canonical 文档和已有 Evidence，不要求用户重复粘贴全部背景。

## 当前任务

- `technical-feasibility.md`：继续风险驱动技术预研与可行性验证，执行当前最高优先级且尚未完成的 Research Item。

## 规则

- 阶段任务 Prompt 不能重新定义 canonical architecture。
- 如果 Prompt 与 `requirements.md` / `architecture.md` / `implementation-contracts.md` 冲突，以 canonical 文档为准，并修复 Prompt 漂移。
- 已完成的 Research Item 不重复执行，除非已有 Evidence 失效、环境变化或任务明确要求重新验证。
- 新增任务 Prompt 应保持聚焦，避免复制整个仓库设计文档。
