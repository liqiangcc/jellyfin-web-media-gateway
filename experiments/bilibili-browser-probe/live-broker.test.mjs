import test from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import net from 'node:net';
import { PassThrough, Readable } from 'node:stream';
import { once } from 'node:events';
import {
  brokerServer, consumeResponseBody, createDisposableProfile, removeDisposableProfile, LIVE_BROWSER_ARGS,
} from './live.mjs';

async function listen(server) {
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  return server.address().port;
}

function fakeRequest(options, callback) {
  const upstream = new PassThrough();
  process.nextTick(() => {
    const pathname = options.path.split('?', 1)[0];
    const reply = Readable.from([Buffer.from(pathname === '/redirect' ? '' : pathname === '/error' ? 'error' : 'allowed')]);
    reply.statusCode = pathname === '/redirect' ? 302 : pathname === '/error' ? 503 : 200;
    reply.headers = { 'content-type': 'video/mp4', ...(pathname === '/redirect' ? { location: 'https://private.bilibili.com/final' } : {}) };
    callback(reply);
  });
  return upstream;
}

function proxyGet(port, target) {
  return new Promise((resolve, reject) => {
    const req = http.request({ host: '127.0.0.1', port, path: target, method: 'GET' }, (res) => {
      const chunks = [];
      res.on('data', (chunk) => chunks.push(chunk));
      res.on('end', () => resolve({ status: res.statusCode, body: Buffer.concat(chunks).toString() }));
    });
    req.on('error', reject); req.end();
  });
}

test('the live broker enforces host, DNS, redirect and upgrade policy through one seam', async (t) => {
  const state = { requests: 0, responseBytes: 0, responseLimit: 1024, denied: [], pins: new Map() };
  const broker = brokerServer(state, {
    request: fakeRequest,
    resolveAddress: async (host) => { if (host === 'private.bilibili.com') throw new Error('non-public'); return '127.0.0.1'; },
  });
  const port = await listen(broker);
  t.after(() => broker.close());

  assert.deepEqual(await proxyGet(port, 'https://www.bilibili.com/media'), { status: 200, body: '/media' });
  assert.equal((await proxyGet(port, 'https://private.bilibili.com/media')).status, 403);
  assert.equal((await proxyGet(port, 'https://www.bilibili.com/redirect')).status, 302);
  assert.equal((await proxyGet(port, 'https://private.bilibili.com/final')).status, 403);
  assert.equal((await proxyGet(port, 'http://www.bilibili.com/plain')).status, 403);

  const socket = net.connect(port, '127.0.0.1');
  await once(socket, 'connect');
  socket.write('GET / HTTP/1.1\r\nHost: www.bilibili.com\r\nConnection: Upgrade\r\nUpgrade: websocket\r\n\r\n');
  const upgrade = await once(socket, 'data');
  assert.match(upgrade[0].toString(), /403 Forbidden/);
  socket.destroy();
  assert.ok(state.denied.some((entry) => entry.kind === 'upgrade'));
});

test('CONNECT uses the same broker byte budget and denies private DNS', async (t) => {
  const remote = net.createServer((socket) => { socket.end(Buffer.alloc(32, 7)); });
  const remotePort = await listen(remote);
  t.after(() => remote.close());
  const state = { requests: 0, responseBytes: 0, responseLimit: 8, denied: [], pins: new Map() };
  const broker = brokerServer(state, {
    resolveAddress: async (host) => { if (host === 'private.bilibili.com') throw new Error('non-public'); return '127.0.0.1'; },
    connect: () => { const socket = net.connect(remotePort, '127.0.0.1'); socket.once('connect', () => socket.emit('secureConnect')); return socket; },
  });
  const port = await listen(broker);
  t.after(() => broker.close());
  const client = net.connect(port, '127.0.0.1');
  await once(client, 'connect');
  client.write('CONNECT www.bilibili.com:443 HTTP/1.1\r\nHost: www.bilibili.com:443\r\n\r\n');
  await once(client, 'close').catch(() => {});
  assert.ok(state.responseBytes > state.responseLimit);
  assert.equal((await proxyGet(port, 'https://private.bilibili.com/final')).status, 403);
  client.destroy();
});

test('independent body reader counts success/error/over-budget bytes and cancellation', async () => {
  const success = Readable.from([Buffer.from('ok')]);
  success.statusCode = 200; success.headers = { 'content-type': 'video/mp4' };
  assert.deepEqual(await consumeResponseBody(success, 8), { status_class: '2xx', bytes: 2, content_type: 'video/mp4' });
  const error = Readable.from([Buffer.from('error')]);
  error.statusCode = 503; error.headers = { 'content-type': 'text/plain' };
  assert.deepEqual(await consumeResponseBody(error, 8), { status_class: '5xx', bytes: 5, content_type: 'text/plain' });
  const over = Readable.from([Buffer.alloc(9)]);
  over.statusCode = 200; over.headers = {};
  await assert.rejects(consumeResponseBody(over, 8), /budget/);
  const controller = new AbortController();
  const never = new PassThrough(); never.statusCode = 200; never.headers = {};
  const cancelled = consumeResponseBody(never, 8, controller.signal);
  controller.abort();
  await assert.rejects(cancelled, /cancelled/);
});

test('profile lifecycle and browser transport restrictions are explicit', async () => {
  const profile = await createDisposableProfile();
  await removeDisposableProfile(profile);
  assert.deepEqual(LIVE_BROWSER_ARGS, ['--disable-quic', '--disable-features=WebTransport', '--disable-background-networking']);
});
