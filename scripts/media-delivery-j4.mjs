import fs from 'node:fs';
import { execFileSync } from 'node:child_process';
import { chromium } from 'playwright-core';

const base = process.env.MEDIA_DELIVERY_HARNESS_URL;
const startPath = process.env.MEDIA_DELIVERY_START_PATH;
const expiryStartPath = process.env.MEDIA_DELIVERY_EXPIRY_START_PATH;
const candidate = process.env.CANDIDATE_SHA;
const output = process.env.MEDIA_DELIVERY_PROOF || 'media-delivery-j4-proof.json';
if (!base || !startPath || !expiryStartPath || !candidate) throw new Error('missing hosted media-delivery harness inputs');

const evidence = {
  candidate_sha: candidate,
  browser: 'isolated Playwright Chromium (sandbox enabled)',
  browser_sandbox_status: 'unverified',
  browser_sandbox_fields: {},
  browser_no_disable_switches: false,
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
  stale_output_status: null,
  stale_cleanup_output_count: null,
  expiry_start_status: null,
  expired_output_status: null,
  expiry_cleanup_output_count: null,
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

function parseSandboxFields(text) {
  const fields = {};
  for (const line of text.split(/\r?\n/).slice(0, 128)) {
    const match = line.match(/^\s*([A-Za-z][A-Za-z0-9 _-]{2,80}?):?\s+(Yes|No|Enabled|Disabled|Active|Inactive)\s*$/i);
    if (!match || !/(sandbox|namespace|seccomp|privilege)/i.test(match[1])) continue;
    fields[match[1].trim()] = match[2].toLowerCase();
  }
  return fields;
}

let stage = 'launch';
let browser;
try {
  browser = await chromium.launch({
    executablePath: process.env.CHROME_PATH || '/usr/bin/google-chrome',
    headless: true,
    chromiumSandbox: true,
  });
  const processArgs = execFileSync('ps', ['-eo', 'args='], { encoding: 'utf8' });
  const chromeArgs = processArgs
    .split('\n')
    .filter(line => /(^|\s|\/)(google-chrome|chrome|chromium)(\s|$)/i.test(line))
    .join('\n');
  evidence.browser_no_disable_switches = !/--no-sandbox|--disable-setuid-sandbox/i.test(chromeArgs);
  if (!evidence.browser_no_disable_switches) throw new Error('Chromium sandbox disabling switch observed');
  const sandboxPage = await browser.newPage();
  let sandboxText = '';
  try {
    await sandboxPage.goto('chrome://sandbox', { waitUntil: 'domcontentloaded' });
    sandboxText = await sandboxPage.locator('body').innerText();
  } catch {
    sandboxText = '';
  }
  await sandboxPage.close();
  evidence.browser_sandbox_fields = parseSandboxFields(sandboxText);
  const sandboxValues = Object.values(evidence.browser_sandbox_fields);
  evidence.browser_sandbox_status = sandboxValues.length > 0 && sandboxValues.every(value => ['yes', 'enabled', 'active'].includes(value)) ? 'enabled' : 'unverified';
  if (evidence.browser_sandbox_status !== 'enabled') throw new Error('Chromium sandbox status unavailable');
  stage = 'gateway_route';
  const page = await browser.newPage();
  await page.goto(`${base}/display?profile=tv`, { waitUntil: 'domcontentloaded' });
  const result = await page.evaluate(async ({ path, expiryStartPath }) => {
    const start = await fetch(path, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: '{}',
    });
    const body = await start.text();
    if (!start.ok) throw new Error(`delivery start failed: ${start.status}: ${body}`);
    const payload = JSON.parse(body);
    const sessionId = payload.binding?.session_id;
    if (typeof sessionId !== 'string' || sessionId.length === 0) throw new Error('delivery binding missing session');
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
    if (stale.status !== 404) throw new Error(`one-shot start capability was reusable: ${stale.status}`);
    const invalidate = await fetch(`/__harness/invalidate/${encodeURIComponent(sessionId)}`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: '{}',
    });
    if (!invalidate.ok) throw new Error(`harness authority invalidation failed: ${invalidate.status}`);
    const staleOutput = await fetch(capabilityUrl);
    const staleCount = await fetch('/__harness/output-count');
    if (!staleCount.ok) throw new Error(`stale output count failed: ${staleCount.status}`);
    const expiryStart = await fetch(expiryStartPath, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: '{}',
    });
    if (!expiryStart.ok) throw new Error(`expiry delivery start failed: ${expiryStart.status}`);
    const expiryPayload = JSON.parse(await expiryStart.text());
    if (typeof expiryPayload.gateway_path !== 'string' || expiryPayload.gateway_path.length === 0) {
      throw new Error('expiry delivery response missing gateway path');
    }
    const expiryUrl = new URL(expiryPayload.gateway_path, location.origin).href;
    await new Promise(resolve => setTimeout(resolve, 3000));
    const expiredOutput = await fetch(expiryUrl);
    const expiryCount = await fetch('/__harness/output-count');
    if (!expiryCount.ok) throw new Error(`expiry output count failed: ${expiryCount.status}`);
    return {
      start_status: start.status,
      media_status: media.status,
      media_content_type: media.headers.get('content-type'),
      media_content_length: Number(media.headers.get('content-length')),
      media_bytes: Array.from(bytes),
      range_status: range.status,
      range_content_range: range.headers.get('content-range'),
      stale_start_status: stale.status,
      stale_output_status: staleOutput.status,
      stale_cleanup_output_count: await staleCount.json(),
      expiry_start_status: expiryStart.status,
      expired_output_status: expiredOutput.status,
      expiry_cleanup_output_count: await expiryCount.json(),
      browser_src_is_gateway_capability: new URL(video.src).pathname.startsWith('/media/delivery/'),
      browser_loadeddata: loadeddata,
      browser_frame_decoded: frameDecoded,
      browser_ready_state: video.readyState,
      browser_duration_seconds: Number.isFinite(video.duration) ? video.duration : null,
      browser_current_time_seconds: video.currentTime,
      browser_video_width: video.videoWidth,
      browser_video_height: video.videoHeight,
    };
  }, { path: startPath, expiryStartPath });

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
  if (evidence.stale_output_status !== 410 || evidence.stale_cleanup_output_count !== 0) throw new Error('stale Gateway output was not revoked and cleaned');
  if (evidence.expiry_start_status !== 200 || ![404, 410].includes(evidence.expired_output_status) || evidence.expiry_cleanup_output_count !== 0) throw new Error('expired Gateway output was not rejected and cleaned');
  fs.writeFileSync(output, `${JSON.stringify(evidence, null, 2)}\n`);
} catch (error) {
  evidence.stage = stage;
  evidence.error_code = errorCode(error);
  fs.writeFileSync(output, `${JSON.stringify(evidence, null, 2)}\n`);
  throw error;
} finally {
  if (browser) await browser.close();
}
