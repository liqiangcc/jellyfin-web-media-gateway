# Task #119 — Establish deterministic local-only DevTools transport for Chrome Beta

## Identity

- Issue: #119
- Task ID: `ENV-ARM64-CHROME-BETA-LOCAL-CDP-TRANSPORT`
- Kind: environment transport verification / provisioning
- Parent blocked task: #118
- Upstream: #116 remains blocked; #117 remains draft
- Planning base: `d62b4a3cfdbde296413786c077ef35c444ae98b5`
- Eligible environment: `env:ubuntu-arm64`
- Worker: independent Ubuntu ARM64 Worker on accepted phone

## Trigger evidence

#118 Attempt 2 established all of the following:

- official `com.chrome.beta` is installed and distinct from `com.android.chrome`;
- verified launcher starts Chrome Beta successfully;
- Chrome Beta PID is present while running;
- a `chrome_devtools_remote` abstract socket is observed while Beta is running;
- Ubuntu-side temporary `socat` loopback bridge did not yield bounded `/json/version` or `/json/list` responses;
- teardown and safe-output boundaries passed.

Therefore this Task must not repeat browser provisioning. The only unresolved prerequisite is local-only DevTools transport usable from Ubuntu ARM64.

## Goal

Establish one deterministic and reversible transport:

`Ubuntu ARM64 Worker -> local/process-private endpoint -> exact Chrome Beta DevTools endpoint`

Then prove it with bounded field-only CDP health checks.

## Execution contract

### T0 — Claim and fixed-state check

1. Read live #119.
2. Claim only when OPEN + `status:ready` + `env:ubuntu-arm64` + no owner.
3. Confirm task package SHA supplied by Publication Gate.
4. Confirm #118 remains blocked and #117 remains draft.
5. Post `[EXECUTION CHECKPOINT]` before changing device state.

### T1 — Exact Chrome Beta endpoint identity

1. Confirm `com.chrome.beta` is installed.
2. Record only package identity/version and verified launcher identity.
3. Start only Chrome Beta with a bounded timeout.
4. Confirm Beta PID is present.
5. Inspect only `/proc/net/unix` entries relevant to Chrome DevTools and determine the exact socket identity attributable to the running Beta instance.
6. Do not enumerate normal Chrome tabs/profile/session state.

PASS T1 only when the Worker can select one exact Beta-owned DevTools socket rather than assuming a generic socket name.

### T2 — Preferred Termux-host transport

Preferred first path because #118's Ubuntu-side abstract-socket bridge failed:

1. Check whether Termux already has a trusted Unix/TCP bridge utility capable of `ABSTRACT-CONNECT` or equivalent.
2. If `socat` is absent in Termux, installation is permitted only from the already-configured official Termux repository. Do not download an APK/binary from arbitrary URLs.
3. From the Termux host context, connect only to the exact Beta DevTools abstract socket identified in T1.
4. Bridge it to the narrowest endpoint Ubuntu can consume:
   - first choice: shared filesystem Unix socket;
   - second choice: `127.0.0.1` listener if and only if it is reachable only locally and no public interface is exposed.
5. If a shared Unix socket is used, permissions must be the minimum required for Ubuntu to connect and the socket must be removed during teardown.

Do not bind `0.0.0.0`, LAN, Tailscale, or another externally reachable address.

### T3 — Fallback on-demand ADB transport

Run only if T2 cannot establish usable transport.

1. Check for a trusted packaged ADB client in Termux/Ubuntu repositories.
2. ADB use is allowed only as an on-demand local transport.
3. Do not permanently enable TCP adbd solely for this Task.
4. Do not bypass Android pairing, authorization, lock-screen, or other device confirmation.
5. If an ordinary local abstract-socket forward can be created safely, bind the client side only to loopback/process-local scope.
6. If safe ADB transport cannot be established within these rules, BLOCK rather than widening exposure.

### T4 — Bounded CDP proof

With Chrome Beta running and the accepted local transport active:

1. Probe `/json/version` with a hard timeout.
2. Parse only an allowlisted bounded summary such as:
   - response reachable yes/no;
   - browser product/version present yes/no or sanitized product family/version;
   - protocol version present yes/no.
3. Probe `/json/list` with a hard timeout.
4. Parse only:
   - valid array yes/no;
   - bounded target count;
   - page target present yes/no.
5. Do not print raw JSON, target URLs, titles, websocketDebuggerUrl, headers, body content, query strings, tokens, cookies, or profile/session data.

PASS T4 requires both bounded checks PASS.

### T5 — Teardown

Always run, including on BLOCK/FAIL:

1. Stop/remove every temporary bridge, listener, Unix socket, ADB forward and temp probe file created by this Task.
2. Force-stop only `com.chrome.beta`.
3. Verify:
   - Beta PID absent;
   - selected Beta DevTools socket absent;
   - temporary loopback listener absent;
   - temporary shared Unix socket absent;
   - no public/LAN/Tailscale DevTools listener exists;
   - normal `com.android.chrome` state unchanged.
4. Safe-output boundary PASS.

## Success criteria

Overall PASS requires all of:

- T1 exact Beta DevTools endpoint identity PASS;
- T2 or T3 deterministic local-only transport PASS;
- `/json/version` bounded field check PASS;
- `/json/list` bounded field check PASS;
- T5 teardown PASS;
- safe-output boundary PASS.

A transport that only opens a socket but cannot return the two bounded CDP health responses is not PASS.

## Block conditions

BLOCK if any of the following is required:

- use or inspection of normal `com.android.chrome` profile/tab/session state;
- public/LAN/Tailscale/`0.0.0.0` DevTools listener;
- arbitrary APK/binary download;
- permanent TCP adbd solely for this Task;
- bypass of device authorization/pairing/security confirmation;
- Bilibili/site navigation;
- widening to browser-diag MCP, yt-dlp, resolver, broker, R008, #116/#117/#67/#113/#68.

## Codex control-plane rule

This is not Task network evidence. For the phone Ubuntu Codex/GPT process only:

- remove inherited `NO_PROXY` and `no_proxy`;
- explicitly set HTTP/HTTPS/ALL proxy variables (upper and lower case as needed) to `http://127.0.0.1:7890`;
- this proxy authorization applies only to Codex/GPT control-plane traffic, not Chrome/CDP/Bilibili/task probes.

## Durable report

On PASS, post a bounded `[EXECUTION REPORT]`, move to `status:review`, release owner, STOP.

On blocker, post `[BLOCKER REPORT]` with the exact failed stage and sanitized evidence, move to `status:blocked`, release owner, STOP.
