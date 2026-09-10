# Session Bootstrap — Policy-bound separated A/V remux delivery

```text
GitHub Issue: #271
Task Contract: docs/tasks/271-r005-media-delivery/task.md
Expected worker: cloud
Expected environment: env:cloud
Handoff profile: docs/tasks/handoffs/cloud.md
```

Read `AGENTS.md`, Issue #271 and all comments, this Task Contract, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, current `main@2bf8ce4d27ce937a2649bccaa0fd3ffbf1bead10`, and accepted #265/#268 evidence.

Before claiming, verify `status:ready`, `env:cloud`, owner-free and package readback from GitHub. Implement only the generic policy-bound remux delivery contract in the Task Contract. Do not perform live Bilibili/login/target actions or phone/TV/VNC/CDP work; do not touch #191/#195; do not run local build/test/package/install. All required verification must use GitHub Actions with exact Candidate SHA. Post one `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, set `status:review` or `status:blocked`, release ownership and stop. Do not merge or close Issue.
