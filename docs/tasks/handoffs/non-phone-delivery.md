# Coordinator Handoff — Non-phone Web delivery

Build routing: AGENTS.md §4.1 requires remote GitHub-hosted Actions for every compilation/build, including test binaries. #146 R3 prepares verified artifacts and a compile-free launcher; #67 R20 consumes them. Never use old target-toolchain/cargo paths. Read actual live package links before claim.

This is a project coordination/resume profile, not a Worker Task Contract. User direction: temporarily do not deploy the phone; complete ordinary Linux real Bilibili Web playback. Long-term phone/TV requirements are preserved. See docs/product-roadmap.md and docs/non-phone-web-playback-plan.md.

## Resume from GitHub

Read AGENTS.md, canonical startup set, live main, all open Issue state, relevant comment history and Candidate/PR/Actions. Do not infer completion from this file. Use repository `.agents/skills/task-publisher/SKILL.md` and `task-reviewer/SKILL.md` for publication and review; use task-worker only for a separately identified ready Attempt.

Active delivery graph:

```text
#146 NON-PHONE-EXECUTION-PREP
→ #67 R20 ordinary Linux real extraction
→ #68 real Web product playback

#147 navigation workflow repair — independent, no phone/site dependency
```

Package paths:

- #146: docs/tasks/146-non-phone-execution-prep/task.md + prompt.md
- #147: docs/tasks/147-ci-navigation-workflow-repair/task.md + prompt.md
- #67: docs/tasks/67-generic-ytdlp-bilibili-real/task.md + prompt.md
- #68: docs/tasks/68-bilibili-web-e2e/task.md + prompt.md

## Authority at each transition

1. If #146/#147 are ready, select #146 first when actual authenticated SSH tx-node capability exists. #147 can proceed independently while live work is blocked. Fresh read/claim is mandatory; never launch duplicate owners.
2. Worker executes exactly one Task/Attempt and STOPs after durable report/state/owner release. Coordinator reviews required Evidence and Candidate/freshness, records decision in GitHub, merges only reviewed exact head, and posts Final Acceptance before done/close.
3. After #146 accepted, fill #67's explicit publication completion fields with actual accepted runbook/host/provenance, revise prompt only if bootstrap changes, commit package, read back, ready/env, queue verify, then emit downstream entry. #146 host readiness is not site PASS.
4. #67 BLOCKED → preserve same Issue/Candidate and identify failed layer. No phone recovery, probe loops or expanded privileges. No new infrastructure Task without a concrete independent blocker. Replacement source or runtime needs Contract Revision before next Attempt.
5. #67 FAIL on media shape → decide minimum generic capability task from exact stage/reason. Keep #68 draft; do not conceal FAIL or promote unsupported to conditional PASS.
6. #67 Final Acceptance PASS → fill #68 exact accepted protocol/streams/source/runtime/offline artifact/#146 browser path/Candidate fields; resolve freshness against live main; complete Publication Gate and output its entry.
7. #68 accepted → record ordinary-Linux real Web milestone and reproducible runbook. Do not automatically start navigation/auth/TV/phone/performance or declare #22 accepted.

## Deferred existing Issues

Keep #142/#131/#113/#9 blocked with preserved owner-free history and explicit temporary deferral. Keep #7/#16/#22/#72/#26/#27 draft until their own dependencies and chosen product need exist. User's pause is not an instruction to delete previous Evidence, stop unknown remote services, close failures as successful, or rewrite phone contracts into hosted proof.

## Publication and recovery

A plan/commit/PR alone does not publish a Task. Independent GitHub Issue + task/prompt read-back, correct env/status + no owner, and worker-equivalent queue search are required before emitting a Worker entry. If a stale session has a durable Candidate/PR, preserve it on the same Issue, use recovery/terminal-write protocols and a new Attempt only after proper release/republication.

For a new Codex coordinator session the user may say:

```text
作为 Coordinator 接手非手机 Web 播放目标。读取 AGENTS.md 和 docs/tasks/handoffs/non-phone-delivery.md，从 GitHub 当前 Issue/PR/Actions 恢复 #146 → #67 → #68 主线及独立 #147；按仓库协议 Review、治理和发布下一任务。暂不部署或恢复手机，不降低真实来源、安全、物理 TV/手机最终验收边界。Worker 每个 Attempt 报告后停止，Coordinator 才推进下一阶段；所有决定与可恢复入口写回 GitHub。
```

This coordinator entry does not itself claim a Worker Task or authorize launching duplicate agents. Use the exact ready Issue's env:cloud profile for individual Worker handoffs after read-back.
