# Session Bootstrap — R005 Vault-bound candidate capture

- GitHub Issue: #257
- Task Contract: docs/tasks/257-r005-vault-candidate-capture/task.md
- Expected Worker: cloud
- Environment: env:cloud
- Handoff profile: docs/tasks/handoffs/cloud.md

Read AGENTS.md, Issue #257 and all relevant comments, the Task Contract, issue-lifecycle-protocol.md, execution-anchor-recovery-protocol.md and freshness-integration-protocol.md, then read accepted #243/#251 and current main. Confirm status:ready, env:cloud, no owner and exact base before claiming.

Execute only #257. All compilation, tests and packaging must run through GitHub Actions; no local build/test/install. Do not perform live login, Bilibili, tx-node, browser, VNC/CDP, phone or #191/#195 work. Keep profile/Secret material inside the Vault/worker boundary and preserve the separate #246 target gate.

After completion, post [EXECUTION REPORT] or [BLOCKER REPORT], set status:review, release ownership and stop. Do not merge/close or start another Task.

