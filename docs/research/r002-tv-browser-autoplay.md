# R002 TV Browser Remote Playback Probe

This document describes the reproducible probe delivered by Issue #6. It is
mechanics and instrumentation evidence only; a real television/browser and
audible observation are required for the R002 result in Issue #7.

## Probe surface

- `/display` is the viewport-immersive Web Display. It consumes the existing
  R001 Gateway media capability and never receives upstream headers.
- `/control` is a minimal same-origin phone/controller page.
- `POST /api/v1/display-probe/commands` accepts `{ "request_id": "..." }`.
  A repeated request ID is idempotent and does not enqueue another event.
- `/display` polls `/api/v1/display-probe/events?after=<cursor>` as the
  reconnectable remote-event transport. The event contains only a sequence,
  request ID and `play` kind.
- `/api/v1/display-probe/state` exposes bounded, structured diagnostics for
  the current probe run. It contains play resolve/reject results, browser
  error name/message, activation source, Fullscreen result, lifecycle,
  visibility, media-end and transport observations. It does not include media
  URLs, Cookie, Authorization, Vault or upstream credentials.
- `POST /api/v1/display-probe/reset` clears the in-memory probe evidence; the
  Display button also reloads the page. This is the reset boundary for each
  manual case.

Each remote command is consumed once by the Display page. The Display sets
`muted=false` and `volume=1`, calls `HTMLMediaElement.play()` once, and records
the Promise `resolve` or `reject` plus a bounded browser error. It does not
retry a rejected play or silently downgrade it to muted playback. The
“Press OK to enable remote playback” button is the explicit one-time
activation/bootstrap path. Fullscreen is optional; the page remains usable in
the viewport-immersive layout when the request is unavailable or rejected.

## Manual entry for Issue #7

1. Run the accepted candidate on the Gateway reachable by the TV and phone.
2. Open `/display?profile=tv` on the real TV/browser and use the probe’s
   **Reset / reload probe** button before each case. Open `/control` on the
   phone or use the equivalent same-origin POST.
3. Send a remote attempt from the phone:

   ```sh
   curl -fsS -X POST "$GATEWAY/api/v1/display-probe/commands" \
     -H "origin: $GATEWAY" \
     -H 'content-type: application/json' \
     -d '{"request_id":"case-a-1"}'
   ```

4. Read the Display status and save `/api/v1/display-probe/state` after each
   attempt. Record audible output separately; a `play` resolve is not by
   itself proof that the TV produced audible sound.
5. Use the same command surface for the repeated-play, idle, refresh,
   restart/resume and reconnect cases. Do not use browser autoplay flags,
   DevTools overrides, synthetic activation or a desktop browser as the TV
   acceptance evidence.

The physical verifier should record, for every case, the request ID, whether
the command arrived, audible result, `play()` result/error, activation needed,
Fullscreen result or viewport fallback, visibility/lifecycle and reconnect
observations. The final `PASS | CONDITIONAL PASS | FAIL | BLOCKED` classification
belongs only to Issue #7.
