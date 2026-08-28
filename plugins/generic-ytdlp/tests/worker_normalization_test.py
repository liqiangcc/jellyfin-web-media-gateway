import io
import importlib.util
import unittest
from pathlib import Path

from yt_dlp.extractor.generic import GenericIE
from yt_dlp.networking import Response


WORKER_PATH = Path(__file__).parents[1] / "worker" / "worker.py"
SPEC = importlib.util.spec_from_file_location("generic_ytdlp_worker", WORKER_PATH)
assert SPEC and SPEC.loader
worker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(worker)


class DirectFallbackNormalizationTest(unittest.TestCase):
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
