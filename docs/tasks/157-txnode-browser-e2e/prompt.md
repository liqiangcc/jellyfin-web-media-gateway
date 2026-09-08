# Session Bootstrap — TXNODE-BROWSER-E2E Deploy and verify browser Control on tx-node

你正在执行 liqiangcc/jellyfin-web-media-gateway 的 GitHub Issue #157。

## Execution Context

~~~text
GitHub Issue: #157
Task Contract: docs/tasks/157-txnode-browser-e2e/task.md
Expected worker: cloud
Expected environment label: env:cloud
Downstream handoff profile: docs/tasks/handoffs/cloud.md
~~~

## Start

Use:

~~~text
$task-worker Execute Issue #157 using docs/tasks/157-txnode-browser-e2e/prompt.md.
~~~

Before claiming, read AGENTS.md, Issue #157 and relevant comments, this Task Contract, the lifecycle/recovery/freshness protocols, accepted #154, and .github/workflows/browser-control-e2e.yml. Confirm status:ready + env:cloud + no active owner, freeze current origin/main, and preflight tx-node plus Chrome DevTools MCP.

This is target deployment and verification. Build/test binaries only through GitHub-hosted Actions; never compile locally or on tx-node. Transfer only exact SHA-256 verified artifact and run Gateway under gateway-verify uid/gid 1001 with loopback-only binding. Use a fresh isolated Chrome DevTools MCP context on the tx-node browser for the product Display/Control loop. Direct Bilibili activity is diagnostic only; #67/#68 and phone/TV work are out of scope.

Follow the Worker lifecycle: claim Attempt N, post one checkpoint only after a durable anchor if useful, perform only #157, then post EXECUTION REPORT or BLOCKER REPORT, set status:review or status:blocked, release ownership and stop. Do not merge, close Issue #157 or start another Task.

