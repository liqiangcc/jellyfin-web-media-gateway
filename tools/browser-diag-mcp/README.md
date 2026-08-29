# Browser diagnostic MCP

This is a standalone, stdlib-only MCP for the Issue #116 dedicated Chrome Beta
diagnostic path. It accepts one operator-configured filesystem `AF_UNIX` relay
socket and communicates over stdio using JSON-RPC/MCP messages.

The relay and Chrome Beta lifecycle are external prerequisites. The tool does
not start a browser, create a listener, use TCP, inspect browser profiles, or
remove the relay socket. The operator must stop the Termux-host AF_UNIX relay
and Chrome Beta after the neutral proof.

The repository also includes the narrow relay used by the accepted transport:

```sh
/data/data/com.termux/files/usr/bin/python3 \
  tools/browser-diag-mcp/af_unix_relay.py \
  --listen /data/local/tmp/issue-119-chrome-beta-cdp.sock \
  --upstream @chrome_devtools_remote
```

Run that process in the Termux host context, keep it attached for the proof,
and stop it with `SIGINT`/`SIGTERM`; it removes only its explicitly configured
shared socket. The upstream is fixed to an Android abstract AF_UNIX endpoint,
and the relay has no TCP mode.

The allowlist is fixed at process startup. Example:

```sh
python3 tools/browser-diag-mcp/browser_diag_mcp.py \
  --socket /data/local/tmp/issue-119-chrome-beta-cdp.sock \
  --allowed-host example.com
```

The exact socket path is deployment state supplied by the accepted #119
transport; it is not a repository default. Only the following tools are
exposed: `health`, `list_targets`, `open_url`, `reload`,
`network_capture_start`, `network_summary`, and `network_capture_stop`.

Outputs contain status classes, bounded enums, opaque in-process target handles,
and coarse timing only. They do not contain URLs, titles, bodies, arbitrary
headers, cookies, authorization, storage, tokens, media payloads, or raw CDP
events. `open_url` accepts only HTTPS URLs without query strings/fragments or
userinfo whose exact hostname was configured at startup.

Run focused tests with:

```sh
python3 -m unittest discover -s tools/browser-diag-mcp/tests -v
```

## ARM64 neutral proof

Run this only in the accepted phone Ubuntu/Termux environment and only with the dedicated `com.chrome.beta` browser. Browser/CDP task traffic must have all HTTP proxy variables unset; the Codex control plane may continue to use `127.0.0.1:7890` in its own process.

1. Confirm stable Chrome is not running, then start only the verified Beta launcher:

```sh
unset HTTP_PROXY HTTPS_PROXY ALL_PROXY http_proxy https_proxy all_proxy NO_PROXY no_proxy
/system/bin/pidof com.android.chrome >/dev/null 2>&1 && { echo stable_chrome_running; exit 1; } || true
/system/bin/am force-stop com.chrome.beta
/system/bin/am start -n com.chrome.beta/com.google.android.apps.chrome.Main >/dev/null
```

2. Wait boundedly for the exact accepted Beta endpoint, then start the Termux-host Python relay on a filesystem Unix socket visible to Ubuntu:

```sh
RELAY=/data/local/tmp/issue-116-chrome-beta-cdp.sock
for i in 1 2 3 4 5 6 7 8 9 10; do
  grep -q '@chrome_devtools_remote' /proc/net/unix && break
  sleep 0.5
done
grep -q '@chrome_devtools_remote' /proc/net/unix
rm -f "$RELAY"
/data/data/com.termux/files/usr/bin/python3 tools/browser-diag-mcp/af_unix_relay.py \
  --listen "$RELAY" --upstream @chrome_devtools_remote &
RELAY_PID=$!
trap 'kill "$RELAY_PID" 2>/dev/null || true; wait "$RELAY_PID" 2>/dev/null || true; rm -f "$RELAY"; /system/bin/am force-stop com.chrome.beta >/dev/null 2>&1 || true' EXIT INT TERM
for i in 1 2 3 4 5 6 7 8 9 10; do [ -S "$RELAY" ] && break; sleep 0.2; done
[ -S "$RELAY" ]
```

3. Start the MCP with an immutable neutral-host allowlist. The client must retry `health` only within a small bounded startup window until both version/target health pass; socket appearance alone is not CDP readiness. After health PASS, an MCP client must call, in order: `initialize`, `tools/list`, `health`, `list_targets`, `network_capture_start(target-1)`, `open_url(https://example.com/)`, `network_summary`, and `network_capture_stop`. The proof may report only the bounded fields defined by this tool; it must not publish page bodies, headers, URLs with query strings, cookies, auth state, raw CDP events, or browser profile data.

```sh
python3 tools/browser-diag-mcp/browser_diag_mcp.py \
  --socket "$RELAY" \
  --allowed-host example.com \
  --browser-product-family ChromeBeta
```

4. Exit the MCP/client and trigger the trap. Final checks must show the relay socket absent, Beta stopped, stable `com.android.chrome` still absent, and no TCP DevTools listener on `0.0.0.0`, LAN, or Tailscale.
