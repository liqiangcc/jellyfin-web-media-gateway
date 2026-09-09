# Session Bootstrap — R005 Generic Browser Auth Experience

- GitHub Issue: #259
- Task Contract: `docs/tasks/259-r005-generic-browser-auth-experience/task.md`
- Expected Worker: cloud
- Environment: env:cloud
- Handoff profile: `docs/tasks/handoffs/cloud.md`

Read `AGENTS.md`, Issue #259 and all relevant comments, the Task Contract, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, and the accepted #243/#251/#255/#257 contracts before claiming.

Confirm `status:ready`, env eligibility, no active owner, and the exact current main/base before starting. Execute only #259. All compilation, tests and packaging must run through GitHub Actions; do not run local build/test/install. Do not perform live login, Bilibili, tx-node, browser/VNC/CDP, phone/TV/deployment or #191/#195 work. Preserve the Vault, Origin/CSRF, SSRF, opaque-reference and failure-isolation boundaries.

After completion, post `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, set `status:review`, release ownership and stop. Do not merge or close the Issue or start another Task.