# Session Bootstrap — Issue #118

You are the independent `ubuntu-arm64` Worker for Issue #118.

Read GitHub live state and `docs/tasks/118-env-arm64-android-diag-browser-provision/task.md` first.

Claim only if Issue #118 is OPEN + `status:ready` + `env:ubuntu-arm64` + no owner. The claim must be durable on GitHub before any provisioning action.

## Mission

Provision only the prerequisites needed by blocked #116:

- a dedicated Android diagnostic browser distinct from normal `com.android.chrome`;
- an on-demand local-only CDP transport usable from the Ubuntu ARM64 diagnostic environment.

Start with bounded discovery. Prefer an already-installed separate official Chrome Beta/Dev/Canary/Chromium-style package. If installation is required, use only a trusted device source or operator-provided official artifact. If device/operator confirmation is required, report BLOCKED and STOP; do not bypass it.

For CDP transport, use the narrowest reversible local mechanism. Loopback/process-local/abstract-socket only. Never expose DevTools to `0.0.0.0`, LAN, Tailscale or public networks, and never enable permanent TCP adbd solely for this Task.

## Strict prohibitions

Do NOT:

- attach to/start/enumerate the user's normal Chrome profile/tabs for diagnosis;
- copy Cookie/Auth/login/profile/storage/cache/history/password state;
- download arbitrary APKs or use unofficial mirrors;
- navigate Bilibili;
- execute #67/#113/#117/#68 or yt-dlp/generic-ytdlp/resolver/broker/R008 paths;
- implement the browser-diag MCP here;
- change UA/Referer/headers, emulate fingerprint, use proxy/rotation or bypass access controls.

## Evidence

Publish only bounded capability metadata permitted by task.md. A neutral version/target health proof is sufficient. No page bodies, raw CDP dumps, arbitrary headers, full query URLs, tokens or media URLs.

On PASS, write `[EXECUTION REPORT]`, transition to `status:review`, release owner, STOP.

On prerequisite failure, write `[BLOCKER REPORT]`, transition to `status:blocked`, release owner, STOP.

Do not publish or execute #116/#117 yourself.