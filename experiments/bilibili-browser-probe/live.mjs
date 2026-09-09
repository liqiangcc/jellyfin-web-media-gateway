#!/usr/bin/env node
/*
 * Opt-in live browser probe. It is intentionally separate from the synthetic
 * fixture path and is never enabled by the hosted workflow. URLs and response
 * bodies stay in the process; only the sanitized observation leaves it.
 */
import dns from 'node:dns/promises';
import http from 'node:http';
import https from 'node:https';
import net from 'node:net';
import tls from 'node:tls';
import { Transform } from 'node:stream';
import { chmod, mkdtemp, rm } from 'node:fs/promises';
import { once } from 'node:events';
import { chromium } from 'playwright-core';
import { navigationDescriptor, validateLiveInvocation } from '../../plugins/bilibili/live_selector.mjs';
import { summarizeObservation, SCHEMA_VERSION } from '../../plugins/bilibili/experimental_probe.mjs';
import { classifyError, classifyNavigationLifecycle, classifyTransport, diagnosticCounters } from './diagnostic.mjs';
import { createFinalizer, terminationClass } from './finalizer.mjs';
import { createStageTracker } from './stage-markers.mjs';

const MAX_REQUESTS = 200;
const MAX_RESPONSE_BYTES = 32 * 1024 * 1024;
const MAX_CANDIDATES = 64;
const MAX_INDEPENDENT_REQUESTS = 8;
const MAX_INDEPENDENT_BYTES = 4 * 1024 * 1024;
const MAX_INDEPENDENT_PER_REQUEST = 1024 * 1024;
const SECRET_HEADERS = new Set(['cookie', 'authorization', 'proxy-authorization', 'set-cookie']);
const SAFE_HEADERS = new Set(['accept-ranges', 'content-range', 'content-type', 'content-length', 'etag', 'last-modified']);
const MEDIA_TYPES = /^(video\/|audio\/|application\/(vnd\.apple\.mpegurl|dash\+xml))/i;
export const LIVE_BROWSER_ARGS = Object.freeze(['--disable-quic', '--disable-features=WebTransport', '--disable-background-networking']);

function listen(server) {
  server.listen(0, '127.0.0.1');
  return once(server, 'listening').then(() => server.address().port);
}

function publicIpv4(address) {
  const octets = address.split('.').map(Number);
  if (octets.length !== 4 || octets.some((item) => !Number.isInteger(item) || item < 0 || item > 255)) return false;
  const [a, b] = octets;
  return !(a === 0 || a === 10 || a === 127 || a >= 224 || (a === 100 && b >= 64 && b <= 127) ||
    (a === 169 && b === 254) || (a === 172 && b >= 16 && b <= 31) || (a === 192 && b === 0) ||
    (a === 192 && b === 168) || (a === 192 && b === 2) || (a === 198 && (b === 18 || b === 19 || b === 51)) ||
    (a === 203 && b === 0));
}

function publicAddress(address) {
  if (net.isIPv4(address)) return publicIpv4(address);
  if (net.isIPv6(address)) {
    const normalized = address.toLowerCase();
    if (normalized.startsWith('::ffff:')) return publicIpv4(normalized.slice('::ffff:'.length));
    return normalized !== '::' && normalized !== '::1' && !normalized.startsWith('ff') && !normalized.startsWith('2001:db8') && !normalized.startsWith('fc') && !normalized.startsWith('fd') &&
      !normalized.startsWith('fe8') && !normalized.startsWith('fe9') && !normalized.startsWith('fea') && !normalized.startsWith('feb');
  }
  return false;
}

function allowedHost(host) {
  const value = host.toLowerCase().replace(/\.$/, '');
  return value === 'bilibili.com' || value.endsWith('.bilibili.com') ||
    value.endsWith('.bilivideo.com') || value.endsWith('.bilivideo.cn') || value.endsWith('.hdslb.com');
}

function safeRequestHeaders(headers) {
  return Object.fromEntries(Object.entries(headers).filter(([name]) => !SECRET_HEADERS.has(name.toLowerCase())));
}

function statusClass(status) {
  if (!Number.isInteger(status)) return 'unknown';
  return `${Math.floor(status / 100)}xx`;
}

// Process-level failures are published by probe.mjs using the same finite process_termination marker.
function recordStage(state, event, fields = {}) {
  state.stages?.record(event, { ...diagnosticCounters(state), ...fields });
}

export function shouldRecordFailure(state, phaseHint, lifecycleOutcome) {
  const explicitNavigationLifecycle = phaseHint === 'chromium_navigation' &&
    typeof lifecycleOutcome === 'string' && lifecycleOutcome !== 'unknown';
  return !(state.failure || state.transportFinalized ||
    (state.transport && !state.responseLimitTriggered && !explicitNavigationLifecycle));
}

function recordFailure(state, error, phaseHint, status, transportStage, lifecycleOutcome) {
  // Once the broker has emitted a successful CONNECT response, a later socket
  // callback belongs to the already observed transport outcome. The explicit
  // response-budget path or an explicitly settled navigation lifecycle is the
  // only failure that may supersede it before the tunnel cleanup callback runs.
  if (!shouldRecordFailure(state, phaseHint, lifecycleOutcome)) return state.failure;
  const counters = diagnosticCounters(state);
  state.failure = { ...classifyError(error, phaseHint, { ...counters, status, transport_stage: transportStage, lifecycle_outcome: lifecycleOutcome }), lifecycle_outcome: lifecycleOutcome || 'unknown' };
  return state.failure;
}

function recordTransportSuccess(state, stage, status) {
  if (state.failure || state.transport) return state.transport;
  state.transport = classifyTransport({ stage, outcome: 'success', status, ...diagnosticCounters(state) });
  return state.transport;
}

async function publicAddressFor(host, pins) {
  const answers = await dns.lookup(host, { all: true, verbatim: false });
  if (!answers.length || answers.some(({ address }) => !publicAddress(address))) throw new Error('non-public DNS answer denied');
  const addresses = answers.map(({ address }) => address).sort();
  const previous = pins.get(host);
  if (previous && previous.join(',') !== addresses.join(',')) throw new Error('DNS pin changed');
  pins.set(host, addresses);
  return addresses[0];
}

export function brokerServer(state, overrides = {}) {
  const resolveAddress = overrides.resolveAddress || publicAddressFor;
  const makeRequest = overrides.request || https.request;
  const makeConnect = overrides.connect || tls.connect;
  const onTunnelClosed = overrides.onTunnelClosed || (() => {});
  const server = http.createServer(async (req, res) => {
    recordStage(state, 'broker_request_start');
    if (state.requests++ >= MAX_REQUESTS) { recordStage(state, 'broker_request_result', { status_class: '4xx', transport_outcome: 'failure' }); res.writeHead(429); res.end(); return; }
    let target;
    try { target = new URL(req.url); } catch { recordStage(state, 'broker_request_result', { status_class: '4xx', transport_outcome: 'failure' }); res.writeHead(400); res.end(); return; }
    const host = target.hostname.toLowerCase();
    if (target.protocol !== 'https:' || target.port && target.port !== '443' || !allowedHost(host)) {
      state.denied.push({ kind: 'http', host: host || 'unknown', reason: 'host-or-scheme-policy' });
      recordStage(state, 'broker_request_result', { status_class: '4xx', transport_outcome: 'failure' });
      res.writeHead(403); res.end('denied'); return;
    }
    let address;
    try { address = await resolveAddress(host, state.pins); } catch {
      recordFailure(state, { code: 'ERR_DNS_ADDRESS_POLICY' }, 'dns_address_policy', undefined, 'resolve_policy');
      state.denied.push({ kind: 'http', host, reason: 'dns-address-policy' });
      recordStage(state, 'broker_request_result', { status_class: '4xx', transport_stage: 'resolve_policy', transport_outcome: 'failure' });
      res.writeHead(403); res.end('denied'); return;
    }
    const options = {
      host: address, port: 443, servername: host, path: `${target.pathname}${target.search}`,
      method: req.method, headers: { ...safeRequestHeaders(req.headers), host, connection: 'close' },
      rejectUnauthorized: true,
    };
    const upstream = makeRequest(options, (reply) => {
      recordStage(state, 'broker_request_result', { status_class: statusClass(reply.statusCode), transport_outcome: 'success' });
      res.writeHead(reply.statusCode || 502, reply.headers);
      reply.on('data', (chunk) => {
        state.responseBytes += chunk.length;
        if (state.responseBytes > (state.responseLimit || MAX_RESPONSE_BYTES)) {
          reply.destroy(new Error('response budget exceeded'));
          if (!res.writableEnded) res.end();
          return;
        }
        if (!res.writableEnded) res.write(chunk);
      });
      reply.on('end', () => { if (!res.writableEnded) res.end(); });
      reply.on('error', () => { if (!res.writableEnded) res.end(); });
    });
    upstream.on('error', (error) => {
      recordFailure(state, error, 'broker_connect');
      recordStage(state, 'broker_request_result', { transport_outcome: 'failure' });
      if (!res.writableEnded) { res.writeHead(502); res.end(); }
    });
    req.pipe(upstream);
  });
  server.on('connect', async (req, client, head) => {
    recordStage(state, 'broker_request_start');
    if (state.requests++ >= MAX_REQUESTS) { recordStage(state, 'broker_request_result', { status_class: '4xx', transport_outcome: 'failure' }); client.end('HTTP/1.1 429 Too Many Requests\r\n\r\n'); return; }
    const [host, portText] = String(req.url).split(':');
    const port = Number(portText || 443);
    if (!allowedHost(host) || port !== 443) {
      state.denied.push({ kind: 'connect', host: String(host).toLowerCase(), reason: 'host-or-port-policy' });
      recordStage(state, 'broker_request_result', { status_class: '4xx', transport_outcome: 'failure' });
      client.end('HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n'); return;
    }
    let address;
    try { address = await resolveAddress(host.toLowerCase(), state.pins); } catch {
      recordFailure(state, { code: 'ERR_DNS_ADDRESS_POLICY' }, 'dns_address_policy', undefined, 'resolve_policy');
      state.denied.push({ kind: 'connect', host: String(host).toLowerCase(), reason: 'dns-address-policy' });
      recordStage(state, 'broker_request_result', { status_class: '4xx', transport_stage: 'resolve_policy', transport_outcome: 'failure' });
      client.end('HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n'); return;
    }
    const upstream = makeConnect({ host: address, port, servername: host, rejectUnauthorized: true });
    upstream.once('secureConnect', () => {
      client.write('HTTP/1.1 200 Connection Established\r\nProxy-agent: bounded-bilibili-probe\r\n\r\n');
      recordStage(state, 'broker_request_result', { status_class: '2xx', transport_stage: 'proxy_response', transport_outcome: 'success' });
      recordStage(state, 'transport_outcome', { status_class: '2xx', transport_stage: 'tls_handshake', transport_outcome: 'success' });
      recordTransportSuccess(state, 'proxy_response', 200);
      if (head?.length) upstream.write(head);
      let closed = false;
      const closeTunnel = () => {
        if (closed) return;
        closed = true;
        client.unpipe(upstream); upstream.unpipe(counted);
        counted.destroy(); upstream.destroy(); client.destroy();
        if (!state.failure) state.transport = classifyTransport({ stage: 'downstream_close', outcome: 'success', ...diagnosticCounters(state) });
        recordStage(state, 'transport_outcome', { transport_stage: 'downstream_close', transport_outcome: state.failure ? 'failure' : 'success' });
        state.transportFinalized = true;
        onTunnelClosed({ client_destroyed: client.destroyed, upstream_destroyed: upstream.destroyed, counter_destroyed: counted.destroyed });
      };
      const counted = new Transform({ transform(chunk, encoding, callback) {
        state.responseBytes += chunk.length;
        if (state.responseBytes > (state.responseLimit || MAX_RESPONSE_BYTES)) {
          state.responseLimitTriggered = true;
          recordFailure(state, { code: 'BROKER_CONNECT' }, 'broker_connect', undefined, 'downstream_close');
          recordStage(state, 'transport_outcome', { transport_stage: 'downstream_close', transport_outcome: 'failure' });
          closeTunnel(); callback(new Error('response budget exceeded')); return;
        }
        callback(null, chunk, encoding);
      } });
      counted.on('error', () => {
        closeTunnel();
      });
      client.pipe(upstream);
      upstream.pipe(counted).pipe(client);
    });
    upstream.on('error', (error) => { recordFailure(state, error, 'broker_connect'); recordStage(state, 'broker_request_result', { transport_outcome: 'failure' }); recordStage(state, 'transport_outcome', { transport_stage: 'tcp_connect', transport_outcome: 'failure' }); client.destroy(); });
    client.on('error', () => upstream.destroy());
  });
  server.on('upgrade', (req, socket) => {
    recordStage(state, 'broker_request_start');
    state.requests += 1;
    state.denied.push({ kind: 'upgrade', host: String(req.headers.host || 'unknown').toLowerCase(), reason: 'upgrade-disabled' });
    recordStage(state, 'broker_request_result', { status_class: '4xx', transport_outcome: 'failure' });
    socket.end('HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n');
  });
  return server;
}

export function consumeResponseBody(response, maxBytes, signal) {
  return new Promise((resolve, reject) => {
    let bytes = 0;
    let settled = false;
    const fail = (error) => { if (!settled) { settled = true; response.destroy?.(); reject(error); } };
    const onAbort = () => fail(new Error('independent consumer cancelled'));
    if (signal?.aborted) { onAbort(); return; }
    signal?.addEventListener('abort', onAbort, { once: true });
    response.on('data', (chunk) => {
      bytes += chunk.length;
      if (bytes > maxBytes) { fail(new Error('independent request budget exceeded')); }
    });
    response.on('error', fail);
    response.on('end', () => {
      if (settled) return;
      settled = true;
      signal?.removeEventListener('abort', onAbort);
      resolve({ status_class: statusClass(response.statusCode), bytes, content_type: String(response.headers?.['content-type'] || 'unknown').split(';', 1)[0] });
    });
  });
}

export function requestOne(candidate, pins, maxBytes, signal) {
  return new Promise((resolve, reject) => {
    let target;
    try { target = new URL(candidate.url); } catch { reject(new Error('candidate URL malformed')); return; }
    if (target.protocol !== 'https:' || !allowedHost(target.hostname)) { reject(new Error('candidate host denied')); return; }
    publicAddressFor(target.hostname, pins).then((address) => {
      const rangeEnd = Math.max(0, maxBytes - 1);
      const req = https.request({ host: address, port: 443, servername: target.hostname, path: `${target.pathname}${target.search}`, method: 'GET', headers: { host: target.hostname, range: `bytes=0-${rangeEnd}`, 'user-agent': 'bilibili-browser-probe/1', referer: 'https://www.bilibili.com/' }, rejectUnauthorized: true, signal }, (res) => {
        consumeResponseBody(res, maxBytes, signal).then(resolve, reject);
      });
      req.on('error', reject); req.end();
    }, reject).catch(reject);
  });
}

export async function createDisposableProfile() {
  const profile = await mkdtemp('/tmp/bilibili-live-probe-');
  await chmod(profile, 0o700);
  return profile;
}

export function removeDisposableProfile(profile) {
  return rm(profile, { recursive: true, force: true });
}

export async function runLive(invocation, browserPath = process.env.CHROME_PATH || '/usr/bin/google-chrome') {
  const admitted = validateLiveInvocation(invocation);
  const state = { requests: 0, responseBytes: 0, metadataBytes: 0, denied: [], pins: new Map(), stages: createStageTracker() };
  const candidates = new Map();
  const broker = brokerServer(state);
  const brokerPort = await listen(broker);
  let browser;
  let context;
  let profile;
  let result;
  let caughtError;
  let navigationActive = false;
  const finalizer = createFinalizer(state.stages);
  try {
    profile = await createDisposableProfile();
    recordStage(state, 'browser_launch_start');
    try {
      context = await chromium.launchPersistentContext(profile, { executablePath: browserPath, headless: true, timeout: admitted.timeout_ms, args: [
        `--proxy-server=http://127.0.0.1:${brokerPort}`, '--proxy-bypass-list=<-loopback>',
        ...LIVE_BROWSER_ARGS,
        '--no-first-run', '--no-default-browser-check',
      ], serviceWorkers: 'block' });
      recordStage(state, 'browser_launch_result', { status_class: '2xx', transport_outcome: 'success' });
    } catch (error) {
      recordStage(state, 'browser_launch_result', { transport_outcome: 'failure' });
      throw error;
    }
    browser = context.browser();
    await context.route('**/*', async (route) => {
      const headers = safeRequestHeaders(await route.request().allHeaders());
      await route.continue({ headers });
    });
    const page = await context.newPage();
    const observeLifecycle = (event, lifecycleOutcome) => {
      if (!navigationActive || state.lifecycleOutcome) return;
      state.lifecycleOutcome = lifecycleOutcome;
      recordStage(state, event, { lifecycle_outcome: lifecycleOutcome, transport_outcome: 'failure' });
    };
    browser?.on('disconnected', () => observeLifecycle('browser_disconnect', 'browser_disconnected'));
    context.on('close', () => observeLifecycle('page_lifecycle_result', 'page_closed'));
    page.on('close', () => observeLifecycle('page_lifecycle_result', 'page_closed'));
    page.on('crash', () => observeLifecycle('page_lifecycle_result', 'page_crashed'));
    const pendingObservations = new Set();
    page.on('response', async (response) => {
      const task = (async () => {
        if (candidates.size >= MAX_CANDIDATES || response.status() < 200 || response.status() >= 300) return;
        let url;
        try { url = new URL(response.url()); } catch { return; }
        if (!allowedHost(url.hostname)) return;
        const headers = await response.allHeaders().catch(() => ({}));
        const contentType = String(headers['content-type'] || '').split(';', 1)[0];
        if (!MEDIA_TYPES.test(contentType)) return;
        const id = candidates.size + 1;
        const mediaPath = `${url.pathname}${url.search}`.toLowerCase();
        const role = contentType.startsWith('audio/') ? 'audio' : /(^|[/?=&_-])video([/?=&_.-]|$)/.test(mediaPath) ? 'video' : 'muxed';
        candidates.set(id, { url: url.href, role, codec: 'unknown', container: contentType.split('/')[1] || 'unknown', status_class: statusClass(response.status()), range_supported: 'unknown', header_names: Object.keys(headers).filter((name) => SAFE_HEADERS.has(name.toLowerCase())), egress_allowed: true, expiry_hint: 'unknown' });
      })();
      pendingObservations.add(task);
      task.finally(() => pendingObservations.delete(task));
    });
    navigationActive = true;
    recordStage(state, 'navigation_start');
    let navigationResponse;
    try {
      navigationResponse = await page.goto(admitted.navigation.url, { waitUntil: 'domcontentloaded', timeout: admitted.timeout_ms });
      navigationActive = false;
      recordStage(state, 'navigation_promise_result', { lifecycle_outcome: 'fulfilled', transport_outcome: 'success' });
      const navigationStatus = navigationResponse?.status();
      recordStage(state, 'navigation_status', { status_class: statusClass(navigationStatus) });
      if (navigationResponse?.status() >= 300) {
        recordFailure(state, { code: 'ERR_HTTP_RESPONSE_CODE_FAILURE' }, 'http_status', navigationResponse.status());
        throw new Error('navigation returned a non-success status');
      }
      recordStage(state, 'navigation_end', { status_class: statusClass(navigationStatus), transport_outcome: 'success' });
    } catch (error) {
      const lifecycleOutcome = state.lifecycleOutcome || classifyNavigationLifecycle(error);
      navigationActive = false;
      recordStage(state, 'navigation_promise_result', { lifecycle_outcome: lifecycleOutcome, transport_outcome: 'failure' });
      recordFailure(state, error, 'chromium_navigation', undefined, undefined, lifecycleOutcome);
      recordStage(state, 'navigation_status', { transport_stage: state.failure?.transport_stage, transport_outcome: 'failure' });
      recordStage(state, 'navigation_end', { transport_stage: state.failure?.transport_stage, transport_outcome: 'failure' });
      throw error;
    }
    navigationActive = false;
    await page.waitForTimeout(Math.min(3000, admitted.timeout_ms));
    await Promise.allSettled([...pendingObservations]);
    const partMatch = new URL(page.url()).pathname.includes(`/video/${admitted.navigation.bvid}`) && new URL(page.url()).searchParams.get('p') === String(admitted.navigation.part);
    await context.close(); context = undefined; browser = undefined;
    const independent = [];
    let totalBytes = 0;
    for (const candidate of candidates.values()) {
      if (independent.length >= MAX_INDEPENDENT_REQUESTS || totalBytes >= MAX_INDEPENDENT_BYTES) break;
      try {
        const remaining = Math.min(MAX_INDEPENDENT_PER_REQUEST, MAX_INDEPENDENT_BYTES - totalBytes);
        const controller = new AbortController();
        const timer = setTimeout(() => controller.abort(), Math.min(admitted.timeout_ms, 15000));
        const result = await requestOne(candidate, state.pins, remaining, controller.signal).finally(() => clearTimeout(timer));
        totalBytes += result.bytes;
        independent.push({ ...result, role: candidate.role });
      } catch { independent.push({ status_class: 'unknown', bytes: 0, content_type: 'unknown', role: candidate.role }); }
    }
    const roles = independent.filter((item) => item.status_class === '2xx').map((item) => item.role);
    const sourceKind = roles.includes('video') && roles.includes('audio') ? 'av_separated' : roles.length ? 'muxed' : 'unknown';
    const raw = { schema_version: SCHEMA_VERSION, selector: { site_id: 'bilibili', content_id: admitted.navigation.bvid, part: admitted.navigation.part }, part_match: partMatch, source_kind: sourceKind, events: state.requests, resources: candidates.size, metadata_bytes: JSON.stringify([...candidates.values()].map(({ url, ...item }) => item)).length, independent_after_browser_exit: independent.some((item) => item.status_class === '2xx'), candidates: [...candidates.values()].map(({ url, ...item }) => ({ ...item, independent: undefined })) };
    const observation = summarizeObservation(raw);
    state.metadataBytes = raw.metadata_bytes;
    observation.budget = { ...observation.budget, request_count: state.requests, response_bytes: state.responseBytes, response_budget: MAX_RESPONSE_BYTES, independent_requests: independent.length, independent_bytes: totalBytes };
    observation.containment = { broker: 'public-host-pinned', dns_pin: 'public-address-pinned', redirects: 'revalidated-per-hop', connect: 'public-host-only', websocket: 'no-upgrade-export', service_worker: 'disabled-for-observation', quic: 'disabled', secret_headers: 'stripped-and-not-exported' };
    observation.independent_consumer = { requests: independent.length, bytes: totalBytes, results: independent.map(({ role, status_class, bytes, content_type }) => ({ role, status_class, bytes, content_type })) };
    observation.cleanup = { browser_exit: 'complete', temporary_profile: 'removed-in-finally', ephemeral_candidates: 'cleared-in-finally' };
    state.stages.record('finalizer_entry');
    result = { schema_version: SCHEMA_VERSION, termination: 'normal', observation, denied_count: state.denied.length, stage_markers: state.stages.seal() };
  } catch (error) {
    caughtError = error;
  } finally {
    candidates.clear();
    if (context) await context.close().catch(() => {});
    if (browser) await browser.close().catch(() => {});
    if (profile) await removeDisposableProfile(profile).catch(() => {});
    broker.close();
    state.pins.clear(); state.denied.length = 0;
  }
  if (!result) {
    const diagnostic = state.failure || recordFailure(state, caughtError, 'chromium_navigation') || classifyError(caughtError, 'unknown', diagnosticCounters(state));
    result = finalizer.finalize({ result: 'failure', termination: terminationClass(caughtError), diagnostic, activity: { page_navigation: true }, cleanup: {
      browser_exit: 'complete', broker_close: 'complete', temporary_profile: 'complete', ephemeral_candidates: 'complete', dns_pins: 'complete', staging: 'unknown',
    } });
  }
  return result;
}
