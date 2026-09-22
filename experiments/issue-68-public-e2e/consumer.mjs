import fs from 'node:fs';
import { execFileSync } from 'node:child_process';
import { chromium } from 'playwright-core';

const base = process.env.BASE_URL || 'http://127.0.0.1:8787';
const selector = 'BV14V411W7r5';
const evidencePath = process.env.EVIDENCE_PATH || 'public-e2e-evidence.json';
const evidence = {
  selector,
  site: 'bilibili',
  public_source: 'frozen-public-no-login-non-drm',
  source_request: { status_class: 'not-started', display_http_request_count: 0, media_request_count: 0, bytes: 0 },
  source_worker_observation: { request_count: null, byte_count: null, request_budget: 200, byte_budget: 32 * 1024 * 1024, counter_source: 'gateway_browser_network_events', enforcement: 'gateway_public_browser_limits' },
  display: { registered_by_product_page: false, session_attached: false, same_origin_capability: false },
  media: { http_status_class: 'not-observed', content_type_class: 'not-observed', range_class: 'not-observed', bytes: 0 },
  progress: { loadedmetadata: false, loadeddata: false, ready_state: 0, current_time: 0, dimensions: false, duration_observed: false },
  control: { actions: [], stale_revision: false, display_refresh_reconnect: false, control_refresh_reconnect: false },
  sandbox: { low_privilege: false, pid_namespaces: false, network_namespaces: false, seccomp_bpf: false, tsync: false, no_disable_switches: false },
  cleanup: { browser_closed: false, bounded_residue_check: false },
};

function writeEvidence() {
  fs.writeFileSync(evidencePath, JSON.stringify(evidence, null, 2));
}

function classForStatus(status) {
  if (status >= 200 && status < 300) return '2xx';
  if (status >= 300 && status < 400) return '3xx';
  if (status >= 400 && status < 500) return '4xx';
  if (status >= 500 && status < 600) return '5xx';
  return 'other';
}

function boundedHeader(response, name) {
  const value = response.headers()[name];
  return value && value.length < 128 ? value : undefined;
}

async function main() {
  if (typeof process.getuid === 'function' && process.getuid() === 0) throw new Error('low_privilege_required');
  evidence.sandbox.low_privilege = true;
  const browser = await chromium.launch({ executablePath: process.env.CHROME_PATH, headless: true, chromiumSandbox: true, args: [] });
  try {
    const args = execFileSync('ps', ['-u', String(process.getuid()), '-o', 'args='], { encoding: 'utf8' });
    evidence.sandbox.no_disable_switches = !/(--no-sandbox|--disable-setuid-sandbox|--disable-namespace-sandbox)/i.test(args);
    if (!evidence.sandbox.no_disable_switches) throw new Error('sandbox_disable_switch_observed');

    const sandboxPage = await browser.newPage();
    await sandboxPage.goto('chrome://sandbox', { waitUntil: 'domcontentloaded', timeout: 15000 });
    const sandboxText = await sandboxPage.locator('body').innerText().catch(() => '');
    const fields = {
      pid_namespaces: /PID namespaces\s*:\s*(?:Yes|Enabled)/i,
      network_namespaces: /Network namespaces\s*:\s*(?:Yes|Enabled)/i,
      seccomp_bpf: /Seccomp-BPF sandbox\s*:\s*(?:Yes|Enabled)/i,
      tsync: /TSYNC\s*:\s*(?:Yes|Enabled)/i,
    };
    for (const [key, pattern] of Object.entries(fields)) {
      evidence.sandbox[key] = pattern.test(sandboxText);
      if (!evidence.sandbox[key]) throw new Error(`sandbox_field_missing:${key}`);
    }
    await sandboxPage.close();

    const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
    page.on('request', () => { evidence.source_request.display_http_request_count += 1; });
    page.on('response', response => {
      const url = response.url();
      if (!url.startsWith(new URL(base).origin)) return;
      if (!/\/(stream|media\/delivery)\//.test(new URL(url).pathname)) return;
      evidence.source_request.media_request_count += 1;
      evidence.media.http_status_class = classForStatus(response.status());
      evidence.media.content_type_class = (boundedHeader(response, 'content-type') || 'unknown').split(';', 1)[0];
      evidence.media.range_class = boundedHeader(response, 'content-range') ? 'content-range' : (boundedHeader(response, 'accept-ranges') ? 'accept-ranges' : 'unknown');
      const length = Number(boundedHeader(response, 'content-length') || 0);
      if (Number.isSafeInteger(length) && length >= 0 && length <= 64 * 1024 * 1024) evidence.media.bytes += length;
    });
    page.on('requestfailed', () => { evidence.source_request.failed_requests = (evidence.source_request.failed_requests || 0) + 1; });

    const registrationResponsePromise = page.waitForResponse(response => response.url().endsWith('/api/v1/displays/register') && response.request().method() === 'POST', { timeout: 30000 });
    const heartbeatResponsePromise = page.waitForResponse(response => response.url().includes('/heartbeat') && response.request().method() === 'POST', { timeout: 30000 });
    await page.goto(`${base}/display`, { waitUntil: 'domcontentloaded', timeout: 30000 });
    const registrationResponse = await registrationResponsePromise;
    const registration = await registrationResponse.json();
    if (!registration.display_id || !registration.registration_id || !registration.lease_token) throw new Error('display_registration_missing');
    evidence.display.registered_by_product_page = true;
    evidence.display.heartbeat_observed = await heartbeatResponsePromise.then(() => true).catch(() => false);
    if (!evidence.display.heartbeat_observed) throw new Error('display_heartbeat_missing');

    const created = await page.evaluate(async ({ selector: value, displayId }) => {
      const source = `https://www.bilibili.com/video/${value}`;
      const response = await fetch('/api/v1/sessions', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ request_id: 'public-e2e-1', source, display_id: displayId }) });
      const body = await response.json();
      return { status: response.status, body };
    }, { selector, displayId: registration.display_id });
    evidence.source_request.status_class = classForStatus(created.status);
    if (created.status !== 200) {
      evidence.source_request.error_class = 'public_source_create_failed';
      throw new Error(`public_source_create_${created.status}`);
    }
    const sessionId = created.body.session_id;
    let revision = Number(created.body.session_revision);
    if (!sessionId || !Number.isSafeInteger(revision)) throw new Error('session_authority_missing');

    await page.waitForFunction(() => {
      const video = document.querySelector('video');
      return Boolean(video && video.src && video.src.startsWith(location.origin + '/') && video.readyState >= 1);
    }, null, { timeout: 60000 });
    const initialContext = await page.evaluate(async () => {
      const registration = JSON.parse(sessionStorage.getItem('gateway.tv.display.v1') || 'null');
      if (!registration?.display_id || !registration?.lease_token) return { status: 0, session_id: null };
      const response = await fetch(`/api/v1/displays/${encodeURIComponent(registration.display_id)}/rendering`, { headers: { 'x-display-lease': registration.lease_token } });
      const body = await response.json();
      return { status: response.status, session_id: body.context?.session_id || null };
    });
    if (initialContext.status !== 200 || initialContext.session_id !== sessionId) throw new Error('display_session_authority_mismatch');
    evidence.display.session_attached = true;
    const attached = await page.evaluate(() => {
      const video = document.querySelector('video');
      const src = video?.src || '';
      return { same_origin: src.startsWith(location.origin + '/'), gateway_path: /\/(stream|media\/delivery)\//.test(new URL(src).pathname), ready_state: video?.readyState || 0, current_time: Number(video?.currentTime || 0), duration: Number.isFinite(video?.duration) ? video.duration : null, width: video?.videoWidth || 0, height: video?.videoHeight || 0 };
    });
    evidence.display.session_attached = evidence.display.session_attached && Boolean(attached.same_origin && attached.gateway_path);
    evidence.display.same_origin_capability = evidence.display.session_attached;
    evidence.progress.ready_state = attached.ready_state;
    evidence.progress.current_time = attached.current_time;
    evidence.progress.dimensions = attached.width > 0 && attached.height > 0;
    evidence.progress.duration_observed = attached.duration !== null;
    evidence.progress.loadedmetadata = attached.ready_state >= 1;
    evidence.progress.loadeddata = attached.ready_state >= 2;
    if (!evidence.display.session_attached) throw new Error('display_not_attached_to_gateway_media');

    const activate = page.locator('#activate');
    if (await activate.count()) await activate.click().catch(() => {});
    async function command(type, extra = {}) {
      const result = await page.evaluate(async ({ sessionId: id, expectedRevision, type: commandType, extra: commandExtra }) => {
        const response = await fetch(`/api/v1/sessions/${encodeURIComponent(id)}/commands`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ request_id: `public-e2e-${commandType}-${Date.now()}`, expected_session_revision: expectedRevision, command: { type: commandType, ...commandExtra } }) });
        const body = await response.json();
        return { status: response.status, body };
      }, { sessionId, expectedRevision: revision, type, extra });
      if (result.status === 409) { evidence.control.stale_revision = true; throw new Error('stale_revision'); }
      if (result.status < 200 || result.status >= 300) throw new Error(`control_${type}_${result.status}`);
      revision = Number(result.body.session_revision);
      if (!Number.isSafeInteger(revision)) throw new Error('control_revision_missing');
      evidence.control.actions.push(type);
    }
    await command('play');
    await command('pause');
    await command('seek', { position_ms: 0 });
    await command('play');

    const reloadRegistration = page.waitForResponse(response => response.url().endsWith('/api/v1/displays/register') && response.request().method() === 'POST', { timeout: 30000 });
    const reloadHeartbeat = page.waitForResponse(response => response.url().includes('/heartbeat') && response.request().method() === 'POST', { timeout: 30000 });
    await page.reload({ waitUntil: 'domcontentloaded', timeout: 30000 });
    const reconnected = await reloadRegistration;
    const reconnectedBody = await reconnected.json();
    if (!reconnectedBody.display_id || !reconnectedBody.registration_id || !reconnectedBody.lease_token) throw new Error('display_reconnect_registration_missing');
    if (reconnectedBody.display_id !== registration.display_id) throw new Error('display_identity_changed_on_reconnect');
    evidence.display.reconnect_heartbeat = await reloadHeartbeat.then(() => true).catch(() => false);
    if (!evidence.display.reconnect_heartbeat) throw new Error('display_reconnect_heartbeat_missing');
    await page.waitForFunction(() => Boolean(document.querySelector('video')), null, { timeout: 30000 });
    const reconnectedContext = await page.evaluate(async () => {
      const current = JSON.parse(sessionStorage.getItem('gateway.tv.display.v1') || 'null');
      if (!current?.display_id || !current?.lease_token) return { status: 0, session_id: null };
      const response = await fetch(`/api/v1/displays/${encodeURIComponent(current.display_id)}/rendering`, { headers: { 'x-display-lease': current.lease_token } });
      const body = await response.json();
      return { status: response.status, session_id: body.context?.session_id || null };
    });
    if (reconnectedContext.status !== 200 || reconnectedContext.session_id !== sessionId) throw new Error('display_reconnect_session_mismatch');
    evidence.control.display_refresh_reconnect = true;

    const controlPage = await browser.newPage();
    const controlView = controlPage.waitForResponse(response => response.url().includes(`/api/v1/control/${encodeURIComponent(sessionId)}`), { timeout: 30000 });
    await controlPage.goto(`${base}/control?session_id=${encodeURIComponent(sessionId)}`, { waitUntil: 'domcontentloaded', timeout: 30000 });
    if (!(await controlView).ok()) throw new Error('control_view_refresh_failed');
    const controlReload = controlPage.waitForResponse(response => response.url().includes(`/api/v1/control/${encodeURIComponent(sessionId)}`), { timeout: 30000 });
    await controlPage.reload({ waitUntil: 'domcontentloaded', timeout: 30000 });
    if (!(await controlReload).ok()) throw new Error('control_view_reconnect_failed');
    evidence.control.control_refresh_reconnect = true;
    await controlPage.close();
    await command('stop');
    writeEvidence();
  } finally {
    await browser.close();
    evidence.cleanup.browser_closed = true;
    writeEvidence();
  }
}

try {
  await main();
} catch (error) {
  const message = String(error?.message || '');
  if (/^sandbox_field_missing:/.test(message)) evidence.failure_class = 'sandbox_field_missing';
  else if (message === 'sandbox_disable_switch_observed') evidence.failure_class = 'sandbox_disable_switch_observed';
  else if (message === 'low_privilege_required') evidence.failure_class = 'low_privilege_required';
  else if (message === 'display_heartbeat_missing' || message === 'display_reconnect_heartbeat_missing') evidence.failure_class = 'display_heartbeat_missing';
  else if (/^public_source_create_/.test(message)) evidence.failure_class = 'public_source_create_failed';
  else if (message === 'stale_revision') evidence.failure_class = 'stale_revision';
  else if (/^control_/.test(message)) evidence.failure_class = 'control_command_failed';
  else if (/^display_/.test(message)) evidence.failure_class = 'display_authority_failed';
  else if (/^control_view_/.test(message)) evidence.failure_class = 'control_view_failed';
  else evidence.failure_class = 'bounded_failure';
  writeEvidence();
  process.exitCode = 1;
}
