# Offline Bilibili browser acquisition probe

This directory is an experiment for Issue #165. It uses only a synthetic page
and a loopback allowlist broker. It does not accept a URL, CDP endpoint, browser
profile, cookies, authorization headers, or a short-lived media URL from a
caller. It is not a production SiteAdapter and is not enabled by Gateway.

The hosted workflow installs the pinned `playwright-core` package, discovers an
allowlisted preinstalled Chromium, runs the contract and containment tests, then
builds a manifest-addressed artifact. The artifact consumer starts from the
downloaded artifact and reads the synthetic media after the browser exits.
