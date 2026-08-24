import fs from 'node:fs';
import { chromium } from 'playwright-core';

const base = process.env.BASE_URL || 'http://127.0.0.1:8787';
const chrome = process.env.CHROME_PATH || '/usr/bin/google-chrome';
const evidence = {
  base,
  browser: {},
  commands: [],
  playAttempts: [],
  fullscreen: [],
  lifecycle: [],
  transport: [],
  failures: [],
};

const browser = await chromium.launch({
  executablePath: chrome,
  headless: true,
  // No autoplay policy override is used. The first remote attempt must face
  // the browser's ordinary audible-playback policy.
  args: ['--no-sandbox'],
});
const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
page.on('requestfailed', request => evidence.failures.push({
  url: request.url().replace(/https?:\/\/[^/]+/, '[origin]'),
  failure: request.failure(),
}));

async function postCommand(requestId) {
  const response = await page.request.post(`${base}/api/v1/display-probe/commands`, {
    data: { request_id: requestId },
  });
  const body = await response.json();
  evidence.commands.push(body);
  if (!response.ok()) throw new Error(`remote command failed: ${response.status()}`);
  return body;
}

async function diagnostics() {
  const response = await page.request.get(`${base}/api/v1/display-probe/state?nonce=${Date.now()}`, { timeout: 10000 });
  if (!response.ok()) throw new Error(`diagnostics failed: ${response.status()}`);
  return response.json();
}

async function waitForState(predicate, description) {
  const deadline = Date.now() + 15000;
  while (Date.now() < deadline) {
    const state = await diagnostics();
    if (predicate(state)) return state;
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  throw new Error(`timed out waiting for ${description}`);
}

async function waitForPlayAttempt(commandId) {
  return waitForState(
    state => state.telemetry.some(item => item.kind === 'play_attempt' && item.command_id === commandId),
    `play attempt for ${commandId}`,
  );
}

await page.goto(`${base}/display`, { waitUntil: 'domcontentloaded', timeout: 30000 });
evidence.browser = await page.evaluate(() => ({
  userAgent: navigator.userAgent,
  viewport: { width: innerWidth, height: innerHeight },
  immersiveClass: document.querySelector('#display-shell')?.className,
  fullscreenAvailable: Boolean(document.documentElement.requestFullscreen),
}));

// Remote attempt before any page interaction: result may resolve or reject,
// but both outcomes must be recorded rather than hidden.
await postCommand('r002-remote-before-activation');
await waitForPlayAttempt('r002-remote-before-activation');

// Repeat the same request_id to prove command idempotency does not create a
// second browser event or a second play() attempt.
const duplicate = await postCommand('r002-remote-before-activation');
if (!duplicate.duplicate) throw new Error('duplicate request_id was not reported as duplicate');

// A real Playwright click supplies normal user activation for the bootstrap
// path. This is probe mechanics evidence, not physical-TV acceptance.
await page.locator('#activate').click();
await waitForState(
  state => state.telemetry.some(item => item.kind === 'play_attempt' && item.detail === 'source=activation'),
  'activation play attempt',
);
await postCommand('r002-remote-after-activation');
await waitForPlayAttempt('r002-remote-after-activation');

await page.locator('#fullscreen').click();
if (evidence.commands.length !== 3) throw new Error(`expected 3 command responses, got ${evidence.commands.length}`);
if (evidence.commands.filter(command => command.duplicate).length !== 1) throw new Error('expected exactly one idempotent duplicate response');
await waitForState(
  state => state.telemetry.filter(item => item.kind === 'play_attempt').length >= 3,
  'three play attempts',
);
const finalState = await diagnostics();
evidence.playAttempts = finalState.telemetry.filter(item => item.kind === 'play_attempt');
evidence.fullscreen = finalState.telemetry.filter(item => item.kind === 'fullscreen');
evidence.lifecycle = finalState.telemetry.filter(item => ['lifecycle', 'visibility', 'media'].includes(item.kind));
evidence.transport = finalState.telemetry.filter(item => ['transport', 'network'].includes(item.kind));
fs.writeFileSync('r002-probe-debug.json', JSON.stringify({ telemetry: finalState.telemetry }, null, 2));
if (evidence.playAttempts.length !== 3) throw new Error(`expected activation + 2 remote play attempts, got ${evidence.playAttempts.length}`);
if (!evidence.playAttempts.every(attempt => ['resolve', 'reject'].includes(attempt.result))) throw new Error('play result telemetry was incomplete');
for (const commandId of ['r002-remote-before-activation', 'r002-remote-after-activation']) {
  if (evidence.playAttempts.filter(attempt => attempt.command_id === commandId).length !== 1) {
    throw new Error(`expected exactly one play attempt for ${commandId}`);
  }
}
if (evidence.fullscreen.length === 0) throw new Error('fullscreen result was not observable');
if (!evidence.lifecycle.some(item => item.result === 'ready' || item.result === 'pageshow')) throw new Error('lifecycle telemetry was not observed');
if (!evidence.transport.some(item => item.result === 'connected')) throw new Error('remote transport connection was not observed');
if (JSON.stringify(evidence).match(/(Bearer\s+|Cookie|r001-fixture-secret)/i)) throw new Error('secret-like material appeared in probe evidence');

fs.writeFileSync('r002-probe.json', JSON.stringify(evidence, null, 2));
await browser.close();
console.log(JSON.stringify({
  browser: evidence.browser,
  commands: evidence.commands.map(command => ({ sequence: command.command.sequence, duplicate: command.duplicate })),
  playAttempts: evidence.playAttempts.map(attempt => ({ command_id: attempt.command_id, result: attempt.result, error_name: attempt.error_name || null })),
  fullscreen: evidence.fullscreen.map(item => item.result),
  lifecycle: evidence.lifecycle.map(item => item.result),
  transport: evidence.transport.map(item => item.result),
}, null, 2));
