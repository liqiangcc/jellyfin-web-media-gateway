# R001 Media Path Proof — Evidence Record

Date: 2026-08-24

Issue: #3

Attempt: 1

Result: **PASS for the R001 media-path scope**

This record is evidence for R001 only. It does **not** close the full Web-only Core feasibility gate and does not claim R002, R003, R004, R005, R006, R007, or the full R008 scope.

## Proven path

The executable path exercised by the proof is:

```text
public media input
→ SiteAdapterRegistry
→ generic-direct SiteAdapter
→ SourceLocator(v1)
→ SiteAdapter.resolve
→ ResolvedMedia
→ scoped short-lived Media Gateway capability
→ Media Gateway
→ Web Display <video>
```

The media capability is bound to stable test identities for `session_id`, `item_id`, `item_revision`, and `resource_id`. These fields are used only as opaque identity binding for R001 capability/replay rejection. R001 does not define Playback command CAS, telemetry revision, media-refresh freshness, display generation, handoff generation, or authority semantics; those remain owned by R007.

## Public evidence

Primary public MP4 used by the GitHub Actions proof:

- host: `raw.githubusercontent.com`
- repository source: `mediaelement/mediaelement-files/big_buck_bunny.mp4`
- request path exposed to the browser: Gateway capability path, not the upstream URL

Public HLS proof source:

- host: `devstreaming-cdn.apple.com`
- master playlist, rewritten variant, and rewritten segment path were fetched through Gateway capabilities

The first proposed Google sample MP4 returned HTTP 403 from the GitHub-hosted runner. That was treated as an unavailable external source, not as a Gateway PASS. The proof source was changed and then re-run successfully.

## Browser proof

Verified executable candidate: `217859a9f620be8f8874cab8a21b10ccd2b2bc6c`

GitHub Actions run: `32730313377`

Execution plane:

- GitHub-hosted `ubuntu-24.04` x86_64
- runner image `20260816.277.1`
- Rust `1.97.1`
- Google Chrome `151.0.7922.137`
- ffmpeg `6.1.1-3ubuntu5`
- Node `v22.23.2`

Observed MP4 browser result:

- duration: `60.095011s`
- readyState: `4`
- play advanced currentTime beyond `0.5s`
- pause held at `0.512104s`
- seek target: `33.05225605s`
- observed seek position: `33.052256s`
- browser `/stream/` request used `Range: bytes=0-`

The browser proof also played a deterministic protected MP4 fixture through the Gateway. Browser-visible Gateway requests contained no Authorization header and no Cookie, and the fixture secret did not appear in the browser evidence or server artifact. Final browser metrics showed `active_streams=0`.

Accepted browser evidence artifact for the executable candidate:

- artifact id: `9521162868`
- digest: `sha256:db91c9d766fa439b54e088233d47b87d072ec236e1ef83251dd3d716758419db`

## HTTP/Range and capability behavior

Deterministic tests and public smoke establish:

- public MP4 `Range: bytes=0-1023` returns HTTP 206, exactly 1024 bytes, with matching `Content-Range`;
- the real Chromium request uses Range through the Gateway;
- an upstream that ignores a requested Range is rejected as HTTP 502 `UPSTREAM_RANGE_UNSUPPORTED` rather than silently returning ambiguous seek semantics;
- invalid capability returns 404 `INVALID_MEDIA_CAPABILITY`;
- expired capability returns 410 `EXPIRED_MEDIA_CAPABILITY`;
- cross-session/item/revision/resource capability replay returns 403 `MEDIA_CAPABILITY_BINDING_MISMATCH` before upstream access;
- a caller-chosen `/stream?url=...` path does not exist and does not contact upstream;
- redirect targets are revalidated against egress policy on every hop;
- redirect to link-local/private metadata space is rejected;
- upstream 403/404 remain bounded, explicit outcomes;
- the deterministic interrupted HLS segment produces HTTP 502 `UPSTREAM_REQUEST_FAILED`;
- active streamed-body accounting returns to zero after abort/close.

## HLS result

R001 HLS result is **PASS for the HTTP manifest/variant/segment Gateway path**, with these concrete behaviors:

- a deterministic HLS entry redirect is followed only after redirect-target egress revalidation;
- redirect query material remains server-side and is not exposed in the rewritten browser-facing manifest;
- master playlist passes through Gateway capability;
- ordinary relative URI lines are rewritten;
- `URI="..."` attributes are rewritten;
- relative and query-bearing upstream child URIs resolve against the upstream playlist before receiving a new scoped capability;
- upstream host and upstream query material are not exposed in the rewritten browser-facing child path;
- rewritten variant is retrievable;
- rewritten public segment is retrievable;
- missing segment returns a concrete 404;
- interrupted deterministic segment returns a concrete 502 error code;
- child capabilities inherit the parent binding identity and server-side upstream access context.

R001 does **not** claim Chromium-native HLS playback. The required real-browser playback proof is MP4; HLS was verified at the concrete HTTP manifest/variant/segment level. Browser-HLS support, player-library selection, and TV-specific HLS behavior remain later compatibility work.

## Secret and egress boundary

Public `ResolvedStream.public_headers` rejects `Cookie`, `Authorization`, and `Proxy-Authorization` as secret material.

The protected fixture requires a server-side bearer credential. Direct fixture access without it is unauthorized; the Gateway injects it server-side, while the browser sees only an opaque scoped capability path.

Public-web egress rejects non-public IPv4/IPv6 targets and validates every redirect target. Fixture loopback access is a separate test-only egress scope and cannot be selected by a browser URL parameter.

## Cleanup / boundedness

J4 repeatedly starts a streamed response, consumes one chunk, aborts it, and waits for active stream accounting to return to zero for **100 cycles**.

Observed contract after the loop:

- `active_streams == 0`;
- capability count remains within the configured bound;
- the repeated streamed bytes are not retained as in-memory media objects.

This is the R001 bounded abort/reconnect proof. It is not a substitute for R003 ARM64 CPU/RSS/temperature measurements or later 30/60-minute stability runs.

## Jobs

Required R001 jobs on run `32730313377`:

- J1 deterministic x64 / job `97440836359`: fmt, clippy with warnings denied, workspace unit/contract tests — PASS
- J2 Chromium MP4 play/pause/seek + protected secret boundary / job `97440836345` — PASS
- J3 public MP4 Range + public HLS master/variant/segment smoke / job `97440836280` — PASS
- J4 bounded abort/reconnect cleanup 100x / job `97440836004` — PASS

Accepted public-smoke evidence artifact for the executable candidate:

- artifact id: `9521139065`
- digest: `sha256:1664866deb1cd10b06a54aeacfefeae1b141a05c0af69dd5506f436354f50574`

The workflow uses read-only repository contents permission for accepted verification runs.

## Explicitly deferred / unsupported in R001

R001 does not implement or prove:

- DASH;
- remux;
- transcoding;
- Jellyfin integration;
- TV autoplay/audible playback policy;
- phone resource handling;
- real authenticated site plugins;
- Native Site Panel / Browser Worker;
- R007 Playback command/revision/media-refresh/handoff concurrency semantics.

Those boundaries must not be inferred from the R001 PASS result.