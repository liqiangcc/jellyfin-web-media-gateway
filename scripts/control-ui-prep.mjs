import fs from 'node:fs';
import { chromium } from 'playwright-core';

const base = process.env.BASE_URL || 'http://127.0.0.1:8787';
const sessionId = process.env.SESSION_ID;
const chrome = process.env.CHROME_PATH || '/usr/bin/google-chrome';
if (!sessionId) throw new Error('SESSION_ID is required');

const evidence = {
  route: '/control?session_id=[bounded]',
  sessionSelector: 'provided-by-gated-harness',
  pages: [],
  commands: [],
  reconnect: { disconnectObserved: false, recoveryObserved: false },
  browser: { consoleMessages: 0, failedRequests: 0, storageEntries: 0 },
  negatives: {},
};
const forbidden = /Bearer\s+|Cookie|resolved_media|control-ui-harness-media|password|qr/i;
const browser = await chromium.launch({
  executablePath: chrome,
  headless: true,
  args: ['--no-sandbox'],
});
const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
const requestPaths = [];
const consoleMessages = [];
page.on('request', request => {
  const url = new URL(request.url());
  requestPaths.push(`${request.method()} ${url.pathname}${url.search ? '?[query]' : ''}`);
});
page.on('console', message => consoleMessages.push({ type: message.type(), text: message.text() }));
page.on('requestfailed', request => {
  evidence.browser.failedRequests += 1;
  if (!request.url().includes('/api/v1/sessions/')) {
    consoleMessages.push({ type: 'requestfailed', text: request.url().replace(/^https?:\/\/[^/]+/, '[origin]') });
  }
});

async function waitFor(selector, predicate = value => Boolean(value), description = selector) {
  const deadline = Date.now() + 20000;
  let value = '';
  while (Date.now() < deadline) {
    value = (await page.locator(selector).textContent()) || '';
    if (predicate(value)) return value;
    await page.waitForTimeout(100);
  }
  throw new Error(`timed out waiting for ${description}: ${value}`);
}

async function jsonResponse(response) {
  const body = await response.json();
  if (!response.ok()) throw new Error(`HTTP ${response.status()}: ${body.code || 'request failed'}`);
  return body;
}

async function view() {
  return jsonResponse(await page.request.get(`${base}/api/v1/control/${encodeURIComponent(sessionId)}`));
}

async function command(requestId, expectedRevision, value) {
  const response = await page.request.post(`${base}/api/v1/sessions/${encodeURIComponent(sessionId)}/commands`, {
    headers: { origin: base, 'content-type': 'application/json' },
    data: { request_id: requestId, expected_session_revision: expectedRevision, command: value },
  });
  const body = await response.json();
  evidence.commands.push({ requestId: requestId.startsWith('control-') ? '[generated]' : requestId, status: response.status(), code: body.code || body.status });
  return { response, body };
}

async function clickAndWait(id, feedback) {
  await page.locator(id).click();
  await waitFor('#feedback', value => value.includes(feedback)
    || (feedback === 'Command accepted' && value.includes('Gateway event received')),
  `${id} feedback`);
}

try {
  await page.goto(`${base}/control?session_id=${encodeURIComponent(sessionId)}`, { waitUntil: 'domcontentloaded', timeout: 30000 });
  await waitFor('#connection', value => value.includes('Connected'), 'initial connection');
  await waitFor('#item', value => value.includes('control-ui-item'), 'server-produced Now Playing view');
  await waitFor('#display-status', value => value === 'Online', 'accepted Web Display status');
  evidence.pages.push({ route: '/control', item: await page.locator('#item').textContent(), display: await page.locator('#display-status').textContent() });

  // The real session may already be playing. Use the server-projected
  // control availability rather than assuming a fixture playback state, then
  // exercise both accepted Play and Pause paths.
  if (!(await page.locator('#play').isEnabled())) await clickAndWait('#pause', 'Command accepted');
  await clickAndWait('#play', 'Command accepted');
  await clickAndWait('#pause', 'Command accepted');
  await page.locator('#seek-position').fill('4200');
  await clickAndWait('#seek', 'Command accepted');
  await clickAndWait('#stop', 'Command accepted');

  const initialStorage = await page.evaluate(() => ({
    local: Object.keys(localStorage),
    session: Object.keys(sessionStorage),
    text: document.body.innerText,
  }));
  evidence.browser.consoleMessages = consoleMessages.length;
  evidence.browser.storageEntries = initialStorage.local.length + initialStorage.session.length;
  if (forbidden.test(initialStorage.text) || forbidden.test(JSON.stringify(initialStorage))) {
    throw new Error('forbidden or secret-like browser state was observed');
  }

  // Establish a local playing view so the stale command below remains a
  // server-projectedly valid Pause even after the competing command advances
  // the authoritative revision.
  await clickAndWait('#play', 'Command accepted');

  // Hold the UI at an old view while a competing command advances the
  // server revision. The event response is bounded to the old cursor so the
  // UI must discover the conflict through the existing command API.
  const oldView = await page.evaluate(() => window.__controlUi.getView());
  await page.route(`**/api/v1/sessions/${sessionId}/events*`, async route => {
    await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ session_id: sessionId, cursor: oldView.freshness.event_cursor, events: [], snapshot_required: false, reason: null }) });
  });
  const competing = await command('browser-competing-command', oldView.freshness.playback.session_revision, { type: 'pause' });
  if (!competing.response.ok()) throw new Error(`competing command failed: ${competing.body.code}`);
  await clickAndWait('#pause', 'stale');
  evidence.negatives.revisionConflict = true;
  await page.unroute(`**/api/v1/sessions/${sessionId}/events*`);

  // Rebuild a playing view before exercising request identity reuse so the
  // UI-generated Pause command is enabled by the fresh server projection.
  await clickAndWait('#play', 'Command accepted');

  // Force the UI-generated request id to be reused, then make the transport
  // return the real R007 REQUEST_ID_MISMATCH response from the same endpoint.
  const fixedNow = 1700000000000;
  const fixedRandom = 0.123456;
  await page.route(`**/api/v1/sessions/${sessionId}/commands`, async route => {
    const first = await route.fetch();
    const firstBody = await first.json();
    const request = route.request().postDataJSON();
    const mismatch = await fetch(`${base}/api/v1/sessions/${sessionId}/commands`, {
      method: 'POST',
      headers: { origin: base, 'content-type': 'application/json' },
      body: JSON.stringify({ request_id: request.request_id, expected_session_revision: request.expected_session_revision, command: { type: request.command.type === 'play' ? 'pause' : 'play' } }),
    });
    const mismatchBody = await mismatch.text();
    await route.fulfill({ status: mismatch.status, headers: { 'content-type': 'application/json' }, body: mismatchBody || JSON.stringify(firstBody) });
  });
  await page.evaluate(([now, random]) => { Date.now = () => now; Math.random = () => random; }, [fixedNow, fixedRandom]);
  await clickAndWait('#pause', 'identity was already used');
  await page.evaluate(() => { delete Date.now; delete Math.random; });
  evidence.negatives.requestIdMismatch = true;
  await page.unroute(`**/api/v1/sessions/${sessionId}/commands`);

  // Abort one event poll and let the next poll use the production endpoint.
  let abortNextEvent = true;
  await page.route(`**/api/v1/sessions/${sessionId}/events*`, async route => {
    if (abortNextEvent) {
      abortNextEvent = false;
      evidence.reconnect.disconnectObserved = true;
      await route.abort('failed');
    } else {
      await route.continue();
    }
  });
  await page.evaluate(() => window.__controlUi.pollEvents());
  await page.waitForTimeout(1200);
  await page.evaluate(() => window.__controlUi.pollEvents());
  await waitFor('#connection', value => value.includes('Connected'), 'event reconnect recovery');
  evidence.reconnect.recoveryObserved = true;
  await page.unroute(`**/api/v1/sessions/${sessionId}/events*`);

  await page.goto(`${base}/control?session_id=s-missing`, { waitUntil: 'domcontentloaded', timeout: 30000 });
  await waitFor('#connection', value => value.includes('no longer exists'), 'missing session recovery');
  if (!(await page.locator('#item').textContent()).includes('Unavailable')) throw new Error('missing session did not discard stale view');
  evidence.negatives.missingSession = true;
  await page.goto(`${base}/control?session_id=${encodeURIComponent(sessionId)}`, { waitUntil: 'domcontentloaded', timeout: 30000 });
  await waitFor('#connection', value => value.includes('Connected'), 'fresh session rebuild');

  const finalState = await page.evaluate(() => ({
    text: document.body.innerText,
    local: Object.keys(localStorage),
    session: Object.keys(sessionStorage),
  }));
  const allEvidence = JSON.stringify({ evidence, requestPaths, consoleMessages, finalState });
  if (forbidden.test(allEvidence)) throw new Error('forbidden or secret-like material appeared in browser evidence');
  evidence.requestPaths = requestPaths.filter(path => path.includes('/api/v1/control/') || path.includes('/commands') || path.includes('/events'));
  evidence.browser.consoleMessages = consoleMessages.length;
  fs.writeFileSync('control-ui-proof.json', JSON.stringify(evidence, null, 2));
  console.log(JSON.stringify(evidence, null, 2));
} finally {
  await browser.close();
}
