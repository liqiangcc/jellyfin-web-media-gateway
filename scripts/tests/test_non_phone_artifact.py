import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile

spec = importlib.util.spec_from_file_location('artifact', Path(__file__).parents[1] / 'non-phone-execution/artifact.py')
a = importlib.util.module_from_spec(spec)
spec.loader.exec_module(a)


class AdmissionTests(unittest.TestCase):
    def test_traversal_and_unexpected_content(self):
        for name in ('../bin/x', '/bin/x', 'cache/site-packages/../x', 'cache//x', '.git/config', 'tools/key', 'cache/site-packages/.netrc', 'cache\\x'):
            self.assertFalse(a.allowed(name), name)
        self.assertTrue(a.allowed('source/plugins/generic-ytdlp/worker/worker.py'))

    def test_admission_rejects_archive_before_execution(self):
        for attack in ('digest', 'candidate', 'missing', 'mutated', 'traversal', 'symlink', 'duplicate'):
            with self.subTest(attack=attack), tempfile.TemporaryDirectory() as temp:
                root = Path(temp) / 'runtime'
                files = {k: {'size': 1, 'sha256': hashlib.sha256(b'x').hexdigest(), 'executable': k in a.EXECUTABLES} for k in a.REQUIRED}
                m = dict(schema=1, candidate='a'*40, workflow_sha='a'*40, runtime=a.RUNTIME, source_root=str(root/'source'), target='x86_64-unknown-linux-gnu', run_id=1, run_attempt=1, files=files)
                raw = json.dumps(m).encode()
                archive = Path(temp) / 'artifact.zip'
                with zipfile.ZipFile(archive, 'w') as z:
                    z.writestr('manifest.json', raw)
                    for name in files:
                        if attack == 'missing' and name == 'bin/ytdlp-sandbox':
                            continue
                        z.writestr(name, b'y' if attack == 'mutated' else b'x')
                    if attack == 'traversal':
                        z.writestr('../outside', b'x')
                    if attack == 'symlink':
                        info = zipfile.ZipInfo('cache/site-packages/link')
                        info.create_system = 3
                        info.external_attr = 0o120777 << 16
                        z.writestr(info, 'outside')
                    if attack == 'duplicate':
                        z.writestr('bin/ytdlp-sandbox', b'x')
                e = dict(candidate='b'*40 if attack == 'candidate' else 'a'*40, run_id=1, run_attempt=1, manifest_sha256=hashlib.sha256(raw).hexdigest(), digest='sha256:' + ('0'*64 if attack == 'digest' else a.file_hash(archive)))
                with patch.object(a, 'ROOT', root), patch.object(a, 'context'), patch.object(a, 'owned_root_parent', return_value=Path(temp)):
                    with self.assertRaises(a.AdmissionError):
                        a.admit(archive, e)
                self.assertFalse(root.exists())
                self.assertEqual(list(Path(temp).glob('admission-*')), [])

    def test_unpublished_or_wrong_sample_never_executes(self):
        with patch.object(a.subprocess, 'run') as run:
            for source in ('https://user:secret@example.test/', 'https://example.test/'):
                with self.assertRaises(a.AdmissionError):
                    a.smoke({}, source)
            run.assert_not_called()
