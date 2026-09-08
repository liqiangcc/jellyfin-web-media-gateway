#!/usr/bin/env node
/* Compile-free consumer: verifies a downloaded artifact and reads media after browser exit. */
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { spawn } from 'node:child_process';

const here = path.dirname(fileURLToPath(import.meta.url));
const artifact = path.resolve(process.argv[2] || here);
const { verify } = await import(pathToFileURL(path.join(artifact, 'experiments/bilibili-browser-probe/artifact.mjs')));
const manifest = await verify(artifact);
const browserPath = process.env.CHROME_PATH;
if (!browserPath || !path.isAbsolute(browserPath)) throw new Error('consumer requires an explicit external CHROME_PATH');
if (path.resolve(browserPath).startsWith(`${artifact}${path.sep}`)) throw new Error('browser must be external to the artifact');
if (!(await fs.stat(browserPath)).isFile()) throw new Error('external browser executable is missing');
const { createRequire } = await import('node:module');
const resolvedRuntime = createRequire(import.meta.url).resolve('playwright-core', { paths: [artifact] });
if (!resolvedRuntime.startsWith(`${artifact}${path.sep}`)) throw new Error('playwright-core resolved outside downloaded artifact');
const result = await new Promise((resolve, reject) => {
  const child = spawn(process.execPath, [path.join(artifact, manifest.entry)], { cwd: artifact, env: { ...process.env, CHROME_PATH: browserPath, PROBE_ARTIFACT_CONSUMER: '1' }, stdio: ['ignore', 'pipe', 'pipe'] });
  let stdout = ''; let stderr = '';
  child.stdout.on('data', (data) => { stdout += data; }); child.stderr.on('data', (data) => { stderr += data; });
  child.on('error', reject); child.on('close', (code) => code === 0 ? resolve({ stdout, stderr }) : reject(new Error(stderr || `probe exit ${code}`)));
});
const evidence = JSON.parse(result.stdout);
if (evidence.observation.independent_consumer?.status_class !== '2xx') throw new Error('browser-exit independent consumer did not read media');
const independentResults = evidence.observation.independent_consumer?.results || [];
if (!independentResults.some((item) => item.role === 'muxed' && item.status_class === '2xx')) throw new Error('muxed fixture was not independently readable');
if (!independentResults.some((item) => item.role === 'video' && item.status_class === '2xx') || !independentResults.some((item) => item.role === 'audio' && item.status_class === '2xx')) throw new Error('AV-separated fixtures were not independently readable');
if (!['loopback-allowlist', 'public-host-pinned'].includes(evidence.observation.containment?.broker)) throw new Error('containment evidence missing');
console.log(JSON.stringify({ result: 'PASS', candidate_sha: manifest.candidate_sha, browser_exit: true, independent_consumer: evidence.observation.independent_consumer, request_count: evidence.observation.budget.request_count, denied_exits: evidence.broker_requests?.filter((item) => !item.allowed).length ?? evidence.denied_count ?? 0 }, null, 2));
