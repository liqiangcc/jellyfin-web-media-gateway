# Session Bootstrap — R008 marker retention

You are preparing to execute `liqiangcc/jellyfin-web-media-gateway` Issue #219.

## Execution Context

```text
GitHub Issue: #219
Task Contract: docs/tasks/219-r008-marker-retention/task.md
Expected worker: cloud-codex
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Requested model: gpt-5.6-luna, reasoning high
Fast: enabled by current user routing; may be selected
```

## Start Protocol

Read the live Issue #219 and relevant #188/#215 comments, then read `AGENTS.md`, the Task Contract, lifecycle/recovery/freshness protocols and the canonical documents named by that contract. Confirm #219 is `status:ready`, `env:cloud`, and owner-free before claim; this package is currently materialized as `status:draft` pending the Coordinator Publication Gate.

After claim, implement only the finite marker-retention correction described by `task.md`. Use GitHub-hosted Actions for all required build/test/package verification. Do not access tx-node, launch or attach to a browser, access Bilibili, run playback, use VNC/CDP/phone/TV, perform production actions, or run local build/test/package/install. Do not process, claim, edit or wait on #191 or #195. A later #188 target rerun is Coordinator-controlled.

At completion, post the required report on #219, set `status:review` or `status:blocked`, release ownership and stop. Do not mark done or close.
