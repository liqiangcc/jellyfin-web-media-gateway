# Task #119 — ENV-ARM64-CHROME-BETA-LOCAL-CDP-TRANSPORT

## Identity

- Issue: #119
- Kind: environment transport verification / provisioning
- Planning base: `d62b4a3cfdbde296413786c077ef35c444ae98b5`
- Environment: `env:ubuntu-arm64`
- Parent authority: #118 Coordinator Review Attempt 2, Decision `SPLIT`
- Upstream accepted fact: official `com.chrome.beta` version `153.0.8010.18` is installed, distinct from `com.android.chrome`, starts via `com.chrome.beta/com.google.android.apps.chrome.Main`, and exposes a `chrome_devtools_remote` abstract socket during bounded execution.
- #118 remains blocked until this Task PASS.
- #116 remains blocked; #117 remains draft.

## Goal

Establish one deterministic, reversible, local-only transport from the Ubuntu ARM64 Worker to the **exact Chrome Beta DevTools endpoint**, then prove bounded `/json/version` and `/json/list` health and deterministic teardown.

Do not reinstall/provision a browser. Do not implement browser-diag MCP. Do not access Bilibili.

## Control-plane rule

Codex/GPT control-plane on this phone must:

- remove inherited `NO_PROXY` and `no_proxy` for the Codex process;
- explicitly use existing `127.0.0.1:7890` for HTTP/HTTPS/ALL proxy variables.

This proxy rule is **control-plane only**. Task browser/CDP traffic must not use that proxy.

## T0 — claim and fixed authority

Before device work:

1. Re-read live #119.
2. Require `OPEN + status:ready + env:ubuntu-arm64 + no owner`.
3. Claim #119 according to repository Worker protocol.
4. Record the exact Task Package SHA.
5. Do not substitute moving `main` after claim.

If the gate does not match, STOP without device work.

## T1 — exact Beta endpoint identity

Start only the dedicated package using the verified launcher:

`com.chrome.beta/com.google.android.apps.chrome.Main`

Use hard timeouts for Android activity-manager operations.

Bounded observations only:

- Chrome Beta PID present/absent;
- candidate DevTools abstract socket names from `/proc/net/unix`;
- associate the endpoint with the Beta process/package using bounded process/socket ownership evidence where available;
- do not enumerate targets/tabs from stable Chrome;
- do not inspect any profile/session storage.

Do **not** assume generic `chrome_devtools_remote` is the correct endpoint merely because it exists. The transport proof must use the endpoint attributable to Chrome Beta.

If exact Beta endpoint identity cannot be established safely, BLOCK.

## T2 — preferred Termux-host bridge

First determine whether the Android/Termux host context can connect to the exact Beta abstract socket.

Preferred path:

`Chrome Beta abstract DevTools socket -> Termux-host local bridge -> Ubuntu-visible process-local/shared Unix socket or 127.0.0.1 endpoint`

Rules:

- Prefer already-installed trusted tools.
- If Termux `socat` is required and absent, installation is allowed only from the configured official Termux repository.
- Any TCP listener must bind exactly `127.0.0.1`; prefer shared Unix socket where practical.
- Never bind `0.0.0.0`, LAN, Wi-Fi, Tailscale, or another remotely reachable address.
- No proxy for CDP traffic.
- Temporary files/sockets must use Task-specific paths and be removed during teardown.

Perform a bounded transport handshake before HTTP health checks. Do not output raw payloads.

## T3 — ADB fallback only if T2 cannot work

Only if the Termux-host bridge is not viable, investigate a standard on-demand ADB abstract-socket forward.

Allowed only when all are true:

- ADB tooling is obtained from a trusted platform/Termux package source;
- no device security/confirmation is bypassed;
- no permanent TCP adbd is enabled solely for this Task;
- forwarding is on-demand and deterministic to tear down;
- exposure remains loopback/process-local only.

Do not change Android security settings merely to force PASS. If safe ADB transport is unavailable, BLOCK.

## T4 — bounded neutral CDP proof

With the exact Beta endpoint and accepted transport active, validate only:

### `/json/version`

PASS requires a parseable response containing expected bounded protocol/product metadata. Output only booleans/coarse identifiers needed for the Task, for example:

- `version_health=PASS|FAIL`
- `browser_product_family=ChromeBeta|other`
- `protocol_version_present=yes|no`

Do not output raw JSON, websocket debugger URLs, user-agent strings if they contain unnecessary detail, or arbitrary fields.

### `/json/list`

PASS requires a parseable bounded target list. Output only:

- `target_health=PASS|FAIL`
- bounded `target_count`
- bounded target-type summary if needed.

Never output titles, target URLs, websocket URLs, page content, headers, cookies, storage, or tokens.

Both checks must PASS. Socket existence alone is not PASS.

## T5 — teardown

Order:

1. stop temporary bridge/forward;
2. force-stop only `com.chrome.beta` with a hard timeout;
3. remove Task temp files/sockets;
4. verify Beta PID absent;
5. verify exact Beta DevTools endpoint absent;
6. verify loopback bridge/forward listener absent;
7. verify stable `com.android.chrome` state is unchanged;
8. safe-output check PASS.

Cleanup failures are blockers; do not hide them.

## Hard boundaries

- Dedicated `com.chrome.beta` only.
- Never attach to, enumerate, read, start for diagnosis, or copy normal `com.android.chrome` profile/tabs/state.
- No Cookie/Auth/login/password/local-storage/session-storage/cache/history/profile extraction or replay.
- No raw DevTools event dumps or raw `/json/*` responses.
- No page/body data, arbitrary headers, URLs/titles, signed/query URLs, tokens, or media payloads.
- No public/LAN/Tailscale/`0.0.0.0` DevTools listener.
- No Bilibili/site navigation.
- No proxy for browser/CDP Task traffic.
- No UA/Referer/header spoofing, fingerprint emulation, CAPTCHA/challenge/access-control bypass.
- No #116/#117/#67/#113/#68 execution.
- No yt-dlp/generic-ytdlp/R008/broker/resolver/sandbox widening.
- No product/runtime changes.

## Success

`PASS` requires all:

- exact Chrome Beta DevTools endpoint identity established;
- deterministic local-only transport usable from Ubuntu ARM64;
- `/json/version` field check PASS;
- `/json/list` field check PASS;
- teardown PASS;
- safe-output boundary PASS;
- stable Chrome unchanged.

## Blocked

`BLOCKED` if a deterministic local-only path cannot be established within these boundaries, or if health/teardown cannot be proven.

On blocker:

1. post `[BLOCKER REPORT]` with sanitized bounded evidence;
2. set `status:blocked`;
3. release owner;
4. STOP.

## Normal finish

This is verification/environment-only and does not require a product PR. On PASS:

1. post a durable execution report with transport authority and sanitized health/teardown evidence;
2. set `status:review`;
3. release owner;
4. STOP for Coordinator Review.
