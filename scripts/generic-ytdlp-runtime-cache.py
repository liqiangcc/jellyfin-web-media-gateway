#!/usr/bin/env python3
"""Prepare and verify the repository-owned frozen yt-dlp runtime cache.

Only the fixed source/version below are accepted.  ``prepare`` is the normal
smoke-path operation; ``invalidate`` is a bounded user-owned cleanup path.
The helper deliberately emits no setup output because pip/git diagnostics can
contain URLs or credentials supplied by the setup environment.
"""

from __future__ import annotations

import fcntl
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
from typing import Callable, Iterable


APP_NAME = "jellyfin-web-media-gateway"
CACHE_NAME = "generic-ytdlp"
FROZEN_VERSION = "2026.08.19"
FROZEN_COMMIT = "3a08beaf031ab68f966401ead017ac81fe8486cf"
FROZEN_SOURCE = (
    "yt-dlp @ git+https://github.com/yt-dlp/yt-dlp.git@"
    + FROZEN_COMMIT
)
SCHEMA_VERSION = 1
STAGING_PREFIX = ".staging-"
INSTALL_TIMEOUT_SECONDS = 300


class CacheError(Exception):
    """A bounded setup/verification failure with no sensitive detail."""


def _require_non_root() -> int:
    uid = os.geteuid()
    if uid == 0:
        raise CacheError("non-root user required")
    return uid


def _lstat(path: Path) -> os.stat_result:
    try:
        return path.lstat()
    except OSError as error:
        raise CacheError("cache storage unavailable") from error


def _owned(path: Path, uid: int) -> bool:
    return _lstat(path).st_uid == uid


def _ensure_owned_directory(path: Path, uid: int) -> None:
    try:
        if path.is_symlink():
            raise CacheError("cache symlink rejected")
        path.mkdir(mode=0o700, parents=True, exist_ok=True)
    except OSError as error:
        raise CacheError("cache storage unavailable") from error
    stat = _lstat(path)
    if not path.is_dir() or stat.st_uid != uid:
        raise CacheError("cache storage ownership rejected")


def _assert_owned_tree(path: Path, uid: int) -> None:
    if path.is_symlink():
        raise CacheError("cache symlink rejected")
    if not _owned(path, uid):
        raise CacheError("cache ownership rejected")
    if path.is_dir():
        for entry in path.rglob("*"):
            if entry.is_symlink() or not _owned(entry, uid):
                raise CacheError("cache tree ownership rejected")


def _remove_owned_tree(path: Path, uid: int) -> None:
    if not path.exists() and not path.is_symlink():
        return
    _assert_owned_tree(path, uid)
    try:
        if path.is_dir() and not path.is_symlink():
            shutil.rmtree(path)
        else:
            path.unlink()
    except OSError as error:
        raise CacheError("cache cleanup failed") from error


def _cache_parent(uid: int) -> Path:
    configured = os.environ.get("XDG_CACHE_HOME")
    if configured:
        base = Path(configured)
    else:
        home = os.environ.get("HOME")
        if not home:
            raise CacheError("user cache unavailable")
        base = Path(home) / ".cache"
    if not base.is_absolute() or base == Path("/"):
        raise CacheError("user cache path rejected")
    _ensure_owned_directory(base, uid)
    app_directory = base / APP_NAME
    _ensure_owned_directory(app_directory, uid)
    parent = app_directory / CACHE_NAME
    _ensure_owned_directory(parent, uid)
    return parent


def _cache_paths(uid: int) -> tuple[Path, Path]:
    parent = _cache_parent(uid)
    cache = parent / f"{FROZEN_VERSION}-{FROZEN_COMMIT}"
    return parent, cache


def _staging_directories(parent: Path) -> Iterable[Path]:
    try:
        entries = tuple(parent.iterdir())
    except OSError as error:
        raise CacheError("cache storage unavailable") from error
    return (entry for entry in entries if entry.name.startswith(STAGING_PREFIX))


def _cleanup_staging(parent: Path, uid: int) -> None:
    for entry in _staging_directories(parent):
        _remove_owned_tree(entry, uid)


def _runtime_environment(site_dir: Path) -> dict[str, str]:
    # Cache verification and extractor execution never inherit setup proxy
    # variables or caller Python import paths.
    return {
        "PATH": os.environ.get("PATH", ""),
        "PYTHONNOUSERSITE": "1",
        "PYTHONPATH": str(site_dir),
    }


def _setup_environment() -> dict[str, str]:
    # Retain ordinary setup routing, including an explicitly supplied proxy,
    # but remove pip/git configuration that could change the fixed source or
    # executable policy.  No value from this environment is persisted.
    environment = dict(os.environ)
    for name in (
        "PYTHONPATH",
        "PYTHONHOME",
        "PIP_CONFIG_FILE",
        "PIP_FIND_LINKS",
        "PIP_INDEX_URL",
        "PIP_EXTRA_INDEX_URL",
        "PIP_NO_INDEX",
        "PIP_TARGET",
        "PIP_USER",
        "GIT_CONFIG_GLOBAL",
        "GIT_CONFIG_SYSTEM",
        "GIT_SSH_COMMAND",
        "GIT_ASKPASS",
        "GIT_EXEC_PATH",
    ):
        environment.pop(name, None)
    environment.update(
        {
            "PYTHONNOUSERSITE": "1",
            "PIP_DISABLE_PIP_VERSION_CHECK": "1",
            "PIP_NO_INPUT": "1",
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_TERMINAL_PROMPT": "0",
        }
    )
    return environment


def _verify_code(site_dir: Path) -> str:
    encoded = json.dumps(str(site_dir))
    return f"""
import importlib.metadata
import json
import pathlib
import yt_dlp

site = pathlib.Path({encoded}).resolve()
module = pathlib.Path(yt_dlp.__file__).resolve()
assert site == module or site in module.parents
assert yt_dlp.version.__version__ == {FROZEN_VERSION!r}
distribution = importlib.metadata.distribution('yt-dlp')
location = pathlib.Path(distribution._path).resolve()
assert site == location or site in location.parents
direct_url = json.loads((location / 'direct_url.json').read_text(encoding='utf-8'))
assert direct_url.get('vcs_info', {{}}).get('commit_id') == {FROZEN_COMMIT!r}
"""


def verify_cache(python: str | os.PathLike[str], cache: Path, uid: int) -> bool:
    if not cache.exists() or cache.is_symlink() or not cache.is_dir():
        return False
    _assert_owned_tree(cache, uid)
    marker = cache / "verified.json"
    site_dir = cache / "site-packages"
    try:
        marker_data = json.loads(marker.read_text(encoding="utf-8"))
        if marker_data != {
            "schema": SCHEMA_VERSION,
            "version": FROZEN_VERSION,
            "commit": FROZEN_COMMIT,
            "site_packages": "site-packages",
        }:
            return False
    except (OSError, ValueError, TypeError):
        return False
    if not site_dir.is_dir() or site_dir.is_symlink():
        return False
    try:
        result = subprocess.run(
            [str(python), "-c", _verify_code(site_dir)],
            env=_runtime_environment(site_dir),
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
            timeout=30,
        )
    except (OSError, subprocess.SubprocessError):
        return False
    return result.returncode == 0


def _install_frozen(
    python: str | os.PathLike[str], site_dir: Path, runner: Callable[..., int] | None = None
) -> bool:
    del runner  # Kept out of the public CLI; tests replace this function itself.
    try:
        site_dir.mkdir(mode=0o700, parents=True, exist_ok=False)
    except OSError as error:
        raise CacheError("cache staging unavailable") from error
    command = [
        str(python),
        "-m",
        "pip",
        "install",
        "--target",
        str(site_dir),
        "--disable-pip-version-check",
        "--no-cache-dir",
        "--no-deps",
        FROZEN_SOURCE,
    ]
    try:
        process = subprocess.Popen(
            command,
            env=_setup_environment(),
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True,
        )
        try:
            return process.wait(timeout=INSTALL_TIMEOUT_SECONDS) == 0
        except (KeyboardInterrupt, subprocess.TimeoutExpired):
            try:
                os.killpg(process.pid, signal.SIGTERM)
            except OSError:
                pass
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except OSError:
                    pass
                process.wait()
            return False
    except OSError:
        return False


def _write_marker(stage: Path) -> None:
    marker = stage / "verified.json"
    temporary = stage / ".verified.json.tmp"
    data = {
        "schema": SCHEMA_VERSION,
        "version": FROZEN_VERSION,
        "commit": FROZEN_COMMIT,
        "site_packages": "site-packages",
    }
    try:
        temporary.write_text(json.dumps(data, sort_keys=True) + "\n", encoding="utf-8")
        os.replace(temporary, marker)
    except OSError as error:
        raise CacheError("cache provenance write failed") from error


def prepare(python: str | os.PathLike[str]) -> tuple[str, Path]:
    uid = _require_non_root()
    parent, cache = _cache_paths(uid)
    lock_path = parent / ".cache.lock"
    try:
        lock = lock_path.open("a+")
    except OSError as error:
        raise CacheError("cache lock unavailable") from error
    try:
        if not _owned(lock_path, uid):
            raise CacheError("cache lock ownership rejected")
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
        _cleanup_staging(parent, uid)
        if verify_cache(python, cache, uid):
            return "hit", cache / "site-packages"
        if cache.exists() or cache.is_symlink():
            _remove_owned_tree(cache, uid)
        stage = Path(tempfile.mkdtemp(prefix=STAGING_PREFIX, dir=parent))
        try:
            if not _install_frozen(python, stage / "site-packages"):
                raise CacheError("frozen runtime setup failed")
            _write_marker(stage)
            if not verify_cache(python, stage, uid):
                raise CacheError("frozen runtime provenance failed")
            os.replace(stage, cache)
            stage = Path()
            return "prepared", cache / "site-packages"
        finally:
            if str(stage) != "." and (stage.exists() or stage.is_symlink()):
                _remove_owned_tree(stage, uid)
    finally:
        try:
            fcntl.flock(lock.fileno(), fcntl.LOCK_UN)
        finally:
            lock.close()


def invalidate() -> None:
    uid = _require_non_root()
    parent, cache = _cache_paths(uid)
    _cleanup_staging(parent, uid)
    _remove_owned_tree(cache, uid)


def main(argv: list[str]) -> int:
    if len(argv) != 2 or argv[1] not in {"prepare", "invalidate"}:
        return 64
    try:
        if argv[1] == "invalidate":
            invalidate()
            return 0
        state, site_dir = prepare("python3")
        # The caller captures this path; it is never part of the durable safe
        # smoke summary.  The state is a bounded hit/prepared classification.
        print(state)
        print(site_dir)
        return 0
    except CacheError:
        return 75


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
