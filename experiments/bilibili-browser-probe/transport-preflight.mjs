#!/usr/bin/env node
/*
 * One-shot, no-page transport admission for the already-approved target.
 * The authority is owned by the Bilibili plugin; this entry point accepts no
 * URL, host, proxy, profile, header, selector, or credential arguments.
 */
import http from 'node:http';
import { once } from 'node:events';
import { pathToFileURL } from 'node:url';
import { brokerServer } from './live.mjs';
import { classifyTransport, diagnosticCounters } from './diagnostic.mjs';
import { PREFLIGHT_AUTHORITY } from '../../plugins/bilibili/live_selector.mjs';

const PREFLIGHT_TIMEOUT_MS = 12000;

function listen(server) {
  server.listen(0, '127.0.0.1');
  return once(server, 'listening').then(() => server.address().port);
}

function closeServer(server) {
  server.closeAllConnections?.();
  return new Promise((resolve) => server.close(() => resolve()));
}

/**
 * Establishes one broker CONNECT and immediately closes it. No page or media
 * request is sent. The result is limited to the diagnostic schema and cleanup
 * state, making this safe to persist as target evidence.
 */
export async function runTransportPreflight() {
  const state = { requests: 0, responseBytes: 0, metadataBytes: 0, denied: [], pins: new Map() };
  const broker = brokerServer(state);
  let port;
  let request;
  let timer;
  let output;
  try {
    port = await listen(broker);
    const result = await new Promise((resolve) => {
      let settled = false;
      const finish = () => {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        resolve();
      };
      timer = setTimeout(finish, PREFLIGHT_TIMEOUT_MS);
      request = http.request({
        host: '127.0.0.1', port, method: 'CONNECT',
        path: `${PREFLIGHT_AUTHORITY.host}:${PREFLIGHT_AUTHORITY.port}`,
        timeout: PREFLIGHT_TIMEOUT_MS,
      });
      request.once('connect', (response, socket) => {
        // Receiving the broker's 200 is the only response observation. The
        // socket is closed before any tunneled HTTP bytes can be sent.
        if (response.statusCode !== 200) finish();
        socket.once('close', finish);
        socket.destroy();
        finish();
      });
      request.once('timeout', () => request.destroy());
      request.once('error', finish);
      request.end();
    });
    void result;
    output = {
      schema_version: 2,
      diagnostic: state.failure || state.transport || classifyTransport({ stage: 'unknown', outcome: 'unknown', ...diagnosticCounters(state) }),
      activity: { page_navigation: false, media_request: false, selector: false, consumer: false },
      cleanup: { request_destroyed: Boolean(request?.destroyed), broker_closed: false, pins_cleared: false },
    };
  } finally {
    request?.destroy();
    await closeServer(broker).catch(() => {});
    state.pins.clear();
    state.denied.length = 0;
    output ||= {
      schema_version: 2,
      diagnostic: state.failure || state.transport || classifyTransport({ stage: 'unknown', outcome: 'unknown', ...diagnosticCounters(state) }),
      activity: { page_navigation: false, media_request: false, selector: false, consumer: false },
      cleanup: { request_destroyed: Boolean(request?.destroyed), broker_closed: false, pins_cleared: false },
    };
    output.cleanup = { request_destroyed: Boolean(request?.destroyed), broker_closed: true, pins_cleared: true };
  }
  return output;
}

if (import.meta.url === pathToFileURL(process.argv[1] || '').href) {
  const result = await runTransportPreflight();
  process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
}
