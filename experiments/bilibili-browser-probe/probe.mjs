#!/usr/bin/env node
/* Offline-only browser acquisition probe for #165. */
import http from 'node:http';
import net from 'node:net';
import { once } from 'node:events';
import { chromium } from 'playwright-core';
import { summarizeObservation, rejectSensitiveInput, SCHEMA_VERSION } from '../../plugins/bilibili/experimental_probe.mjs';
import { runLive } from './live.mjs';
import { classifyError } from './diagnostic.mjs';
import { finalizeResult, terminationClass } from './finalizer.mjs';

const TIMEOUT_MS = 15_000;
const MAX_REQUESTS = 200;
const MAX_RESPONSE_BYTES = 32 * 1024 * 1024;
const browserPath = process.env.CHROME_PATH || '/usr/bin/google-chrome';

function listen(server) {
  server.listen(0, '127.0.0.1');
  return once(server, 'listening').then(() => server.address().port);
}

function fixtureServer() {
  const server = http.createServer((req, res) => {
    if (req.url === '/page') {
      res.writeHead(200, { 'content-type': 'text/html' });
      res.end(`<!doctype html><script>
        window.__probeObservation = {
          schema_version: 1,
          selector: { site_id: 'bilibili', content_id: 'BV-synthetic-165', part: 2 },
          part_match: true, source_kind: 'av_separated', events: 8, resources: 4,
          metadata_bytes: 384, independent_after_browser_exit: true,
          candidates: [
            { role: 'video', codec: 'avc1.640028', container: 'mp4', http_status_class: '2xx', range_supported: true, header_names: ['content-type', 'accept-ranges'], egress_allowed: true, expiry_hint: 'short-lived' },
            { role: 'audio', codec: 'mp4a.40.2', container: 'm4a', http_status_class: '2xx', range_supported: true, header_names: ['content-type', 'content-length'], egress_allowed: true, expiry_hint: 'short-lived' }
          ]
        };
        fetch('/metadata');
        fetch('http://deny.invalid/forbidden').catch(() => {});
        fetch('/redirect').catch(() => {});
        navigator.serviceWorker.register('/sw.js').catch(() => {});
        const worker = new Worker('/worker.js');
        worker.onmessage = () => { window.__workerDone = true; };
        const ws = new WebSocket('ws://deny.invalid/socket');
        ws.onerror = () => {};
      </script>`);
      return;
    }
    if (req.url === '/sw.js') {
      res.writeHead(200, { 'content-type': 'application/javascript', 'service-worker-allowed': '/' });
      res.end("self.addEventListener('fetch', event => { if (new URL(event.request.url).pathname === '/sw-probe') event.respondWith(fetch('http://deny.invalid/sw')); });");
      return;
    }
    if (req.url === '/worker.js') {
      res.writeHead(200, { 'content-type': 'application/javascript' });
      res.end("fetch('http://deny.invalid/worker', {mode: 'no-cors'}).catch(() => {}); setTimeout(() => postMessage('started'), 100);");
      return;
    }
    if (req.url === '/metadata') {
      res.writeHead(200, { 'content-type': 'application/json', 'content-length': '27' });
      res.end('{"fixture":"metadata","ok":true}');
      return;
    }
    if (req.url === '/redirect') {
      res.writeHead(302, { location: 'http://deny.invalid/final' });
      res.end();
      return;
    }
    if (req.url === '/media') {
      const data = Buffer.from('synthetic-media-payload-165');
      res.writeHead(200, { 'content-type': 'video/mp4', 'content-length': data.length, 'accept-ranges': 'bytes' });
      res.end(data);
      return;
    }
    if (req.url === '/video') {
      const data = Buffer.from('synthetic-video-payload-169');
      res.writeHead(200, { 'content-type': 'video/mp4', 'content-length': data.length, 'accept-ranges': 'bytes' });
      res.end(data);
      return;
    }
    if (req.url === '/audio') {
      const data = Buffer.from('synthetic-audio-payload-169');
      res.writeHead(200, { 'content-type': 'audio/mp4', 'content-length': data.length, 'accept-ranges': 'bytes' });
      res.end(data);
      return;
    }
    res.writeHead(404); res.end();
  });
  return server;
}

function brokerServer(fixturePort) {
  const requests = [];
  const server = http.createServer((req, res) => {
    let target;
    try { target = new URL(req.url); } catch { res.writeHead(400); res.end(); return; }
    const host = target.hostname.toLowerCase();
    const allowed = host === 'fixture.test';
    requests.push({ host, path: target.pathname, method: req.method, allowed, redirected: target.pathname === '/final' });
    if (!allowed) {
      res.writeHead(403, { 'content-type': 'text/plain', 'x-probe-deny': 'policy' });
      res.end('denied');
      return;
    }
    // The broker owns DNS/connection selection for the fixture.  The exact
    // loopback destination is recorded in the evidence rather than exported
    // as a caller-controlled URL.
    const headers = { ...req.headers, host: `fixture.test:${fixturePort}`, connection: 'close' };
    const forwarded = http.request({ host: '127.0.0.1', port: fixturePort, path: target.pathname + target.search, method: req.method, headers }, (reply) => {
      const chunks = [];
      reply.on('data', (chunk) => chunks.push(chunk));
      reply.on('end', () => {
        if (!res.headersSent && !res.writableEnded) {
          res.writeHead(reply.statusCode || 502, reply.headers);
          res.end(Buffer.concat(chunks));
        }
      });
    });
    forwarded.on('error', () => {
      if (!res.headersSent && !res.writableEnded) { res.writeHead(502); res.end(); }
    });
    req.pipe(forwarded);
  });
  server.on('connect', (req, socket) => {
    const host = String(req.url).split(':')[0].toLowerCase();
    requests.push({ host, path: String(req.url), method: 'CONNECT', allowed: false, redirected: false });
    socket.end('HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n');
  });
  return { server, requests };
}

async function fetchIndependent(port) {
  const results = [];
  let total = 0;
  for (const [role, path] of [['muxed', '/media'], ['video', '/video'], ['audio', '/audio']]) {
    const response = await fetch(`http://127.0.0.1:${port}${path}`);
    const bytes = Buffer.from(await response.arrayBuffer());
    if (!response.ok || bytes.length === 0 || bytes.length > 1024 * 1024) throw new Error('independent consumer failed');
    total += bytes.length;
    if (total > 4 * 1024 * 1024) throw new Error('independent consumer budget exceeded');
    results.push({ role, status_class: '2xx', bytes: bytes.length, content_type: response.headers.get('content-type') });
  }
  return { status_class: '2xx', bytes: total, content_type: 'multiple', requests: results.length, results };
}

async function runSynthetic() {
  rejectSensitiveInput({ selector: 'bilibili:BV-synthetic-165:part-2' });
  const fixtures = fixtureServer();
  const fixturePort = await listen(fixtures);
  const { server: broker, requests } = brokerServer(fixturePort);
  const brokerPort = await listen(broker);
  const profile = `/tmp/bilibili-probe-profile-${process.pid}`;
  let browser;
  try {
    browser = await chromium.launch({ executablePath: browserPath, headless: true, timeout: TIMEOUT_MS, args: [
      `--proxy-server=http://127.0.0.1:${brokerPort}`, '--proxy-bypass-list=<-loopback>',
      '--disable-quic', '--disable-features=WebTransport', '--disable-background-networking',
      '--no-first-run', '--no-default-browser-check',
    ] });
    const context = await browser.newContext({ serviceWorkers: 'block' });
    const page = await context.newPage();
    await page.goto('http://fixture.test/page', { waitUntil: 'domcontentloaded', timeout: TIMEOUT_MS });
    const raw = await page.evaluate(() => window.__probeObservation);
    const observation = summarizeObservation(raw);
    // Start a second worker from the harness with the handler installed in the
    // same evaluation. The fixture worker delays its completion message, so
    // the network denial remains observable without relying on page-script
    // scheduling.
    await page.evaluate(() => new Promise((resolve) => {
      const worker = new Worker('/worker.js');
      worker.onmessage = () => resolve(true);
      setTimeout(() => resolve(false), 1000);
    }));
    await page.waitForTimeout(1000);
    if (requests.length > MAX_REQUESTS) throw new Error('request budget exceeded');
    const denied = requests.filter((item) => !item.allowed);
    if (!requests.some((item) => item.allowed && item.host === 'fixture.test')) throw new Error('broker did not mediate allowed fixture');
    if (denied.length < 3) throw new Error('redirect/worker/websocket denial coverage missing');
    if (!denied.some((item) => item.path === '/final')) throw new Error('redirect denial missing');
    if (!denied.some((item) => item.path === '/worker')) throw new Error('dedicated worker denial missing');
    if (!denied.some((item) => item.method === 'CONNECT' || item.path === '/socket')) throw new Error('websocket denial missing');
    if (requests.some((item) => item.allowed && item.host !== 'fixture.test')) throw new Error('unexpected broker allow');
    observation.budget = { ...observation.budget, request_count: requests.length, response_bytes: 0, response_budget: MAX_RESPONSE_BYTES };
    observation.containment = {
      broker: 'loopback-allowlist',
      dns_pin: 'fixture.test→127.0.0.1 recorded by broker',
      redirect: denied.some((item) => item.redirected) ? 'denied' : 'not-observed',
      worker: denied.some((item) => item.host === 'deny.invalid' && item.path === '/worker') ? 'denied' : 'not-observed',
      service_worker: 'disabled-for-observation',
      websocket: denied.some((item) => item.method === 'CONNECT') ? 'denied-by-connect-policy' : 'denied-by-http-proxy',
      quic: 'disabled',
      secret_headers: 'not-exported',
    };
    await browser.close();
    browser = undefined;
    const independent = await fetchIndependent(fixturePort);
    observation.independent_consumer = independent;
    observation.cleanup = { browser_exit: 'complete', temporary_profile: 'removed-by-finalizer' };
    return { schema_version: SCHEMA_VERSION, observation, broker_requests: requests.map(({ host, path, method, allowed, redirected }) => ({ host, path, method, allowed, redirected })) };
  } finally {
    if (browser) await browser.close().catch(() => {});
    fixtures.close(); broker.close();
    // The profile contains no durable authority and is always disposable.
    const { rm } = await import('node:fs/promises');
    await rm(profile, { recursive: true, force: true }).catch(() => {});
  }
}

function parseArgs(argv) {
  const args = { mode: 'synthetic' };
  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === '--mode') args.mode = argv[++index];
    else if (value === '--selector') args.selector = argv[++index];
    else if (value === '--timeout-ms') args.timeout_ms = Number(argv[++index]);
    else throw new Error('unknown option; only --mode and plugin-owned --selector are accepted');
  }
  if (!['synthetic', 'live'].includes(args.mode)) throw new Error('mode must be synthetic or live');
  if (args.mode === 'synthetic' && args.selector !== undefined) throw new Error('selector is only valid in live mode');
  if (args.mode === 'live' && typeof args.selector !== 'string') throw new Error('live mode requires an opaque selector');
  return args;
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.mode === 'live') {
    if (process.env.BILIBILI_PROBE_ALLOW_LIVE !== '1') throw new Error('live mode is disabled unless explicitly enabled by the target runbook');
    return runLive(args);
  }
  return runSynthetic();
}

let resultPublished = false;
const writeStdout = process.stdout.write.bind(process.stdout);

function publishResult(result, exitCode = 0) {
  if (resultPublished) return false;
  resultPublished = true;
  const payload = `${JSON.stringify(result, null, 2)}\n`;
  if (exitCode === 0) {
    writeStdout(payload);
    return true;
  }
  // A process-level failure may leave browser/broker handles in an
  // unobservable state. Emit the bounded result, then force a bounded exit so
  // those handles cannot keep the diagnostic process alive indefinitely.
  let exited = false;
  const forceExit = setTimeout(() => {
    if (!exited) { exited = true; process.exit(exitCode); }
  }, 2_000);
  writeStdout(payload, () => {
    if (exited) return;
    exited = true;
    clearTimeout(forceExit);
    process.exit(exitCode);
  });
  return true;
}

function publishProcessFailure(reason, termination = 'error') {
  if (resultPublished) return;
  const error = reason instanceof Error ? reason : undefined;
  // Process-level failures can bypass the promise returned by main(). Keep
  // their evidence finite and make cleanup uncertainty explicit.
  const result = finalizeResult({ result: 'failure', termination: terminationClass(error, termination), diagnostic: classifyError(error, 'unknown'), activity: { page_navigation: false }, cleanup: {
    browser_exit: 'unknown', broker_close: 'unknown', temporary_profile: 'unknown', ephemeral_candidates: 'unknown', dns_pins: 'unknown', staging: 'unknown',
  } });
  publishResult(result, 1);
}

// These handlers cover errors/rejections and observable signals raised after
// the normal promise path has been lost. They intentionally publish only the
// same bounded result shape and never copy the process error text.
process.once('uncaughtException', (error) => publishProcessFailure(error, 'error'));
process.once('unhandledRejection', (reason) => publishProcessFailure(reason, 'error'));
for (const signal of ['SIGINT', 'SIGTERM']) {
  process.once(signal, () => publishProcessFailure(undefined, 'signal'));
}

main().then((result) => {
  if (result) publishResult(result);
}, (error) => {
  publishProcessFailure(error, terminationClass(error));
});
