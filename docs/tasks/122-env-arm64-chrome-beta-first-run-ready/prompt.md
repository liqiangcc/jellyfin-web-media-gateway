# Session Bootstrap — Issue #122

You are an independent Ubuntu ARM64 environment-provisioning Worker.

Execute Issue #122 using `docs/tasks/122-env-arm64-chrome-beta-first-run-ready/task.md` as the canonical contract.

## Entry gate

1. Live-read #122, its Publication Gate, #117 Attempt 1 blocker review, #116 Final Acceptance and #119 Final Acceptance.
2. Proceed only if #122 is `OPEN + status:ready + env:ubuntu-arm64 + no owner`.
3. Claim durably, read back owner/state, and post one bounded `[EXECUTION CHECKPOINT]` before device mutation.

## Goal

Repair only the reusable dedicated Chrome Beta environment state:

- complete Beta first-run inside `com.chrome.beta` only;
- remain logged out/no account/no sync/no profile import;
- restore verified launcher -> exact Beta endpoint -> accepted Termux AF_UNIX -> bounded MCP health;
- prove that state survives a force-stop/restart;
- do **not** `pm clear` at successful teardown.

## Permission boundary

Full local command permissions may be granted by the operator to avoid per-command approval, but that does not widen Task scope.

- Codex/GPT control plane may use `127.0.0.1:7890`.
- Browser/CDP Task traffic must clear/bypass HTTP proxy variables.
- Never use or inspect normal `com.android.chrome` state.

## P1 first-run interaction

Use only bounded dedicated-Beta UI inspection needed to identify first-run choices. Prefer the least-state path:

- required terms acceptance only if unavoidable;
- decline sign-in/account/sync/import/default-browser/personalization when optional;
- no Google/account addition;
- no normal-browser import.

If a choice is ambiguous about personal-state import or account use, BLOCK rather than guess.

Do not publish the raw UI hierarchy; report only the semantic choices made.

## Verification

Run P2-P4 exactly as the Task Contract requires:

- verified Beta launcher no longer lands in FirstRunActivity;
- stable Chrome absent;
- exact Beta endpoint two bounded cycles;
- accepted Termux Python AF_UNIX local relay;
- bounded version/list or MCP health/list-target fields only;
- force-stop/remove temp relay;
- restart without `pm clear` and prove first-run does not return;
- final cleanup while preserving initialized logged-out state.

No Bilibili/public-site navigation and no #117 measurements.

## Report

PASS: post `[EXECUTION REPORT]`, transition to `status:review`, release owner and STOP.

BLOCK: post sanitized `[BLOCKER REPORT]`, transition to `status:blocked`, release owner and STOP.

Do not merge/close/done or create/execute another Task.
