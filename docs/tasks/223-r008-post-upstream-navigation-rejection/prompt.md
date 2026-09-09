# Session Bootstrap — R008 post-upstream navigation rejection

You are preparing to execute `liqiangcc/jellyfin-web-media-gateway` Issue #223.

## Execution Context

```text
GitHub Issue: #223
Task Contract: docs/tasks/223-r008-post-upstream-navigation-rejection/task.md
Expected worker: cloud-codex
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Requested model: gpt-5.6-luna, reasoning high
Fast: enabled per current user routing; may be selected
```

## Start Protocol

Read the live Issue #223 and relevant #188/#219 history, then read `AGENTS.md`, the Task Contract, lifecycle/recovery/freshness protocols and the canonical documents named by the contract. Confirm the Publication Gate has made #223 `status:ready`, `env:cloud`, and owner-free before claiming it. After claim, follow the Task Contract exactly, use GitHub-hosted Actions for implementation verification, and only use the bounded target plan after the exact-Candidate artifact gate.

This Task is limited to the post-upstream navigation rejection diagnostic/correction. Preserve the fixed plugin-owned selector and all no-playback, SSRF/egress, secret, source-runtime, no-local-build/test/package/install, no VNC/CDP, no phone/TV, and no #191/#195 boundaries. Do not change #188. A concrete failure or explicit unknown is valid evidence; do not infer a cause without direct finite ordering evidence.

At completion, post `[EXECUTION REPORT]` or a genuine `[BLOCKER REPORT]` on #223, transition to `status:review` or `status:blocked`, release ownership and stop. Do not set done, close the Issue or begin another Task.
