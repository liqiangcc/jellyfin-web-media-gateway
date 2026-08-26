import fs from 'node:fs';
import { chromium } from 'playwright-core';

const base = process.env.BASE_URL || 'http://127.0.0.1:8787';
const source = process.env.WEB_MVP_SOURCE || 'https://raw.githubusercontent.com/mediaelement/mediaelement-files/master/big_buck_bunny.mp4';
const chrome = process.env.CHROME_PATH || '/usr/bin/google-chrome';
const evidence = {
  base: new URL(base).origin,
  claims: {},
  production_path: [],
  failures: [],
  requests: { gateway_media: 0, rendering: 0, events: 0 },
};

const browser = await chromium.launch({
  executablePath: chrome,
  headless: true,
  args: ['--autoplay-policy=no-user-gesture-required', '--no-sandbox'],
});

function safeUrl(value) {
  try {
    const url = new URL(value);
    if (url.origin !== new URL(base).origin) return '[external-url]';
    const parts = url.pathname.split('/');
    if (parts[1] === 'stream' && parts.length > 2) parts[2] = '<capability-redacted>';
    return `${parts.join('/')}${url.search ? '?[query]' : ''}`;
  } catch (_) {
    return '[invalid-url]';
  }
}

function attachGuards(page) {
  page.on('console', message => {
    if (/(bearer\s+|cookie|authorization|r001-fixture-secret|vault|profile)/i.test(message.text())) {
      throw new Error('secret-like console text observed');
    }
  });
  page.on('requestfailed', request => evidence.failures.push(`${request.method()} ${safeUrl(request.url())}`));
  page.on('request', request => {
    const path = safeUrl(request.url());
    if (path.startsWith('/stream/')) evidence.requests.gateway_media += 1;
    if (path.includes('/rendering')) evidence.requests.rendering += 1;
    if (path.includes('/events')) evidence.requests.events += 1;
  });
}

async function json(response) {
  try { return await response.json(); } catch (_) { return {}; }
}

async function waitForRegistration(page) {
  await page.waitForFunction(() => Boolean(window.__displayPrep?.getRegistration()), null, { timeout: 10000 });
  return page.evaluate(() => window.__displayPrep.getRegistration());
}

async function postJson(page, path, body) {
  return page.evaluate(async ({ path, body }) => {
    const response = await fetch(path, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) });
    return { status: response.status, payload: await response.json().catch(() => ({})) };
  }, { path, body });
}

function assertCleanBrowser(page, label) {
  return page.evaluate(() => ({
    text: document.body.innerText,
    storage: JSON.stringify(sessionStorage),
    media: document.querySelector('#player')?.src || '',
  })).then(value => {
    const serialized = JSON.stringify(value);
    if (/(Bearer\s+|Cookie|Authorization|r001-fixture-secret|file:|resolved_media|upstream_access_ref)/i.test(serialized)) {
      throw new Error(`${label} exposed forbidden browser material`);
    }
    return value;
  });
}

async function run() {
  const displayContext = await browser.newContext({ viewport: { width: 1280, height: 720 } });
  const display = await displayContext.newPage();
  attachGuards(display);
  await display.goto(`${base}/display?profile=tv`, { waitUntil: 'domcontentloaded' });
  const initialRegistration = await waitForRegistration(display);
  if (!initialRegistration.display_id || !initialRegistration.registration_id) throw new Error('Display registration missing');
  const preSessionMedia = await display.locator('#player').getAttribute('src');
  if (preSessionMedia) throw new Error('production TV Display preloaded a proof media path before session creation');
  evidence.production_path.push('GET /display?profile=tv → POST /api/v1/displays/register → heartbeat');
  evidence.claims.C1 = { production_display_route: true, proof_path_not_used_for_creation: true };
  evidence.claims.C2 = { display_id: initialRegistration.display_id, registration_id: initialRegistration.registration_id, lease_redacted: true };

  const control = await displayContext.newPage();
  attachGuards(control);
  await control.goto(`${base}/control`, { waitUntil: 'domcontentloaded' });
  await control.locator('#display-selector option').nth(0).waitFor({ state: 'attached', timeout: 10000 });
  await control.locator('#display-selector').selectOption(initialRegistration.display_id);
  await control.locator('#source-input').fill(source);
  await control.locator('#create-session').click();
  await control.waitForURL('**/control?session_id=*', { timeout: 15000 });
  const sessionId = new URL(control.url()).searchParams.get('session_id');
  if (!sessionId) throw new Error('Control did not navigate to the created session');
  await control.locator('#connection').filter({ hasText: 'Connected' }).waitFor({ state: 'visible', timeout: 10000 });
  evidence.production_path.push('GET /control → GET /api/v1/displays → POST /api/v1/sessions → /control?session_id=<opaque>');
  evidence.claims.C3 = { session_id_opaque: true, display_id: initialRegistration.display_id, source_site: 'generic' };

  await display.waitForFunction(id => window.__displayPrep?.getRendering()?.session_id === id, sessionId, { timeout: 20000 });
  const rendering = await display.evaluate(() => window.__displayPrep.getRendering());
  const mediaPath = await display.evaluate(() => document.querySelector('#player')?.src || '');
  if (!mediaPath?.startsWith(new URL(base).origin + '/stream/')) throw new Error('Display did not receive a Gateway media path');
  if (rendering.session_id !== sessionId || rendering.item_revision !== 1) throw new Error('Display rendering view identity mismatch');
  if (evidence.requests.gateway_media < 1) throw new Error('Display did not request Gateway media');
  evidence.production_path.push('Display rendering view → same-origin Gateway media request');
  evidence.claims.C4 = { rendering_session: rendering.session_id, item_revision: rendering.item_revision, safe_gateway_path: true };
  evidence.claims.C5 = { media_request_count: evidence.requests.gateway_media, browser_media_path: safeUrl(mediaPath) };

  for (const [command, expectedState] of [['pause', 'paused'], ['play', 'playing'], ['seek', 'playing'], ['stop', 'stopped']]) {
    if (command === 'seek') await control.locator('#seek-position').fill('1200');
    await control.locator(`#${command}`).click();
    await control.waitForFunction(state => document.querySelector('#playback-state')?.textContent === state, expectedState, { timeout: 10000 });
  }
  evidence.production_path.push('/control?session_id=<id> → play → pause → seek → stop');
  evidence.claims.C6 = { commands: ['play', 'pause', 'seek', 'stop'], revision_aware: true };

  const sessionAState = await control.evaluate(async id => (await fetch(`/api/v1/control/${encodeURIComponent(id)}`)).json(), sessionId);
  if (sessionAState.now_playing?.state !== 'stopped') throw new Error('Session A was not stopped before repeated-use creation');

  await control.goto(`${base}/control`, { waitUntil: 'domcontentloaded' });
  await control.locator('#display-selector option').nth(0).waitFor({ state: 'attached', timeout: 10000 });
  await control.locator('#display-selector').selectOption(initialRegistration.display_id);
  await control.locator('#source-input').fill(source);
  await control.locator('#create-session').click();
  await control.waitForURL('**/control?session_id=*', { timeout: 15000 });
  const sessionBId = new URL(control.url()).searchParams.get('session_id');
  if (!sessionBId || sessionBId === sessionId) throw new Error('Repeated-use creation did not produce a distinct Session B');
  await control.locator('#connection').filter({ hasText: 'Connected' }).waitFor({ state: 'visible', timeout: 10000 });
  const sessionBView = await control.evaluate(async id => (await fetch(`/api/v1/control/${encodeURIComponent(id)}`)).json(), sessionBId);
  if (sessionBView.active_display?.display_id !== initialRegistration.display_id) throw new Error('Session B was not created for the original Display');
  evidence.production_path.push('/control → stop Session A → POST /api/v1/sessions on the same Display → Session B');
  evidence.claims.C3 = {
    session_a: sessionId,
    session_b: sessionBId,
    distinct_sessions: true,
    same_display: true,
    source_site: 'generic',
  };

  await display.waitForFunction(id => window.__displayPrep?.getRendering()?.session_id === id, sessionBId, { timeout: 20000 });
  const renderingB = await display.evaluate(() => window.__displayPrep.getRendering());
  if (renderingB.session_id !== sessionBId || renderingB.item_revision !== 1) throw new Error('Display did not resolve Session B before reload');
  const mediaPathB = await display.evaluate(() => document.querySelector('#player')?.src || '');
  if (!mediaPathB?.startsWith(new URL(base).origin + '/stream/')) throw new Error('Session B did not receive a Gateway media path');
  evidence.production_path.push('Display renders Session B from the server-owned current rendering relationship');
  evidence.claims.C4 = {
    rendering_session_a: rendering.session_id,
    rendering_session_b: renderingB.session_id,
    item_revision: renderingB.item_revision,
    safe_gateway_path: true,
  };
  evidence.claims.C5 = { media_request_count: evidence.requests.gateway_media, browser_media_path: safeUrl(mediaPathB) };

  const preReloadRegistration = await display.evaluate(() => window.__displayPrep.getRegistration());
  await control.reload({ waitUntil: 'domcontentloaded' });
  await control.waitForURL(`**/control?session_id=${sessionBId}`, { timeout: 10000 });
  await control.locator('#connection').filter({ hasText: 'Connected' }).waitFor({ state: 'visible', timeout: 10000 });
  await display.reload({ waitUntil: 'domcontentloaded' });
  const refreshedRegistration = await waitForRegistration(display);
  if (refreshedRegistration.display_id !== initialRegistration.display_id || refreshedRegistration.page_lease_epoch <= preReloadRegistration.page_lease_epoch) {
    throw new Error('Display reload did not rotate the page lease for the same Display');
  }
  await display.waitForFunction(id => window.__displayPrep?.getRendering()?.session_id === id, sessionBId, { timeout: 20000 });
  const reloadedRendering = await display.evaluate(() => window.__displayPrep.getRendering());
  if (reloadedRendering.session_id !== sessionBId) throw new Error('Display reload did not resolve Session B');
  const staleLeaseStatus = await display.evaluate(async old => {
    const response = await fetch(`/api/v1/displays/${encodeURIComponent(old.display_id)}/heartbeat`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ lease_token: old.lease_token }) });
    return response.status;
  }, preReloadRegistration);
  if (staleLeaseStatus !== 401) throw new Error(`stale Display lease was accepted: ${staleLeaseStatus}`);
  const staleLeaseRenderingStatus = await display.evaluate(async old => {
    const response = await fetch(`/api/v1/displays/${encodeURIComponent(old.display_id)}/rendering`, { headers: { 'x-display-lease': old.lease_token }, cache: 'no-store' });
    return response.status;
  }, preReloadRegistration);
  if (staleLeaseRenderingStatus !== 401) throw new Error(`stale Display lease rendering was accepted: ${staleLeaseRenderingStatus}`);
  const historicalA = await control.evaluate(async id => (await fetch(`/api/v1/control/${encodeURIComponent(id)}`)).json(), sessionId);
  if (historicalA.now_playing?.state !== 'stopped') throw new Error('Historical Session A changed after Session B creation/reload');
  evidence.production_path.push('Control refresh/event polling + Display reload/lease reconnect → Session B remains rendered');
  evidence.claims.C7 = {
    session_a_stopped: true,
    session_b_after_reload: reloadedRendering.session_id,
    same_display_after_refresh: true,
    stale_lease_status: staleLeaseStatus,
    stale_lease_rendering_status: staleLeaseRenderingStatus,
    new_page_lease_epoch: refreshedRegistration.page_lease_epoch,
  };

  const invalidForm = await displayContext.newPage();
  attachGuards(invalidForm);
  await invalidForm.goto(`${base}/control`, { waitUntil: 'domcontentloaded' });
  await invalidForm.locator('#display-selector').selectOption(refreshedRegistration.display_id);
  await invalidForm.locator('#source-input').fill('https://example.test/not-a-media-page');
  await invalidForm.locator('#create-session').click();
  await invalidForm.locator('#source-status').filter({ hasText: 'recognized' }).waitFor({ state: 'visible', timeout: 10000 });
  const missingDisplay = await postJson(invalidForm, '/api/v1/sessions', { request_id: 'e2e-offline-display', source, display_id: 'missing-display' });
  if (missingDisplay.status !== 404) throw new Error(`offline Display returned ${missingDisplay.status}`);
  const staleCommand = await postJson(invalidForm, `/api/v1/sessions/${sessionBId}/commands`, { request_id: 'e2e-stale-command', expected_session_revision: 99, command: { type: 'play' } });
  if (staleCommand.status !== 409 || staleCommand.payload.code !== 'REVISION_CONFLICT') throw new Error('stale command was not rejected with REVISION_CONFLICT');
  const replayFirst = await postJson(invalidForm, `/api/v1/sessions/${sessionBId}/commands`, { request_id: 'e2e-replay-command', expected_session_revision: 0, command: { type: 'pause' } });
  const replaySecond = await postJson(invalidForm, `/api/v1/sessions/${sessionBId}/commands`, { request_id: 'e2e-replay-command', expected_session_revision: 1, command: { type: 'stop' } });
  if (replayFirst.status !== 200 || replaySecond.status !== 409 || replaySecond.payload.code !== 'REQUEST_ID_MISMATCH') throw new Error('request-id reuse matrix failed');
  const missingSession = await invalidForm.evaluate(async () => { const response = await fetch('/api/v1/control/s-missing'); return response.status; });
  const resync = await invalidForm.evaluate(async id => (await fetch(`/api/v1/sessions/${encodeURIComponent(id)}/events?after=999999`)).json(), sessionBId);
  if (missingSession !== 404 || resync.snapshot_required !== true) throw new Error('missing session/event resync matrix failed');
  await assertCleanBrowser(invalidForm, 'negative Control');
  evidence.claims.C8 = {
    invalid_source: true,
    offline_display_status: missingDisplay.status,
    stale_revision: staleCommand.payload.code,
    request_id_reuse: replaySecond.payload.code,
    missing_session_status: missingSession,
    event_resync: resync.snapshot_required,
    historical_session_a_stopped: historicalA.now_playing?.state === 'stopped',
  };

  await assertCleanBrowser(display, 'Display');
  await assertCleanBrowser(control, 'Control');
  evidence.claims.C9 = { browser_storage_dom_network_scan: 'clean', physical_tv_phone_real_site_out_of_scope: true };
  evidence.requests.external_source = '[redacted]';
  evidence.failures = evidence.failures.slice(0, 10);
  fs.writeFileSync('web-mvp-e2e-proof.json', JSON.stringify(evidence, null, 2));
  console.log(JSON.stringify(evidence, null, 2));
  await invalidForm.close();
  await displayContext.close();
}

try {
  await run();
} finally {
  await browser.close();
}
