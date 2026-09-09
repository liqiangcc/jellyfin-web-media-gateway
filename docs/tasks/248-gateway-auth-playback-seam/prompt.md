# Session Bootstrap — Gateway authenticated Browser Worker playback seam

```text
GitHub Issue: #248
Task Contract: docs/tasks/248-gateway-auth-playback-seam/task.md
Expected worker: cloud
Expected environment: env:cloud
Handoff profile: docs/tasks/handoffs/cloud.md
```

Read `AGENTS.md`, Issue #248 and all comments, this Task Contract, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, and accepted #237/#240/#243 evidence.

Confirm `status:ready`, `env:cloud`, no active owner, and matching capabilities before claim. Claim Attempt N, implement only this contract, and preserve the existing Vault/R008/PlaybackSession/Display/Site Plugin boundaries. Do not run local build/test/package/install. Use GitHub Actions on the exact Candidate SHA. Do not perform live Bilibili/login/tx-node/phone/TV/VNC/CDP activity, do not touch #191/#195, and do not claim playback.

On success post `[EXECUTION REPORT]` with Candidate and exact Actions evidence, set `status:review`, release ownership and stop. On blocker post `[BLOCKER REPORT]`, set `status:blocked`, release ownership and stop. Do not close the Issue or start #246.
