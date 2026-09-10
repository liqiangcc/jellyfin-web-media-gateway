import fs from 'node:fs';
import { chromium } from 'playwright-core';

const base = process.env.MEDIA_DELIVERY_HARNESS_URL;
const startPath = process.env.MEDIA_DELIVERY_START_PATH;
const candidate = process.env.CANDIDATE_SHA;
const output = process.env.MEDIA_DELIVERY_PROOF || 'media-delivery-j4-proof.json';
if (!base || !startPath || !candidate) throw new Error('missing hosted media-delivery harness inputs');

const evidence = {
  candidate_sha: candidate,
  browser: 'isolated Playwright Chromium',
  authority_entry: 'GatewayService::start_media_delivery',
  route: 'POST /api/v1/media-delivery/{token}/start',
  start_status: null,
  media_status: null,
  media_content_type: null,
  media_content_length: null,
  media_bytes: null,
  range_status: null,
  range_content_range: null,
  browser_readable: false,
  browser_duration_seconds: null,
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
    if (depth < 3 && ['moov', 'trak', 'mdia', 'minf'].includes(type)) {
      visit(offset + header, offset + size, depth + 1);
    }
    offset += size;
    }
  }
  visit(0, bytes.length, 0);
  return boxes;
}

const browser = await chromium.launch({
  executablePath: process.env.CHROME_PATH || '/usr/bin/google-chrome',
  headless: true,
  args: ['--no-sandbox'],
});
try {
  const page = await browser.newPage();
  await page.goto(`${base}/display?profile=tv`, { waitUntil: 'domcontentloaded' });
  const result = await page.evaluate(async path => {
    const start = await fetch(path, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: '{}',
    });
    const payload = await start.json();
    if (!start.ok) throw new Error(`delivery start failed: ${start.status}`);
    const media = await fetch(payload.gateway_path);
    const bytes = new Uint8Array(await media.arrayBuffer());
    const range = await fetch(payload.gateway_path, { headers: { Range: 'bytes=0-15' } });
    const browserReadable = await new Promise((resolve, reject) => {
      const video = document.createElement('video');
      const url = URL.createObjectURL(new Blob([bytes], { type: 'video/mp4' }));
      const timeout = setTimeout(() => reject(new Error('Chromium media metadata timeout')), 10000);
      video.preload = 'metadata';
      video.onloadedmetadata = () => {
        clearTimeout(timeout);
        URL.revokeObjectURL(url);
        resolve({ readable: Number.isFinite(video.duration) && video.duration > 0, duration: video.duration });
      };
      video.onerror = () => {
        clearTimeout(timeout);
        URL.revokeObjectURL(url);
        reject(new Error('Chromium could not parse Gateway fMP4 output'));
      };
      video.src = url;
    });
    return {
      start_status: start.status,
      media_status: media.status,
      media_content_type: media.headers.get('content-type'),
      media_content_length: Number(media.headers.get('content-length')),
      media_bytes: bytes,
      range_status: range.status,
      range_content_range: range.headers.get('content-range'),
      browser_readable: browserReadable.readable,
      browser_duration_seconds: browserReadable.duration,
    };
  }, startPath);

  evidence.start_status = result.start_status;
  evidence.media_status = result.media_status;
  evidence.media_content_type = result.media_content_type;
  evidence.media_content_length = result.media_content_length;
  evidence.media_bytes = result.media_bytes.length;
  evidence.range_status = result.range_status;
  evidence.range_content_range = result.range_content_range;
  evidence.browser_readable = result.browser_readable;
  evidence.browser_duration_seconds = result.browser_duration_seconds;
  evidence.boxes = parseBoxes(result.media_bytes);

  if (evidence.start_status !== 200 || evidence.media_status !== 200) throw new Error('Gateway HTTP route/resource was not successful');
  if (evidence.media_content_type !== 'video/mp4') throw new Error(`unexpected content type: ${evidence.media_content_type}`);
  if (evidence.media_content_length !== evidence.media_bytes) throw new Error('content length mismatch');
  if (!evidence.browser_readable) throw new Error('Chromium could not read the Gateway fMP4 output');
  for (const required of ['ftyp', 'moov', 'mvex', 'moof', 'mdat']) {
    if (!evidence.boxes.includes(required)) throw new Error(`fragmented MP4 box missing: ${required}`);
  }
  if (evidence.range_status !== 206 || !/^bytes 0-15\//.test(evidence.range_content_range || '')) {
    throw new Error('Gateway byte-range resource check failed');
  }
  fs.writeFileSync(output, `${JSON.stringify(evidence, null, 2)}\n`);
} finally {
  await browser.close();
}
