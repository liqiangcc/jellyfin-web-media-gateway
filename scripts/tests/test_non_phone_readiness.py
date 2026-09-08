import importlib.util
from pathlib import Path
import unittest
import urllib.request
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('readiness', Path(__file__).parents[1] / 'non-phone-execution/readiness.py')
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


class ReadinessTests(unittest.TestCase):
    def test_root_rejected_before_browser(self):
        with patch.object(m.os, 'getuid', return_value=0), self.assertRaisesRegex(m.ReadinessError, 'ROOT_REJECTED'):
            m.identity()

    def test_browser_does_not_disable_sandbox_or_open_cdp(self):
        args = m.browser_argv('/owned/profile', 'http://127.0.0.1:1234/')
        self.assertFalse(any('no-sandbox' in a or 'remote-debugging' in a or 'autoplay-policy' in a for a in args))
        self.assertIn('--user-data-dir=/owned/profile', args)

    def test_fixture_rejects_wrong_origin_and_host(self):
        server, thread, proof = m.fixture()
        url = f'http://127.0.0.1:{server.server_port}'
        try:
            with urllib.request.urlopen(url + '/') as response:
                self.assertEqual(response.status, 200)
            for headers in ({'Origin': 'https://untrusted.invalid'}, {'Origin': url, 'Host': 'untrusted.invalid'}):
                with self.assertRaises(urllib.error.HTTPError) as error:
                    urllib.request.urlopen(urllib.request.Request(url + '/proof', data=b'', headers=headers))
                self.assertEqual(error.exception.code, 403)
            with urllib.request.urlopen(urllib.request.Request(url + '/proof', data=b'', headers={'Origin': url})) as response:
                self.assertEqual(response.status, 204)
            self.assertTrue(all(proof.values()))
        finally:
            server.shutdown()
            server.server_close()
            thread.join()

    def test_failure_diagnostics_are_fixed(self):
        with patch.object(m, 'browser_proof', side_effect=OSError('secret-token')), patch('builtins.print') as output:
            self.assertEqual(m.main(), 75)
            self.assertNotIn('secret-token', output.call_args.args[0])
