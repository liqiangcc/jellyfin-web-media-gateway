#!/usr/bin/env python3
"""Bounded, anonymous same-host Chrome fixture proof; never builds or installs."""
import http.server
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import tempfile
import threading


class ReadinessError(Exception):
    pass


def identity():
    fields = dict(line.split(':', 1) for line in Path('/proc/self/status').read_text().splitlines() if ':' in line)
    if os.getuid() == 0 or os.geteuid() == 0:
        raise ReadinessError('ROOT_REJECTED')
    if any(int(fields[key].strip(), 16) for key in ('CapInh', 'CapPrm', 'CapEff', 'CapBnd', 'CapAmb')):
        raise ReadinessError('CAPABILITIES_REJECTED')
    if fields['NoNewPrivs'].strip() != '1' or os.getgroups() != [os.getgid()]:
        raise ReadinessError('PRIVILEGE_REJECTED')
    return {'uid': os.getuid(), 'gid': os.getgid(), 'capabilities': 'zero', 'no_new_privs': True}


def browser_argv(profile, url):
    return ['/usr/bin/google-chrome', '--headless', '--no-first-run',
            '--no-default-browser-check', '--disable-background-networking',
            '--disable-component-update', '--disable-sync',
            '--user-data-dir=' + str(profile), '--dump-dom',
            '--virtual-time-budget=5000', url]


def fixture():
    proof = {'host': False, 'origin': False}

    class Handler(http.server.BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def do_GET(self):
            expected = '127.0.0.1:' + str(self.server.server_port)
            if self.headers.get('Host') != expected or self.path != '/':
                self.send_error(403)
                return
            proof['host'] = True
            data = b'''<!doctype html><title>fixture</title><script>
fetch('/proof', {method:'POST'}).then(r => {if(r.ok) document.title='GATEWAY_FIXTURE_PASS'});
</script>'''
            self.send_response(200)
            self.send_header('Content-Type', 'text/html')
            self.send_header('Content-Length', str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def do_POST(self):
            expected = '127.0.0.1:' + str(self.server.server_port)
            ok = (self.path == '/proof' and self.headers.get('Host') == expected
                  and self.headers.get('Origin') == 'http://' + expected
                  and self.headers.get('Content-Length', '0') == '0')
            proof['origin'] = ok
            self.send_response(204 if ok else 403)
            self.end_headers()

    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    server.daemon_threads = True
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    return server, thread, proof


def browser_proof():
    report = identity()
    server, thread, proof = fixture()
    port = server.server_port
    proc = None
    try:
        with tempfile.TemporaryDirectory(prefix='gateway-fixture-') as work:
            with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as err:
                proc = subprocess.Popen(browser_argv(Path(work) / 'profile', f'http://127.0.0.1:{port}/'),
                    env={'HOME': os.environ['HOME'], 'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8'},
                    stdin=subprocess.DEVNULL, stdout=out, stderr=err, start_new_session=True)
                try:
                    proc.wait(timeout=45)
                finally:
                    try:
                        os.killpg(proc.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    proc.wait(timeout=5)
                out.seek(0)
                dom = out.read(65537)
                # Never print Chrome diagnostics, DOM or inherited environment.
                if proc.returncode != 0 or len(dom) > 65536 or b'<title>GATEWAY_FIXTURE_PASS</title>' not in dom or not all(proof.values()):
                    raise ReadinessError('BROWSER_FIXTURE_UNAVAILABLE')
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=2)
    with socket.socket() as probe:
        if probe.connect_ex(('127.0.0.1', port)) == 0:
            raise ReadinessError('FIXTURE_CLEANUP_FAILED')
    return dict(report, result='PASS', browser_host='tx-node', topology='same-host-loopback',
                host=True, origin=True, profile='temporary-isolated-removed',
                sandbox='normal', cdp='no-listener', fixture_cleanup=True)


def main():
    try:
        print(json.dumps(browser_proof(), sort_keys=True))
        return 0
    except (ReadinessError, OSError, subprocess.SubprocessError) as exc:
        code = str(exc) if isinstance(exc, ReadinessError) else 'READINESS_UNAVAILABLE'
        print(json.dumps({'result': 'BLOCKED', 'code': code}))
        return 75


if __name__ == '__main__':
    raise SystemExit(main())
