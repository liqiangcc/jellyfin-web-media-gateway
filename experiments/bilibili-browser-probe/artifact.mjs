#!/usr/bin/env node
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const files = [
  'experiments/bilibili-browser-probe/probe.mjs',
  'experiments/bilibili-browser-probe/contract.test.mjs',
  'experiments/bilibili-browser-probe/artifact.mjs',
  'experiments/bilibili-browser-probe/consumer.mjs',
  'plugins/bilibili/experimental_probe.mjs',
  'plugins/bilibili/package.json',
  'docs/research/bilibili-browser-probe-runbook.md',
];
const sha256 = async (file) => crypto.createHash('sha256').update(await fs.readFile(file)).digest('hex');

async function build(output) {
  const candidate = process.env.CANDIDATE_SHA || 'uncommitted';
  if (candidate !== 'uncommitted' && !/^[0-9a-f]{40}$/.test(candidate)) throw new Error('CANDIDATE_SHA must be exact 40-hex SHA');
  await fs.rm(output, { recursive: true, force: true });
  await fs.mkdir(output, { recursive: true });
  const entries = [];
  for (const relative of files) {
    const source = path.join(root, relative);
    const destination = path.join(output, relative);
    await fs.mkdir(path.dirname(destination), { recursive: true });
    await fs.copyFile(source, destination);
    const stat = await fs.stat(destination);
    entries.push({ path: relative, size: stat.size, sha256: await sha256(destination) });
  }
  const manifest = {
    schema_version: 1,
    artifact: 'bilibili-browser-probe',
    candidate_sha: candidate,
    platform: 'portable-node-chromium-hosted-x64',
    entry: 'experiments/bilibili-browser-probe/probe.mjs',
    runtime: { node: process.version, browser: process.env.BROWSER_VERSION || 'discovered-by-workflow', playwright_core: '1.55.0' },
    files: entries,
    limits: { navigation_seconds: 120, requests_per_session: 200, response_bytes_per_session: 33554432, metadata_bytes: 1048576, independent_requests: 8, independent_bytes: 4194304, per_request_bytes: 1048576 },
  };
  await fs.writeFile(path.join(output, 'manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`);
  return manifest;
}

async function verify(directory) {
  const manifest = JSON.parse(await fs.readFile(path.join(directory, 'manifest.json'), 'utf8'));
  if (manifest.schema_version !== 1 || manifest.artifact !== 'bilibili-browser-probe') throw new Error('manifest schema rejected');
  for (const item of manifest.files) {
    const file = path.join(directory, item.path);
    const stat = await fs.stat(file);
    if (stat.size !== item.size || await sha256(file) !== item.sha256) throw new Error(`digest mismatch: ${item.path}`);
  }
  return manifest;
}

const command = process.argv[2];
const target = path.resolve(process.argv[3] || 'probe-artifact');
if (command === 'build') console.log(JSON.stringify(await build(target), null, 2));
else if (command === 'verify') console.log(JSON.stringify(await verify(target), null, 2));
else throw new Error('usage: artifact.mjs build|verify <directory>');
