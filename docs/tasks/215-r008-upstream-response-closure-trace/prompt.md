# Session Bootstrap — R008 upstream response closure trace

You are preparing to execute `liqiangcc/jellyfin-web-media-gateway` Issue #215.

## Execution Context

```text
GitHub Issue: #215
Task Contract: docs/tasks/215-r008-upstream-response-closure-trace/task.md
Expected worker: cloud-codex
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Requested model: gpt-5.6-luna, reasoning high
Fast: enabled by current user routing; may be selected
```

## Start Protocol

Read the live Issue #215 and relevant comments, then read `AGENTS.md`, the Task Contract, lifecycle/recovery/freshness protocols and the canonical documents named by that contract. Read back #188 Attempt 8 Final Acceptance and #211 Final Acceptance before claim.

Confirm Issue #215 is `status:ready`, `env:cloud`, and owner-free before claim. The Coordinator must complete the Publication Gate before this package enters the ready queue. After claim, follow the Task Contract and use GitHub-hosted Actions for all required verification.

## Task-specific entry

Work only on the finite upstream response/socket lifecycle trace described by `task.md`, preserving explicit unknown fallback and all redaction, finalizer, cleanup, SSRF/egress and no-playback boundaries. Do not guess a Chromium/network cause. Do not access tx-node, launch or attach to a browser, access Bilibili, run playback, use VNC/CDP/phone/TV, perform production actions, or run local build/test/package/install. Do not process, claim, edit or wait on #191 or #195. Any later #188 target rerun is Coordinator-controlled.

At completion, post the required report on #215, set `status:review` or `status:blocked`, release ownership and stop. Do not mark done or close.
