# Session Bootstrap — Issue #119

You are the independent `ubuntu-arm64` Worker for Issue #119.

Read and obey:

1. repository `AGENTS.md`;
2. `docs/tasks/119-env-arm64-chrome-beta-local-cdp-transport/task.md` as the canonical execution contract;
3. live GitHub Issue #119 before claim.

Do not execute from this bootstrap summary alone.

Key reminders:

- Task is transport-only: exact Chrome Beta DevTools endpoint -> deterministic local-only transport -> bounded `/json/version` + `/json/list` -> teardown.
- `com.chrome.beta` is already installed; do not reprovision another browser.
- Never touch normal `com.android.chrome` profile/tabs/state.
- Prefer exact endpoint identification and a Termux-host process-local/loopback bridge; ADB is fallback only if safe/on-demand.
- No public/LAN/Tailscale/`0.0.0.0` DevTools exposure.
- No Bilibili/site navigation and no raw browser/CDP sensitive output.
- Codex/GPT control-plane only: remove inherited `NO_PROXY/no_proxy` and use existing `127.0.0.1:7890`; do not apply this proxy to Task browser/CDP traffic.
- Use hard timeouts for Android lifecycle and transport probes.
- PASS requires both bounded CDP health checks plus teardown.
- On blocker: report -> `status:blocked` -> release owner -> STOP.
- On PASS: report -> `status:review` -> release owner -> STOP.
