import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { build, verify, dependency } from './artifact.mjs';

const runtimeRoot = process.env.PLAYWRIGHT_RUNTIME_ROOT;

test('builds and verifies a clean self-contained runtime inventory', async (t) => {
  if (!runtimeRoot) return t.skip('hosted runtime staging is required');
  const temp = await fs.mkdtemp(path.join(os.tmpdir(), 'bilibili-artifact-test-'));
  t.after(() => fs.rm(temp, { recursive: true, force: true }));
  const bundle = path.join(temp, 'bundle');
  const manifest = await build(bundle);
  assert.equal(manifest.dependency.name, dependency.name);
  assert.equal(manifest.dependency.version, dependency.version);
  await verify(bundle);
  const { createRequire } = await import('node:module');
  const resolved = createRequire(import.meta.url).resolve('playwright-core', { paths: [bundle] });
  assert.ok(resolved.startsWith(`${bundle}${path.sep}`));
});

test('rejects missing, digest-mismatched, version-drifted and symlinked dependency files', async (t) => {
  if (!runtimeRoot) return t.skip('hosted runtime staging is required');
  const temp = await fs.mkdtemp(path.join(os.tmpdir(), 'bilibili-artifact-negative-'));
  t.after(() => fs.rm(temp, { recursive: true, force: true }));
  const bundle = path.join(temp, 'bundle');
  await build(bundle);
  const manifestPath = path.join(bundle, 'manifest.json');
  const entry = path.join(bundle, dependency.package_path, 'package.json');
  const original = await fs.readFile(entry);
  await fs.writeFile(entry, Buffer.from(`${original}x`));
  await assert.rejects(verify(bundle), /digest mismatch/);
  await fs.writeFile(entry, original);
  const originalManifest = await fs.readFile(manifestPath);
  const manifest = JSON.parse(originalManifest);
  manifest.dependency.files[0].size += 1;
  await fs.writeFile(manifestPath, `${JSON.stringify(manifest)}\n`);
  await assert.rejects(verify(bundle), /digest mismatch|dependency provenance/);
  await fs.writeFile(manifestPath, originalManifest);
  await fs.rm(entry);
  await fs.symlink('/tmp/outside-playwright-package.json', entry);
  await assert.rejects(verify(bundle), /symlinked|missing/);
});
