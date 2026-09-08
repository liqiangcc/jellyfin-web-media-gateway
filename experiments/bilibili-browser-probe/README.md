# Bilibili browser acquisition probe

This directory is an experiment for Issues #165 and #169. Synthetic mode is
the default and uses a loopback allowlist broker. An explicit live mode is
available only to a target runbook with a plugin-owned opaque selector:

```text
BILIBILI_PROBE_ALLOW_LIVE=1 node probe.mjs --mode live --selector bilibili:BV14V411W7r5:part-2
```

The caller cannot provide a URL, CDP endpoint, browser profile, cookies,
authorization headers, proxy, or short-lived media URL. Live navigation is
constructed by `plugins/bilibili/live_selector.mjs`; its broker owns DNS,
public-address checks, redirects and CONNECT. Candidate URLs remain ephemeral
server-side and only bounded sanitized metadata is emitted. This is not a
production SiteAdapter and is not enabled by Gateway.

The hosted workflow installs the exact locked `playwright-core@1.55.0` package
with lifecycle scripts disabled, copies its regular runtime files into the
manifest-addressed artifact, and discovers an external system Chrome. The
artifact contains no browser binary and a clean target directory needs no
`npm install`, workspace `node_modules`, profile, proxy, or secret. Verify the
downloaded manifest before running the bundle's no-install command. Hosted CI
never enables live mode.
