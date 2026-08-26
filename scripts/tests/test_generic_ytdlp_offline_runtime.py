from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock
import zipfile


HELPER_PATH = Path(__file__).parents[1] / "generic-ytdlp-offline-runtime.py"
SPEC = importlib.util.spec_from_file_location("generic_ytdlp_offline_runtime", HELPER_PATH)
assert SPEC and SPEC.loader
runtime = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = runtime
SPEC.loader.exec_module(runtime)


def write_wheel(path: Path, version: str = runtime.FROZEN_VERSION) -> None:
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_DEFLATED) as wheel:
        wheel.writestr(
            "yt_dlp/__init__.py",
            "from .version import __version__\n",
        )
        wheel.writestr("yt_dlp/version.py", f"__version__ = {version!r}\n")
        wheel.writestr(
            "yt_dlp-2026.8.19.dist-info/METADATA",
            f"Metadata-Version: 2.1\nName: yt-dlp\nVersion: {version}\n",
        )
        wheel.writestr(
            "yt_dlp-2026.8.19.dist-info/WHEEL",
            "Wheel-Version: 1.0\nRoot-Is-Purelib: true\nTag: py3-none-any\n",
        )
        wheel.writestr("yt_dlp-2026.8.19.dist-info/RECORD", "")


def write_bundle(root: Path, candidate: str = "a" * 40) -> Path:
    bundle = root / runtime.RUNTIME_NAME
    artifacts = bundle / "artifacts"
    artifacts.mkdir(parents=True)
    artifact = artifacts / "yt_dlp-2026.8.19-py3-none-any.whl"
    write_wheel(artifact)
    digest = hashlib.sha256(artifact.read_bytes()).hexdigest()
    manifest = runtime._manifest_template(artifact.name, digest, candidate)
    (bundle / "manifest.json").write_text(json.dumps(manifest) + "\n", encoding="utf-8")
    (bundle / "SHA256SUMS").write_text(
        f"{digest}  artifacts/{artifact.name}\n", encoding="utf-8"
    )
    return bundle


class OfflineBundleTests(unittest.TestCase):
    def test_bundle_hash_and_platform_contract(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            manifest = runtime.verify_bundle(write_bundle(Path(temporary)))
            self.assertEqual(manifest["source_commit"], runtime.FROZEN_COMMIT)
            self.assertEqual(manifest["platform_compatibility"], runtime.PLATFORM_COMPATIBILITY)

    def test_hash_mismatch_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            bundle = write_bundle(Path(temporary))
            artifact = next((bundle / "artifacts").glob("*.whl"))
            artifact.write_bytes(artifact.read_bytes() + b"tamper")
            with self.assertRaises(runtime.OfflineRuntimeError):
                runtime.verify_bundle(bundle)

    def test_manifest_extra_field_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            bundle = write_bundle(Path(temporary))
            manifest_path = bundle / "manifest.json"
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["proxy"] = "must-not-persist"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            with self.assertRaises(runtime.OfflineRuntimeError):
                runtime.verify_bundle(bundle)

    def test_offline_install_promotes_and_warm_reuses(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            bundle = write_bundle(root)
            cache_root = root / "cache"
            with mock.patch.object(runtime, "_require_non_root", return_value=runtime.os.geteuid()), mock.patch.dict(
                runtime.os.environ, {"XDG_CACHE_HOME": str(cache_root)}, clear=False
            ):
                state, site_dir, manifest = runtime.install_bundle(bundle, sys.executable)
                self.assertEqual(state, "prepared")
                self.assertTrue(runtime._verify_import(sys.executable, site_dir, manifest))
                state, warm_dir, _ = runtime.install_bundle(bundle, sys.executable)
                self.assertEqual(state, "hit")
                self.assertEqual(site_dir, warm_dir)
                self.assertEqual(list(site_dir.parent.parent.glob(runtime.STAGING_PREFIX + "*")), [])

    def test_root_is_rejected(self) -> None:
        with mock.patch.object(runtime.os, "geteuid", return_value=0):
            with self.assertRaises(runtime.OfflineRuntimeError):
                runtime._require_non_root()


if __name__ == "__main__":
    unittest.main()
