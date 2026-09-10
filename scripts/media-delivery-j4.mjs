import fs from 'node:fs';
import { chromium } from 'playwright-core';

const base = process.env.MEDIA_DELIVERY_HARNESS_URL;
const startPath = process.env.MEDIA_DELIVERY_START_PATH;
const candidate = process.env.CANDIDATE_SHA;
const output = process.env.MEDIA_DELIVERY_PROOF || 'media-delivery-j4-proof.json';
if (!base || !startPath || !candidate) throw new Error('missing hosted media-delivery harness inputs');

const evidence = {
  candidate_sha: candidate,
  browser: 'isolated Playwright Chromium (sandbox enabled)',
  authority_entry: 'GatewayService::start_media_delivery',
  route: 'POST /api/v1/media-delivery/{token}/start',
  stage: 'launch',
  start_status: null,
  media_status: null,
  media_content_type: null,
  media_content_length: null,
  media_bytes: null,
  range_status: null,
  range_content_range: null,
  stale_start_status: null,
  browser_src_is_gateway_capability: false,
  browser_loadeddata: false,
  browser_frame_decoded: false,
  browser_ready_state: null,
  browser_duration_seconds: null,
  browser_current_time_seconds: null,
  browser_video_width: null,
  browser_video_height: null,
  boxes: [],
};

function parseBoxes(bytes) {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const boxes = [];
  function visit(start, end, depth) {
    let offset = start;
    while (offset + 8 <= end) {
      let size = view.getUint32(offset);
      const type = new TextDecoder().decode(bytes.slice(offset + 4, offset + 8));
      let header = 8;
      if (size === 1) {
        if (offset + 16 > end) break;
        const high = view.getUint32(offset + 8);
        const low = view.getUint32(offset + 12);
        if (high !== 0) throw new Error('fixture box exceeds bounded browser buffer');
        size = low;
        header = 16;
      }
      if (size < header || offset + size > end) break;
      boxes.push(type);
      if (depth < 4 && ['moov', 'trak', 'mdia', 'minf', 'dinf', 'stbl'].includes(type)) {
        visit(offset + header, offset + size, depth + 1);
      }
      offset += size;
    }
  }
  visit(0, bytes.length, 0);
  return boxes;
}

function errorCode(error) {
  const message = String(error?.message || error);
  if (message.includes('delivery start failed')) return 'gateway_start_failed';
  if (message.includes('Range')) return 'range_check_failed';
  if (message.includes('Chromium')) return 'browser_decode_failed';
  if (message.includes('fragmented MP4')) return 'container_check_failed';
  return 'j4_smoke_failed';
}

let stage = 'launch';
const browser = await chromium.launch({
  executablePath: process.env.CHROME_PATH || '/usr/bin/google-chrome',
  headless: true,
});
try {
  stage = 'gateway_route';
  const page = await browser.newPage();
  await page.goto(`${base}/display?profile=tv`, { waitUntil: 'domcontentloaded' });
  const result = await page.evaluate(async path => {
    const start = await fetch(path, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: '{}',
    });
    const body = await start.text();
    if (!start.ok) throw new Error(`delivery start failed: ${start.status}: ${body}`);
    const payload = JSON.parse(body);
    const capabilityUrl = new URL(payload.gateway_path, location.origin).href;
    const media = await fetch(capabilityUrl);
    const bytes = new Uint8Array(await media.arrayBuffer());
    const range = await fetch(capabilityUrl, { headers: { Range: 'bytes=0-15' } });
    const video = document.createElement('video');
    video.preload = 'auto';
    video.muted = true;
    video.playsInline = true;
    video.src = capabilityUrl;
    document.body.append(video);
    const loadeddata = await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('Chromium media loadeddata timeout')), 10000);
      video.onloadeddata = () => {
        clearTimeout(timeout);
        resolve(true);
      };
      video.onerror = () => {
        clearTimeout(timeout);
        reject(new Error('Chromium could not decode Gateway fMP4 output'));
      };
      video.load();
    });
    let frameDecoded = video.readyState >= 2;
    if (frameDecoded && typeof video.requestVideoFrameCallback === 'function') {
      frameDecoded = await Promise.race([
        new Promise(resolve => video.requestVideoFrameCallback(() => resolve(true))),
        new Promise(resolve => setTimeout(() => resolve(false), 5000)),
      ]);
    }
    const stale = await fetch(path, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: '{}',
    });
    return {
      start_status: start.status,
      media_status: media.status,
      media_content_type: media.headers.get('content-type'),
      media_content_length: Number(media.headers.get('content-length')),
      media_bytes: Array.from(bytes),
      range_status: range.status,
      range_content_range: range.headers.get('content-range'),
      stale_start_status: stale.status,
      browser_src_is_gateway_capability: new URL(video.src).pathname.startsWith('/media/delivery/'),
      browser_loadeddata: loadeddata,
      browser_frame_decoded: frameDecoded,
      browser_ready_state: video.readyState,
      browser_duration_seconds: Number.isFinite(video.duration) ? video.duration : null,
      browser_current_time_seconds: video.currentTime,
      browser_video_width: video.videoWidth,
      browser_video_height: video.videoHeight,
    };
  }, startPath);

  const mediaBytes = Uint8Array.from(result.media_bytes);
  evidence.stage = 'assertions';
  Object.assign(evidence, result, { media_bytes: mediaBytes.length, boxes: parseBoxes(mediaBytes) });
  if (evidence.start_status !== 200 || evidence.media_status !== 200) throw new Error('Gateway HTTP route/resource was not successful');
  if (evidence.media_content_type !== 'video/mp4') throw new Error(`unexpected content type: ${evidence.media_content_type}`);
  if (evidence.media_content_length !== evidence.media_bytes) throw new Error('content length mismatch');
  if (!evidence.browser_src_is_gateway_capability || !evidence.browser_loadeddata || evidence.browser_ready_state < 2) throw new Error('Chromium did not load Gateway capability');
  for (const required of ['ftyp', 'moov', 'mvex', 'moof', 'mdat']) {
    if (!evidence.boxes.includes(required)) throw new Error(`fragmented MP4 box missing: ${required}`);
  }
  if (evidence.range_status !== 206 || !/^bytes 0-15\//.test(evidence.range_content_range || '')) throw new Error('Gateway byte-range resource check failed');
  if (evidence.stale_start_status !== 404) throw new Error('one-shot start capability was reusable');
  fs.writeFileSync(output, `${JSON.stringify(evidence, null, 2)}\n`);
} catch (error) {
  evidence.stage = stage;
  evidence.error_code = errorCode(error);
  fs.writeFileSync(output, `${JSON.stringify(evidence, null, 2)}\n`);
  throw error;
} finally {
  await browser.close();
}
