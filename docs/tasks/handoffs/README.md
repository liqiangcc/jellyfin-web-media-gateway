# Environment-specific Downstream Handoff Profiles

Task Contract 与 Session Entry 必须分离：

```text
Task Contract
= 做什么、证明什么

Environment Handoff
= 在哪个客户端/环境里如何启动这个 Task
```

Coordinator 在 Publication Gate + target queue verification 全部 PASS 后，按 Issue 的真实 eligible `env:*` 选择本目录中的 handoff profile。

## Profiles

```text
env:web-gpt       → web-gpt.md
env:ubuntu-arm64  → ubuntu-arm64.md
env:wsl           → wsl.md
env:windows       → windows.md
env:cloud         → cloud.md
env:manual-tv     → manual-tv.md
```

规则：

1. 一个 Task 只有一个 eligible environment 时，只输出对应的一份可复制入口。
2. 一个 Task 有多个 eligible environments 时，每个环境分别输出独立复制块；不要把多个环境揉成一个提示词。
3. 所有 `<issue>` / `<slug>` / `<environment>` 必须在对外 handoff 前替换为发布后 GitHub read-back 得到的真实值。
4. handoff 只负责启动导航，不复制 `task.md`，不重定义 Scope / Claims / Success Criteria。
5. `env:web-gpt` 使用 Web ChatGPT + GitHub connector 入口，不要求 repo-scoped Codex Skill。
6. Codex 环境优先使用 `$task-worker`；Skill 不可见时才使用 profile 中的 fallback。
7. `env:manual-tv` 是人工验证入口，不伪装成 Codex/Actions 执行。
8. Queue verification 失败时不得输出任何 handoff。

如果新增新的稳定环境标签，必须先新增对应 profile，再允许 `task-publisher` 向该环境发布独立 Worker Task。
