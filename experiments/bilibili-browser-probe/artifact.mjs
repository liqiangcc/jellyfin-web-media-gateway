#!/usr/bin/env node
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const dependency = {
  name: 'playwright-core', version: '1.55.0', package_path: 'node_modules/playwright-core',
  resolved: 'https://registry.npmjs.org/playwright-core/-/playwright-core-1.55.0.tgz',
  integrity: 'sha512-GvZs4vU3U5ro2nZpeiwyb0zuFaqb9sUiAJuyrWpcGouD8y9/HLgGbNRjIph7zU9D3hnPaisMl9zG9CgFi/biIg==',
};
const staticFiles = [
  'experiments/bilibili-browser-probe/probe.mjs', 'experiments/bilibili-browser-probe/live.mjs',
  'experiments/bilibili-browser-probe/diagnostic.mjs',
  'experiments/bilibili-browser-probe/artifact.mjs', 'experiments/bilibili-browser-probe/consumer.mjs',
  'experiments/bilibili-browser-probe/package.json', 'experiments/bilibili-browser-probe/package-lock.json',
  'plugins/bilibili/experimental_probe.mjs',
  'plugins/bilibili/live_selector.mjs', 'plugins/bilibili/package.json', 'docs/research/bilibili-browser-probe-runbook.md',
];
const packageTopLevel = new Set(['LICENSE', 'NOTICE', 'README.md', 'ThirdPartyNotices.txt', 'browsers.json', 'cli.js', 'index.d.ts', 'index.js', 'index.mjs', 'lib', 'package.json', 'types']);
// Playwright's bin/ tree contains browser installer helpers; the target uses
// external system Chrome, so none of those scripts belong in the runtime.
const excludedPackagePrefixes = ['bin/'];
const sha256 = async (file) => crypto.createHash('sha256').update(await fs.readFile(file)).digest('hex');
const relativeSafe = (value) => typeof value === 'string' && value.length > 0 && !path.isAbsolute(value) && !value.split('/').includes('..');

async function walk(directory) {
  const result = [];
  const visit = async (current) => {
    for (const name of (await fs.readdir(current)).sort()) {
      const file = path.join(current, name); const stat = await fs.lstat(file);
      if (stat.isSymbolicLink()) throw new Error(`symlink rejected: ${file}`);
      if (stat.isDirectory()) await visit(file);
      else if (stat.isFile()) result.push(file);
      else throw new Error(`non-regular runtime entry rejected: ${file}`);
    }
  };
  await visit(directory); return result;
}

async function copyRegular(source, destination) {
  const stat = await fs.lstat(source);
  if (!stat.isFile()) throw new Error(`source must be a regular file: ${source}`);
  await fs.mkdir(path.dirname(destination), { recursive: true }); await fs.copyFile(source, destination);
}

async function runtimeEntries(runtimeRoot, output) {
  const packageRoot = path.join(runtimeRoot, 'node_modules', dependency.name);
  const packageStat = await fs.lstat(packageRoot).catch(() => null);
  if (!packageStat?.isDirectory() || packageStat.isSymbolicLink()) throw new Error('pinned playwright-core package path is missing or symlinked');
  const packageJson = JSON.parse(await fs.readFile(path.join(packageRoot, 'package.json'), 'utf8'));
  if (packageJson.name !== dependency.name || packageJson.version !== dependency.version) throw new Error('playwright-core package version/name drift');
  const files = await walk(packageRoot);
  for (const file of files) {
    const relative = path.relative(packageRoot, file).split(path.sep).join('/');
    if (excludedPackagePrefixes.some((prefix) => relative.startsWith(prefix))) continue;
    if (!packageTopLevel.has(relative.split('/')[0])) throw new Error(`unexpected playwright-core package entry: ${relative}`);
    await copyRegular(file, path.join(output, dependency.package_path, relative));
  }
  return Promise.all((await walk(path.join(output, dependency.package_path))).map(async (file) => ({
    path: path.relative(output, file).split(path.sep).join('/'), size: (await fs.stat(file)).size, sha256: await sha256(file),
  })));
}

async function build(output) {
  const candidate = process.env.CANDIDATE_SHA || 'uncommitted';
  if (candidate !== 'uncommitted' && !/^[0-9a-f]{40}$/.test(candidate)) throw new Error('CANDIDATE_SHA must be exact 40-hex SHA');
  const runtimeRoot = process.env.PLAYWRIGHT_RUNTIME_ROOT;
  if (!runtimeRoot) throw new Error('PLAYWRIGHT_RUNTIME_ROOT is required; install the frozen package in hosted Actions first');
  await fs.rm(output, { recursive: true, force: true }); await fs.mkdir(output, { recursive: true });
  const entries = [];
  for (const relative of staticFiles) {
    const destination = path.join(output, relative); await copyRegular(path.join(root, relative), destination);
    entries.push({ path: relative, size: (await fs.stat(destination)).size, sha256: await sha256(destination) });
  }
  entries.push(...await runtimeEntries(path.resolve(runtimeRoot), output)); entries.sort((a, b) => a.path.localeCompare(b.path));
  const manifest = {
    schema_version: 2, artifact: 'bilibili-browser-probe', candidate_sha: candidate,
    platform: 'portable-node-chromium-hosted-x64', entry: 'experiments/bilibili-browser-probe/probe.mjs',
    runtime: { node: process.version, browser: process.env.BROWSER_VERSION || 'external-system-browser-required', browser_external: true },
    dependency: { ...dependency, files: entries.filter((item) => item.path.startsWith(`${dependency.package_path}/`)) }, files: entries,
    limits: { navigation_seconds: 120, requests_per_session: 200, response_bytes_per_session: 33554432, metadata_bytes: 1048576, independent_requests: 8, independent_bytes: 4194304, per_request_bytes: 1048576 },
  };
  await fs.writeFile(path.join(output, 'manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`); return manifest;
}

async function verify(directory) {
  const base = path.resolve(directory); const manifest = JSON.parse(await fs.readFile(path.join(base, 'manifest.json'), 'utf8'));
  if (manifest.schema_version !== 2 || manifest.artifact !== 'bilibili-browser-probe') throw new Error('manifest schema rejected');
  if (process.env.CANDIDATE_SHA && manifest.candidate_sha !== process.env.CANDIDATE_SHA) throw new Error('artifact candidate mismatch');
  if (!/^[0-9a-f]{40}$/.test(manifest.candidate_sha) && manifest.candidate_sha !== 'uncommitted') throw new Error('manifest candidate rejected');
  if (manifest.runtime?.browser_external !== true) throw new Error('manifest requires an external browser');
  const provenance = { ...manifest.dependency, files: undefined }; const expectedProvenance = { ...dependency, files: undefined };
  if (JSON.stringify(provenance) !== JSON.stringify(expectedProvenance)) throw new Error('dependency provenance mismatch');
  if (!Array.isArray(manifest.files) || !Array.isArray(manifest.dependency.files)) throw new Error('manifest file inventory missing');
  const listed = new Map(manifest.files.map((item) => [item.path, item]));
  if (listed.size !== manifest.files.length) throw new Error('duplicate manifest path');
  const packagePrefix = `${dependency.package_path}/`;
  if (manifest.files.filter((item) => item.path.startsWith(packagePrefix)).length !== manifest.dependency.files.length) throw new Error('dependency inventory mismatch');
  const allFiles = await walk(base);
  for (const item of manifest.files) {
    if (!relativeSafe(item.path) || !Number.isSafeInteger(item.size) || !/^[0-9a-f]{64}$/.test(item.sha256)) throw new Error(`invalid manifest file entry: ${item.path}`);
    const file = path.resolve(base, item.path); if (!file.startsWith(`${base}${path.sep}`)) throw new Error(`manifest path outside artifact: ${item.path}`);
    const stat = await fs.lstat(file).catch(() => null);
    if (!stat?.isFile() || stat.isSymbolicLink()) throw new Error(`manifest file is missing or symlinked: ${item.path}`);
    if (stat.size !== item.size || await sha256(file) !== item.sha256) throw new Error(`digest mismatch: ${item.path}`);
  }
  const actual = allFiles.map((file) => path.relative(base, file).split(path.sep).join('/')).filter((file) => file !== 'manifest.json').sort();
  if (JSON.stringify(actual) !== JSON.stringify([...listed.keys()].sort())) throw new Error('artifact contains unmanifested files');
  const packageJson = JSON.parse(await fs.readFile(path.join(base, dependency.package_path, 'package.json'), 'utf8'));
  if (packageJson.name !== dependency.name || packageJson.version !== dependency.version) throw new Error('packaged dependency version mismatch');
  if (!manifest.dependency.files.every((item) => listed.get(item.path)?.sha256 === item.sha256 && listed.get(item.path)?.size === item.size)) throw new Error('dependency provenance inventory mismatch');
  return manifest;
}

export { build, verify, dependency };
if (import.meta.url === pathToFileURL(process.argv[1] || '').href) {
  const command = process.argv[2]; const target = path.resolve(process.argv[3] || 'probe-artifact');
  if (command === 'build') console.log(JSON.stringify(await build(target), null, 2));
  else if (command === 'verify') console.log(JSON.stringify(await verify(target), null, 2));
  else throw new Error('usage: artifact.mjs build|verify <directory>');
}
