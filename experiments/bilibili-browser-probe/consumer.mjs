#!/usr/bin/env node
/* Compile-free consumer: verifies a downloaded artifact and reads media after browser exit. */
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';

const here = path.dirname(fileURLToPath(import.meta.url));
const artifact = path.resolve(process.argv[2] || here);
const manifest = JSON.parse(await fs.readFile(path.join(artifact, 'manifest.json'), 'utf8'));
if (manifest.candidate_sha !== process.env.CANDIDATE_SHA) throw new Error('artifact candidate mismatch');
const result = await new Promise((resolve, reject) => {
  const child = spawn(process.execPath, [path.join(artifact, manifest.entry)], { cwd: artifact, env: { ...process.env, PROBE_ARTIFACT_CONSUMER: '1' }, stdio: ['ignore', 'pipe', 'pipe'] });
  let stdout = ''; let stderr = '';
  child.stdout.on('data', (data) => { stdout += data; }); child.stderr.on('data', (data) => { stderr += data; });
  child.on('error', reject); child.on('close', (code) => code === 0 ? resolve({ stdout, stderr }) : reject(new Error(stderr || `probe exit ${code}`)));
});
const evidence = JSON.parse(result.stdout);
if (evidence.observation.independent_consumer?.status_class !== '2xx') throw new Error('browser-exit independent consumer did not read media');
if (evidence.observation.containment?.broker !== 'loopback-allowlist') throw new Error('containment evidence missing');
console.log(JSON.stringify({ result: 'PASS', candidate_sha: manifest.candidate_sha, browser_exit: true, independent_consumer: evidence.observation.independent_consumer, request_count: evidence.observation.budget.request_count, denied_exits: evidence.broker_requests.filter((item) => !item.allowed).length }, null, 2));
