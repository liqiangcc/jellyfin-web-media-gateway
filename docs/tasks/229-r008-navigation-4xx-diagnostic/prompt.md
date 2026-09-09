# Session Bootstrap — Classify the remaining main-navigation 4xx response

You are executing GitHub Issue #229 in `liqiangcc/jellyfin-web-media-gateway`.

```text
GitHub Issue: #229
Task Contract: docs/tasks/229-r008-navigation-4xx-diagnostic/task.md
Expected worker: cloud
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Model: gpt-5.6-luna, reasoning high
Fast: enabled for this Task
```

Before any Attempt, read the live Issue and relevant history, `AGENTS.md`, the Task Contract, lifecycle/recovery/freshness protocols, and its referenced canonical architecture/security documents. Confirm Publication Gate, `status:ready`, `env:cloud`, no owner, and required capabilities before claiming. If those conditions are absent, stop.

Use GitHub-hosted Actions for all required build/test/package verification. Local build/test/package/install and target execution are forbidden until the contract authorizes them. Keep the diagnostic finite and sanitized; preserve plugin authority, SSRF/egress, TLS ownership, secret, finalizer, cleanup, source-runtime, no-playback, and no-device boundaries. Do not process, claim, edit, or wait on #191/#195.

When the exact Candidate and hosted evidence are complete, follow the contract’s reporting protocol: post `[EXECUTION REPORT]` or a genuine `[BLOCKER REPORT]`, set `status:review` or `status:blocked`, release ownership, and stop. Do not set done or close the Issue.
