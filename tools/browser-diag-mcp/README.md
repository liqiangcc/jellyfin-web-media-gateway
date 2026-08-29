# browser-diag MCP

`browser-diag-mcp` is a standalone, Python-standard-library-only diagnostic
server for the dedicated Android browser used by Issue #116. It connects to a
local shared Unix socket carrying Chrome DevTools traffic; it never opens a TCP
listener.

The MCP surface is fixed to:

- `health`
- `list_targets`
- `open_url`
- `reload`
- `network_capture_start`
- `network_summary`
- `network_capture_stop`

There is no generic CDP command tool. `open_url` accepts only HTTPS URLs whose
exact DNS host was provided at process startup. MCP clients cannot read or
change that allowlist. Target handles are process-local opaque values, and the
server does not return target titles or URLs. Network events are reduced on
ingress to bounded status classes and enums; headers, bodies, DOM, storage,
raw event streams, full URLs, tokens, and media data are neither returned nor
logged.

## Local deterministic tests

The integration fixture is an AF_UNIX fake CDP server; it does not require a
browser or network access.

```sh
python3 -m unittest discover -s tools/browser-diag-mcp/tests -v
python3 -m py_compile tools/browser-diag-mcp/browser_diag_mcp.py tools/browser-diag-mcp/tests/test_browser_diag_mcp.py
```

## ARM64 neutral-proof handoff (do not run in cloud)

Issue #119 accepted a Termux-host AF_UNIX relay from the exact dedicated Chrome
Beta abstract DevTools endpoint to the Ubuntu-visible shared socket
`/data/local/tmp/issue-119-chrome-beta-cdp.sock`. The ARM64 Worker must first
recreate that already-accepted on-demand relay and verify its local-only socket
permissions. It must not attach to stable `com.android.chrome`.

With the relay active, start the MCP for the neutral host `example.com`:

```sh
python3 tools/browser-diag-mcp/browser_diag_mcp.py \
  --cdp-unix-socket /data/local/tmp/issue-119-chrome-beta-cdp.sock \
  --allow-host example.com \
  --browser-family ChromeBeta
```

Configure the ARM64 Worker MCP client to launch that exact argv over stdio, then
perform only this sequence:

1. `health` and require `transport_status=ready`.
2. `list_targets`; select its single opaque page `target_id` without publishing
   a title or URL.
3. `network_capture_start(target_id)`.
4. `open_url(target_id, "https://example.com/")`.
5. `network_summary(target_id)` and record only the returned field names and
   bounded values.
6. `network_capture_stop(target_id)` and terminate the MCP process.
7. Stop the Issue #119 relay, remove the shared socket, force-stop only
   `com.chrome.beta`, and verify the Beta endpoint, relay socket, and any
   listener are absent. Do not stop, enumerate, or inspect stable Chrome.

The neutral proof is target evidence. These instructions document it but do
not claim that it has run in cloud.
