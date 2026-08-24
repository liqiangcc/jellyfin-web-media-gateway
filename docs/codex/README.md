# Codex 任务入口

本目录保存可以直接交给 Codex 执行的阶段性任务提示词。

长期、不随单次任务变化的仓库规则放在根 `AGENTS.md`；本目录只描述“当前阶段要完成什么”。

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
