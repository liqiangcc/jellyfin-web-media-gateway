from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock


HELPER_PATH = Path(__file__).parents[1] / "generic-ytdlp-runtime-cache.py"
SPEC = importlib.util.spec_from_file_location("generic_ytdlp_runtime_cache", HELPER_PATH)
assert SPEC and SPEC.loader
cache = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = cache
SPEC.loader.exec_module(cache)


def write_fixture(site_dir: Path, version: str = cache.FROZEN_VERSION, commit: str = cache.FROZEN_COMMIT) -> None:
    package = site_dir / "yt_dlp"
    distribution = site_dir / "yt_dlp-0.0.dist-info"
    package.mkdir(parents=True)
    distribution.mkdir()
    (package / "__init__.py").write_text("from .version import __version__\n", encoding="utf-8")
    (package / "version.py").write_text(f"__version__ = {version!r}\n", encoding="utf-8")
    (distribution / "METADATA").write_text(
        f"Metadata-Version: 2.1\nName: yt-dlp\nVersion: {version}\n", encoding="utf-8"
    )
    (distribution / "direct_url.json").write_text(
        json.dumps({"vcs_info": {"commit_id": commit}}), encoding="utf-8"
    )


def write_marker(cache_dir: Path, version: str = cache.FROZEN_VERSION, commit: str = cache.FROZEN_COMMIT) -> None:
    (cache_dir / "verified.json").write_text(
        json.dumps(
            {
                "schema": cache.SCHEMA_VERSION,
                "version": version,
                "commit": commit,
                "site_packages": "site-packages",
            }
        ),
        encoding="utf-8",
    )


class FrozenCacheTests(unittest.TestCase):
    def make_cache(self, root: Path, version: str = cache.FROZEN_VERSION, commit: str = cache.FROZEN_COMMIT) -> Path:
        cache_dir = root / f"{cache.FROZEN_VERSION}-{cache.FROZEN_COMMIT}"
        (cache_dir / "site-packages").mkdir(parents=True)
        write_fixture(cache_dir / "site-packages", version, commit)
        write_marker(cache_dir, version, commit)
        return cache_dir

    def test_exact_provenance_and_cache_import_are_required(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            cache_dir = self.make_cache(Path(temporary))
            self.assertTrue(cache.verify_cache(sys.executable, cache_dir, os.geteuid()))

            (cache_dir / "site-packages" / "yt_dlp" / "version.py").write_text(
                "__version__ = 'wrong'\n", encoding="utf-8"
            )
            self.assertFalse(cache.verify_cache(sys.executable, cache_dir, os.geteuid()))

    def test_runtime_environment_scrubs_setup_proxy_and_import_state(self) -> None:
        with mock.patch.dict(
            os.environ,
            {
                "HTTP_PROXY": "http://setup-sentinel.invalid",
                "HTTPS_PROXY": "http://setup-sentinel.invalid",
                "ALL_PROXY": "http://setup-sentinel.invalid",
                "PYTHONPATH": "/caller-controlled",
            },
            clear=False,
        ):
            environment = cache._runtime_environment(Path("/fixed/site-packages"))
            self.assertEqual(environment["PYTHONPATH"], "/fixed/site-packages")
            self.assertNotIn("HTTP_PROXY", environment)
            self.assertNotIn("HTTPS_PROXY", environment)
            self.assertNotIn("ALL_PROXY", environment)

            setup = cache._setup_environment()
            self.assertEqual(setup["HTTPS_PROXY"], "http://setup-sentinel.invalid")
            self.assertNotIn("PYTHONPATH", setup)
            self.assertNotIn("PIP_INDEX_URL", setup)
            self.assertNotIn("GIT_SSH_COMMAND", setup)

    @unittest.skipIf(os.geteuid() == 0, "cache preparation must run as a non-root user")
    def test_cold_prepare_warm_offline_reuse_and_setup_proxy_sentinel(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            with mock.patch.dict(
                os.environ,
                {
                    "XDG_CACHE_HOME": temporary,
                    "HTTPS_PROXY": "http://setup-sentinel.invalid",
                    "PIP_NO_INDEX": "1",
                },
                clear=False,
            ):
                seen_setup_proxy: list[str] = []

                def fake_install(_python: str, site_dir: Path) -> bool:
                    seen_setup_proxy.append(cache._setup_environment()["HTTPS_PROXY"])
                    site_dir.mkdir(parents=True)
                    write_fixture(site_dir)
                    return True

                with mock.patch.object(cache, "_install_frozen", side_effect=fake_install):
                    state, site_dir = cache.prepare(sys.executable)
                self.assertEqual(state, "prepared")
                self.assertEqual(seen_setup_proxy, ["http://setup-sentinel.invalid"])
                self.assertTrue(site_dir.is_dir())

                with mock.patch.object(cache, "_install_frozen", side_effect=AssertionError("warm cache installed")):
                    state, reused = cache.prepare(sys.executable)
                self.assertEqual(state, "hit")
                self.assertEqual(reused, site_dir)

    @unittest.skipIf(os.geteuid() == 0, "cache preparation must run as a non-root user")
    def test_mismatch_rebuild_and_interrupted_stage_never_promotes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            with mock.patch.dict(os.environ, {"XDG_CACHE_HOME": temporary}, clear=False):
                parent, cache_dir = cache._cache_paths(os.geteuid())
                old = self.make_cache(parent, version="old", commit=cache.FROZEN_COMMIT)

                def rebuild(_python: str, site_dir: Path) -> bool:
                    site_dir.mkdir(parents=True)
                    write_fixture(site_dir)
                    return True

                with mock.patch.object(cache, "_install_frozen", side_effect=rebuild):
                    state, _ = cache.prepare(sys.executable)
                self.assertEqual(state, "prepared")
                self.assertFalse(old.exists())
                self.assertTrue(cache.verify_cache(sys.executable, cache_dir, os.geteuid()))

                cache.invalidate()

                def interrupted(_python: str, site_dir: Path) -> bool:
                    site_dir.mkdir(parents=True)
                    (site_dir / "partial").write_text("incomplete", encoding="utf-8")
                    return False

                with mock.patch.object(cache, "_install_frozen", side_effect=interrupted):
                    with self.assertRaises(cache.CacheError):
                        cache.prepare(sys.executable)
                self.assertFalse(cache_dir.exists())
                self.assertEqual(list(parent.glob(cache.STAGING_PREFIX + "*")), [])


if __name__ == "__main__":
    unittest.main()
