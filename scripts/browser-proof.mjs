import fs from 'node:fs';
import { chromium } from 'playwright-core';

const base = process.env.BASE_URL || 'http://127.0.0.1:8787';
const chrome = process.env.CHROME_PATH || '/usr/bin/google-chrome';
const evidence = { base, public: {}, secret: {}, requests: [], console: [], failures: [] };

const browser = await chromium.launch({
  executablePath: chrome,
  headless: true,
  args: ['--autoplay-policy=no-user-gesture-required', '--no-sandbox'],
});
const page = await browser.newPage();
page.on('console', msg => evidence.console.push({ type: msg.type(), text: msg.text() }));
page.on('requestfailed', req => evidence.failures.push({ url: req.url(), failure: req.failure() }));
page.on('request', req => {
  if (req.url().includes('/stream/')) {
    const headers = req.headers();
    evidence.requests.push({ url: req.url(), range: headers.range || null, authorization: headers.authorization || null, cookie: headers.cookie || null });
  }
});

async function waitReady() {
  await page.waitForFunction(() => {
    const v = document.querySelector('#player');
    return v && Number.isFinite(v.duration) && v.duration > 0 && v.readyState >= 2;
  }, null, { timeout: 45000 });
}

await page.goto(`${base}/display`, { waitUntil: 'domcontentloaded', timeout: 45000 });
await waitReady();
const meta = await page.$eval('#player', v => ({ duration: v.duration, readyState: v.readyState, currentTime: v.currentTime }));
await page.$eval('#player', v => v.play());
await page.waitForFunction(() => document.querySelector('#player').currentTime > 0.5, null, { timeout: 20000 });
await page.$eval('#player', v => v.pause());
const pausedAt = await page.$eval('#player', v => v.currentTime);
await page.waitForTimeout(750);
const pausedAfter = await page.$eval('#player', v => v.currentTime);
if (Math.abs(pausedAfter - pausedAt) > 0.25) throw new Error(`pause did not hold: ${pausedAt} -> ${pausedAfter}`);
const target = Math.min(Math.max(meta.duration * 0.55, 1.5), meta.duration - 0.5);
await page.$eval('#player', (v, t) => { v.currentTime = t; }, target);
await page.waitForFunction(t => Math.abs(document.querySelector('#player').currentTime - t) < 0.9, target, { timeout: 15000 });
const seeked = await page.$eval('#player', v => v.currentTime);
evidence.public = { ...meta, pausedAt, pausedAfter, seekTarget: target, seeked };

const publicRequests = evidence.requests.filter(r => r.url.includes('/stream/'));
if (publicRequests.length === 0) throw new Error('no Gateway media request observed');
if (!publicRequests.some(r => r.range)) throw new Error('browser proof observed no Range request');
if (publicRequests.some(r => r.authorization || r.cookie)) throw new Error('browser sent secret headers on public Gateway media request');

const beforeSecret = evidence.requests.length;
await page.goto(`${base}/secret-display`, { waitUntil: 'domcontentloaded', timeout: 30000 });
await waitReady();
await page.$eval('#player', v => v.play());
await page.waitForFunction(() => document.querySelector('#player').currentTime > 0.3, null, { timeout: 10000 });
await page.$eval('#player', v => { v.pause(); v.removeAttribute('src'); v.load(); });
const secretRequests = evidence.requests.slice(beforeSecret);
if (secretRequests.length === 0) throw new Error('no protected fixture Gateway request observed');
if (secretRequests.some(r => r.authorization || r.cookie)) throw new Error('fixture secret crossed into browser-visible request');
if (JSON.stringify(secretRequests).includes('r001-fixture-secret')) throw new Error('fixture secret appeared in browser evidence');
evidence.secret = { requestCount: secretRequests.length, noAuthorizationOrCookie: true };

await page.waitForTimeout(500);
const metrics = await (await page.request.get(`${base}/metrics`)).json();
if (metrics.active_streams !== 0) throw new Error(`active streams did not return to zero: ${JSON.stringify(metrics)}`);
evidence.metrics = metrics;

fs.writeFileSync('browser-proof.json', JSON.stringify(evidence, null, 2));
await browser.close();
console.log(JSON.stringify({ public: evidence.public, secret: evidence.secret, metrics }, null, 2));
