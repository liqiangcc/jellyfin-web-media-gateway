# Coordinator Handoff — Non-phone Web delivery

用户方向：Coordinator 负责技术研究、Task 拆分/发布、Review 与闭环；具体实现交后续 Worker。本轮不启动实现 Worker。模型要求 Luna high，Fast 可用时开启并记录实际配置。所有编译（含 test binaries）必须远程 GitHub-hosted Actions；不在本机/tx-node 编译，不部署手机。

## Resume from durable authority

读取 AGENTS.md 启动集、live main、docs/product-roadmap.md、docs/research/bilibili-browser-acquisition.md，再读取相关 Issue history、task.md、Candidate/PR/Actions/artifacts。实时状态/owner 只从 GitHub 恢复。发布/Review 使用仓库 task-publisher/task-reviewer skill；本入口不 claim 独立 Worker Task。

当前推进图：

```text
#165 BILIBILI-BROWSER-PROBE-PREP（离线实现/边界/hosted artifact）
→ #166 BILIBILI-BROWSER-SOURCE-REAL（冻结依赖后实站可移植性）
→ 按 Evidence 拆最小媒体交付/站点集成 Task
→ 正式修订 #68 source route 与依赖，发布真实产品 E2E

#67 generic-ytdlp 保留 BLOCK，只有具体外部/契约变化才考虑新 Attempt。
```

Task packages：

- docs/tasks/165-bilibili-browser-probe-prep/task.md + prompt.md
- docs/tasks/166-bilibili-browser-source-real/task.md + prompt.md
- docs/tasks/67-generic-ytdlp-bilibili-real/task.md + prompt.md
- docs/tasks/68-bilibili-web-e2e/task.md + prompt.md

## Transition ownership

1. #165 没有 #67 硬依赖；发布前读回 Issue/task/prompt，最后 ready/env、再查目标队列。Worker 只完成离线范围；真实站点请求不属于 #165。
2. #165 Evidence 全部审查后，Coordinator 在 GitHub 留 Review，接受 exact Candidate/artifact/runbook，再决定 #166 是否可冻结。实现完成不是实站 PASS。
3. #166 发布前必须修订冻结字段：artifact/run/job/digest、runbook、低权限用户/浏览器/出站边界/命令/清理/预算。未满足则 draft，不借用当前浏览器或私有 profile。
4. #166 负面实验可以完整交付，但不能解锁媒体/产品 PASS。只有所需取源与独立消费者 Claims PASS 才冻结最小实现设计；先判断 muxed/AV-separated，不能预先要求 remux。Core 不加站点逻辑。
5. 复用 #68 产品闭环 Issue；旧 generic-ytdlp-only Contract 需要正式 revision，不能直接喂 browser CDN URL 或 seed session。发布前冻结真实 URL input → Registry/plugin → Gateway → Display 的验证链及音视频/控制/重连证据。
6. #67 不重复相同样本/runtime 试错，保留所有 Attempt 与 Review。#27 继续 draft Native Panel/resource umbrella，Browser 取源分支不恢复 Auth/phone capacity。
7. Worker 报告后 review/blocked、释放 owner、STOP；Coordinator 写 COORDINATOR REVIEW，再按条件 FINAL ACCEPTANCE/done/close。保留可恢复 Candidate/PR；无证据不得写 PASS。

物理 TV #7、手机 #9/#142/#131/#113、Jellyfin #16、Core #22 保留其原证据职责。桌面/VNC 功能证据不能替代 audible autoplay、手机资源或完整 Core Gate。

新的 Coordinator 可使用：

```text
作为 Coordinator 接手。读取 AGENTS.md、docs/tasks/handoffs/non-phone-delivery.md，从 GitHub 当前状态恢复 #165/#166 Browser 取源研究分支及 #67/#68 历史/契约。只做预研、任务发布与 Review 闭环；实现交后续 Luna high Worker。暂不部署手机，全部编译走 GitHub-hosted Actions。按每项 task.md 审查 Evidence、更新 GitHub 并发布下一项，不把浏览器原站可播放当 Gateway E2E。
```
