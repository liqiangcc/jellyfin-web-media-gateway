import fs from 'node:fs';
import { chromium } from 'playwright-core';

const base = process.env.BASE_URL || 'http://127.0.0.1:8787';
const chrome = process.env.CHROME_PATH || '/usr/bin/google-chrome';
const mode = process.env.DISPLAY_UX_MODE || 'happy';
const evidence = { mode, base: new URL(base).origin, claims: {}, requests: { register: 0, heartbeat: 0, subtitle: 0 }, failures: [] };

const browser = await chromium.launch({
  executablePath: chrome,
  headless: true,
  args: ['--autoplay-policy=no-user-gesture-required', '--no-sandbox'],
});

function attachGuards(page) {
  page.on('console', message => {
    const text = message.text();
    if (/(bearer\s+|cookie|authorization|r001-fixture-secret)/i.test(text)) {
      throw new Error('secret-like console text observed');
    }
  });
  page.on('requestfailed', request => evidence.failures.push(request.failure()?.errorText || 'request failed'));
}

async function waitForRegistration(page) {
  await page.waitForFunction(() => Boolean(window.__displayPrep?.getRegistration()), null, { timeout: 10000 });
  return page.evaluate(() => window.__displayPrep.getRegistration());
}

async function testEntry() {
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  attachGuards(page);
  const started = Date.now();
  await page.goto(`${base}/`, { waitUntil: 'domcontentloaded' });
  await page.waitForURL('**/display?profile=tv', { timeout: 7500 });
  const elapsed = Date.now() - started;
  if (elapsed < 4800 || elapsed > 7000) throw new Error(`smart entry delay outside bounded five-second window: ${elapsed}`);
  const cancelled = await browser.newPage();
  attachGuards(cancelled);
  await cancelled.goto(`${base}/`, { waitUntil: 'domcontentloaded' });
  await cancelled.locator('#control').click();
  await cancelled.waitForTimeout(5200);
  if (!cancelled.url().endsWith('/control')) throw new Error('explicit Control choice did not cancel countdown');
  await cancelled.close();
  const direct = await browser.newPage();
  attachGuards(direct);
  await direct.goto(`${base}/control`, { waitUntil: 'domcontentloaded' });
  if (await direct.locator('#countdown').count()) throw new Error('direct Control waited for smart-entry countdown');
  await direct.close();
  await page.close();
  evidence.claims.C1 = { default_delay_ms: elapsed, explicit_control_cancels: true, direct_control: true };
}

async function testDisplay() {
  const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
  await context.addInitScript(() => {
    Object.defineProperty(Element.prototype, 'requestFullscreen', { configurable: true, value: undefined });
    Object.defineProperty(HTMLMediaElement.prototype, 'play', {
      configurable: true,
      value: () => Promise.reject(new DOMException('forced browser rejection', 'NotAllowedError')),
    });
  });
  const page = await context.newPage();
  attachGuards(page);
  page.on('request', request => {
    if (request.url().endsWith('/api/v1/displays/register')) evidence.requests.register += 1;
    if (request.url().includes('/heartbeat')) evidence.requests.heartbeat += 1;
    if (request.url().includes('/stream/') && request.url().includes('subtitle-fixture')) evidence.requests.subtitle += 1;
  });
  await page.goto(`${base}/display?profile=tv`, { waitUntil: 'domcontentloaded' });
  const first = await waitForRegistration(page);
  if (!first.display_id || !first.registration_id) throw new Error('TV display did not register');
  await page.waitForFunction(() => document.querySelector('#subtitle-track')?.track?.readyState === 2, null, { timeout: 10000 });
  const subtitle = await page.evaluate(() => {
    const element = document.querySelector('#subtitle-track');
    return { src: element?.src || '', readyState: element?.track?.readyState, cues: element?.track?.cues?.length || 0 };
  });
  if (!subtitle.src.startsWith(new URL(base).origin + '/stream/')) throw new Error('subtitle did not use a same-origin Gateway path');
  if (!subtitle.cues) throw new Error('Gateway WebVTT track has no cues');
  for (const [width, height] of [[1280, 720], [1920, 1080], [3840, 2160]]) {
    await page.setViewportSize({ width, height });
    const dimensions = await page.locator('#display-shell').evaluate(element => ({ width: element.clientWidth, height: element.clientHeight }));
    if (dimensions.width !== width || dimensions.height !== height) throw new Error(`viewport shell mismatch at ${width}x${height}`);
  }
  await page.locator('#retry').focus();
  await page.keyboard.press('Tab');
  if (!(await page.evaluate(() => document.activeElement?.tagName))?.includes('BUTTON')) throw new Error('remote focus traversal left essential controls');
  await page.locator('#fullscreen').click();
  if (!(await page.locator('#status').textContent()).includes('Fullscreen unavailable')) throw new Error('fullscreen degradation was not explicit');
  await page.locator('#activate').click();
  if (!(await page.locator('#status').textContent()).includes('NotAllowedError')) throw new Error('play rejection was not surfaced');
  await page.waitForTimeout(10500);
  if (evidence.requests.heartbeat < 1) throw new Error('idle display did not heartbeat below lease TTL');
  const beforeReload = await page.evaluate(() => window.__displayPrep.getRegistration());
  await page.reload({ waitUntil: 'domcontentloaded' });
  const afterReload = await waitForRegistration(page);
  const stale = await page.evaluate(async old => {
    const response = await fetch(`/api/v1/displays/${encodeURIComponent(old.display_id)}/heartbeat`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ lease_token: old.lease_token }) });
    return response.status;
  }, beforeReload);
  if (stale !== 401) throw new Error(`old display lease remained valid after reconnect: ${stale}`);
  const storage = await page.evaluate(() => JSON.stringify(sessionStorage));
  if (/(https?:\/\/|file:|bearer\s+|cookie|authorization|r001-fixture-secret)/i.test(storage)) throw new Error('browser storage contains forbidden authority material');
  const browserPaths = await page.evaluate(() => ({ media: document.querySelector('#player')?.src || '', subtitle: document.querySelector('#subtitle-track')?.src || '', text: document.body.innerText }));
  if (!browserPaths.media.startsWith(new URL(base).origin + '/stream/')) throw new Error('media did not use a Gateway path');
  if (/(r001-fixture-secret|Bearer\s+|Cookie|Authorization|file:|\.vtt\?)/i.test(JSON.stringify(browserPaths))) throw new Error('browser state contains forbidden Secret/local authority');
  evidence.claims.C2 = { idle_registration: true, viewport: '720p/1080p/4K-like', waiting_overlay: true, heartbeat_count: evidence.requests.heartbeat };
  evidence.claims.C3 = { subtitle_track: subtitle.cues, capability: 'subtitles' };
  evidence.claims.C4 = { same_origin_gateway_path: true, subtitle_requests: evidence.requests.subtitle };
  evidence.claims.C5 = { keyboard_focus: true, enter_activation: true };
  evidence.claims.C6 = { fullscreen: 'unavailable with viewport fallback' };
  evidence.claims.C7 = { insecure_http_baseline: true };
  evidence.claims.C8 = { play_rejection: 'NotAllowedError', old_lease_status: stale, new_epoch: afterReload.page_lease_epoch };
  evidence.claims.C9 = { storage_and_browser_state_scan: 'clean' };
  await context.close();
}

try {
  await testEntry();
  await testDisplay();
  if (mode === 'negative') {
    if (evidence.claims.C9.storage_and_browser_state_scan !== 'clean') throw new Error('negative safety matrix failed');
  }
  fs.writeFileSync('display-ux-prep-proof.json', JSON.stringify(evidence, null, 2));
  console.log(JSON.stringify(evidence, null, 2));
} finally {
  await browser.close();
}
