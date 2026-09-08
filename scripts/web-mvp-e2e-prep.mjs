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
  requests: { gateway_media: 0, gateway_media_responses: [], rendering: 0, events: 0 },
};

const browser = await chromium.launch({
  executablePath: chrome,
  headless: true,
  // Keep Chromium's ordinary autoplay policy. The production Display's
  // activation button below must provide the user-equivalent gesture.
  args: ['--no-sandbox'],
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
  page.on('requestfailed', request => {
    const failure = request.failure()?.errorText || '';
    // Replacing Session A's video source with Session B intentionally aborts
    // the old media request in Chromium. Keep real failed requests visible,
    // but do not report this expected browser-side cancellation as a product
    // failure.
    if (safeUrl(request.url()).startsWith('/stream/') && /(?:ERR_ABORTED|ABORTED|NS_BINDING_ABORTED)/i.test(failure)) return;
    evidence.failures.push(`${request.method()} ${safeUrl(request.url())}`);
  });
  page.on('request', request => {
    const path = safeUrl(request.url());
    if (path.startsWith('/stream/')) evidence.requests.gateway_media += 1;
    if (path.includes('/rendering')) evidence.requests.rendering += 1;
    if (path.includes('/events')) evidence.requests.events += 1;
  });
  page.on('response', response => {
    const path = safeUrl(response.url());
    if (!path.startsWith('/stream/')) return;
    evidence.requests.gateway_media_responses.push({ method: response.request().method(), status: response.status(), path });
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

async function mediaState(page) {
  return page.evaluate(() => {
    const player = document.querySelector('#player');
    const error = player?.error;
    return {
      readyState: player?.readyState ?? 0,
      duration: player?.duration ?? Number.NaN,
      currentTime: player?.currentTime ?? Number.NaN,
      paused: player?.paused ?? true,
      ended: player?.ended ?? false,
      error: error ? { code: error.code, message: error.message || '' } : null,
    };
  });
}

async function assertUsableMedia(page, label) {
  await page.waitForFunction(() => {
    const player = document.querySelector('#player');
    return Boolean(player && player.readyState >= HTMLMediaElement.HAVE_METADATA && Number.isFinite(player.duration) && player.duration > 0 && !player.error);
  }, null, { timeout: 20000 });
  const state = await mediaState(page);
  const responses = evidence.requests.gateway_media_responses;
  const unsuccessful = responses.filter(response => response.status < 200 || response.status >= 300);
  if (unsuccessful.length) throw new Error(`${label} Gateway media response was not successful: ${unsuccessful.map(response => response.status).join(',')}`);
  if (!responses.length) throw new Error(`${label} had no Gateway media response`);
  // HAVE_METADATA is the minimum usable readiness for a bounded VOD source;
  // the activation/progression check below proves that Chromium can actually
  // start and advance playback rather than merely exposing metadata.
  if (state.readyState < 1 || !Number.isFinite(state.duration) || state.duration <= 0 || state.error) {
    throw new Error(`${label} media is not usable: readyState=${state.readyState} duration=${state.duration} error=${state.error?.code || 'none'}`);
  }
  return state;
}

function assertNoUnexpectedMediaFailures(label) {
  const unsuccessful = evidence.requests.gateway_media_responses.filter(response => response.status < 200 || response.status >= 300);
  const failedRequests = evidence.failures.filter(failure => failure.includes('/stream/'));
  if (unsuccessful.length || failedRequests.length) {
    throw new Error(`${label} Gateway media failures: responses=${unsuccessful.map(response => response.status).join(',') || 'none'} requests=${failedRequests.length}`);
  }
}

async function activateAndAssertProgression(page) {
  const before = await mediaState(page);
  await page.locator('#activate').click();
  await page.waitForFunction(() => {
    const player = document.querySelector('#player');
    return Boolean(player && !player.paused && !player.error);
  }, null, { timeout: 10000 });
  await page.waitForFunction(start => {
    const player = document.querySelector('#player');
    return Boolean(player && !player.paused && !player.error && player.currentTime > start + 0.25);
  }, before.currentTime, { timeout: 20000 });
  const after = await mediaState(page);
  if (after.paused || after.error || !(after.currentTime > before.currentTime + 0.25)) {
    throw new Error(`Display activation did not advance media: before=${before.currentTime} after=${after.currentTime} paused=${after.paused} error=${after.error?.code || 'none'}`);
  }
  return { before, after };
}

async function waitForDisplayState(page, state, target = null) {
  await page.waitForFunction(({ state: expected, target: seekTarget }) => {
    const player = document.querySelector('#player');
    if (!player || player.error) return false;
    if (expected === 'paused') return player.paused;
    if (expected === 'playing') return !player.paused && player.currentTime > (seekTarget ?? 0) + 0.25;
    if (expected === 'seeked') return !player.paused && Math.abs(player.currentTime - seekTarget) < 0.5;
    if (expected === 'stopped') return player.paused && player.currentTime <= 0.1;
    return false;
  }, { state, target }, { timeout: 15000 });
  return mediaState(page);
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
  const readyMedia = await assertUsableMedia(display, 'Session A');
  const activation = await activateAndAssertProgression(display);
  evidence.production_path.push('Display rendering view → same-origin Gateway media request');
  evidence.production_path.push('Display #activate user gesture → media readiness → currentTime progression');
  evidence.claims.C4 = { rendering_session: rendering.session_id, item_revision: rendering.item_revision, safe_gateway_path: true };
  evidence.claims.C5 = {
    media_request_count: evidence.requests.gateway_media,
    media_response_statuses: evidence.requests.gateway_media_responses.map(response => response.status),
    browser_media_path: safeUrl(mediaPath),
    ready_state: readyMedia.readyState,
    duration_seconds: readyMedia.duration,
    activation: { user_gesture: true, before_current_time: activation.before.currentTime, after_current_time: activation.after.currentTime, progressed: true },
  };

  const commandMedia = {};
  const pausedBefore = await mediaState(display);
  await control.locator('#pause').click();
  await control.waitForFunction(() => document.querySelector('#playback-state')?.textContent === 'paused', null, { timeout: 10000 });
  const paused = await waitForDisplayState(display, 'paused');
  await display.waitForTimeout(800);
  const pausedAfter = await mediaState(display);
  if (Math.abs(pausedAfter.currentTime - paused.currentTime) > 0.1) throw new Error('Display media advanced after pause command');
  commandMedia.pause = { before: pausedBefore, after: pausedAfter, server_state: 'paused', progression_stopped: true };

  const playedBefore = await mediaState(display);
  await control.locator('#play').click();
  await control.waitForFunction(() => document.querySelector('#playback-state')?.textContent === 'playing', null, { timeout: 10000 });
  const played = await waitForDisplayState(display, 'playing', playedBefore.currentTime);
  if (played.paused || !(played.currentTime > playedBefore.currentTime + 0.25)) throw new Error('Display media did not resume after play command');
  commandMedia.play = { before: playedBefore, after: played, server_state: 'playing', progression_resumed: true };

  const seekTarget = 1.2;
  await control.locator('#seek-position').fill('1200');
  await control.locator('#seek').click();
  await control.waitForFunction(() => document.querySelector('#playback-state')?.textContent === 'playing', null, { timeout: 10000 });
  const seeked = await waitForDisplayState(display, 'seeked', seekTarget);
  if (seeked.paused || Math.abs(seeked.currentTime - seekTarget) >= 0.5) throw new Error(`Display media did not seek near ${seekTarget}s: ${seeked.currentTime}`);
  commandMedia.seek = { target_seconds: seekTarget, after: seeked, server_state: 'playing', target_applied: true };

  await control.locator('#stop').click();
  await control.waitForFunction(() => document.querySelector('#playback-state')?.textContent === 'stopped', null, { timeout: 10000 });
  const stopped = await waitForDisplayState(display, 'stopped');
  if (!stopped.paused || stopped.currentTime > 0.1) throw new Error(`Display media did not stop/reset: paused=${stopped.paused} currentTime=${stopped.currentTime}`);
  commandMedia.stop = { after: stopped, server_state: 'stopped', reset_to_start: true };
  evidence.production_path.push('/control?session_id=<id> → play → pause → seek → stop');
  evidence.claims.C6 = { commands: ['play', 'pause', 'seek', 'stop'], revision_aware: true, display_media_sync: commandMedia };

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
  const readyMediaB = await assertUsableMedia(display, 'Session B');
  assertNoUnexpectedMediaFailures('Session B');
  evidence.production_path.push('Display renders Session B from the server-owned current rendering relationship');
  evidence.claims.C4 = {
    rendering_session_a: rendering.session_id,
    rendering_session_b: renderingB.session_id,
    item_revision: renderingB.item_revision,
    safe_gateway_path: true,
  };
  evidence.claims.C5 = {
    ...evidence.claims.C5,
    browser_media_path_after_session_b: safeUrl(mediaPathB),
    session_b_ready_state: readyMediaB.readyState,
    session_b_duration_seconds: readyMediaB.duration,
  };

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
  assertNoUnexpectedMediaFailures('final browser journey');
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
