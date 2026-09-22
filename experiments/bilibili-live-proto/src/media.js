// Prototype Media Gateway: issues task-bound media paths, proxies upstream
// bytes with Range forwarding, and injects upstream headers server-side so
// no Referer/UA/secret ever reaches the client. Mirrors the real Media
// Gateway seam; deliberately has no token signing (single trusted user).

import { spawn } from 'node:child_process';
import { createWriteStream, existsSync, mkdirSync, readFileSync } from 'node:fs';
import { mkdtemp } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { Readable } from 'node:stream';
import { pipeline } from 'node:stream/promises';

/** Proxy one upstream stream through the gateway with Range support. */
export async function proxyStream(req, res, stream) {
  const headers = { ...stream.upstream_headers };
  const range = req.headers.range;
  if (range) headers.Range = range;

  const upstream = await fetch(stream.url, { headers });
  res.writeHead(upstream.status, {
    'content-type': upstream.headers.get('content-type') || 'video/mp4',
    ...(upstream.headers.get('content-length')
      ? { 'content-length': upstream.headers.get('content-length') }
      : {}),
    ...(upstream.headers.get('content-range')
      ? { 'content-range': upstream.headers.get('content-range') }
      : {}),
    'accept-ranges': 'bytes',
  });
  if (req.method === 'HEAD' || !upstream.body) return res.end();
  try {
    await pipeline(Readable.fromWeb(upstream.body), res);
  } catch (err) {
    // Client aborts are normal for video elements (range probing,
    // pausing). Swallow them; only real upstream failures matter.
    if (!res.destroyed) console.error('[proxy]', err.message);
  } finally {
    upstream.body?.cancel().catch(() => {});
  }
}

/**
 * Remux separate DASH video+audio into HLS with stream copy (no re-encode).
 * Empirically answers the "is remux needed / does it work" question.
 * Returns the local m3u8 path once ffmpeg has written the first segments.
 */
export async function remuxToHls(video, audio, sessionId) {
  const dir = await mkdtemp(join(tmpdir(), `proto-remux-${sessionId}-`));
  const playlist = join(dir, 'index.m3u8');
  const args = [
    '-headers',
    Object.entries(video.upstream_headers)
      .map(([k, v]) => `${k}: ${v}\r\n`)
      .join(''),
    '-i',
    video.url,
    '-headers',
    Object.entries(audio.upstream_headers)
      .map(([k, v]) => `${k}: ${v}\r\n`)
      .join(''),
    '-i',
    audio.url,
    '-c',
    'copy',
    '-f',
    'hls',
    '-hls_time',
    '6',
    '-hls_list_size',
    '0',
    // list_size=0 keeps every segment listed so a client can play from
    // the beginning while the remux is still running. `vod` would defer
    // the playlist until EOF; default size=5 drops early segments.
    playlist,
  ];
  const proc = spawn('ffmpeg', args, { stdio: ['ignore', 'ignore', 'pipe'] });
  let stderrTail = '';
  proc.stderr.on('data', (d) => {
    stderrTail = (stderrTail + d).slice(-4000);
  });
  proc.on('exit', (code) => {
    if (code !== 0) console.error(`[remux] ffmpeg exited ${code}: ${stderrTail}`);
  });
  // Wait until the playlist exists (first segment written).
  for (let i = 0; i < 300 && !existsSync(playlist); i++) {
    await new Promise((r) => setTimeout(r, 200));
  }
  if (!existsSync(playlist)) throw new Error('remux produced no playlist');
  return { dir, playlist };
}

export function serveFile(res, path, contentType) {
  res.writeHead(200, { 'content-type': contentType });
  res.end(readFileSync(path));
}
