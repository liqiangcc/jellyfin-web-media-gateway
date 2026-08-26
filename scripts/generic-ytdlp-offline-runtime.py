#!/usr/bin/env python3
"""Build and consume the repository-owned frozen yt-dlp runtime bundle.

The build command is the only place that may acquire the fixed upstream
source.  The install command accepts only an already-built bundle and uses a
local wheel with pip's index and dependency resolution disabled.
"""

from __future__ import annotations

import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import zipfile
from typing import Any, Iterable


APP_NAME = "jellyfin-web-media-gateway"
CACHE_NAME = "generic-ytdlp-offline"
RUNTIME_NAME = "generic-ytdlp-offline-runtime"
FROZEN_VERSION = "2026.08.19"
FROZEN_COMMIT = "3a08beaf031ab68f966401ead017ac81fe8486cf"
FROZEN_SOURCE = (
    "https://github.com/yt-dlp/yt-dlp.git"
)
SCHEMA_VERSION = 1
ARTIFACT_FORMAT = "python-wheel"
PLATFORM_COMPATIBILITY = "platform-independent: py3-none-any"
TRUST_ANCHOR_PATH = Path(__file__).with_name("generic-ytdlp-offline-runtime.lock.json")
WHEEL_NAME_RE = re.compile(r"^yt_dlp-[0-9][A-Za-z0-9.]*-py3-none-any\.whl$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")
STAGING_PREFIX = ".staging-"
MAX_WHEEL_BYTES = 64 * 1024 * 1024
INSTALL_TIMEOUT_SECONDS = 300


class OfflineRuntimeError(Exception):
    """A bounded failure safe to expose to the caller."""


def _require_non_root() -> int:
    uid = os.geteuid()
    if uid == 0:
        raise OfflineRuntimeError("non-root user required")
    return uid


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    try:
        with path.open("rb") as source:
            for chunk in iter(lambda: source.read(1024 * 1024), b""):
                digest.update(chunk)
    except OSError as error:
        raise OfflineRuntimeError("artifact unreadable") from error
    return digest.hexdigest()


def _owned(path: Path, uid: int) -> bool:
    try:
        return path.lstat().st_uid == uid
    except OSError:
        return False


def _ensure_owned_directory(path: Path, uid: int) -> None:
    try:
        if path.is_symlink():
            raise OfflineRuntimeError("cache symlink rejected")
        path.mkdir(mode=0o700, parents=True, exist_ok=True)
        stat = path.lstat()
    except OSError as error:
        raise OfflineRuntimeError("cache storage unavailable") from error
    if not path.is_dir() or stat.st_uid != uid:
        raise OfflineRuntimeError("cache storage ownership rejected")


def _assert_owned_tree(path: Path, uid: int) -> None:
    if path.is_symlink() or not _owned(path, uid):
        raise OfflineRuntimeError("cache ownership rejected")
    if path.is_dir():
        try:
            entries = tuple(path.iterdir())
        except OSError as error:
            raise OfflineRuntimeError("cache storage unavailable") from error
        for entry in entries:
            _assert_owned_tree(entry, uid)


def _remove_owned_tree(path: Path, uid: int) -> None:
    if not path.exists() and not path.is_symlink():
        return
    _assert_owned_tree(path, uid)
    try:
        if path.is_dir():
            shutil.rmtree(path)
        else:
            path.unlink()
    except OSError as error:
        raise OfflineRuntimeError("cache cleanup failed") from error


def _cache_parent(uid: int) -> Path:
    configured = os.environ.get("XDG_CACHE_HOME")
    if configured:
        base = Path(configured)
    else:
        home = os.environ.get("HOME")
        if not home:
            raise OfflineRuntimeError("user cache unavailable")
        base = Path(home) / ".cache"
    if not base.is_absolute() or base == Path("/"):
        raise OfflineRuntimeError("cache path rejected")
    _ensure_owned_directory(base, uid)
    app = base / APP_NAME
    _ensure_owned_directory(app, uid)
    parent = app / CACHE_NAME
    _ensure_owned_directory(parent, uid)
    return parent


def _manifest_template(artifact_filename: str, artifact_sha256: str, candidate: str) -> dict[str, str | int]:
    return {
        "schema_version": SCHEMA_VERSION,
        "runtime_name": RUNTIME_NAME,
        "yt_dlp_version": FROZEN_VERSION,
        "source_commit": FROZEN_COMMIT,
        "artifact_filename": artifact_filename,
        "artifact_sha256": artifact_sha256,
        "artifact_format": ARTIFACT_FORMAT,
        "python_compatibility": "python>=3.9",
        "platform_compatibility": PLATFORM_COMPATIBILITY,
        "build_candidate_sha": candidate,
    }


MANIFEST_KEYS = frozenset(_manifest_template("placeholder", "0" * 64, "0" * 40))


def _read_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, ValueError) as error:
        raise OfflineRuntimeError("manifest unreadable") from error
    if not isinstance(value, dict):
        raise OfflineRuntimeError("manifest shape rejected")
    return value


def _validate_manifest(value: dict[str, Any]) -> dict[str, str | int]:
    if set(value) != MANIFEST_KEYS:
        raise OfflineRuntimeError("manifest fields rejected")
    if value.get("schema_version") != SCHEMA_VERSION:
        raise OfflineRuntimeError("manifest schema rejected")
    string_fields = (
        "runtime_name",
        "yt_dlp_version",
        "source_commit",
        "artifact_filename",
        "artifact_sha256",
        "artifact_format",
        "python_compatibility",
        "platform_compatibility",
        "build_candidate_sha",
    )
    if any(not isinstance(value.get(field), str) for field in string_fields):
        raise OfflineRuntimeError("manifest value rejected")
    if value["runtime_name"] != RUNTIME_NAME or value["yt_dlp_version"] != FROZEN_VERSION:
        raise OfflineRuntimeError("runtime identity rejected")
    if value["source_commit"] != FROZEN_COMMIT or value["artifact_format"] != ARTIFACT_FORMAT:
        raise OfflineRuntimeError("source identity rejected")
    if value["python_compatibility"] != "python>=3.9":
        raise OfflineRuntimeError("python compatibility rejected")
    if value["platform_compatibility"] != PLATFORM_COMPATIBILITY:
        raise OfflineRuntimeError("platform compatibility rejected")
    filename = value["artifact_filename"]
    if not WHEEL_NAME_RE.fullmatch(filename) or Path(filename).name != filename:
        raise OfflineRuntimeError("artifact filename rejected")
    if not SHA256_RE.fullmatch(value["artifact_sha256"]):
        raise OfflineRuntimeError("artifact hash rejected")
    if not COMMIT_RE.fullmatch(value["build_candidate_sha"]):
        raise OfflineRuntimeError("candidate identity rejected")
    return value


TRUST_ANCHOR_KEYS = frozenset(
    {
        "schema_version",
        "runtime_name",
        "yt_dlp_version",
        "source_commit",
        "artifact_filename",
        "artifact_sha256",
        "artifact_format",
        "platform_compatibility",
    }
)


def _load_trust_anchor() -> dict[str, str | int]:
    try:
        value = json.loads(TRUST_ANCHOR_PATH.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, ValueError) as error:
        raise OfflineRuntimeError("artifact trust anchor unavailable") from error
    if not isinstance(value, dict) or set(value) != TRUST_ANCHOR_KEYS:
        raise OfflineRuntimeError("artifact trust anchor shape rejected")
    if value.get("schema_version") != SCHEMA_VERSION:
        raise OfflineRuntimeError("artifact trust anchor schema rejected")
    if any(not isinstance(value.get(key), str) for key in TRUST_ANCHOR_KEYS - {"schema_version"}):
        raise OfflineRuntimeError("artifact trust anchor value rejected")
    if (
        value["runtime_name"] != RUNTIME_NAME
        or value["yt_dlp_version"] != FROZEN_VERSION
        or value["source_commit"] != FROZEN_COMMIT
        or value["artifact_format"] != ARTIFACT_FORMAT
        or value["platform_compatibility"] != PLATFORM_COMPATIBILITY
    ):
        raise OfflineRuntimeError("artifact trust anchor identity rejected")
    if (
        not WHEEL_NAME_RE.fullmatch(value["artifact_filename"])
        or not SHA256_RE.fullmatch(value["artifact_sha256"])
    ):
        raise OfflineRuntimeError("artifact trust anchor hash rejected")
    return value


def _wheel_metadata(artifact: Path) -> tuple[str, str, str]:
    if not artifact.is_file() or artifact.is_symlink():
        raise OfflineRuntimeError("artifact missing")
    try:
        if artifact.stat().st_size > MAX_WHEEL_BYTES:
            raise OfflineRuntimeError("artifact too large")
        with zipfile.ZipFile(artifact) as wheel:
            names = wheel.namelist()
            if any(
                not name or name.startswith("/") or ".." in Path(name).parts
                for name in names
            ):
                raise OfflineRuntimeError("artifact paths rejected")
            metadata_name = next(
                (name for name in names if name.endswith(".dist-info/METADATA")), None
            )
            wheel_name = next((name for name in names if name.endswith(".dist-info/WHEEL")), None)
            if metadata_name is None or wheel_name is None:
                raise OfflineRuntimeError("wheel metadata missing")
            metadata = wheel.read(metadata_name).decode("utf-8")
            wheel_metadata = wheel.read(wheel_name).decode("utf-8")
    except (OSError, UnicodeError, zipfile.BadZipFile, KeyError) as error:
        raise OfflineRuntimeError("artifact format rejected") from error
    fields = {}
    for line in metadata.splitlines():
        if ": " in line:
            key, value = line.split(": ", 1)
            fields.setdefault(key, value)
    package_version = fields.get("Version", "")
    normalized_version = ".".join(part.lstrip("0") or "0" for part in FROZEN_VERSION.split("."))
    if fields.get("Name", "").lower() != "yt-dlp" or package_version not in {
        FROZEN_VERSION,
        normalized_version,
    }:
        raise OfflineRuntimeError("wheel package identity rejected")
    if "Root-Is-Purelib: true" not in wheel_metadata or "Tag: py3-none-any" not in wheel_metadata:
        raise OfflineRuntimeError("wheel platform compatibility rejected")
    return fields["Name"], package_version, wheel_name


def verify_bundle(bundle: Path) -> dict[str, str | int]:
    if not bundle.is_dir() or bundle.is_symlink():
        raise OfflineRuntimeError("bundle directory rejected")
    manifest_path = bundle / "manifest.json"
    sums_path = bundle / "SHA256SUMS"
    artifacts_path = bundle / "artifacts"
    if any(path.is_symlink() for path in (manifest_path, sums_path, artifacts_path)):
        raise OfflineRuntimeError("bundle symlink rejected")
    manifest = _validate_manifest(_read_json(manifest_path))
    trust_anchor = _load_trust_anchor()
    for key in TRUST_ANCHOR_KEYS - {"schema_version"}:
        if manifest[key] != trust_anchor[key]:
            raise OfflineRuntimeError("artifact trust anchor mismatch")
    artifact = bundle / "artifacts" / str(manifest["artifact_filename"])
    if artifact.is_symlink():
        raise OfflineRuntimeError("artifact symlink rejected")
    actual_hash = _sha256(artifact)
    if actual_hash != manifest["artifact_sha256"] or actual_hash != trust_anchor["artifact_sha256"]:
        raise OfflineRuntimeError("artifact hash mismatch")
    try:
        sums = (bundle / "SHA256SUMS").read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeError) as error:
        raise OfflineRuntimeError("artifact checksums unreadable") from error
    if sums != [f"{actual_hash}  artifacts/{manifest['artifact_filename']}"]:
        raise OfflineRuntimeError("artifact checksum contract rejected")
    _wheel_metadata(artifact)
    return manifest


def _candidate_sha() -> str:
    try:
        result = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            check=True,
            text=True,
            timeout=15,
        )
    except (OSError, subprocess.SubprocessError) as error:
        raise OfflineRuntimeError("candidate identity unavailable") from error
    candidate = result.stdout.strip()
    if not COMMIT_RE.fullmatch(candidate):
        raise OfflineRuntimeError("candidate identity rejected")
    return candidate


def _run_silent(command: list[str], *, cwd: Path | None = None, timeout: int = 600) -> None:
    try:
        subprocess.run(
            command,
            cwd=cwd,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=True,
            timeout=timeout,
        )
    except (OSError, subprocess.SubprocessError) as error:
        raise OfflineRuntimeError("fixed runtime build failed") from error


def build_bundle(output: Path, python: str = "python3") -> dict[str, str | int]:
    candidate = _candidate_sha()
    if output.exists():
        if output.is_symlink() or not output.is_dir() or any(output.iterdir()):
            raise OfflineRuntimeError("output bundle must be new and empty")
    else:
        try:
            output.mkdir(mode=0o755, parents=True)
        except OSError as error:
            raise OfflineRuntimeError("output bundle unavailable") from error
    parent = output.parent
    temporary = Path(tempfile.mkdtemp(prefix=".generic-ytdlp-build-", dir=parent))
    try:
        source = temporary / "source"
        artifacts = temporary / "artifacts"
        artifacts.mkdir()
        _run_silent(["git", "clone", "--no-checkout", FROZEN_SOURCE, str(source)], timeout=600)
        _run_silent(["git", "checkout", "--detach", FROZEN_COMMIT], cwd=source, timeout=120)
        try:
            checked_out = subprocess.run(
                ["git", "rev-parse", "HEAD"],
                cwd=source,
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                check=True,
                text=True,
                timeout=30,
            ).stdout.strip()
        except (OSError, subprocess.SubprocessError) as error:
            raise OfflineRuntimeError("source provenance unavailable") from error
        if checked_out != FROZEN_COMMIT:
            raise OfflineRuntimeError("source provenance mismatch")
        _run_silent(
            [python, "-m", "pip", "wheel", "--disable-pip-version-check", "--no-deps", "--wheel-dir", str(artifacts), str(source)],
            timeout=900,
        )
        wheels = sorted(artifacts.glob("*.whl"))
        if len(wheels) != 1:
            raise OfflineRuntimeError("wheel output rejected")
        artifact = wheels[0]
        if not WHEEL_NAME_RE.fullmatch(artifact.name):
            raise OfflineRuntimeError("wheel filename rejected")
        _wheel_metadata(artifact)
        final_artifacts = output / "artifacts"
        final_artifacts.mkdir()
        final_artifact = final_artifacts / artifact.name
        shutil.copyfile(artifact, final_artifact)
        manifest = _manifest_template(artifact.name, _sha256(final_artifact), candidate)
        (output / "manifest.json").write_text(
            json.dumps(manifest, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8"
        )
        (output / "SHA256SUMS").write_text(
            f"{manifest['artifact_sha256']}  artifacts/{artifact.name}\n", encoding="utf-8"
        )
        return verify_bundle(output)
    except OfflineRuntimeError:
        _remove_output_safely(output)
        raise
    finally:
        shutil.rmtree(temporary, ignore_errors=True)


def _remove_output_safely(output: Path) -> None:
    if not output.exists() or output.is_symlink():
        return
    if output.parent == output or output == Path("/"):
        return
    shutil.rmtree(output, ignore_errors=True)


def _offline_environment(site_dir: Path | None = None) -> dict[str, str]:
    environment = {"PATH": os.environ.get("PATH", "")}
    environment.update(
        {
            "PYTHONNOUSERSITE": "1",
            "PIP_CONFIG_FILE": os.devnull,
            "PIP_DISABLE_PIP_VERSION_CHECK": "1",
            "PIP_NO_INDEX": "1",
        }
    )
    if site_dir is not None:
        environment["PYTHONPATH"] = str(site_dir)
    return environment


def _verify_import(python: str | os.PathLike[str], site_dir: Path, manifest: dict[str, str | int]) -> bool:
    encoded_site = json.dumps(str(site_dir.resolve()))
    expected_version = json.dumps(FROZEN_VERSION)
    code = f"""
import importlib.metadata
import pathlib
import yt_dlp
site = pathlib.Path({encoded_site})
module = pathlib.Path(yt_dlp.__file__).resolve()
assert site == module or site in module.parents
assert yt_dlp.version.__version__ == {expected_version}
distribution = importlib.metadata.distribution('yt-dlp')
location = pathlib.Path(distribution._path).resolve()
assert site == location or site in location.parents
assert not (location / 'direct_url.json').exists()
"""
    try:
        result = subprocess.run(
            [str(python), "-c", code],
            env=_offline_environment(site_dir),
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
            timeout=30,
        )
    except (OSError, subprocess.SubprocessError):
        return False
    del manifest
    return result.returncode == 0


def _cache_paths(uid: int, manifest: dict[str, str | int]) -> tuple[Path, Path]:
    parent = _cache_parent(uid)
    cache = parent / f"{manifest['yt_dlp_version']}-{manifest['artifact_sha256']}"
    return parent, cache


def _staging_directories(parent: Path) -> Iterable[Path]:
    try:
        return tuple(entry for entry in parent.iterdir() if entry.name.startswith(STAGING_PREFIX))
    except OSError as error:
        raise OfflineRuntimeError("cache storage unavailable") from error


def _cleanup_staging(parent: Path, uid: int) -> None:
    for entry in _staging_directories(parent):
        _remove_owned_tree(entry, uid)


def _marker(manifest: dict[str, str | int]) -> dict[str, str | int]:
    return {
        "schema_version": SCHEMA_VERSION,
        "runtime_name": RUNTIME_NAME,
        "yt_dlp_version": manifest["yt_dlp_version"],
        "source_commit": manifest["source_commit"],
        "artifact_filename": manifest["artifact_filename"],
        "artifact_sha256": manifest["artifact_sha256"],
        "artifact_format": manifest["artifact_format"],
        "site_packages": "site-packages",
    }


def verify_cache(
    python: str | os.PathLike[str], cache: Path, uid: int, manifest: dict[str, str | int]
) -> bool:
    if not cache.exists() or cache.is_symlink() or not cache.is_dir():
        return False
    _assert_owned_tree(cache, uid)
    site_dir = cache / "site-packages"
    try:
        marker = _read_json(cache / "verified.json")
    except OfflineRuntimeError:
        return False
    if marker != _marker(manifest) or not site_dir.is_dir() or site_dir.is_symlink():
        return False
    return _verify_import(python, site_dir, manifest)


def _remove_direct_url(site_dir: Path) -> None:
    try:
        entries = tuple(site_dir.iterdir())
    except OSError as error:
        raise OfflineRuntimeError("runtime staging unavailable") from error
    for entry in entries:
        if entry.is_dir() and entry.name.startswith("yt_dlp-") and entry.name.endswith(".dist-info"):
            direct_url = entry / "direct_url.json"
            if direct_url.exists() or direct_url.is_symlink():
                try:
                    direct_url.unlink()
                except OSError as error:
                    raise OfflineRuntimeError("runtime provenance cleanup failed") from error


def _install_wheel(python: str, artifact: Path, site_dir: Path) -> None:
    try:
        site_dir.mkdir(mode=0o700, parents=True, exist_ok=False)
    except OSError as error:
        raise OfflineRuntimeError("runtime staging unavailable") from error
    command = [
        python,
        "-m",
        "pip",
        "install",
        "--disable-pip-version-check",
        "--no-index",
        "--no-deps",
        "--no-cache-dir",
        "--target",
        str(site_dir),
        str(artifact),
    ]
    try:
        result = subprocess.run(
            command,
            env=_offline_environment(),
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
            timeout=INSTALL_TIMEOUT_SECONDS,
        )
    except (OSError, subprocess.SubprocessError) as error:
        raise OfflineRuntimeError("offline wheel install failed") from error
    if result.returncode != 0:
        raise OfflineRuntimeError("offline wheel install failed")
    _remove_direct_url(site_dir)


def install_bundle(bundle: Path, python: str = "python3") -> tuple[str, Path, dict[str, str | int]]:
    uid = _require_non_root()
    manifest = verify_bundle(bundle)
    parent, cache = _cache_paths(uid, manifest)
    lock_path = parent / ".cache.lock"
    try:
        lock = lock_path.open("a+")
    except OSError as error:
        raise OfflineRuntimeError("cache lock unavailable") from error
    try:
        if not _owned(lock_path, uid):
            raise OfflineRuntimeError("cache lock ownership rejected")
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
        _cleanup_staging(parent, uid)
        if verify_cache(python, cache, uid, manifest):
            return "hit", cache / "site-packages", manifest
        if cache.exists() or cache.is_symlink():
            _remove_owned_tree(cache, uid)
        stage = Path(tempfile.mkdtemp(prefix=STAGING_PREFIX, dir=parent))
        try:
            artifact = bundle / "artifacts" / str(manifest["artifact_filename"])
            _install_wheel(python, artifact, stage / "site-packages")
            marker = stage / "verified.json"
            marker.write_text(json.dumps(_marker(manifest), sort_keys=True) + "\n", encoding="utf-8")
            if not verify_cache(python, stage, uid, manifest):
                raise OfflineRuntimeError("runtime provenance verification failed")
            os.replace(stage, cache)
            stage = Path()
            return "prepared", cache / "site-packages", manifest
        finally:
            if str(stage) != "." and (stage.exists() or stage.is_symlink()):
                _remove_owned_tree(stage, uid)
    finally:
        try:
            fcntl.flock(lock.fileno(), fcntl.LOCK_UN)
        finally:
            lock.close()


def main(argv: list[str]) -> int:
    if len(argv) != 3 or argv[1] not in {"build", "verify", "install"}:
        return 64
    try:
        path = Path(argv[2])
        if argv[1] == "build":
            manifest = build_bundle(path)
            print("built")
            print(manifest["artifact_filename"])
            print(manifest["artifact_sha256"])
            return 0
        if argv[1] == "verify":
            manifest = verify_bundle(path)
            print("verified")
            print(manifest["artifact_sha256"])
            return 0
        state, site_dir, manifest = install_bundle(path)
        print(state)
        print(site_dir)
        print(manifest["artifact_sha256"])
        return 0
    except OfflineRuntimeError:
        return 75


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
