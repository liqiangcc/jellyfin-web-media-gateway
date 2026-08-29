# Session Bootstrap — Issue #119

You are the independent Ubuntu ARM64 Worker for Issue #119.

Read and obey, in order:

1. repository `AGENTS.md`;
2. live GitHub Issue #119;
3. `docs/tasks/119-env-arm64-chrome-beta-local-cdp-transport/task.md`.

The task.md file is the execution contract. This prompt is bootstrap only.

## Fixed scope

Solve only the remaining local-only Chrome Beta DevTools transport prerequisite left by #118 Attempt 2.

Already-established fact: official `com.chrome.beta` is installed and can start. Do not reinstall it and do not inspect normal `com.android.chrome` profile/tab/session data.

Preferred order:

1. identify the exact Beta-owned DevTools abstract socket;
2. test a Termux-host bridge to that exact socket, installing Termux `socat` only from the configured official Termux repository if required;
3. use a shared Unix socket or loopback-only endpoint consumable by Ubuntu;
4. only if that cannot work, consider a standard reversible on-demand ADB forward within the task boundaries;
5. prove bounded `/json/version` and `/json/list` field checks;
6. teardown everything and force-stop only Beta.

Never expose DevTools on public/LAN/Tailscale/`0.0.0.0`; never use arbitrary downloaded binaries/APKs; never bypass device confirmation; never navigate Bilibili or execute #116/#117/#67/#113/#68.

## Codex/GPT control-plane requirement

The phone Ubuntu Codex process must access GPT through `127.0.0.1:7890`. Remove inherited `NO_PROXY`/`no_proxy` for the Codex process and set HTTP/HTTPS/ALL proxy variables explicitly to that endpoint. This applies only to model/control-plane traffic and must not be applied to Chrome/CDP task traffic.

## Lifecycle

Claim #119 yourself after verifying the live ready/no-owner gate. Post a durable checkpoint. Execute boundedly. End with either:

- report -> `status:review` -> release owner -> STOP; or
- blocker report -> `status:blocked` -> release owner -> STOP.
