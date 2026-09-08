#!/usr/bin/env python3
"""Hosted-only package assembly; runtime source is built untouched at fixed path."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('artifact', HERE / 'artifact.py')
a = importlib.util.module_from_spec(spec)
spec.loader.exec_module(a)


def run(argv, **kw):
    return subprocess.run(argv, check=True, **kw)


def main():
    assert os.environ.get('GITHUB_ACTIONS') == 'true'
    candidate = os.environ['CANDIDATE_SHA']
    assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip() == candidate
    root = a.ROOT
    source = root / 'source'
    source.mkdir(parents=True)
    run(['git', 'clone', '--no-checkout', '--no-hardlinks', '.', str(source)])
    run(['git', '-C', str(source), 'checkout', '--detach', a.RUNTIME])
    assert subprocess.check_output(['git', '-C', str(source), 'rev-parse', 'HEAD'], text=True).strip() == a.RUNTIME
    env = dict(os.environ, CARGO_TARGET_DIR=str(root / 'target'), PYTHONDONTWRITEBYTECODE='1')
    run(['python3', str(source / 'scripts/generic-ytdlp-offline-runtime.py'), 'build', str(root / 'bundle')], cwd=source, env=env)
    cold = subprocess.check_output(['python3', str(source / 'scripts/generic-ytdlp-offline-runtime.py'), 'install', str(root / 'bundle')], env=dict(env, XDG_CACHE_HOME=str(root / 'build-cache')), text=True).splitlines()
    assert cold[0] == 'prepared'
    env['YTDLP_SOURCE'] = cold[1]
    run(['cargo', 'build', '--locked', '-p', 'generic-ytdlp', '--features', 'runtime-prep', '--bins'], cwd=source, env=env)
    messages = subprocess.check_output(['cargo', 'test', '--locked', '-p', 'generic-ytdlp', '--features', 'runtime-prep', '--test', 'runtime', '--no-run', '--message-format=json'], cwd=source, env=env, text=True)
    tests = [json.loads(s)['executable'] for s in messages.splitlines() if json.loads(s).get('reason') == 'compiler-artifact' and json.loads(s).get('target', {}).get('name') == 'runtime' and json.loads(s).get('executable')]
    assert len(tests) == 1
    run(['cargo', 'test', '--locked', '-p', 'generic-ytdlp', '--features', 'runtime-prep', '--test', 'runtime'], cwd=source, env=env)
    run(['bash', 'scripts/test-generic-ytdlp-clean-build.sh'], cwd=source, env=env)
    out = Path(os.environ['RUNNER_TEMP']) / 'delivery'
    (out / 'bin').mkdir(parents=True)
    for name in ('generic-ytdlp-real-smoke', 'ytdlp-sandbox'):
        shutil.copyfile(root / 'target/debug' / name, out / 'bin' / name)
    shutil.copyfile(tests[0], out / 'bin/runtime-tests')
    for p in (out / 'bin').iterdir():
        run(['strip', str(p)])
        p.chmod(0o755)
    dest = out / 'source/plugins/generic-ytdlp/worker'
    dest.mkdir(parents=True)
    shutil.copyfile(source / 'plugins/generic-ytdlp/worker/worker.py', dest / 'worker.py')
    shutil.copytree(root / 'bundle', out / 'bundle')
    shutil.copytree(Path(cold[1]).parent, out / 'cache', ignore=shutil.ignore_patterns('__pycache__', '*.pyc'))
    (out / 'tools').mkdir()
    for f in ('generic-ytdlp-offline-runtime.py', 'generic-ytdlp-offline-runtime.lock.json'):
        shutil.copyfile(source / 'scripts' / f, out / 'tools' / f)
    shutil.copyfile(HERE / 'artifact.py', out / 'tools/artifact.py')
    files = {}
    for p in out.rglob('*'):
        if p.is_file():
            name = str(p.relative_to(out))
            assert a.allowed(name), name
            p.chmod(0o755 if name in a.EXECUTABLES else 0o644)
            files[name] = dict(sha256=a.file_hash(p), size=p.stat().st_size, executable=name in a.EXECUTABLES)
    manifest = dict(schema=1, candidate=candidate, workflow_sha=candidate, runtime=a.RUNTIME,
        source_root=str(source), target='x86_64-unknown-linux-gnu', minimum_glibc='2.39',
        run_id=int(os.environ['GITHUB_RUN_ID']), run_attempt=int(os.environ['GITHUB_RUN_ATTEMPT']),
        rustc=subprocess.check_output(['rustc', '--version'], text=True).strip(), files=files)
    (out / 'manifest.json').write_text(json.dumps(manifest, sort_keys=True))
    # A separate consumer job receives only this package, never this build tree.
    print('PACKAGED', len(files))


if __name__ == '__main__':
    main()
