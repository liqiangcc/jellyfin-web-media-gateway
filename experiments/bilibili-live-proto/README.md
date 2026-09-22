# bilibili-live-proto (research spike)

Disposable Node prototype answering the empirical questions that the Rust
contract pipeline defers:

1. What media shape does anonymous public Bilibili actually return?
   (muxed `durl` MP4 vs separated DASH A/V candidates, URL expiry,
   required upstream headers)
2. Does a same-origin gateway proxy with injected Referer/UA suffice, or
   is separated-A/V remux required? (`ffmpeg -c copy` HLS remux path)
3. Does the result actually play on iPhone Safari over the tailnet?

**Boundary**: contract-mirrored module layout (`sites/bilibili.js` ↔
SiteAdapter, `media.js` ↔ Media Gateway, `session.js` ↔
PlaybackSession identity), but deliberately no CAS/Vault/EgressPolicy —
those are owned by the Rust layer and are NOT what this spike validates.
Anonymous public content only; no login, cookies, or vault material.

**Not part of the workspace.** Zero dependencies, no build step:

```bash
PROTO_BIND=100.64.98.39 PROTO_PORT=8899 node src/server.js
```

- `GET /control` — paste a `bilibili.com/video/BV…[?p=N]` URL
- `GET /display` — polls and plays the current session (iPhone)
- `POST /api/play {source}` — recognize → resolve → session → media_url
- `GET /stream/<sid>/<idx>` — upstream bytes proxied with Range +
  server-side Referer/UA (client never sees them)
- `GET /hls/<sid>/index.m3u8` — ffmpeg stream-copy remux of DASH A/V

Evidence gathered here feeds `plugins/bilibili` and #68 contract review.
This spike is throwaway per repo experiment rules.
