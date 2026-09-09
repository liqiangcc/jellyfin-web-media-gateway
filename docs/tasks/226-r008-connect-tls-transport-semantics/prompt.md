# Session Bootstrap — Resolve CONNECT TLS transport semantics after navigation rejection

You are executing GitHub Issue #226 in `liqiangcc/jellyfin-web-media-gateway`.

```text
GitHub Issue: #226
Task Contract: docs/tasks/226-r008-connect-tls-transport-semantics/task.md
Expected worker: cloud
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Model: gpt-5.6-luna, reasoning high
Fast: enabled for this Task
```

Before any Attempt, read the live Issue and all relevant comments, `AGENTS.md`, the Task Contract, lifecycle/recovery/freshness protocols, and the canonical architecture/security documents named by the contract. Confirm Publication Gate, `status:ready`, `env:cloud`, no owner, and the required capabilities before claiming. If those conditions are absent, stop.

Use GitHub-hosted Actions for implementation verification and artifacts. No local build/test/package/install is allowed. The transport investigation is limited to the broker CONNECT/request semantics and deterministic bounded regressions described by `task.md`; do not widen authority or weaken security. Do not process, claim, edit, or wait on #191/#195, and do not run target, browser, Bilibili, playback, VNC/CDP, phone, or TV work until the contract explicitly authorizes a later Coordinator-controlled target step.

After the exact Candidate and hosted evidence are complete, follow the Task Contract’s reporting protocol: post `[EXECUTION REPORT]` or a genuine `[BLOCKER REPORT]`, set the Issue to `status:review` or `status:blocked`, release ownership, and stop. Do not set done or close the Issue.
