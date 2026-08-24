# R004 Jellyfin DisplayAdapter PoC — Attempt Evidence Skeleton

This document describes the repository-side adapter mechanics for R004-PREP (Issue #15). It is not a real Jellyfin Android TV result and must not be used to declare R004 `PASS`, `CONDITIONAL PASS`, or `FAIL`. Physical/client evidence belongs to Issue #16.

## Boundary

```text
Playback Coordinator / R007 authority
  → display-adapter-api::DisplayAdapter
    → jellyfin-display-adapter::JellyfinDisplayAdapter
      → configured Jellyfin Server
```

`display-adapter-api` carries `DisplayContext` on every request and result. The adapter only returns candidate/probe/status evidence; it has no reference to `PlaybackSession`, `active_display`, or the R007 handoff commit path. `GatewayMediaCapability` contains only a Gateway-issued media URL and display identity. It has no upstream Cookie, Authorization, or Session Vault access.

The Jellyfin API key is injected into `X-Emby-Token` by the adapter's server-side `JellyfinCredential`. It is never serialized, returned in a display result, placed in a media capability, or emitted by fixture test evidence. The endpoint is a `ConfiguredJellyfinService` supplied by deployment configuration; there is no browser/plugin URL or `allow_private` override.

## Protocol behavior

- `GET /Sessions` discovers sessions, sorts them by stable session ID, and filters offline displays from registration. Probe/prepare select exactly one online target or return a stable missing/offline/ambiguous error.
- Start posts a temporary Gateway media URL and `StartPositionTicks`, then polls `GET /Sessions` until `IsPlaying=true` and `IsPaused=false`. A 2xx command response without that observation returns `PlaybackNotConfirmed`, not success.
- Pause, unpause, seek, stop, and status map to Jellyfin session control endpoints. Adapter errors distinguish auth, server, target, media, rejection, timeout, and confirmation failures.
- Jellyfin ticks use 10,000 ticks per millisecond. Conversion uses checked multiplication and nearest-millisecond reporting; tests retain requested, reported, and error positions for later handoff observation.

## Reproducible Issue #16 manual procedure

Issue #16 should use a trusted Gateway candidate and a Jellyfin Server on the same LAN as the Android TV/client.

1. Configure one Jellyfin service endpoint in the Gateway deployment and inject its API key server-side. Do not paste the key into a browser URL, a display page, a media capability, or a log collection command.
2. Register/identify the Android TV session from adapter discovery. Record the session/device ID, online state, and adapter capabilities.
3. Resolve one non-DRM R001 Gateway media capability bound to test `session_id`, `item_id`, `item_revision`, and `resource_id`. Record the Gateway expected position `P` and the R007 display generation/context.
4. Run probe → prepare → start at `P`; record command response time, first confirmed `IsPlaying`, Jellyfin reported ticks/ms, and visible/audible TV position. Keep the source display running until the Coordinator's handoff commit is confirmed.
5. Exercise pause, resume, seek, stop, and status. Record each command result, reported position, and whether the TV actually changed state.
6. Repeat with Jellyfin unavailable, the TV offline, incompatible media, an expired Gateway capability, and delayed/no-playback confirmation. For each failure record the stable adapter error and whether the original Web Display kept playing.

If Jellyfin reports command accepted but the TV does not play, record exact candidate SHA, adapter request timestamp, HTTP status, target session ID, `StartPositionTicks`, every PlayState sample until timeout, TV model/Jellyfin client version, and Gateway/R007 context. Do not call that a successful start and do not stop the committed source display based on the 2xx response alone.

## Hosted evidence boundary

Required J1/J2 GitHub Actions jobs run against the exact pull-request head SHA. J1 runs formatting, clippy, the workspace suite, and accepted R001/R007 regressions. J2 runs the deterministic API fixture, command/failure coverage, position conversion, and fake-key redaction check. Hosted fixture evidence proves adapter mechanics only; it cannot prove Android TV playback, audible output, physical remote behavior, or final R004 research status.
