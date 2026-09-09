# Session Bootstrap — R008 downstream-close navigation correction

You are preparing to execute `liqiangcc/jellyfin-web-media-gateway` Issue #211.

## Execution Context

```text
GitHub Issue: #211
Task Contract: docs/tasks/211-r008-downstream-close-navigation-correction/task.md
Expected worker: cloud-codex
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Requested model: gpt-5.6-luna, reasoning high
Fast: enabled by user; may be selected
```

## Start Protocol

Read the live Issue #211 and relevant comments, then read `AGENTS.md`, the Task Contract, lifecycle/recovery/freshness protocols and the canonical documents named by that contract. Read back #188 Attempt 7 Final Acceptance and #207 Final Acceptance before claim.

Confirm Issue #211 is `status:ready`, `env:cloud`, and owner-free before claim. The Coordinator must complete the Publication Gate before this package enters the ready queue. After claim, follow the Task Contract and use GitHub-hosted Actions for all required verification.

## Task-specific entry

Work only on the evidence-supported downstream-close navigation correction described by `task.md`. If evidence cannot support a safe correction, preserve explicit unknown classification and add only a narrow deterministic diagnostic seam. Do not guess a Chromium/network cause. Do not access tx-node, browser, Bilibili, playback, VNC/CDP, phone/TV or production, and do not process #188 target execution, #191 or #195. Do not run local build/test/package/install.

At completion, post the required report on #211, set `status:review` or `status:blocked`, release ownership and stop. Do not mark done or close.

