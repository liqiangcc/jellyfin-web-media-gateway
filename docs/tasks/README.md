# 执行任务目录

本目录保存由 Web Coordinator 为独立 Worker 会话准备的版本化执行契约。

Worker 可以是：

- Web Worker（默认最高优先级）；
- WSL Codex；
- Windows Codex；
- Ubuntu ARM64 Codex；
- Cloud Codex；
- 真实电视 / manual worker。

## 1. 目录规则

需要版本化执行契约的 GitHub Issue 使用独立目录：

```text
docs/tasks/<issue-number>-<slug>/task.md
```

例如：

```text
docs/tasks/12-r007-playback-concurrency/task.md
```

`task.md` 是执行契约，不是实时状态文件。

## 2. 状态所有权

### GitHub Issue = 实时状态 authority

以下动态信息只在 Issue 中维护：

```text
status
assignee / active owner
claimed environment
claimed at
active branch
PR / commit
blocker
review state
result summary
```

状态标签：

```text
status:draft | status:ready | status:in-progress | status:blocked | status:review | status:done
```

环境标签：

```text
env:web-gpt | env:windows | env:wsl | env:ubuntu-arm64 | env:cloud | env:manual-tv
```

`env:web-gpt` 表示独立 **Web Worker Session** 的执行资格，不表示 Web Coordinator 正在执行该 Task。

### task.md = 稳定执行契约

`task.md` 只保存：

- Goal；
- Context / Why；
- Preferred executor；
- Eligible environments；
- Required capabilities；
- Base commit；
- Preconditions；
- In Scope / Out of Scope；
- Architecture Invariants；
- Commands / Tests；
- Success Criteria；
- Evidence Requirements；
- Failure / Blocked Handling；
- Deliverables。

不再重复保存 status、claim owner、claim time、active branch 和最终实时结果。

如果任务完成后需要长期保存研究结论，写入：

```text
docs/research/<research-id>-<topic>.md
```

普通工程结果由 Issue + PR/commit 保存。

## 3. Web-first 生命周期

```text
Web Coordinator
→ 创建/整理 Issue
→ 判断 Required Capabilities
→ 从 task.template.md 创建 task.md
→ task.md 提交到 main
→ Issue 链接 task.md + base commit
→ 标记 status:ready + eligible env:*
```

调度时先判断：

```text
Web Worker 能否产生完整有效结果？
  ├── 能 → env:web-gpt 优先
  └── 不能 → 路由到具备缺失 capability 的外部 Worker
```

然后：

```text
Worker claim Issue
→ assignee + claimed environment + status:in-progress
→ 读取 AGENTS.md + task.md
→ 只执行 Scope
→ commit / PR / Evidence
→ Issue 记录结果摘要
→ status:review
→ Worker 停止
→ Web Coordinator Review
→ status:done / ready / blocked
```

同一 Task 任一时刻只允许一个 active owner。

## 4. Web Coordinator 与 Web Worker

两种网页会话必须区分：

```text
Web Coordinator Session
= 长生命周期、项目全局状态、调度、Review、Gate 决策

Web Worker Session
= 短生命周期、单 Task、env:web-gpt 执行者
```

Coordinator 不应因为网页也能执行，就把所有实现长期堆在同一会话中。

Web Worker 完成当前 Task 后停止，不自行选择下一项。

## 5. 自助领取

每个 Worker 只查询同时匹配：

```text
status:ready + env:<current-environment>
```

多个 `env:*` 表示多个环境都具备执行资格，不表示并行执行。

领取成功前不得进行写入性工作。

如果两个 Worker 同时尝试领取，以最先成功设置 active owner 和 `status:in-progress` 的 Worker 为准；另一方停止。

没有匹配任务时不自行从 backlog 扩大工作范围。

## 6. 推荐 Worker 指令

### Web Worker

> 读取 `AGENTS.md`、对应 GitHub Issue 和 `docs/tasks/<issue>-<slug>/task.md`；确认并 claim `env:web-gpt` 任务，只执行当前 Scope，提交修改和当前网页环境能够真实提供的 Evidence，把 Issue 转为 `status:review` 后停止，不开始下一项。

### 外部 Codex Worker

> 读取 `AGENTS.md`、对应 GitHub Issue 和 `docs/tasks/<issue>-<slug>/task.md`；确认当前环境满足 Required Capabilities 并 claim 任务，只执行当前 Scope，提交真实测试/Evidence，把 Issue 转为 `status:review` 后停止，不开始下一项。

## 7. Evidence

任务包不能预先宣称实验 PASS。

需要运行时或设备 Evidence 时，Worker 必须记录实际：

```text
Executor
Execution host
Target host/device
OS / architecture
Relevant versions
Network path
Base commit
Commands / steps
Raw evidence location
Result
```

网页源码分析不能冒充 runtime PASS；WSL、Cloud、模拟器或手机浏览器也不能冒充目标 ARM64 / 真实电视 Evidence。

大型日志、Secret、Cookie、Token、账号数据和临时媒体 URL 不得写入任务目录。