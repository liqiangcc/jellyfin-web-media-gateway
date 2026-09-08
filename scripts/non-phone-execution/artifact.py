#!/usr/bin/env python3
"""Compile-free, fail-closed admission for #146's GitHub-hosted runtime bundle.

The bootstrap script and expected.json come from the authenticated control plane,
not from the archive being admitted. No installer or compiler is invoked here.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import platform
import re
import shutil
import stat
import subprocess
import tempfile
import zipfile

RUNTIME = '80fb081b129f8f664124b84ddcc9698039e2cfd1'
ROOT = Path('/home/gateway-verify/non-phone/runtime-' + RUNTIME)
MAX_BYTES = 1024 * 1024 * 1024
MAX_FILES = 6000
REPO = 'liqiangcc/jellyfin-web-media-gateway'
EXECUTABLES = {'bin/generic-ytdlp-real-smoke', 'bin/ytdlp-sandbox', 'bin/runtime-tests'}
REQUIRED = EXECUTABLES | {'source/plugins/generic-ytdlp/worker/worker.py',
    'tools/generic-ytdlp-offline-runtime.py', 'tools/generic-ytdlp-offline-runtime.lock.json',
    'tools/artifact.py', 'cache/verified.json', 'bundle/manifest.json', 'bundle/SHA256SUMS'}
TESTS = ('pinned_worker_uses_actual_ytdlp_request_handler_and_existing_parser',
    'inherited_ipc_capability_supports_multiple_broker_requests',
    'seccomp_denies_worker_custom_handler_and_child_but_ipc_survives',
    'non_cloexec_ambient_fd_is_not_admitted_beyond_broker_fd',
    'diagnostics_consume_secret_sentinel_without_crossing_error_boundary',
    'r008_broker_rejects_secret_userinfo_and_private_targets_before_network')


class AdmissionError(Exception):
    pass


def require(ok):
    if not ok:
        raise AdmissionError('ARTIFACT_REJECTED')


def digest(data):
    return hashlib.sha256(data).hexdigest()


def file_hash(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def allowed(name):
    p = PurePosixPath(name)
    return (bool(name) and not p.is_absolute() and str(p) == name and '\\' not in name
            and not any(x in ('.', '..', '.git', '__pycache__') or x.startswith('.') for x in p.parts)
            and (name in REQUIRED or name.startswith('cache/site-packages/') or name.startswith('bundle/artifacts/')))


def owned_root_parent():
    require(os.geteuid() != 0)
    home = Path('/home/gateway-verify')
    require(home.is_dir() and not home.is_symlink() and home.stat().st_uid == os.getuid())
    parent = home / 'non-phone'
    if not parent.exists():
        parent.mkdir(mode=0o700)
    require(not parent.is_symlink() and parent.is_dir() and parent.stat().st_uid == os.getuid())
    require(parent.stat().st_mode & 0o077 == 0)
    return parent


def context(expected):
    require(expected['repository'] == REPO and expected['runtime'] == RUNTIME)
    require(re.fullmatch('[0-9a-f]{40}', expected['candidate']) is not None)
    require(re.fullmatch('sha256:[0-9a-f]{64}', expected['digest']) is not None)
    require(isinstance(expected['artifact_id'], int) and expected['artifact_id'] > 0)
    require(isinstance(expected['run_id'], int) and expected['run_id'] > 0)
    require(expected['run_attempt'] >= 1 and expected['job_id'] > 0)
    require(re.fullmatch('[0-9a-f]{64}', expected['manifest_sha256']) is not None)
    require(expected['workflow_path'] == '.github/workflows/non-phone-execution.yml')
    require(platform.machine() == 'x86_64' and platform.libc_ver()[0] == 'glibc')
    require(tuple(map(int, platform.libc_ver()[1].split('.'))) >= (2, 39))


def manifest_check(m, expected):
    require(m['schema'] == 1 and m['candidate'] == expected['candidate']
            and m['workflow_sha'] == expected['candidate'] and m['runtime'] == RUNTIME
            and m['source_root'] == str(ROOT / 'source') and m['target'] == 'x86_64-unknown-linux-gnu'
            and m['run_id'] == expected['run_id'] and m['run_attempt'] == expected['run_attempt'])
    files = m['files']
    require(isinstance(files, dict) and REQUIRED <= set(files) and len(files) <= MAX_FILES)
    require(sum(v['size'] for v in files.values()) <= MAX_BYTES)
    for name, meta in files.items():
        require(allowed(name) and 0 <= meta['size'] <= MAX_BYTES)
        require(re.fullmatch('[0-9a-f]{64}', meta['sha256']) is not None)
        require(meta['executable'] == (name in EXECUTABLES))
    return files


def admit(archive, expected):
    context(expected)
    parent = owned_root_parent()
    require(archive.stat().st_size <= MAX_BYTES)
    require('sha256:' + file_hash(archive) == expected['digest'])
    require(not ROOT.exists() and not ROOT.is_symlink())
    stage = Path(tempfile.mkdtemp(prefix='admission-', dir=parent))
    try:
        with zipfile.ZipFile(archive) as z:
            infos = z.infolist()
            require(len(infos) <= MAX_FILES + 1)
            names = [i.filename for i in infos]
            require(len(names) == len(set(names)) and 'manifest.json' in names)
            require(sum(i.file_size for i in infos) <= MAX_BYTES)
            for i in infos:
                mode = i.external_attr >> 16
                require(i.filename == 'manifest.json' or allowed(i.filename))
                require(not i.is_dir() and stat.S_IFMT(mode) in (0, stat.S_IFREG))
                require(not (mode & (stat.S_ISUID | stat.S_ISGID)))
                require(not i.flag_bits & 1)
                require(i.file_size <= MAX_BYTES)
            require(z.getinfo('manifest.json').file_size <= 2 * 1024 * 1024)
            raw = z.read('manifest.json')
            require(digest(raw) == expected['manifest_sha256'])
            m = json.loads(raw)
            files = manifest_check(m, expected)
            require(set(names) == set(files) | {'manifest.json'})
            for name, meta in files.items():
                require(z.getinfo(name).file_size == meta['size'])
                path = stage / name
                path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
                with z.open(name) as src, path.open('xb') as dst:
                    shutil.copyfileobj(src, dst, 1024 * 1024)
                require(file_hash(path) == meta['sha256'])
                path.chmod(0o700 if meta['executable'] else 0o600)
            for directory in stage.rglob('*'):
                if directory.is_dir():
                    directory.chmod(0o700)
            for directory in (stage / 'cache').rglob('*'):
                directory.chmod(0o500 if directory.is_dir() else 0o400)
            (stage / 'cache').chmod(0o500)
            (stage / 'manifest.json').write_bytes(raw)
            receipt = dict(expected, manifest_sha256=digest(raw), python_sha256=file_hash(Path('/usr/bin/python3').resolve()))
            (stage / 'receipt.json').write_text(json.dumps(receipt, sort_keys=True))
            (stage / 'manifest.json').chmod(0o600)
            (stage / 'receipt.json').chmod(0o600)
            stage.rename(ROOT)
    finally:
        if stage.exists():
            for directory in stage.rglob('*'):
                if directory.is_dir():
                    directory.chmod(0o700)
            shutil.rmtree(stage)
    return {'result': 'PASS', 'operation': 'admit', 'files': len(files)}


def verify(expected):
    context(expected)
    owned_root_parent()
    require(ROOT.is_dir() and not ROOT.is_symlink())
    paths = list(ROOT.rglob('*'))
    require(len(paths) <= MAX_FILES * 2)
    for p in [ROOT] + paths:
        s = p.lstat()
        require(s.st_uid == os.getuid() and not s.st_mode & 0o077)
        require(stat.S_ISDIR(s.st_mode) or stat.S_ISREG(s.st_mode))
    receipt = json.loads((ROOT / 'receipt.json').read_text())
    require(all(receipt[k] == v for k, v in expected.items()))
    require(receipt['python_sha256'] == file_hash(Path('/usr/bin/python3').resolve()))
    require(receipt['manifest_sha256'] == file_hash(ROOT / 'manifest.json'))
    m = json.loads((ROOT / 'manifest.json').read_text())
    files = manifest_check(m, expected)
    require({str(p.relative_to(ROOT)) for p in paths if p.is_file()} == set(files) | {'manifest.json', 'receipt.json'})
    for name, meta in files.items():
        p = ROOT / name
        require(p.stat().st_size == meta['size'] and file_hash(p) == meta['sha256'])
        require(bool(p.stat().st_mode & 0o111) == meta['executable'])
    spec = importlib.util.spec_from_file_location('offline', ROOT / 'tools/generic-ytdlp-offline-runtime.py')
    helper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(helper)
    bundle = helper.verify_bundle(ROOT / 'bundle')
    require(helper.verify_cache('/usr/bin/python3', ROOT / 'cache', os.getuid(), bundle))
    return m


def environment():
    return {'HOME': '/home/gateway-verify', 'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8',
            'PYTHONDONTWRITEBYTECODE': '1', 'PYTHONNOUSERSITE': '1',
            'YTDLP_SOURCE': str(ROOT / 'cache/site-packages'),
            'YTDLP_PREP_PYTHONPATH': str(ROOT / 'cache/site-packages'),
            'CARGO_BIN_EXE_ytdlp-sandbox': str(ROOT / 'bin/ytdlp-sandbox'),
            'GENERIC_YTDLP_TEST_WORKER_PATH': str(ROOT / 'source/plugins/generic-ytdlp/worker/worker.py'),
            'PYTHON': '/usr/bin/python3'}


def runtime_identity():
    fields = dict(line.split(':', 1) for line in Path('/proc/self/status').read_text().splitlines() if ':' in line)
    require(os.getuid() != 0 and os.geteuid() != 0 and os.getgroups() == [os.getgid()])
    require(fields['NoNewPrivs'].strip() == '1')
    require(all(int(fields[k].strip(), 16) == 0 for k in ('CapInh', 'CapPrm', 'CapEff', 'CapBnd', 'CapAmb')))


def probe(expected):
    runtime_identity()
    verify(expected)
    env = environment()
    for name in TESTS:
        # All selected tests use the existing in-process fixture broker or deny
        # private/secret targets before networking. No real site selector exists.
        result = subprocess.run([str(ROOT / 'bin/runtime-tests'), '--exact', name, '--test-threads=1'],
            env=env, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, timeout=30)
        require(result.returncode == 0 and b'1 passed; 0 failed' in result.stdout and len(result.stdout) < 65536)
    result = subprocess.run([str(ROOT / 'bin/generic-ytdlp-real-smoke'), 'http://127.0.0.1/metadata'],
        env=env, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, timeout=35)
    require(result.returncode != 0 and b'broker_error_code: BROKER_EGRESS_REJECTED' in result.stdout
            and re.search(rb'broker_request_count: [1-9][0-9]*', result.stdout) is not None)
    verify(expected)
    return {'result': 'PASS', 'operation': 'probe', 'tests': len(TESTS), 'smoke': 'BROKER_EGRESS_REJECTED', 'cache': 'verified-warm'}



def smoke(expected, source):
    # Entry is delivered for downstream #67 only. Its independent preflight and
    # publication gates authorize execution; #146 never calls this operation.
    require(source == 'https://www.bilibili.com/video/BV14V411W7r5/')
    runtime_identity()
    verify(expected)
    result = subprocess.run([str(ROOT / 'bin/generic-ytdlp-real-smoke'), source],
        env=environment(), stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL, timeout=90)
    require(len(result.stdout) <= 4096)
    text = result.stdout.decode('ascii')
    allowed_keys = {'result', 'plugin', 'broker_status_class', 'broker_error_code',
        'broker_request_count', 'protocol', 'stream_count', 'title_length',
        'process_error', 'unsupported_stage', 'fallback_reason'}
    seen = set()
    for line in text.splitlines():
        key, value = line.split(': ', 1)
        require(key in allowed_keys and key not in seen)
        require(re.fullmatch(r'[A-Z0-9_]{1,80}|[0-9]xx|n/a|generic-ytdlp|http-file|hls', value) is not None)
        seen.add(key)
    require({'result', 'plugin', 'broker_request_count', 'stream_count'} <= seen)
    print(text, end='')
    print('runtime_cache: offline-hit')
    return result.returncode


def main():
    p = argparse.ArgumentParser()
    p.add_argument('operation', choices=['admit', 'verify', 'probe', 'smoke'])
    p.add_argument('--expected', required=True, type=Path)
    p.add_argument('--archive', type=Path)
    p.add_argument('--source')
    a = p.parse_args()
    try:
        expected = json.loads(a.expected.read_text())
        if a.operation == 'smoke':
            return smoke(expected, a.source)
        if a.operation == 'admit':
            report = admit(a.archive, expected)
        elif a.operation == 'probe':
            report = probe(expected)
        else:
            verify(expected)
            report = {'result': 'PASS', 'operation': 'verify'}
        print(json.dumps(report, sort_keys=True))
        return 0
    except Exception:
        # Raw exceptions can contain source paths, URLs or child output.
        print('{"result":"BLOCKED","code":"ARTIFACT_REJECTED"}')
        return 75


if __name__ == '__main__':
    raise SystemExit(main())
