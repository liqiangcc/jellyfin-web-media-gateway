import io
import gzip
import importlib.util
import unittest
import zlib
from pathlib import Path

from yt_dlp.extractor.generic import GenericIE
from yt_dlp.networking import Response


WORKER_PATH = Path(__file__).parents[1] / "worker" / "worker.py"
SPEC = importlib.util.spec_from_file_location("generic_ytdlp_worker", WORKER_PATH)
assert SPEC and SPEC.loader
worker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(worker)


class DirectFallbackNormalizationTest(unittest.TestCase):
    @staticmethod
    def _response(body, headers=None):
        return Response(
            io.BytesIO(body),
            "https://fixture.example.test/fallback",
            headers or {"Content-Type": "text/html; charset=utf-8"},
            status=200,
        )

    def test_identity_utf8_webpage_and_json_are_unchanged(self):
        webpage = worker._fallback_response_body(
            self._response(b"<html><body>fixture</body></html>"),
            json_body=False,
            stage=worker.FALLBACK_WEBPAGE,
        )
        self.assertIsInstance(webpage, worker._WebpageScan)
        self.assertTrue(webpage.has_html)
        self.assertFalse(webpage.has_initial_state)
        self.assertFalse(webpage.has_bangumi)

        payload = b'{"code":0,"data":{"fixture":true}}'
        value = worker._fallback_response_body(
            self._response(payload, {"Content-Type": "application/json"}),
            json_body=True,
            stage=worker.FALLBACK_NAV,
        )
        self.assertEqual(value, {"code": 0, "data": {"fixture": True}})

    def test_each_admitted_content_coding_normalizes_before_admission(self):
        payload = b"<html><body>encoded fixture</body></html>"
        for coding, encoded in (
            ("identity", payload),
            ("gzip", gzip.compress(payload, mtime=0)),
            ("deflate", zlib.compress(payload)),
        ):
            with self.subTest(coding=coding):
                body = self._response(
                    encoded,
                    {
                        "Content-Type": "text/html; charset=UTF-8",
                        "Content-Encoding": coding,
                    },
                )
                scan = worker._fallback_response_body(
                    body,
                    json_body=False,
                    stage=worker.FALLBACK_WEBPAGE,
                )
                self.assertTrue(scan.has_html)
                self.assertFalse(scan.has_initial_state)
                self.assertFalse(scan.has_bangumi)

    def test_malformed_or_truncated_admitted_codings_fail_closed(self):
        payload = b"<html>fixture</html>"
        for coding, encoded in (
            ("gzip", gzip.compress(payload, mtime=0)[:-1]),
            ("deflate", zlib.compress(payload)[:-1]),
        ):
            with self.subTest(coding=coding):
                with self.assertRaises(worker.UnsupportedFormat) as raised:
                    worker._fallback_response_body(
                        self._response(
                            encoded,
                            {
                                "Content-Type": "text/html",
                                "Content-Encoding": coding,
                            },
                        ),
                        json_body=False,
                        stage=worker.FALLBACK_WEBPAGE,
                    )
                self.assertEqual(
                    (raised.exception.stage, raised.exception.reason),
                    (worker.FALLBACK_WEBPAGE, worker.RESPONSE_ENCODING),
                )

    def test_unknown_ambiguous_and_nested_codings_fail_closed(self):
        payload = b"<html>fixture</html>"
        cases = (
            ("br", payload),
            ("gzip, deflate", gzip.compress(payload, mtime=0)),
            ("gzip", gzip.compress(gzip.compress(payload, mtime=0), mtime=0)),
        )
        for coding, encoded in cases:
            with self.subTest(coding=coding):
                with self.assertRaises(worker.UnsupportedFormat) as raised:
                    worker._fallback_response_body(
                        self._response(
                            encoded,
                            {
                                "Content-Type": "text/html",
                                "Content-Encoding": coding,
                            },
                        ),
                        json_body=False,
                        stage=worker.FALLBACK_WEBPAGE,
                    )
                self.assertEqual(raised.exception.reason, worker.RESPONSE_ENCODING)

    def test_normalized_bound_is_enforced_after_decompression(self):
        encoded = gzip.compress(b"x" * (worker.MAX_WEBPAGE_SCAN_BYTES + 1), mtime=0)
        with self.assertRaises(worker.UnsupportedFormat) as raised:
            worker._fallback_response_body(
                self._response(
                    encoded,
                    {"Content-Type": "text/html", "Content-Encoding": "gzip"},
                ),
                json_body=False,
                stage=worker.FALLBACK_WEBPAGE,
            )
        self.assertEqual(raised.exception.reason, worker.RESPONSE_BODY_TOO_LARGE)

    def test_oversized_normalized_compressed_webpages_are_admitted_within_scan_bound(self):
        payload = b"<html>" + b"x" * (worker.MAX_BODY + 4096) + b"</html>"
        self.assertGreater(len(payload), worker.MAX_BODY)
        self.assertLessEqual(len(payload), worker.MAX_WEBPAGE_SCAN_BYTES)
        for coding, encoded in (
            ("gzip", gzip.compress(payload, mtime=0)),
            ("deflate", zlib.compress(payload)),
        ):
            with self.subTest(coding=coding):
                self.assertLessEqual(len(encoded), worker.MAX_BODY)
                scan = worker._scan_webpage_body(
                    self._response(
                        encoded,
                        {
                            "Content-Type": "text/html; charset=utf-8",
                            "Content-Encoding": coding,
                        },
                    ),
                    encoded,
                    chunk_size=1,
                )
                self.assertTrue(scan.has_html)
                self.assertFalse(scan.has_initial_state)
                self.assertFalse(scan.has_bangumi)

    def test_marker_detection_survives_input_and_decode_chunk_boundaries(self):
        for marker, attribute in (
            (b"<html", "has_html"),
            (b"__initial_state__", "has_initial_state"),
            (b"bangumi", "has_bangumi"),
        ):
            payload = b"prefix-" + marker + b"-suffix"
            for coding, encoded in (
                ("identity", payload),
                ("gzip", gzip.compress(payload, mtime=0)),
                ("deflate", zlib.compress(payload)),
            ):
                with self.subTest(marker=marker, coding=coding):
                    scan = worker._scan_webpage_body(
                        self._response(
                            encoded,
                            {
                                "Content-Type": "text/html",
                                "Content-Encoding": coding,
                            },
                        ),
                        encoded,
                        chunk_size=1,
                    )
                    self.assertTrue(getattr(scan, attribute))

    def test_normalized_body_continues_to_json_and_html_admission(self):
        with self.assertRaises(worker.UnsupportedFormat) as raised:
            worker._fallback_response_body(
                self._response(
                    gzip.compress(b"{", mtime=0),
                    {
                        "Content-Type": "application/json",
                        "Content-Encoding": "gzip",
                    },
                ),
                json_body=True,
                stage=worker.FALLBACK_NAV,
            )
        self.assertEqual(
            (raised.exception.stage, raised.exception.reason),
            (worker.FALLBACK_NAV, worker.RESPONSE_JSON),
        )

        with self.assertRaises(worker.UnsupportedFormat) as raised:
            webpage = worker._fallback_response_body(
                self._response(
                    zlib.compress(b"not html"),
                    {"Content-Type": "text/html", "Content-Encoding": "deflate"},
                ),
                json_body=False,
                stage=worker.FALLBACK_WEBPAGE,
            )
            if not webpage.has_html:
                raise worker.UnsupportedFormat(
                    worker.FALLBACK_WEBPAGE, worker.WEBPAGE_NOT_HTML
                )
        self.assertEqual(
            (raised.exception.stage, raised.exception.reason),
            (worker.FALLBACK_WEBPAGE, worker.WEBPAGE_NOT_HTML),
        )

    def test_response_metadata_is_strict_and_secret_headers_are_not_decoder_inputs(self):
        payload = b"<html>fixture</html>"
        response = self._response(
            payload,
            {
                "Content-Type": "text/html; charset=utf-8",
                "Set-Cookie": "fixture-secret=must-not-be-read",
                "X-Public": "fixture",
            },
        )
        scan = worker._fallback_response_body(
            response,
            json_body=False,
            stage=worker.FALLBACK_WEBPAGE,
        )
        self.assertTrue(scan.has_html)
        for headers in (
            {"Content-Type": "text/html; charset=iso-8859-1"},
            {"Content-Type": "text/html", "Content-Encoding": "gzip, deflate"},
            {"Content-Type": "text/html", "Content-Encoding": "br"},
        ):
            with self.subTest(headers=headers):
                with self.assertRaises(worker.UnsupportedFormat) as raised:
                    worker._fallback_response_body(
                        self._response(payload, headers),
                        json_body=False,
                        stage=worker.FALLBACK_WEBPAGE,
                    )
                self.assertEqual(raised.exception.reason, worker.RESPONSE_ENCODING)

    def test_trailing_and_concatenated_coding_members_fail_closed(self):
        payload = b"<html>fixture</html>"
        for coding, encoded in (
            ("gzip", gzip.compress(payload, mtime=0) + b"trailing"),
            ("gzip", gzip.compress(payload, mtime=0) + gzip.compress(payload, mtime=0)),
            ("deflate", zlib.compress(payload) + b"trailing"),
        ):
            with self.subTest(coding=coding):
                with self.assertRaises(worker.UnsupportedFormat) as raised:
                    worker._fallback_response_body(
                        self._response(
                            encoded,
                            {"Content-Type": "text/html", "Content-Encoding": coding},
                        ),
                        json_body=False,
                        stage=worker.FALLBACK_WEBPAGE,
                    )
                self.assertEqual(raised.exception.reason, worker.RESPONSE_ENCODING)

    def test_unsupported_stage_is_a_closed_repository_enum(self):
        expected = {
            "PRE_FALLBACK",
            "FALLBACK_WEBPAGE",
            "FALLBACK_NAV",
            "FALLBACK_VIEW",
            "FALLBACK_DETAIL",
            "FALLBACK_PLAYURL",
            "MEDIA_SHAPE",
            "UNCLASSIFIED",
        }
        self.assertEqual(worker.UNSUPPORTED_STAGES, frozenset(expected))
        self.assertEqual(worker.UnsupportedFormat("forged").stage, worker.UNCLASSIFIED)
        self.assertEqual(worker.UnsupportedFormat("forged", "forged").reason, worker.UNCLASSIFIED)

    def test_stage_reason_taxonomy_is_closed_and_stage_scoped(self):
        expected = {
            worker.RESPONSE_STATUS,
            worker.RESPONSE_BODY_TOO_LARGE,
            worker.RESPONSE_ENCODING,
            worker.RESPONSE_JSON,
            worker.RESPONSE_SECRET_FIELD,
            worker.RESPONSE_READ,
            worker.WEBPAGE_NOT_HTML,
            worker.WEBPAGE_BANGUMI,
            worker.NAV_API_ENVELOPE,
            worker.NAV_SHAPE,
            worker.NAV_WBI_SHAPE,
            worker.NAV_WBI_URL,
            worker.VIEW_API_ENVELOPE,
            worker.VIEW_ID_MISMATCH,
            worker.VIEW_TITLE,
            worker.VIEW_PAGES,
            worker.VIEW_CID,
            worker.DETAIL_API_ENVELOPE,
            worker.DETAIL_SHAPE,
            worker.DETAIL_ID_MISMATCH,
            worker.DETAIL_TITLE,
            worker.DETAIL_PAGES,
            worker.DETAIL_CID_MISMATCH,
            worker.DETAIL_TITLE_MISMATCH,
            worker.PLAYURL_API_ENVELOPE,
            worker.PLAYURL_DURL_SHAPE,
            worker.PLAYURL_DASH_PRESENT,
            worker.PLAYURL_SEGMENT_SHAPE,
            worker.PLAYURL_SEGMENT_FIELDS,
            worker.MEDIA_URL_SHAPE,
            worker.MEDIA_URL_SENSITIVE_QUERY,
            worker.MEDIA_EXTENSION,
            worker.MEDIA_HEADERS,
            worker.MEDIA_TITLE,
            worker.MEDIA_NO_MUXED_STREAM,
            worker.UNCLASSIFIED,
        }
        self.assertEqual(worker.UNSUPPORTED_REASONS, frozenset(expected))
        for stage, reasons in worker._REASONS_BY_STAGE.items():
            self.assertTrue(reasons <= worker.UNSUPPORTED_REASONS)
            for reason in reasons:
                result = worker.UnsupportedFormat(stage, reason)
                self.assertEqual((result.stage, result.reason), (stage, reason))
        self.assertEqual(
            worker.UnsupportedFormat(worker.FALLBACK_NAV, worker.VIEW_TITLE).stage,
            worker.UNCLASSIFIED,
        )

    def test_fallback_response_body_classifies_bound_and_read_failures(self):
        class OversizedResponse:
            status = 200

            def read(self, limit):
                return b"x" * limit

        class ReadFailureResponse:
            status = 200

            def read(self, limit):
                raise RuntimeError("fixture failure must not cross the boundary")

        for stage in (worker.FALLBACK_WEBPAGE, worker.FALLBACK_NAV):
            with self.subTest(stage=stage, failure="oversized"):
                with self.assertRaises(worker.UnsupportedFormat) as raised:
                    worker._fallback_response_body(
                        OversizedResponse(),
                        json_body=stage != worker.FALLBACK_WEBPAGE,
                        stage=stage,
                    )
                self.assertEqual(
                    (raised.exception.stage, raised.exception.reason),
                    (stage, worker.RESPONSE_BODY_TOO_LARGE),
                )

            with self.subTest(stage=stage, failure="read"):
                with self.assertRaises(worker.UnsupportedFormat) as raised:
                    worker._fallback_response_body(
                        ReadFailureResponse(),
                        json_body=stage != worker.FALLBACK_WEBPAGE,
                        stage=stage,
                    )
                self.assertEqual(
                    (raised.exception.stage, raised.exception.reason),
                    (stage, worker.RESPONSE_READ),
                )

    def test_frozen_generic_ie_offline_fixture_has_top_level_direct_shape(self):
        with worker._ydl() as ydl:
            extractor = GenericIE(ydl)
            extractor._request_webpage = lambda url, video_id, **kwargs: Response(
                io.BytesIO(b""),
                url,
                {"Content-Type": "application/octet-stream"},
            )
            info = extractor._real_extract(
                "https://fixture.example.test/video.mp4"
            )

        self.assertTrue(info["direct"])
        self.assertEqual(info["ext"], "mp4")
        self.assertNotIn("formats", info)
        formats = worker._formats(info)
        self.assertEqual(formats[0]["url"], info["url"])
        self.assertEqual(formats[0]["ext"], info["ext"])

    def test_known_current_media_extension_is_normalized(self):
        formats = worker._formats(
            {
                "direct": True,
                "url": "https://fixture.example.test/video.mp4",
                "ext": "mp4",
            }
        )
        self.assertEqual(formats[0]["protocol"], None)
        self.assertEqual(formats[0]["format_id"], "direct")

    def test_current_m4v_and_hls_extensions_are_admitted(self):
        for extension, protocol in (("m4v", None), ("m3u8", "m3u8_native")):
            formats = worker._formats(
                {
                    "direct": True,
                    "url": f"https://fixture.example.test/video.{extension}?fixture=1",
                    "ext": extension,
                }
            )
            self.assertEqual(formats[0]["protocol"], protocol)

    def test_non_media_and_unknown_extensions_fail_closed(self):
        for url, extension in (
            ("https://fixture.example.test/document.pdf", "pdf"),
            ("https://fixture.example.test/video.bin", "unknown_video"),
        ):
            with self.subTest(url=url):
                with self.assertRaises(worker.UnsupportedFormat):
                    worker._formats({"direct": True, "url": url, "ext": extension})

    def test_non_direct_shape_fails_closed(self):
        with self.assertRaises(worker.UnsupportedFormat):
            worker._formats(
                {"url": "https://fixture.example.test/video.mp4", "ext": "mp4"}
            )

    def test_secret_headers_remain_rejected_after_normalization(self):
        formats = worker._formats(
            {
                "direct": True,
                "url": "https://fixture.example.test/video.mp4",
                "ext": "mp4",
                "http_headers": {"Authorization": "Bearer fixture-secret"},
            }
        )
        with self.assertRaises(worker.UnsupportedFormat) as raised:
            worker._public_headers({}, formats[0])
        self.assertEqual(raised.exception.reason, worker.MEDIA_HEADERS)

    def test_media_shape_reasons_are_fixed(self):
        with self.assertRaises(worker.UnsupportedFormat) as raised:
            worker._fallback_media_url("ftp://media.example.test/video.mp4")
        self.assertEqual(raised.exception.reason, worker.MEDIA_URL_SHAPE)
        with self.assertRaises(worker.UnsupportedFormat) as raised:
            worker._fallback_media_url(
                "https://media.example.test/video.mp4?token=fixture"
            )
        self.assertEqual(raised.exception.reason, worker.MEDIA_URL_SENSITIVE_QUERY)
        with self.assertRaises(worker.UnsupportedFormat) as raised:
            worker._fallback_media_url("https://media.example.test/video.webm")
        self.assertEqual(raised.exception.reason, worker.MEDIA_EXTENSION)
        with self.assertRaises(worker.UnsupportedFormat) as raised:
            worker._required_text("", stage=worker.MEDIA_SHAPE, reason=worker.MEDIA_TITLE)
        self.assertEqual(raised.exception.reason, worker.MEDIA_TITLE)
        with self.assertRaises(worker.UnsupportedFormat) as raised:
            worker._public_headers(
                {}, {"http_headers": {"Authorization": "Bearer fixture-secret"}}
            )
        self.assertEqual(raised.exception.reason, worker.MEDIA_HEADERS)


if __name__ == "__main__":
    unittest.main()
