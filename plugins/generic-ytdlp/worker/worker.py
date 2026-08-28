"""Repository-owned, anonymous-only yt-dlp API worker.

The worker receives no caller argv/config/plugin/profile/cookie authority. Its
only network capability is the inherited fd 3 socketpair. The Rust launcher
installs the inherited seccomp/no-new-privs boundary before this module runs.
"""

from __future__ import annotations

import binascii
import errno
import io
import json
import os
import socket
import subprocess
import sys
import urllib.parse
import re
from typing import Any

from yt_dlp import YoutubeDL
from yt_dlp.networking import Request, RequestDirector, RequestHandler, Response
from yt_dlp.networking.exceptions import RequestError
from yt_dlp.utils import DownloadError, ExtractorError
from yt_dlp.version import __version__

EXPECTED_VERSION = "2026.08.19"
EXPECTED_COMMIT = "3a08beaf031ab68f966401ead017ac81fe8486cf"
MAX_BODY = 96 * 1024
MAX_TITLE_BYTES = 1024
MAX_URL_BYTES = 16 * 1024
MAX_HEADERS = 32
MAX_HEADER_NAME = 128
MAX_HEADER_VALUE = 4096
DIRECT_MEDIA_EXTENSIONS = frozenset({"mp4", "m4v", "m3u8"})
BILIBILI_API_ORIGIN = "https://api.bilibili.com"
MAX_FALLBACK_TEXT_BYTES = MAX_BODY
SECRET_FIELD_NAMES = frozenset(
    {
        "authorization",
        "cookie",
        "set-cookie",
        "proxy-authorization",
        "access-token",
        "refresh-token",
        "id-token",
        "api-key",
        "signed-url",
        "signature",
        "token",
        "sessdata",
    }
)
SENSITIVE_QUERY_NAMES = frozenset(
    {
        "access_token",
        "auth",
        "expires",
        "hdnts",
        "key",
        "sign",
        "signature",
        "signed",
        "token",
    }
)
_BILIBILI_VIDEO_RE = re.compile(r"^/video/(BV[0-9A-Za-z]{6,})/?$")
# Keep this in lockstep with the Rust protocol bound. It is derived from the
# existing R008 body/header bounds and fixed JSON-escaping/protocol overhead.
MAX_FRAME = (
    MAX_BODY * 2
    + MAX_HEADERS * (2 * (MAX_HEADER_NAME + MAX_HEADER_VALUE) + 8)
    + 4 * 1024
)
_BROKER_STREAM: socket.socket | None = None
_FAILURE_CODE: str | None = None

REQUEST_POLICY_REJECTED = "REQUEST_POLICY_REJECTED"
BROKER_FAILURE = "BROKER_FAILURE"
EXTRACTOR_FAILURE = "EXTRACTOR_FAILURE"
UNSUPPORTED_FORMAT = "UNSUPPORTED_FORMAT"
UNEXPECTED_WORKER_FAILURE = "UNEXPECTED_WORKER_FAILURE"


class UnsupportedFormat(Exception):
    """The fixed first-playback policy cannot represent the extraction."""


class RequestPolicyFailure(RequestError):
    """A repository-owned request rule rejected the operation."""


class BrokerFailure(RequestError):
    """The bounded broker capability rejected or failed the operation."""


class InitialStateFallbackNotApplicable(UnsupportedFormat):
    """The opt-in continuation does not own this response shape."""


def _remember_failure(code: str) -> None:
    global _FAILURE_CODE
    _FAILURE_CODE = code


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
    request = dict(request)
    request["body_hex"] = binascii.hexlify(bytes(request.pop("body", b""))).decode("ascii")
    payload = json.dumps(request, separators=(",", ":")).encode()
    if not payload or len(payload) > MAX_FRAME:
        raise RequestError("broker frame rejected")
    try:
        _BROKER_STREAM.sendall(len(payload).to_bytes(4, "big") + payload)
        length = int.from_bytes(_read_exact(_BROKER_STREAM, 4), "big")
    except (OSError, RuntimeError):
        _remember_failure(BROKER_FAILURE)
        raise BrokerFailure("broker transport failed") from None
    if length == 0 or length > MAX_FRAME:
        _remember_failure(BROKER_FAILURE)
        raise BrokerFailure("broker frame rejected")
    try:
        response = json.loads(_read_exact(_BROKER_STREAM, length))
    except (json.JSONDecodeError, OSError, RuntimeError):
        _remember_failure(BROKER_FAILURE)
        raise BrokerFailure("broker envelope rejected") from None
    if not isinstance(response, dict):
        _remember_failure(BROKER_FAILURE)
        raise BrokerFailure("broker envelope rejected")
    if response.get("error"):
        _remember_failure(BROKER_FAILURE)
        raise BrokerFailure("broker request rejected")
    body_hex = response.get("body_hex", "")
    if not isinstance(body_hex, str) or len(body_hex) > MAX_BODY * 2:
        _remember_failure(BROKER_FAILURE)
        raise BrokerFailure("broker body rejected")
    try:
        response["body"] = binascii.unhexlify(body_hex)
    except (binascii.Error, ValueError):
        _remember_failure(BROKER_FAILURE)
        raise BrokerFailure("broker body rejected") from None
    return response


class BrokerRH(RequestHandler):
    """yt-dlp RequestHandler that has no socket implementation of its own."""

    RH_NAME = "gateway broker"
    _SUPPORTED_URL_SCHEMES = ("http", "https")
    _SUPPORTED_PROXY_SCHEMES = ()
    _SUPPORTED_FEATURES = ()

    def _send(self, request: Request) -> Response:
        if request.proxies:
            _remember_failure(REQUEST_POLICY_REJECTED)
            raise RequestPolicyFailure("proxy configuration is not admitted")
        headers = self._get_headers(request)
        if any(_is_secret_header(name, value) for name, value in headers.items()):
            _remember_failure(REQUEST_POLICY_REJECTED)
            raise RequestPolicyFailure("secret request headers are not admitted")
        if request.data is not None and not isinstance(request.data, bytes):
            _remember_failure(REQUEST_POLICY_REJECTED)
            raise RequestPolicyFailure("streaming request bodies are not admitted")
        response = _broker_request(
            {
                "operation": "http",
                "method": request.method,
                "url": request.url,
                "headers": headers,
                "body": list(request.data or b""),
            }
        )
        body = response.get("body", b"")
        if not isinstance(body, bytes):
            raise RequestError("broker body rejected")
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
    ydl = BrokerOnlyYDL(
        {
            "quiet": True,
            "no_warnings": True,
            "simulate": True,
            "skip_download": True,
            "noplaylist": True,
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
    # auto_init=False is required so the worker owns initialization and never
    # loads arbitrary CLI/plugin state. The frozen built-in extractors are
    # then explicitly installed into this API-owned instance.
    ydl.add_default_info_extractors()
    return ydl


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


def _is_secret_header(name: str, value: str) -> bool:
    normalized_name = name.lower().replace("_", "-")
    normalized_value = value.strip().lower()
    return normalized_name in {
        "authorization",
        "proxy-authorization",
        "cookie",
        "set-cookie",
        "x-api-key",
        "x-auth-token",
        "proxy-authenticate",
        "www-authenticate",
        "api-key",
        "access-token",
        "refresh-token",
        "id-token",
    } or normalized_value.startswith(("bearer ", "basic "))


def _public_headers(info: dict[str, Any], fmt: dict[str, Any]) -> dict[str, str]:
    headers = fmt.get("http_headers") or info.get("http_headers") or {}
    if not isinstance(headers, dict):
        raise UnsupportedFormat
    public: dict[str, str] = {}
    for name, value in headers.items():
        if not isinstance(name, str) or not isinstance(value, str):
            raise UnsupportedFormat
        if _is_secret_header(name, value):
            raise UnsupportedFormat
        public[name] = value
    return public


def _is_muxed(fmt: dict[str, Any]) -> bool:
    # Direct video responses from the generic extractor have no codec fields;
    # a present value means that codec is known, while only the literal
    # ``none`` is an explicit audio-only/video-only marker.
    return fmt.get("vcodec") != "none" and fmt.get("acodec") != "none"


def _reject_secret_fields(value: Any) -> None:
    """Reject secret-bearing fixture/API data before it can be normalized."""
    if isinstance(value, dict):
        for name, child in value.items():
            if not isinstance(name, str):
                raise UnsupportedFormat
            normalized = name.lower().replace("_", "-")
            if normalized in SECRET_FIELD_NAMES:
                raise UnsupportedFormat
            _reject_secret_fields(child)
    elif isinstance(value, list):
        for child in value:
            _reject_secret_fields(child)


def _fallback_response_body(response: Response, *, json_body: bool) -> Any:
    try:
        status = int(response.status)
    except (TypeError, ValueError):
        raise UnsupportedFormat from None
    if not 200 <= status < 300:
        raise UnsupportedFormat
    body = response.read(MAX_FALLBACK_TEXT_BYTES + 1)
    if len(body) > MAX_FALLBACK_TEXT_BYTES:
        raise UnsupportedFormat
    try:
        text = body.decode("utf-8")
    except UnicodeDecodeError:
        raise UnsupportedFormat from None
    if json_body:
        try:
            value = json.loads(text)
        except (TypeError, ValueError):
            raise UnsupportedFormat from None
        if not isinstance(value, dict):
            raise UnsupportedFormat
        _reject_secret_fields(value)
        return value
    return text


def _fallback_json(ydl: BrokerOnlyYDL, url: str) -> dict[str, Any]:
    response = ydl.urlopen(Request(url))
    value = _fallback_response_body(response, json_body=True)
    if not isinstance(value, dict):
        raise UnsupportedFormat
    return value


def _api_data(payload: dict[str, Any]) -> dict[str, Any]:
    if (
        payload.get("code") != 0
        or not isinstance(payload.get("data"), dict)
        or any(name not in {"code", "message", "ttl", "data"} for name in payload)
    ):
        raise UnsupportedFormat
    return payload["data"]


def _required_text(value: Any, *, max_bytes: int = MAX_TITLE_BYTES) -> str:
    if not isinstance(value, str) or not value.strip() or len(value.encode()) > max_bytes:
        raise UnsupportedFormat
    return value


def _required_cid(value: Any) -> str:
    if isinstance(value, bool):
        raise UnsupportedFormat
    if isinstance(value, int):
        cid = str(value)
    elif isinstance(value, str) and value.isdecimal():
        cid = value
    else:
        raise UnsupportedFormat
    if not 1 <= len(cid) <= 32:
        raise UnsupportedFormat
    return cid


def _required_public_url(value: Any) -> str:
    if not isinstance(value, str) or len(value.encode()) > MAX_URL_BYTES:
        raise UnsupportedFormat
    try:
        parsed = urllib.parse.urlparse(value)
        query = urllib.parse.parse_qsl(parsed.query, keep_blank_values=True)
    except ValueError:
        raise UnsupportedFormat from None
    if (
        parsed.scheme != "https"
        or not parsed.hostname
        or parsed.username
        or parsed.password
        or parsed.fragment
        or parsed.query
        or query
    ):
        raise UnsupportedFormat
    return value


def _fallback_media_url(value: Any) -> str:
    if not isinstance(value, str) or len(value.encode()) > MAX_URL_BYTES:
        raise UnsupportedFormat
    try:
        parsed = urllib.parse.urlparse(value)
        query = urllib.parse.parse_qsl(parsed.query, keep_blank_values=True)
    except ValueError:
        raise UnsupportedFormat from None
    if (
        parsed.scheme not in {"http", "https"}
        or not parsed.hostname
        or parsed.username
        or parsed.password
        or parsed.fragment
    ):
        raise UnsupportedFormat
    if any(name.lower() in SENSITIVE_QUERY_NAMES for name, _ in query):
        raise UnsupportedFormat
    extension = parsed.path.rsplit(".", 1)[-1].lower() if "." in parsed.path else ""
    if extension not in {"mp4", "m4v"}:
        raise UnsupportedFormat
    return value


def _bilibili_video_id(url: str) -> str:
    try:
        parsed = urllib.parse.urlparse(url)
        query = urllib.parse.parse_qsl(parsed.query, keep_blank_values=True)
    except ValueError:
        raise InitialStateFallbackNotApplicable from None
    if (
        parsed.scheme != "https"
        or parsed.hostname not in {"www.bilibili.com", "bilibili.com"}
        or parsed.username
        or parsed.password
        or any(name.lower() in SENSITIVE_QUERY_NAMES for name, _ in query)
    ):
        raise InitialStateFallbackNotApplicable
    match = _BILIBILI_VIDEO_RE.fullmatch(parsed.path)
    if not match:
        raise InitialStateFallbackNotApplicable
    return match.group(1)


def _is_missing_initial_state_failure(error: DownloadError) -> bool:
    """Admit only the frozen BiliBiliIE missing-state failure.

    The pinned extractor preserves the originating ExtractorError on the
    DownloadError. Inspecting its structured fields keeps this decision
    independent of rendered diagnostics, while the exact message and
    extractor name prevent this compatibility path from becoming a general
    extractor retry.
    """
    if not isinstance(error, DownloadError):
        return False
    exc_info = getattr(error, "exc_info", None)
    if not isinstance(exc_info, tuple) or len(exc_info) < 2:
        return False
    cause = exc_info[1]
    return (
        isinstance(cause, ExtractorError)
        and cause.orig_msg == "Unable to extract initial state"
        and cause.ie == "BiliBili"
        and not cause.expected
    )


def _extract_initial_state_fallback(url: str) -> dict[str, Any]:
    """Run the narrow no-initial-state/detail-data continuation.

    This is an internal runtime-prep continuation. It only accepts a Bilibili
    video-shaped URL, performs all requests through BrokerRH, and emits one
    un-signed muxed durl. It is not a general extractor fallback.
    """
    video_id = _bilibili_video_id(url)
    with _ydl() as ydl:
        webpage_response = ydl.urlopen(Request(url))
        webpage = _fallback_response_body(webpage_response, json_body=False)
        lowered = webpage.lower()
        if "<html" not in lowered:
            raise UnsupportedFormat
        if "__initial_state__" in lowered:
            raise InitialStateFallbackNotApplicable
        redirected_url = getattr(webpage_response, "url", "")
        if "bangumi" in redirected_url.lower() or "bangumi" in lowered:
            raise UnsupportedFormat

        nav = _api_data(_fallback_json(ydl, f"{BILIBILI_API_ORIGIN}/x/web-interface/nav"))
        wbi_img = nav.get("wbi_img")
        if not isinstance(nav.get("isLogin"), bool) or not isinstance(wbi_img, dict):
            raise UnsupportedFormat
        if set(wbi_img) != {"img_url", "sub_url"}:
            raise UnsupportedFormat
        _required_public_url(wbi_img.get("img_url"))
        _required_public_url(wbi_img.get("sub_url"))
        view = _api_data(
            _fallback_json(ydl, f"{BILIBILI_API_ORIGIN}/x/web-interface/view?bvid={video_id}")
        )
        if view.get("bvid") != video_id:
            raise UnsupportedFormat
        view_title = _required_text(view.get("title"))
        view_pages = view.get("pages")
        if not isinstance(view_pages, list) or not view_pages or not isinstance(view_pages[0], dict):
            raise UnsupportedFormat
        view_cid = _required_cid(view_pages[0].get("cid"))

        detail = _api_data(
            _fallback_json(
                ydl,
                f"{BILIBILI_API_ORIGIN}/x/web-interface/view/detail?bvid={video_id}",
            )
        )
        detail_view = detail.get("View")
        if not isinstance(detail_view, dict) or detail_view.get("bvid") != video_id:
            raise UnsupportedFormat
        detail_title = _required_text(detail_view.get("title"))
        detail_pages = detail_view.get("pages")
        if (
            not isinstance(detail_pages, list)
            or not detail_pages
            or not isinstance(detail_pages[0], dict)
            or _required_cid(detail_pages[0].get("cid")) != view_cid
            or detail_title != view_title
        ):
            raise UnsupportedFormat

        playurl = _api_data(
            _fallback_json(
                ydl,
                f"{BILIBILI_API_ORIGIN}/x/player/playurl?bvid={video_id}&cid={view_cid}&fnval=0",
            )
        )
        durl = playurl.get("durl")
        if not isinstance(durl, list) or len(durl) != 1 or "dash" in playurl:
            raise UnsupportedFormat
        segment = durl[0]
        if not isinstance(segment, dict):
            raise UnsupportedFormat
        if any(
            name not in {"url", "backup_url", "size", "length", "order"}
            for name in segment
        ):
            raise UnsupportedFormat
        media_url = _fallback_media_url(segment.get("url"))
    return {
        "title": detail_title,
        "protection": "clear",
        "streams": [
            {
                "id": "fallback",
                "protocol": "http-file",
                "url": media_url,
                "public_headers": {},
                "upstream_access_ref": None,
            }
        ],
    }


def _formats(info: dict[str, Any]) -> list[dict[str, Any]]:
    formats = info.get("formats")
    if formats is not None:
        if not isinstance(formats, list):
            raise UnsupportedFormat
        return formats

    # GenericIE returns this bounded top-level shape when a non-HTML response
    # has a known media extension but no recognized media MIME type. Normalize
    # it into the same candidate shape as the formats path; do not infer a
    # format for unknown extensions or for non-direct extractor results.
    if info.get("direct") is not True:
        raise UnsupportedFormat
    raw_url = info.get("url")
    ext = info.get("ext")
    if (
        not isinstance(raw_url, str)
        or not isinstance(ext, str)
        or not ext
    ):
        raise UnsupportedFormat
    path = urllib.parse.urlparse(raw_url).path.lower()
    extension = path.rsplit(".", 1)[-1] if "." in path else ""
    if extension not in DIRECT_MEDIA_EXTENSIONS or ext.lower() != extension:
        raise UnsupportedFormat
    return [
        {
            "format_id": "direct",
            "url": raw_url,
            "ext": ext,
            "protocol": "m3u8_native" if extension == "m3u8" else None,
            "vcodec": None,
            "acodec": None,
            "http_headers": info.get("http_headers"),
        }
    ]


def _extract(url: str) -> dict[str, Any]:
    try:
        with _ydl() as ydl:
            info = ydl.extract_info(url, download=False)
    except DownloadError as error:
        if not _is_missing_initial_state_failure(error):
            raise
        try:
            _bilibili_video_id(url)
        except InitialStateFallbackNotApplicable:
            raise
        return _extract_initial_state_fallback(url)

    if not isinstance(info, dict) or info.get("_type") in {"playlist", "multi_video"}:
        raise UnsupportedFormat
    formats = _formats(info)

    for fmt in reversed(formats):
        if not isinstance(fmt, dict) or not _is_muxed(fmt):
            continue
        raw_url = fmt.get("url")
        if not isinstance(raw_url, str):
            continue
        parsed = urllib.parse.urlparse(raw_url)
        if parsed.scheme not in {"http", "https"} or not parsed.netloc:
            continue
        protocol = fmt.get("protocol")
        if protocol in {"m3u8", "m3u8_native"}:
            stream_url = fmt.get("manifest_url") or raw_url
            output_protocol = "hls"
        elif protocol in {"http", "https", None} and (
            not info.get("direct") or fmt.get("ext") not in {None, "unknown", "unknown_video"}
        ):
            stream_url = raw_url
            output_protocol = "http-file"
        else:
            continue
        if not isinstance(stream_url, str):
            continue
        stream_parsed = urllib.parse.urlparse(stream_url)
        if stream_parsed.scheme not in {"http", "https"} or not stream_parsed.netloc:
            continue
        title = info.get("title")
        if not isinstance(title, str) or not title.strip():
            raise UnsupportedFormat
        return {
            "title": title,
            "protection": "clear",
            "streams": [
                {
                    "id": str(fmt.get("format_id") or "primary"),
                    "protocol": output_protocol,
                    "url": stream_url,
                    "public_headers": _public_headers(info, fmt),
                    "upstream_access_ref": None,
                }
            ],
        }
    raise UnsupportedFormat


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
    global _FAILURE_CODE
    if __version__ != EXPECTED_VERSION or os.environ.get("YTDLP_EXPECTED_VERSION") != EXPECTED_VERSION:
        return 42
    if sys.argv[1:] and sys.argv[1] == "crash":
        os._exit(7)
    action = sys.argv[1] if len(sys.argv) > 1 else "probe"
    url = sys.argv[2] if len(sys.argv) > 2 else "https://fixture.example.test/media"
    _FAILURE_CODE = None
    try:
        if action == "probe":
            result = _probe(url)
        elif action == "extract":
            result = _extract(url)
        elif action == "classification-request-policy":
            with _ydl() as ydl:
                ydl.urlopen(Request(url, headers={"Authorization": "Bearer policy-sentinel"}))
            raise AssertionError("request policy fixture unexpectedly passed")
        elif action == "classification-extractor":
            raise DownloadError(
                "extractor failed at https://fixture.invalid/watch?token=query-sentinel "
                "Authorization=Bearer credential-sentinel"
            )
        elif action == "classification-unexpected":
            raise RuntimeError(
                "unexpected https://fixture.invalid/?sig=unexpected-sentinel"
            )
        elif action == "classification-malformed":
            sys.stdout.write("{")  # deterministic parser-negative fixture
            sys.stdout.flush()
            return 0
        elif action == "classification-unknown":
            result = {"error": "UNKNOWN_WORKER_FAILURE"}
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
    except UnsupportedFormat:
        result = {"error": UNSUPPORTED_FORMAT}
    except (RequestPolicyFailure, BrokerFailure, DownloadError, ExtractorError):
        result = {"error": _FAILURE_CODE or EXTRACTOR_FAILURE}
    except Exception:
        result = {"error": _FAILURE_CODE or UNEXPECTED_WORKER_FAILURE}
    sys.stdout.write(json.dumps(result, separators=(",", ":")))
    sys.stdout.flush()
    if _BROKER_STREAM is not None:
        _BROKER_STREAM.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
