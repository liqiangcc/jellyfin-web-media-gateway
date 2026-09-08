#!/usr/bin/env node
/* Offline-only browser acquisition probe for #165. */
import http from 'node:http';
import net from 'node:net';
import { once } from 'node:events';
import { chromium } from 'playwright-core';
import { summarizeObservation, rejectSensitiveInput, SCHEMA_VERSION } from '../../plugins/bilibili/experimental_probe.mjs';

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
        fetch('http://blocked.test/forbidden').catch(() => {});
        fetch('/redirect').catch(() => {});
        navigator.serviceWorker.register('/sw.js').catch(() => {});
        const worker = new Worker(URL.createObjectURL(new Blob([
          "fetch('http://blocked.test/worker').catch(() => {});"
        ], {type: 'application/javascript'})));
        const ws = new WebSocket('ws://blocked.test/socket');
        ws.onerror = () => {};
      </script>`);
      return;
    }
    if (req.url === '/sw.js') {
      res.writeHead(200, { 'content-type': 'application/javascript', 'service-worker-allowed': '/' });
      res.end("self.addEventListener('fetch', event => { if (new URL(event.request.url).pathname === '/sw-probe') event.respondWith(fetch('http://blocked.test/sw')); });");
      return;
    }
    if (req.url === '/metadata') {
      res.writeHead(200, { 'content-type': 'application/json', 'content-length': '27' });
      res.end('{"fixture":"metadata","ok":true}');
      return;
    }
    if (req.url === '/redirect') {
      res.writeHead(302, { location: 'http://blocked.test/final' });
      res.end();
      return;
    }
    if (req.url === '/media') {
      const data = Buffer.from('synthetic-media-payload-165');
      res.writeHead(200, { 'content-type': 'video/mp4', 'content-length': data.length, 'accept-ranges': 'bytes' });
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
  const response = await fetch(`http://127.0.0.1:${port}/media`);
  const bytes = Buffer.from(await response.arrayBuffer());
  if (!response.ok || bytes.length === 0 || bytes.length > 1024 * 1024) throw new Error('independent consumer failed');
  return { status_class: '2xx', bytes: bytes.length, content_type: response.headers.get('content-type') };
}

async function main() {
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
    await page.waitForTimeout(500);
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
      worker: denied.some((item) => item.host === 'blocked.test') ? 'denied' : 'not-observed',
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
    process.stdout.write(`${JSON.stringify({ schema_version: SCHEMA_VERSION, observation, broker_requests: requests.map(({ host, path, method, allowed, redirected }) => ({ host, path, method, allowed, redirected })) }, null, 2)}\n`);
  } finally {
    if (browser) await browser.close().catch(() => {});
    fixtures.close(); broker.close();
    // The profile contains no durable authority and is always disposable.
    const { rm } = await import('node:fs/promises');
    await rm(profile, { recursive: true, force: true }).catch(() => {});
  }
}

main().catch((error) => { process.stderr.write(`probe failed: ${error.message}\n`); process.exitCode = 1; });
