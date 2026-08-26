"""Repository-owned, anonymous-only yt-dlp API worker.

The worker receives no caller argv/config/plugin/profile/cookie authority. Its
only network capability is the inherited fd 3 socketpair. The Rust launcher
installs the inherited seccomp/no-new-privs boundary before this module runs.
"""

from __future__ import annotations

import errno
import io
import json
import os
import socket
import subprocess
import sys
from typing import Any

from yt_dlp import YoutubeDL
from yt_dlp.networking import Request, RequestDirector, RequestHandler, Response
from yt_dlp.networking.exceptions import RequestError
from yt_dlp.version import __version__

EXPECTED_VERSION = "2026.08.19"
EXPECTED_COMMIT = "3a08beaf031ab68f966401ead017ac81fe8486cf"
MAX_FRAME = 128 * 1024
MAX_BODY = 96 * 1024
_BROKER_STREAM: socket.socket | None = None


def _read_exact(stream: socket.socket, size: int) -> bytes:
    chunks: list[bytes] = []
    remaining = size
    while remaining:
        chunk = stream.recv(remaining)
        if not chunk:
            raise RuntimeError("broker closed")
        chunks.append(chunk)
        remaining -= len(chunk)
    return b"".join(chunks)


def _broker_request(request: dict[str, Any]) -> dict[str, Any]:
    global _BROKER_STREAM
    fd = int(os.environ.get("YTDLP_BROKER_FD", "3"))
    if _BROKER_STREAM is None:
        _BROKER_STREAM = socket.socket(fileno=fd)
    payload = json.dumps(request, separators=(",", ":")).encode()
    if not payload or len(payload) > MAX_FRAME:
        raise RequestError("broker frame rejected")
    _BROKER_STREAM.sendall(len(payload).to_bytes(4, "big") + payload)
    length = int.from_bytes(_read_exact(_BROKER_STREAM, 4), "big")
    if length == 0 or length > MAX_FRAME:
        raise RequestError("broker frame rejected")
    response = json.loads(_read_exact(_BROKER_STREAM, length))
    if response.get("error"):
        raise RequestError("broker request rejected")
    if len(response.get("body", [])) > MAX_BODY:
        raise RequestError("broker body rejected")
    return response


class BrokerRH(RequestHandler):
    """yt-dlp RequestHandler that has no socket implementation of its own."""

    RH_NAME = "gateway broker"
    _SUPPORTED_URL_SCHEMES = ("http", "https")
    _SUPPORTED_PROXY_SCHEMES = ()
    _SUPPORTED_FEATURES = ()

    def _send(self, request: Request) -> Response:
        if request.proxies:
            raise RequestError("proxy configuration is not admitted")
        headers = self._get_headers(request)
        if request.data is not None and not isinstance(request.data, bytes):
            raise RequestError("streaming request bodies are not admitted")
        response = _broker_request(
            {
                "operation": "http",
                "method": request.method,
                "url": request.url,
                "headers": headers,
                "body": list(request.data or b""),
            }
        )
        body = bytes(response.get("body", []))
        return Response(
            io.BytesIO(body),
            request.url,
            response.get("headers", {}),
            status=int(response.get("status", 502)),
            reason=response.get("reason", "Bad Gateway"),
        )


class DirectSocketRH(RequestHandler):
    """Negative fixture: an extractor-like handler cannot create AF_INET."""

    RH_NAME = "direct escape fixture"
    _SUPPORTED_URL_SCHEMES = ("http", "https")
    _SUPPORTED_PROXY_SCHEMES = ()
    _SUPPORTED_FEATURES = ()

    def _send(self, request: Request) -> Response:
        del request
        socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        raise AssertionError("direct socket unexpectedly allowed")


class DirectUnixSocketRH(RequestHandler):
    """Negative fixture: no second local socket authority is admitted."""

    RH_NAME = "direct unix escape fixture"
    _SUPPORTED_URL_SCHEMES = ("http", "https")
    _SUPPORTED_PROXY_SCHEMES = ()
    _SUPPORTED_FEATURES = ()

    def _send(self, request: Request) -> Response:
        del request
        socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        raise AssertionError("direct unix socket unexpectedly allowed")


class SilentLogger:
    def stdout(self, *args, **kwargs):
        del args, kwargs

    def error(self, *args, **kwargs):
        del args, kwargs

    warning = error
    debug = error


class BrokerOnlyYDL(YoutubeDL):
    def build_request_director(self, handlers, preferences=None):  # noqa: ARG002
        # Do not call the normal handler set: it includes urllib, requests,
        # websockets and optional curl_cffi implementations.
        return super().build_request_director([BrokerRH], preferences=[])


def _ydl() -> BrokerOnlyYDL:
    return BrokerOnlyYDL(
        {
            "quiet": True,
            "no_warnings": True,
            "simulate": True,
            "skip_download": True,
            "http_headers": {},
            "proxy": None,
            "geo_verification_proxy": None,
            "cookiefile": None,
            "cookiesfrombrowser": None,
            "nocheckcertificate": False,
            "client_certificate": None,
            "client_certificate_key": None,
            "client_certificate_password": None,
            "source_address": None,
            "js_runtimes": {},
            "remote_components": set(),
            "plugins": [],
            "allowed_extractors": ["default"],
            "external_downloader": None,
            "postprocessors": [],
        },
        auto_init=False,
    )


def _probe(url: str) -> dict[str, Any]:
    with _ydl() as ydl:
        response = ydl.urlopen(Request(url))
        body = json.loads(response.read(MAX_BODY).decode())
    if body.get("fixture") != "generic-ytdlp-broker":
        raise RuntimeError("unexpected fixture")
    return {
        "title": body["title"],
        "protection": "clear",
        "streams": [
            {
                "id": "primary",
                "protocol": "http-file",
                "url": "https://cdn.example.test/video.mp4",
                "public_headers": {"Accept": "video/mp4"},
                "upstream_access_ref": None,
            }
        ],
    }


def _network_matrix(url: str) -> dict[str, Any]:
    denied: dict[str, bool] = {}
    for family in (socket.AF_INET, socket.AF_INET6):
        try:
            sock = socket.socket(family, socket.SOCK_STREAM)
            sock.close()
            denied[str(family)] = False
        except OSError as error:
            denied[str(family)] = error.errno == errno.EPERM

    logger = SilentLogger()
    direct = RequestDirector(logger=logger, verbose=False)
    direct.add_handler(DirectSocketRH(logger=logger))
    try:
        direct.send(Request(url))
        custom_denied = False
    except Exception:
        custom_denied = True
    finally:
        direct.close()

    unix_direct = RequestDirector(logger=logger, verbose=False)
    unix_direct.add_handler(DirectUnixSocketRH(logger=logger))
    try:
        unix_direct.send(Request(url))
        unix_denied = False
    except Exception:
        unix_denied = True
    finally:
        unix_direct.close()

    python_executable = os.readlink("/proc/self/exe")
    child = subprocess.run(
        [python_executable, "-c", "import socket; socket.socket(socket.AF_INET, socket.SOCK_STREAM)"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    child_denied = child.returncode != 0
    child_unix = subprocess.run(
        [
            python_executable,
            "-c",
            "import socket; socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)",
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    child_unix_denied = child_unix.returncode != 0
    probe = _probe(url)
    no_new_privs = any(
        line.startswith("NoNewPrivs:") and line.split()[-1] == "1"
        for line in open("/proc/self/status", encoding="ascii")
    )
    seccomp = any(
        line.startswith("Seccomp:") and line.split()[-1] == "2"
        for line in open("/proc/self/status", encoding="ascii")
    )
    return {
        "matrix": {
            "python_af_inet_denied": denied.get(str(socket.AF_INET), False),
            "python_af_inet6_denied": denied.get(str(socket.AF_INET6), False),
            "custom_handler_denied": custom_denied,
            "custom_unix_handler_denied": unix_denied,
            "python_af_unix_denied": unix_denied,
            "child_af_inet_denied": child_denied,
            "child_af_unix_denied": child_unix_denied,
            "broker_ipc_usable": probe["title"] == "fixture media",
            "no_new_privs": no_new_privs,
            "seccomp_filter": seccomp,
        }
    }


def _ambient_fd_report(url: str) -> dict[str, Any]:
    # Keep fd 3 useful while proving that no other inherited descriptor is
    # visible to the worker.
    _probe(url)
    ambient: dict[str, str] = {}
    for entry in os.listdir("/proc/self/fd"):
        try:
            fd = int(entry)
        except ValueError:
            continue
        if fd <= 3:
            continue
        try:
            target = os.readlink(f"/proc/self/fd/{entry}")
        except OSError:
            continue
        # /proc can briefly expose the descriptor used by listdir itself;
        # it is not an inherited authority and disappears before exec.
        if target.startswith("/proc/") and target.endswith("/fd"):
            continue
        ambient[entry] = target
    child_code = (
        "import json, os\n"
        "result = {}\n"
        "for name in os.listdir('/proc/self/fd'):\n"
        "    try:\n"
        "        fd = int(name)\n"
        "        target = os.readlink('/proc/self/fd/' + name)\n"
        "    except (OSError, ValueError):\n"
        "        continue\n"
        "    if fd > 3 and not (target.startswith('/proc/') and target.endswith('/fd')):\n"
        "        result[name] = target\n"
        "print(json.dumps(result))\n"
    )
    child = subprocess.run(
        [sys.executable, "-c", child_code],
        close_fds=False,
        capture_output=True,
        text=True,
        check=False,
    )
    try:
        descendant_ambient = json.loads(child.stdout)
    except json.JSONDecodeError:
        descendant_ambient = {"child_exit": child.returncode}
    return {
        "ambient_fds": ambient,
        "descendant_ambient_fds": descendant_ambient,
        "broker_ipc_usable": True,
    }


def _spawn_long_lived_descendant() -> None:
    pid_file = os.environ.get("YTDLP_DESCENDANT_PID_FILE")
    if not pid_file:
        raise RuntimeError("descendant marker unavailable")
    descendant = subprocess.Popen(
        [sys.executable, "-c", "import time; time.sleep(60)"],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    with open(pid_file, "w", encoding="ascii") as marker:
        marker.write(str(descendant.pid))
        marker.flush()


def main() -> int:
    if __version__ != EXPECTED_VERSION or os.environ.get("YTDLP_EXPECTED_VERSION") != EXPECTED_VERSION:
        return 42
    if sys.argv[1:] and sys.argv[1] == "crash":
        os._exit(7)
    action = sys.argv[1] if len(sys.argv) > 1 else "probe"
    url = sys.argv[2] if len(sys.argv) > 2 else "https://fixture.example.test/media"
    if action == "probe":
        result = _probe(url)
    elif action == "multi-probe":
        _probe(url)
        result = _probe(url)
    elif action == "network-matrix":
        result = _network_matrix(url)
    elif action == "ambient-fd":
        result = _ambient_fd_report(url)
    elif action == "timeout":
        import time

        time.sleep(60)
        return 0
    elif action in {"timeout-descendant", "cancel-descendant"}:
        import time

        _spawn_long_lived_descendant()
        time.sleep(60)
        return 0
    elif action == "cancel-probe-descendant":
        _spawn_long_lived_descendant()
        result = _probe(url)
    elif action == "crash-descendant":
        _spawn_long_lived_descendant()
        os._exit(7)
    elif action == "overflow-descendant":
        _spawn_long_lived_descendant()
        sys.stdout.write("x" * (512 * 1024))
        sys.stdout.flush()
        return 0
    elif action == "overflow":
        sys.stdout.write("x" * (512 * 1024))
        sys.stdout.flush()
        return 0
    elif action == "diagnostic-sentinel":
        sys.stderr.write(
            "source=https://fixture.example.test/watch?sig=signed-query-secret "
            "Authorization=Bearer secret-token Cookie=session-secret\n"
        )
        sys.stderr.flush()
        return 65
    else:
        return 64
    sys.stdout.write(json.dumps(result, separators=(",", ":")))
    sys.stdout.flush()
    if _BROKER_STREAM is not None:
        _BROKER_STREAM.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
