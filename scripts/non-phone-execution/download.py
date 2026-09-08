#!/usr/bin/env python3
"""Authenticated GitHub control-plane lookup; no executable archive content runs."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import zipfile

REPO = 'liqiangcc/jellyfin-web-media-gateway'


def api(path):
    return json.loads(subprocess.check_output(['gh', 'api', f'repos/{REPO}/' + path]))


def main():
    run_id, candidate, output = sys.argv[1:]
    assert run_id.isdecimal() and re.fullmatch('[0-9a-f]{40}', candidate)
    root = Path(output)
    root.mkdir(mode=0o700, parents=True, exist_ok=False)
    run = api('actions/runs/' + run_id)
    assert run['head_sha'] == candidate and run['event'] == 'push'
    assert run['path'] == '.github/workflows/non-phone-execution.yml'
    assert run['head_repository']['full_name'] == REPO
    jobs = api('actions/runs/' + run_id + '/jobs?per_page=100')['jobs']
    build = [j for j in jobs if j['name'] == 'build' and j['conclusion'] == 'success']
    assert len(build) == 1
    artifacts = api('actions/runs/' + run_id + '/artifacts?per_page=100')['artifacts']
    artifacts = [a for a in artifacts if a['name'] == 'non-phone-runtime-' + candidate and not a['expired']]
    assert len(artifacts) == 1
    artifact = artifacts[0]
    assert artifact['size_in_bytes'] <= 1024 * 1024 * 1024
    archive = root / 'artifact.zip'
    with archive.open('xb') as f:
        subprocess.run(['gh', 'api', f'repos/{REPO}/actions/artifacts/{artifact["id"]}/zip'], stdout=f, check=True, timeout=300)
    h = hashlib.sha256()
    with archive.open('rb') as f:
        for data in iter(lambda: f.read(1024 * 1024), b''):
            h.update(data)
    assert 'sha256:' + h.hexdigest() == artifact['digest']
    with zipfile.ZipFile(archive) as z:
        assert z.getinfo('manifest.json').file_size <= 2 * 1024 * 1024
        manifest_hash = hashlib.sha256(z.read('manifest.json')).hexdigest()
    expected = dict(repository=REPO, candidate=candidate, runtime='80fb081b129f8f664124b84ddcc9698039e2cfd1',
        run_id=int(run_id), run_attempt=run['run_attempt'], job_id=build[0]['id'],
        workflow_path=run['path'], artifact_id=artifact['id'], digest=artifact['digest'], manifest_sha256=manifest_hash)
    (root / 'expected.json').write_text(json.dumps(expected, sort_keys=True))
    print(json.dumps(expected, sort_keys=True))


if __name__ == '__main__':
    main()
