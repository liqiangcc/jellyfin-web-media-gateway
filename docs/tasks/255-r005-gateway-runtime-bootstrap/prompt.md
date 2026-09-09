# Session Bootstrap — R005 Gateway runtime bootstrap

- GitHub Issue: #255
- Task Contract: docs/tasks/255-r005-gateway-runtime-bootstrap/task.md
- Expected Worker: cloud
- Environment: env:cloud
- Handoff profile: docs/tasks/handoffs/cloud.md

Start with:
1. Read AGENTS.md, the Issue and all relevant comments.
2. Read the Task Contract, docs/tasks/issue-lifecycle-protocol.md, docs/tasks/execution-anchor-recovery-protocol.md, and docs/tasks/freshness-integration-protocol.md.
3. Read the accepted #237/#240/#243/#248/#251 contracts and current main before coding.
4. Confirm status:ready, env:cloud, no active owner, and exact base before claiming Attempt 1.

Execute only #255. All compilation, tests and packaging must run through GitHub Actions; do not compile, test, install or package locally. Do not perform live Bilibili/login/tx-node/browser/VNC/CDP actions, phone deployment, or #191/#195 work. Preserve the distinction between implementation, hosted verification, and the later authorized target proof in #246.

After completion, post the standard [EXECUTION REPORT] or [BLOCKER REPORT], set the Issue to status:review, release ownership, and stop. Do not start #246 or another Task automatically.

