"""Repository-owned, anonymous-only yt-dlp API worker.

The worker receives no caller argv/config/plugin/profile/cookie authority. Its
only network capability is the inherited fd 3 socketpair. The Rust launcher
installs the inherited seccomp/no-new-privs boundary before this module runs.
"""

from __future__ import annotations

import binascii
import codecs
import errno
import io
import json
import os
import socket
import subprocess
import sys
import urllib.parse
import re
import zlib
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
# Keep fallback documents within the existing R008 response ceiling while
# retaining the full runtime bound used by the broker and adapter.
MAX_FALLBACK_TEXT_BYTES = MAX_BODY
# Webpages are the only fallback response whose normalized text is consumed as
# a marker stream rather than parsed as a complete JSON document.  512 KiB is
# deliberately above the raw 96 KiB broker ceiling to accommodate bounded
# gzip/deflate expansion, while staying below the task's 1 MiB maximum.  The
# scanner retains only marker state and a short cross-chunk suffix.
MAX_WEBPAGE_SCAN_BYTES = 512 * 1024
RESPONSE_STREAM_CHUNK_BYTES = 16 * 1024
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

PRE_FALLBACK = "PRE_FALLBACK"
FALLBACK_WEBPAGE = "FALLBACK_WEBPAGE"
FALLBACK_NAV = "FALLBACK_NAV"
FALLBACK_VIEW = "FALLBACK_VIEW"
FALLBACK_DETAIL = "FALLBACK_DETAIL"
FALLBACK_PLAYURL = "FALLBACK_PLAYURL"
MEDIA_SHAPE = "MEDIA_SHAPE"
UNCLASSIFIED = "UNCLASSIFIED"
UNSUPPORTED_STAGES = frozenset(
    {
        PRE_FALLBACK,
        FALLBACK_WEBPAGE,
        FALLBACK_NAV,
        FALLBACK_VIEW,
        FALLBACK_DETAIL,
        FALLBACK_PLAYURL,
        MEDIA_SHAPE,
        UNCLASSIFIED,
    }
)
RESPONSE_STATUS = "RESPONSE_STATUS"
RESPONSE_BODY_TOO_LARGE = "RESPONSE_BODY_TOO_LARGE"
RESPONSE_ENCODING = "RESPONSE_ENCODING"
RESPONSE_JSON = "RESPONSE_JSON"
RESPONSE_SECRET_FIELD = "RESPONSE_SECRET_FIELD"
RESPONSE_READ = "RESPONSE_READ"
WEBPAGE_NOT_HTML = "WEBPAGE_NOT_HTML"
WEBPAGE_BANGUMI = "WEBPAGE_BANGUMI"
NAV_API_ENVELOPE = "NAV_API_ENVELOPE"
NAV_SHAPE = "NAV_SHAPE"
NAV_WBI_SHAPE = "NAV_WBI_SHAPE"
NAV_WBI_URL = "NAV_WBI_URL"
VIEW_API_ENVELOPE = "VIEW_API_ENVELOPE"
VIEW_ID_MISMATCH = "VIEW_ID_MISMATCH"
VIEW_TITLE = "VIEW_TITLE"
VIEW_PAGES = "VIEW_PAGES"
VIEW_CID = "VIEW_CID"
DETAIL_API_ENVELOPE = "DETAIL_API_ENVELOPE"
DETAIL_SHAPE = "DETAIL_SHAPE"
DETAIL_ID_MISMATCH = "DETAIL_ID_MISMATCH"
DETAIL_TITLE = "DETAIL_TITLE"
DETAIL_PAGES = "DETAIL_PAGES"
DETAIL_CID_MISMATCH = "DETAIL_CID_MISMATCH"
DETAIL_TITLE_MISMATCH = "DETAIL_TITLE_MISMATCH"
PLAYURL_API_ENVELOPE = "PLAYURL_API_ENVELOPE"
PLAYURL_DURL_SHAPE = "PLAYURL_DURL_SHAPE"
PLAYURL_DASH_PRESENT = "PLAYURL_DASH_PRESENT"
PLAYURL_SEGMENT_SHAPE = "PLAYURL_SEGMENT_SHAPE"
PLAYURL_SEGMENT_FIELDS = "PLAYURL_SEGMENT_FIELDS"
MEDIA_URL_SHAPE = "MEDIA_URL_SHAPE"
MEDIA_URL_SENSITIVE_QUERY = "MEDIA_URL_SENSITIVE_QUERY"
MEDIA_EXTENSION = "MEDIA_EXTENSION"
MEDIA_HEADERS = "MEDIA_HEADERS"
MEDIA_TITLE = "MEDIA_TITLE"
MEDIA_NO_MUXED_STREAM = "MEDIA_NO_MUXED_STREAM"
UNSUPPORTED_REASONS = frozenset(
    {
        RESPONSE_STATUS,
        RESPONSE_BODY_TOO_LARGE,
        RESPONSE_ENCODING,
        RESPONSE_JSON,
        RESPONSE_SECRET_FIELD,
        RESPONSE_READ,
        WEBPAGE_NOT_HTML,
        WEBPAGE_BANGUMI,
        NAV_API_ENVELOPE,
        NAV_SHAPE,
        NAV_WBI_SHAPE,
        NAV_WBI_URL,
        VIEW_API_ENVELOPE,
        VIEW_ID_MISMATCH,
        VIEW_TITLE,
        VIEW_PAGES,
        VIEW_CID,
        DETAIL_API_ENVELOPE,
        DETAIL_SHAPE,
        DETAIL_ID_MISMATCH,
        DETAIL_TITLE,
        DETAIL_PAGES,
        DETAIL_CID_MISMATCH,
        DETAIL_TITLE_MISMATCH,
        PLAYURL_API_ENVELOPE,
        PLAYURL_DURL_SHAPE,
        PLAYURL_DASH_PRESENT,
        PLAYURL_SEGMENT_SHAPE,
        PLAYURL_SEGMENT_FIELDS,
        MEDIA_URL_SHAPE,
        MEDIA_URL_SENSITIVE_QUERY,
        MEDIA_EXTENSION,
        MEDIA_HEADERS,
        MEDIA_TITLE,
        MEDIA_NO_MUXED_STREAM,
        UNCLASSIFIED,
    }
)
_COMMON_RESPONSE_REASONS = frozenset(
    {
        RESPONSE_STATUS,
        RESPONSE_BODY_TOO_LARGE,
        RESPONSE_ENCODING,
        RESPONSE_JSON,
        RESPONSE_SECRET_FIELD,
        RESPONSE_READ,
    }
)
_REASONS_BY_STAGE = {
    PRE_FALLBACK: frozenset({UNCLASSIFIED, MEDIA_NO_MUXED_STREAM}),
    FALLBACK_WEBPAGE: _COMMON_RESPONSE_REASONS | frozenset({WEBPAGE_NOT_HTML, WEBPAGE_BANGUMI}),
    FALLBACK_NAV: _COMMON_RESPONSE_REASONS
    | frozenset({NAV_API_ENVELOPE, NAV_SHAPE, NAV_WBI_SHAPE, NAV_WBI_URL}),
    FALLBACK_VIEW: _COMMON_RESPONSE_REASONS
    | frozenset({VIEW_API_ENVELOPE, VIEW_ID_MISMATCH, VIEW_TITLE, VIEW_PAGES, VIEW_CID}),
    FALLBACK_DETAIL: _COMMON_RESPONSE_REASONS
    | frozenset(
        {
            DETAIL_API_ENVELOPE,
            DETAIL_SHAPE,
            DETAIL_ID_MISMATCH,
            DETAIL_TITLE,
            DETAIL_PAGES,
            DETAIL_CID_MISMATCH,
            DETAIL_TITLE_MISMATCH,
        }
    ),
    FALLBACK_PLAYURL: _COMMON_RESPONSE_REASONS
    | frozenset(
        {
            PLAYURL_API_ENVELOPE,
            PLAYURL_DURL_SHAPE,
            PLAYURL_DASH_PRESENT,
            PLAYURL_SEGMENT_SHAPE,
            PLAYURL_SEGMENT_FIELDS,
        }
    ),
    MEDIA_SHAPE: frozenset(
        {
            MEDIA_URL_SHAPE,
            MEDIA_URL_SENSITIVE_QUERY,
            MEDIA_EXTENSION,
            MEDIA_HEADERS,
            MEDIA_TITLE,
            MEDIA_NO_MUXED_STREAM,
        }
    ),
    UNCLASSIFIED: frozenset({UNCLASSIFIED}),
}


class _ResponseEncodingError(Exception):
    """The bounded response normalization contract rejected the input."""


class _ResponseBodyTooLarge(Exception):
    """The normalized response exceeded its fixed fallback body bound."""


# This is deliberately a small, explicit compatibility surface. The broker
# still admits at most MAX_BODY raw response bytes; the decoder below adds no
# new network or storage authority. A response may carry one content-coding
# only, and all text is normalized to UTF-8 before fallback admission.
ADMITTED_CONTENT_CODINGS = frozenset({"identity", "gzip", "deflate"})
ADMITTED_CHARSETS = frozenset({"utf-8"})


class _WebpageScan:
    """Marker-only result for the bounded webpage admission scan."""

    __slots__ = ("has_html", "has_initial_state", "has_bangumi")

    def __init__(self, *, has_html: bool, has_initial_state: bool, has_bangumi: bool):
        self.has_html = has_html
        self.has_initial_state = has_initial_state
        self.has_bangumi = has_bangumi


def _response_header(response: Response, name: str) -> str | None:
    """Return one well-formed response header, rejecting ambiguity."""
    values: list[str] = []
    headers = getattr(response, "headers", {})
    try:
        items = headers.items()
    except AttributeError:
        raise _ResponseEncodingError from None
    for header_name, value in items:
        if not isinstance(header_name, str) or not isinstance(value, str):
            raise _ResponseEncodingError
        if header_name.lower() == name:
            values.append(value)
    if len(values) > 1:
        raise _ResponseEncodingError
    return values[0] if values else None


def _response_content_coding(response: Response) -> str:
    value = _response_header(response, "content-encoding")
    if value is None:
        return "identity"
    tokens = tuple(token.strip().lower() for token in value.split(","))
    if len(tokens) != 1 or tokens[0] not in ADMITTED_CONTENT_CODINGS:
        raise _ResponseEncodingError
    return tokens[0]


def _response_charset(response: Response) -> str:
    value = _response_header(response, "content-type")
    if value is None:
        return "utf-8"
    parts = value.split(";")
    if not parts[0].strip():
        raise _ResponseEncodingError
    charset: str | None = None
    for parameter in parts[1:]:
        name_and_value = parameter.split("=", 1)
        if len(name_and_value) != 2:
            raise _ResponseEncodingError
        parameter_name, parameter_value = (part.strip().lower() for part in name_and_value)
        if parameter_name != "charset":
            continue
        if charset is not None:
            raise _ResponseEncodingError
        if len(parameter_value) >= 2 and parameter_value[0] == parameter_value[-1] == '"':
            parameter_value = parameter_value[1:-1].strip()
        charset = parameter_value
    if charset is None:
        charset = "utf-8"
    if charset not in ADMITTED_CHARSETS:
        raise _ResponseEncodingError
    return charset


def _bounded_inflate(body: bytes, *, wbits: int) -> bytes:
    decoder = zlib.decompressobj(wbits)
    try:
        normalized = decoder.decompress(body, MAX_FALLBACK_TEXT_BYTES + 1)
        if len(normalized) > MAX_FALLBACK_TEXT_BYTES:
            raise _ResponseBodyTooLarge
        flushed = decoder.flush(MAX_FALLBACK_TEXT_BYTES + 1 - len(normalized))
    except _ResponseBodyTooLarge:
        raise
    except (ValueError, zlib.error):
        raise _ResponseEncodingError from None
    normalized += flushed
    if len(normalized) > MAX_FALLBACK_TEXT_BYTES:
        raise _ResponseBodyTooLarge
    # Only one complete coding is admitted. This rejects truncation, trailing
    # bytes and concatenated members instead of silently changing semantics.
    if not decoder.eof or decoder.unused_data or decoder.unconsumed_tail:
        raise _ResponseEncodingError
    return normalized


def _normalize_response_body(response: Response, body: bytes) -> str:
    """Normalize one bounded broker response into the admitted UTF-8 text."""
    if not isinstance(body, bytes):
        raise _ResponseEncodingError
    coding = _response_content_coding(response)
    if coding == "gzip":
        body = _bounded_inflate(body, wbits=16 + zlib.MAX_WBITS)
    elif coding == "deflate":
        body = _bounded_inflate(body, wbits=zlib.MAX_WBITS)
    # The charset is checked even for identity so metadata cannot silently
    # disagree with the bytes admitted to JSON/HTML parsing.
    charset = _response_charset(response)
    try:
        text = body.decode(charset)
    except UnicodeDecodeError:
        raise _ResponseEncodingError from None
    if len(text.encode("utf-8")) > MAX_FALLBACK_TEXT_BYTES:
        raise _ResponseBodyTooLarge
    return text


def _scan_webpage_body(
    response: Response,
    body: bytes,
    *,
    chunk_size: int = RESPONSE_STREAM_CHUNK_BYTES,
) -> _WebpageScan:
    """Incrementally normalize a webpage and retain only admission markers.

    ``body`` is already bounded by the raw broker limit.  Compressed output is
    delivered in bounded pieces to a strict incremental UTF-8 decoder; the
    marker suffix is the only cross-chunk state retained.  This keeps webpage
    expansion bounded without making a second full-page allocation.
    """
    if not isinstance(body, bytes) or not 1 <= chunk_size <= RESPONSE_STREAM_CHUNK_BYTES:
        raise _ResponseEncodingError
    coding = _response_content_coding(response)
    charset = _response_charset(response)
    try:
        decoder = codecs.getincrementaldecoder(charset)(errors="strict")
    except (LookupError, TypeError):
        raise _ResponseEncodingError from None

    markers = ("<html", "__initial_state__", "bangumi")
    marker_tail = ""
    found = [False, False, False]
    normalized_bytes = 0

    def consume(normalized: bytes, *, final: bool = False) -> None:
        nonlocal marker_tail, normalized_bytes
        if not isinstance(normalized, bytes):
            raise _ResponseEncodingError
        normalized_bytes += len(normalized)
        if normalized_bytes > MAX_WEBPAGE_SCAN_BYTES:
            raise _ResponseBodyTooLarge
        try:
            text = decoder.decode(normalized, final=final)
        except UnicodeDecodeError:
            raise _ResponseEncodingError from None
        for character in text:
            if "A" <= character <= "Z":
                character = chr(ord(character) + (ord("a") - ord("A")))
            marker_tail = (marker_tail + character)[-max(map(len, markers)) :]
            for index, marker in enumerate(markers):
                if marker in marker_tail:
                    found[index] = True

    if coding == "identity":
        for offset in range(0, len(body), chunk_size):
            consume(body[offset : offset + chunk_size])
    else:
        wbits = 16 + zlib.MAX_WBITS if coding == "gzip" else zlib.MAX_WBITS
        inflater = zlib.decompressobj(wbits)
        for offset in range(0, len(body), chunk_size):
            pending = body[offset : offset + chunk_size]
            while pending:
                if inflater.eof:
                    raise _ResponseEncodingError
                remaining = MAX_WEBPAGE_SCAN_BYTES - normalized_bytes
                max_output = min(RESPONSE_STREAM_CHUNK_BYTES, remaining + 1)
                try:
                    normalized = inflater.decompress(pending, max_output)
                except (ValueError, zlib.error):
                    raise _ResponseEncodingError from None
                consume(normalized)
                if inflater.unused_data:
                    # This also rejects concatenated members and any trailing
                    # bytes after an otherwise valid gzip/deflate stream.
                    raise _ResponseEncodingError
                next_pending = inflater.unconsumed_tail
                if next_pending == pending and not normalized:
                    raise _ResponseEncodingError
                pending = next_pending
        remaining = MAX_WEBPAGE_SCAN_BYTES - normalized_bytes
        try:
            consume(inflater.flush(min(RESPONSE_STREAM_CHUNK_BYTES, remaining + 1)))
        except (ValueError, zlib.error):
            raise _ResponseEncodingError from None
        if not inflater.eof or inflater.unused_data or inflater.unconsumed_tail:
            raise _ResponseEncodingError

    consume(b"", final=True)
    return _WebpageScan(
        has_html=found[0],
        has_initial_state=found[1],
        has_bangumi=found[2],
    )


class UnsupportedFormat(Exception):
    """The fixed first-playback policy cannot represent the extraction."""

    def __init__(self, stage: str = UNCLASSIFIED, reason: str = UNCLASSIFIED):
        # Keep both wire values repository-owned. Invalid stage/reason pairs
        # fail closed to the single unclassified pair.
        if (
            isinstance(stage, str)
            and stage in UNSUPPORTED_STAGES
            and isinstance(reason, str)
            and reason in _REASONS_BY_STAGE.get(stage, ())
        ):
            self.stage = stage
            self.reason = reason
        else:
            self.stage = UNCLASSIFIED
            self.reason = UNCLASSIFIED
        super().__init__()


class RequestPolicyFailure(RequestError):
    """A repository-owned request rule rejected the operation."""


class BrokerFailure(RequestError):
    """The bounded broker capability rejected or failed the operation."""


class InitialStateFallbackNotApplicable(UnsupportedFormat):
    """The opt-in continuation does not own this response shape."""

    def __init__(self):
        super().__init__(UNCLASSIFIED)


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
        raise UnsupportedFormat(MEDIA_SHAPE, MEDIA_HEADERS)
    public: dict[str, str] = {}
    for name, value in headers.items():
        if not isinstance(name, str) or not isinstance(value, str):
            raise UnsupportedFormat(MEDIA_SHAPE, MEDIA_HEADERS)
        if _is_secret_header(name, value):
            raise UnsupportedFormat(MEDIA_SHAPE, MEDIA_HEADERS)
        public[name] = value
    return public


def _is_muxed(fmt: dict[str, Any]) -> bool:
    # Direct video responses from the generic extractor have no codec fields;
    # a present value means that codec is known, while only the literal
    # ``none`` is an explicit audio-only/video-only marker.
    return fmt.get("vcodec") != "none" and fmt.get("acodec") != "none"


def _reject_secret_fields(value: Any, stage: str) -> None:
    """Reject secret-bearing fixture/API data before it can be normalized."""
    if isinstance(value, dict):
        for name, child in value.items():
            if not isinstance(name, str):
                raise UnsupportedFormat(stage, RESPONSE_SECRET_FIELD)
            normalized = name.lower().replace("_", "-")
            if normalized in SECRET_FIELD_NAMES:
                raise UnsupportedFormat(stage, RESPONSE_SECRET_FIELD)
            _reject_secret_fields(child, stage)
    elif isinstance(value, list):
        for child in value:
            _reject_secret_fields(child, stage)


def _fallback_response_body(response: Response, *, json_body: bool, stage: str) -> Any:
    try:
        status = int(response.status)
    except (TypeError, ValueError):
        raise UnsupportedFormat(stage, RESPONSE_STATUS) from None
    if not 200 <= status < 300:
        raise UnsupportedFormat(stage, RESPONSE_STATUS)
    try:
        # JSON keeps the established 96 KiB normalized/raw read bound.  The
        # webpage scanner admits the same raw MAX_BODY authority, then applies
        # its separate bounded post-coding scan ceiling.
        body = response.read((MAX_FALLBACK_TEXT_BYTES if json_body else MAX_BODY) + 1)
    except Exception:
        raise UnsupportedFormat(stage, RESPONSE_READ) from None
    if len(body) > (MAX_FALLBACK_TEXT_BYTES if json_body else MAX_BODY):
        raise UnsupportedFormat(stage, RESPONSE_BODY_TOO_LARGE)
    if not json_body and stage == FALLBACK_WEBPAGE:
        try:
            return _scan_webpage_body(response, body)
        except _ResponseBodyTooLarge:
            raise UnsupportedFormat(stage, RESPONSE_BODY_TOO_LARGE) from None
        except _ResponseEncodingError:
            raise UnsupportedFormat(stage, RESPONSE_ENCODING) from None
    try:
        text = _normalize_response_body(response, body)
    except _ResponseBodyTooLarge:
        raise UnsupportedFormat(stage, RESPONSE_BODY_TOO_LARGE) from None
    except _ResponseEncodingError:
        raise UnsupportedFormat(stage, RESPONSE_ENCODING) from None
    if json_body:
        try:
            value = json.loads(text)
        except (TypeError, ValueError):
            raise UnsupportedFormat(stage, RESPONSE_JSON) from None
        if not isinstance(value, dict):
            raise UnsupportedFormat(stage, RESPONSE_JSON)
        _reject_secret_fields(value, stage)
        return value
    return text


def _fallback_json(ydl: BrokerOnlyYDL, url: str, stage: str) -> dict[str, Any]:
    response = ydl.urlopen(Request(url))
    value = _fallback_response_body(response, json_body=True, stage=stage)
    if not isinstance(value, dict):
        raise UnsupportedFormat(stage, RESPONSE_JSON)
    return value


def _api_data(payload: dict[str, Any], stage: str, reason: str) -> dict[str, Any]:
    if (
        payload.get("code") != 0
        or not isinstance(payload.get("data"), dict)
        or any(name not in {"code", "message", "ttl", "data"} for name in payload)
    ):
        raise UnsupportedFormat(stage, reason)
    return payload["data"]


def _required_text(
    value: Any, *, stage: str, reason: str, max_bytes: int = MAX_TITLE_BYTES
) -> str:
    if not isinstance(value, str) or not value.strip():
        raise UnsupportedFormat(stage, reason)
    try:
        if len(value.encode()) > max_bytes:
            raise UnsupportedFormat(stage, reason)
    except UnicodeEncodeError:
        raise UnsupportedFormat(stage, reason) from None
    return value


def _required_cid(value: Any, stage: str, reason: str) -> str:
    if isinstance(value, bool):
        raise UnsupportedFormat(stage, reason)
    if isinstance(value, int):
        cid = str(value)
    elif isinstance(value, str) and value.isdecimal():
        cid = value
    else:
        raise UnsupportedFormat(stage, reason)
    if not 1 <= len(cid) <= 32:
        raise UnsupportedFormat(stage, reason)
    return cid


def _required_public_url(value: Any, stage: str, reason: str) -> str:
    if not isinstance(value, str) or len(value.encode()) > MAX_URL_BYTES:
        raise UnsupportedFormat(stage, reason)
    try:
        parsed = urllib.parse.urlparse(value)
        query = urllib.parse.parse_qsl(parsed.query, keep_blank_values=True)
    except ValueError:
        raise UnsupportedFormat(stage, reason) from None
    if (
        parsed.scheme != "https"
        or not parsed.hostname
        or parsed.username
        or parsed.password
        or parsed.fragment
        or parsed.query
        or query
    ):
        raise UnsupportedFormat(stage, reason)
    return value


def _fallback_media_url(value: Any) -> str:
    if not isinstance(value, str) or len(value.encode()) > MAX_URL_BYTES:
        raise UnsupportedFormat(MEDIA_SHAPE, MEDIA_URL_SHAPE)
    try:
        parsed = urllib.parse.urlparse(value)
        query = urllib.parse.parse_qsl(parsed.query, keep_blank_values=True)
    except ValueError:
        raise UnsupportedFormat(MEDIA_SHAPE, MEDIA_URL_SHAPE) from None
    if (
        parsed.scheme not in {"http", "https"}
        or not parsed.hostname
        or parsed.username
        or parsed.password
        or parsed.fragment
    ):
        raise UnsupportedFormat(MEDIA_SHAPE, MEDIA_URL_SHAPE)
    if any(name.lower() in SENSITIVE_QUERY_NAMES for name, _ in query):
        raise UnsupportedFormat(MEDIA_SHAPE, MEDIA_URL_SENSITIVE_QUERY)
    extension = parsed.path.rsplit(".", 1)[-1].lower() if "." in parsed.path else ""
    if extension not in {"mp4", "m4v"}:
        raise UnsupportedFormat(MEDIA_SHAPE, MEDIA_EXTENSION)
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
        webpage = _fallback_response_body(
            webpage_response, json_body=False, stage=FALLBACK_WEBPAGE
        )
        if not isinstance(webpage, _WebpageScan):
            raise UnsupportedFormat(FALLBACK_WEBPAGE, WEBPAGE_NOT_HTML)
        if not webpage.has_html:
            raise UnsupportedFormat(FALLBACK_WEBPAGE, WEBPAGE_NOT_HTML)
        if webpage.has_initial_state:
            raise InitialStateFallbackNotApplicable
        redirected_url = getattr(webpage_response, "url", "")
        if "bangumi" in redirected_url.lower() or webpage.has_bangumi:
            raise UnsupportedFormat(FALLBACK_WEBPAGE, WEBPAGE_BANGUMI)

        nav = _api_data(
            _fallback_json(
                ydl, f"{BILIBILI_API_ORIGIN}/x/web-interface/nav", FALLBACK_NAV
            ),
            FALLBACK_NAV,
            NAV_API_ENVELOPE,
        )
        wbi_img = nav.get("wbi_img")
        if not isinstance(nav.get("isLogin"), bool) or not isinstance(wbi_img, dict):
            raise UnsupportedFormat(FALLBACK_NAV, NAV_SHAPE)
        if set(wbi_img) != {"img_url", "sub_url"}:
            raise UnsupportedFormat(FALLBACK_NAV, NAV_WBI_SHAPE)
        _required_public_url(wbi_img.get("img_url"), FALLBACK_NAV, NAV_WBI_URL)
        _required_public_url(wbi_img.get("sub_url"), FALLBACK_NAV, NAV_WBI_URL)
        view = _api_data(
            _fallback_json(
                ydl,
                f"{BILIBILI_API_ORIGIN}/x/web-interface/view?bvid={video_id}",
                FALLBACK_VIEW,
            ),
            FALLBACK_VIEW,
            VIEW_API_ENVELOPE,
        )
        if view.get("bvid") != video_id:
            raise UnsupportedFormat(FALLBACK_VIEW, VIEW_ID_MISMATCH)
        view_title = _required_text(
            view.get("title"), stage=FALLBACK_VIEW, reason=VIEW_TITLE
        )
        view_pages = view.get("pages")
        if not isinstance(view_pages, list) or not view_pages or not isinstance(view_pages[0], dict):
            raise UnsupportedFormat(FALLBACK_VIEW, VIEW_PAGES)
        view_cid = _required_cid(view_pages[0].get("cid"), FALLBACK_VIEW, VIEW_CID)

        detail = _api_data(
            _fallback_json(
                ydl,
                f"{BILIBILI_API_ORIGIN}/x/web-interface/view/detail?bvid={video_id}",
                FALLBACK_DETAIL,
            ),
            FALLBACK_DETAIL,
            DETAIL_API_ENVELOPE,
        )
        detail_view = detail.get("View")
        if not isinstance(detail_view, dict):
            raise UnsupportedFormat(FALLBACK_DETAIL, DETAIL_SHAPE)
        if detail_view.get("bvid") != video_id:
            raise UnsupportedFormat(FALLBACK_DETAIL, DETAIL_ID_MISMATCH)
        detail_title = _required_text(
            detail_view.get("title"), stage=FALLBACK_DETAIL, reason=DETAIL_TITLE
        )
        detail_pages = detail_view.get("pages")
        if not isinstance(detail_pages, list) or not detail_pages or not isinstance(detail_pages[0], dict):
            raise UnsupportedFormat(FALLBACK_DETAIL, DETAIL_PAGES)
        detail_cid = _required_cid(
            detail_pages[0].get("cid"), FALLBACK_DETAIL, DETAIL_CID_MISMATCH
        )
        if detail_cid != view_cid:
            raise UnsupportedFormat(FALLBACK_DETAIL, DETAIL_CID_MISMATCH)
        if detail_title != view_title:
            raise UnsupportedFormat(FALLBACK_DETAIL, DETAIL_TITLE_MISMATCH)

        playurl = _api_data(
            _fallback_json(
                ydl,
                f"{BILIBILI_API_ORIGIN}/x/player/playurl?bvid={video_id}&cid={view_cid}&fnval=0",
                FALLBACK_PLAYURL,
            ),
            FALLBACK_PLAYURL,
            PLAYURL_API_ENVELOPE,
        )
        if "dash" in playurl:
            raise UnsupportedFormat(FALLBACK_PLAYURL, PLAYURL_DASH_PRESENT)
        durl = playurl.get("durl")
        if not isinstance(durl, list) or len(durl) != 1:
            raise UnsupportedFormat(FALLBACK_PLAYURL, PLAYURL_DURL_SHAPE)
        segment = durl[0]
        if not isinstance(segment, dict):
            raise UnsupportedFormat(FALLBACK_PLAYURL, PLAYURL_SEGMENT_SHAPE)
        if any(
            name not in {"url", "backup_url", "size", "length", "order"}
            for name in segment
        ):
            raise UnsupportedFormat(FALLBACK_PLAYURL, PLAYURL_SEGMENT_FIELDS)
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
            raise UnsupportedFormat(PRE_FALLBACK, UNCLASSIFIED)
        return formats

    # GenericIE returns this bounded top-level shape when a non-HTML response
    # has a known media extension but no recognized media MIME type. Normalize
    # it into the same candidate shape as the formats path; do not infer a
    # format for unknown extensions or for non-direct extractor results.
    if info.get("direct") is not True:
        raise UnsupportedFormat(PRE_FALLBACK, UNCLASSIFIED)
    raw_url = info.get("url")
    ext = info.get("ext")
    if (
        not isinstance(raw_url, str)
        or not isinstance(ext, str)
        or not ext
    ):
        raise UnsupportedFormat(PRE_FALLBACK, UNCLASSIFIED)
    path = urllib.parse.urlparse(raw_url).path.lower()
    extension = path.rsplit(".", 1)[-1] if "." in path else ""
    if extension not in DIRECT_MEDIA_EXTENSIONS or ext.lower() != extension:
        raise UnsupportedFormat(PRE_FALLBACK, UNCLASSIFIED)
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
        raise UnsupportedFormat(PRE_FALLBACK, UNCLASSIFIED)
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
            raise UnsupportedFormat(MEDIA_SHAPE, MEDIA_TITLE)
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
    raise UnsupportedFormat(PRE_FALLBACK, MEDIA_NO_MUXED_STREAM)


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
    except UnsupportedFormat as error:
        result = {
            "error": UNSUPPORTED_FORMAT,
            "unsupported_stage": error.stage,
            "fallback_reason": error.reason,
        }
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
