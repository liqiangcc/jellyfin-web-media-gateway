#!/usr/bin/env python3
"""Reproducible, low-privilege R003 resource measurement harness.

The target workflow checks out this file from a trusted workflow revision and
keeps the candidate checkout separate.  The module intentionally uses only
the Python standard library so target preflight never installs packages.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import math
import os
import platform
import re
import shutil
import signal
import socket
import statistics
import subprocess
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any, Iterable, Sequence


SCHEMA_VERSION = "r003.metrics.v1"
SCENARIOS = (
    "idle",
    "direct-http",
    "direct-hls",
    "direct-4k",
    "remux",
    "transcode-boundary",
    "chromium-baseline",
)
CHECKPOINT_PROFILES = {"5": 300, "30": 1800, "60": 3600}
SHA_RE = re.compile(r"^[0-9a-fA-F]{40}$")
STOP_EVENT = threading.Event()


def request_stop(_signum: int, _frame: Any) -> None:
    """Turn cancellation into normal cleanup so child process groups are reaped."""
    STOP_EVENT.set()


signal.signal(signal.SIGINT, request_stop)
signal.signal(signal.SIGTERM, request_stop)


def now_utc() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat(timespec="milliseconds")


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    temporary.replace(path)


def validate_sha(value: str, name: str = "sha") -> str:
    if not SHA_RE.fullmatch(value):
        raise ValueError(f"{name} must be a full 40-character hexadecimal SHA")
    return value.lower()


def validate_scenario(value: str) -> str:
    if value not in SCENARIOS:
        raise ValueError(f"scenario must be one of: {', '.join(SCENARIOS)}")
    return value


def checkpoint_seconds(profile: str) -> int:
    try:
        return CHECKPOINT_PROFILES[profile]
    except KeyError as exc:
        raise ValueError("checkpoint_profile must be one of: 5, 30, 60") from exc


def checkpoint_schedule(profile: str) -> list[int]:
    """Return the canonical 5m/30m/60m checkpoints within one run."""
    duration = checkpoint_seconds(profile)
    return [value for value in (300, 1800, 3600) if value <= duration]


def resolve_duration(profile: str | None, duration: float | None) -> float:
    if profile is not None:
        expected = checkpoint_seconds(profile)
        if duration is not None and not math.isclose(duration, expected, abs_tol=0.01):
            raise ValueError("duration must match the selected continuous checkpoint profile")
        return float(expected)
    if duration is None or duration <= 0 or duration > 3600:
        raise ValueError("duration_seconds must be greater than 0 and no more than 3600")
    return duration


def measurement_duration(scenario: str, requested_duration: float) -> float:
    """Keep the transcode boundary bounded while aligning collection to workload."""
    validate_scenario(scenario)
    return min(requested_duration, 30.0) if scenario == "transcode-boundary" else requested_duration


def read_text(path: Path) -> str | None:
    try:
        return path.read_text(encoding="utf-8").strip()
    except (OSError, UnicodeError):
        return None


class ProcReader:
    """Read Linux process/system counters, with injectable roots for tests."""

    def __init__(self, proc_root: Path = Path("/proc"), sys_root: Path = Path("/sys")) -> None:
        self.proc_root = proc_root
        self.sys_root = sys_root
        self.clock_ticks = os.sysconf("SC_CLK_TCK")
        self.cpu_count = os.cpu_count() or 1

    def process(self, pid: int) -> dict[str, Any]:
        result: dict[str, Any] = {"pid": pid, "available": False}
        stat = read_text(self.proc_root / str(pid) / "stat")
        status_path = self.proc_root / str(pid) / "status"
        status_text = read_text(status_path)
        if stat is None:
            return result
        closing = stat.rfind(")")
        fields = stat[closing + 2 :].split() if closing >= 0 else []
        if len(fields) < 13:
            return result
        status: dict[str, str] = {}
        if status_text:
            for line in status_text.splitlines():
                if ":" in line:
                    key, value = line.split(":", 1)
                    status[key] = value.strip()
        result.update(
            {
                "available": True,
                "state": fields[0],
                "cpu_ticks": int(fields[11]) + int(fields[12]),
                "rss_bytes": self._kilobytes(status.get("VmRSS")),
                "voluntary_context_switches": self._number(status.get("voluntary_ctxt_switches")),
                "nonvoluntary_context_switches": self._number(status.get("nonvoluntary_ctxt_switches")),
                "name": self._name(pid),
            }
        )
        return result

    def _name(self, pid: int) -> str | None:
        return read_text(self.proc_root / str(pid) / "comm")

    @staticmethod
    def _number(value: str | None) -> int | None:
        if not value:
            return None
        match = re.search(r"-?\d+", value)
        return int(match.group(0)) if match else None

    @classmethod
    def _kilobytes(cls, value: str | None) -> int | None:
        number = cls._number(value)
        return number * 1024 if number is not None else None

    def system_cpu_ticks(self) -> int | None:
        text = read_text(self.proc_root / "stat")
        if not text:
            return None
        for line in text.splitlines():
            if line.startswith("cpu "):
                values = line.split()[1:]
                return sum(int(value) for value in values)
        return None

    def load_average(self) -> list[float] | None:
        text = read_text(self.proc_root / "loadavg")
        if not text:
            return None
        try:
            return [float(value) for value in text.split()[:3]]
        except ValueError:
            return None

    def network_bytes(self) -> dict[str, dict[str, int]]:
        text = read_text(self.proc_root / "net/dev")
        result: dict[str, dict[str, int]] = {}
        if not text:
            return result
        for line in text.splitlines():
            if ":" not in line:
                continue
            name, payload = line.split(":", 1)
            values = payload.split()
            if len(values) < 9:
                continue
            try:
                result[name.strip()] = {"rx_bytes": int(values[0]), "tx_bytes": int(values[8])}
            except ValueError:
                continue
        return result

    def thermal(self) -> dict[str, Any]:
        zones: dict[str, float] = {}
        base = self.sys_root / "class" / "thermal"
        for path in sorted(base.glob("thermal_zone*/temp")):
            raw = read_text(path)
            if raw is None:
                continue
            try:
                zones[path.parent.name] = float(raw) / 1000.0
            except ValueError:
                continue
        return {"available": bool(zones), "celsius": zones}

    def battery(self) -> dict[str, Any]:
        supplies: dict[str, dict[str, str | int | None]] = {}
        base = self.sys_root / "class" / "power_supply"
        for path in sorted(base.glob("*/status")):
            name = path.parent.name
            supplies[name] = {
                "status": read_text(path),
                "capacity_percent": self._number(read_text(path.parent / "capacity")),
            }
        return {"available": bool(supplies), "supplies": supplies}

    def route(self) -> str | None:
        text = read_text(self.proc_root / "net" / "route")
        if not text:
            return None
        for line in text.splitlines()[1:]:
            fields = line.split()
            if len(fields) >= 2 and fields[1] == "00000000":
                return fields[0]
        return None

    def top_processes(self, limit: int = 8) -> list[dict[str, Any]]:
        processes: list[dict[str, Any]] = []
        for path in self.proc_root.iterdir() if self.proc_root.exists() else []:
            if not path.name.isdigit():
                continue
            sample = self.process(int(path.name))
            if sample.get("available"):
                processes.append(
                {
                        "pid": sample["pid"],
                        "name": sample.get("name"),
                        "cpu_ticks": sample.get("cpu_ticks"),
                        "rss_bytes": sample.get("rss_bytes"),
                    }
                )
        return sorted(processes, key=lambda item: item.get("rss_bytes") or 0, reverse=True)[:limit]

    def sample(self, pids: Iterable[int]) -> dict[str, Any]:
        return {
            "processes": {str(pid): self.process(pid) for pid in pids},
            "system_cpu_ticks": self.system_cpu_ticks(),
            "load_average": self.load_average(),
            "network": self.network_bytes(),
            "network_interface": self.route(),
            "thermal": self.thermal(),
            "battery": self.battery(),
            "high_load_processes": self.top_processes(),
        }


def _delta_network(previous: dict[str, Any] | None, current: dict[str, Any], seconds: float) -> dict[str, Any]:
    if not previous or seconds <= 0:
        return {"available": False, "rx_bytes_per_second": None, "tx_bytes_per_second": None}
    rx = tx = 0
    for name, counters in current.get("network", {}).items():
        old = previous.get("network", {}).get(name, {})
        rx += max(0, counters.get("rx_bytes", 0) - old.get("rx_bytes", 0))
        tx += max(0, counters.get("tx_bytes", 0) - old.get("tx_bytes", 0))
    return {
        "available": bool(current.get("network")),
        "rx_bytes_per_second": rx / seconds,
        "tx_bytes_per_second": tx / seconds,
        "interface": current.get("network_interface"),
    }


def enrich_sample(
    raw: dict[str, Any], previous: dict[str, Any] | None, elapsed: float, reader: ProcReader
) -> dict[str, Any]:
    sample = dict(raw)
    sample["elapsed_seconds"] = round(elapsed, 3)
    seconds = elapsed - float(previous.get("elapsed_seconds", 0)) if previous else 0.0
    if previous and seconds > 0 and raw.get("system_cpu_ticks") is not None:
        delta = max(0, raw["system_cpu_ticks"] - (previous.get("system_cpu_ticks") or 0))
        sample["system_cpu_percent"] = min(100.0, max(0.0, delta / reader.clock_ticks / seconds / reader.cpu_count * 100))
    else:
        sample["system_cpu_percent"] = None
    for pid, process in sample["processes"].items():
        old = previous.get("processes", {}).get(pid) if previous else None
        if old and process.get("available") and old.get("available") and seconds > 0:
            delta = max(0, process["cpu_ticks"] - old.get("cpu_ticks", 0))
            process["cpu_percent"] = min(100.0, max(0.0, delta / reader.clock_ticks / seconds / reader.cpu_count * 100))
        else:
            process["cpu_percent"] = None
    sample["network_throughput"] = _delta_network(previous, raw, seconds)
    return sample


def collect_metrics(
    output_dir: Path,
    pids: Sequence[int],
    duration: float,
    interval: float,
    checkpoint_values: Sequence[int],
    metadata: dict[str, Any],
    reader: ProcReader | None = None,
) -> dict[str, Any]:
    if interval <= 0 or interval > 60:
        raise ValueError("sample interval must be greater than 0 and no more than 60 seconds")
    if duration <= 0 or duration > 3600:
        raise ValueError("duration must be greater than 0 and no more than 3600 seconds")
    checkpoints = sorted(set(checkpoint_values))
    if checkpoints and checkpoints[-1] > duration + 0.01:
        raise ValueError("continuous checkpoint must be within the single run duration")
    reader = reader or ProcReader()
    output_dir.mkdir(parents=True, exist_ok=True)
    raw_path = output_dir / "samples.jsonl"
    checkpoint_path = output_dir / "checkpoints.jsonl"
    started = time.monotonic()
    previous: dict[str, Any] | None = None
    checkpoint_index = 0
    sample_count = 0
    status = "COMPLETED"
    error: str | None = None
    run = dict(metadata)
    run.update(
        {
            "schema_version": SCHEMA_VERSION,
            "started_at": now_utc(),
            "sample_interval_seconds": interval,
            "duration_seconds": duration,
            "checkpoint_seconds": checkpoints,
            "pids": list(pids),
            "status": "RUNNING",
        }
    )
    write_json(output_dir / "run.json", run)
    try:
        with raw_path.open("w", encoding="utf-8") as raw_file, checkpoint_path.open("w", encoding="utf-8") as checkpoint_file:
            while not STOP_EVENT.is_set():
                elapsed = time.monotonic() - started
                if elapsed > duration + 0.01:
                    break
                sample = enrich_sample(reader.sample(pids), previous, elapsed, reader)
                sample["timestamp"] = now_utc()
                raw_file.write(json.dumps(sample, sort_keys=True) + "\n")
                raw_file.flush()
                sample_count += 1
                while checkpoint_index < len(checkpoints) and elapsed >= checkpoints[checkpoint_index]:
                    checkpoint = {
                        "schema_version": SCHEMA_VERSION,
                        "checkpoint_seconds": checkpoints[checkpoint_index],
                        "observed_elapsed_seconds": round(elapsed, 3),
                        "sample_count": sample_count,
                        "timestamp": sample["timestamp"],
                    }
                    checkpoint_file.write(json.dumps(checkpoint, sort_keys=True) + "\n")
                    checkpoint_file.flush()
                    checkpoint_index += 1
                previous = sample
                remaining = duration - (time.monotonic() - started)
                if remaining > 0:
                    STOP_EVENT.wait(min(interval, remaining))
            if STOP_EVENT.is_set():
                status = "CANCELLED"
    except Exception as exc:  # preserve a machine-readable failure before re-raising
        status = "FAILED"
        error = f"{type(exc).__name__}: {exc}"
        raise
    finally:
        run.update(
            {
                "ended_at": now_utc(),
                "status": status,
                "error": error,
                "sample_count": sample_count,
                "checkpoints_emitted": checkpoint_index,
            }
        )
        write_json(output_dir / "run.json", run)
    return run


def _numbers(values: Iterable[Any]) -> list[float]:
    return [float(value) for value in values if isinstance(value, (int, float))]


def _slope(points: Sequence[tuple[float, float]]) -> float | None:
    if len(points) < 2:
        return None
    x_mean = statistics.fmean(point[0] for point in points)
    y_mean = statistics.fmean(point[1] for point in points)
    denominator = sum((x - x_mean) ** 2 for x, _ in points)
    return None if denominator == 0 else sum((x - x_mean) * (y - y_mean) for x, y in points) / denominator


def summarize(output_dir: Path) -> dict[str, Any]:
    run = json.loads((output_dir / "run.json").read_text(encoding="utf-8"))
    samples: list[dict[str, Any]] = []
    with (output_dir / "samples.jsonl").open(encoding="utf-8") as handle:
        for line in handle:
            if line.strip():
                samples.append(json.loads(line))
    system_cpu = _numbers(sample.get("system_cpu_percent") for sample in samples)
    rss: dict[str, list[float]] = {}
    temperature: list[float] = []
    for sample in samples:
        for pid, process in sample.get("processes", {}).items():
            if process.get("rss_bytes") is not None:
                rss.setdefault(pid, []).append(float(process["rss_bytes"]))
        temperature.extend(_numbers(sample.get("thermal", {}).get("celsius", {}).values()))
    network_rx = _numbers(sample.get("network_throughput", {}).get("rx_bytes_per_second") for sample in samples)
    network_tx = _numbers(sample.get("network_throughput", {}).get("tx_bytes_per_second") for sample in samples)
    summary = {
        "schema_version": SCHEMA_VERSION,
        "run": run,
        "sample_count": len(samples),
        "metrics": {
            "system_cpu_percent": {
                "average": statistics.fmean(system_cpu) if system_cpu else None,
                "maximum": max(system_cpu) if system_cpu else None,
            },
            "process_rss_bytes": {
                pid: {
                    "average": statistics.fmean(values),
                    "maximum": max(values),
                    "slope_bytes_per_second": _slope(
                        [
                            (sample.get("elapsed_seconds", 0), float(sample["processes"][pid]["rss_bytes"]))
                            for sample in samples
                            if sample.get("processes", {}).get(pid, {}).get("rss_bytes") is not None
                        ]
                    ),
                }
                for pid, values in rss.items()
            },
            "temperature_celsius": {
                "average": statistics.fmean(temperature) if temperature else None,
                "maximum": max(temperature) if temperature else None,
            },
            "network_bytes_per_second": {
                "rx_average": statistics.fmean(network_rx) if network_rx else None,
                "tx_average": statistics.fmean(network_tx) if network_tx else None,
            },
        },
        "checkpoint_count": run.get("checkpoints_emitted", 0),
        "raw_samples": "samples.jsonl",
    }
    write_json(output_dir / "summary.json", summary)
    lines = [
        "# R003 resource measurement summary",
        "",
        f"- Schema: `{SCHEMA_VERSION}`",
        f"- Scenario: `{run.get('scenario', 'unknown')}`",
        f"- Status: `{run.get('status', 'unknown')}`",
        f"- Samples: `{len(samples)}`; checkpoints: `{run.get('checkpoints_emitted', 0)}`",
        f"- Candidate SHA: `{run.get('candidate_sha', 'n/a')}`",
        f"- Harness SHA: `{run.get('harness_sha', 'n/a')}`",
        "",
        "## Metrics",
        "",
        f"- System CPU average/max: `{summary['metrics']['system_cpu_percent']['average']}` / `{summary['metrics']['system_cpu_percent']['maximum']}` percent",
        f"- Temperature average/max: `{summary['metrics']['temperature_celsius']['average']}` / `{summary['metrics']['temperature_celsius']['maximum']}` °C",
        f"- Network RX/TX average: `{summary['metrics']['network_bytes_per_second']['rx_average']}` / `{summary['metrics']['network_bytes_per_second']['tx_average']}` bytes/s",
        "",
        "Raw timestamped samples are retained in `samples.jsonl`; this summary is not a substitute for trend review.",
    ]
    (output_dir / "summary.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    return summary


def command_path(candidates: Sequence[str]) -> str | None:
    for candidate in candidates:
        path = shutil.which(candidate)
        if path:
            return path
    return None


def preflight() -> dict[str, Any]:
    thermal_paths = list(Path("/sys/class/thermal").glob("thermal_zone*/temp"))
    chromium = command_path(("chromium", "chromium-browser", "google-chrome", "google-chrome-stable"))
    capabilities = {
        "python": {"available": True, "version": platform.python_version()},
        "ffmpeg": {"available": bool(shutil.which("ffmpeg")), "path": shutil.which("ffmpeg")},
        "chromium": {"available": bool(chromium), "path": chromium},
        "thermal": {"available": bool(thermal_paths), "readable_paths": [str(path) for path in thermal_paths]},
        "proc": {"available": Path("/proc/stat").is_file(), "path": "/proc"},
        "gateway_commands": {"available": True, "note": "supplied by the target workflow; no package installation"},
    }
    missing = [name for name, value in capabilities.items() if not value.get("available")]
    return {
        "schema_version": SCHEMA_VERSION,
        "checked_at": now_utc(),
        "status": "BLOCKED" if missing else "AVAILABLE",
        "missing": missing,
        "capabilities": capabilities,
        "host": {"hostname": socket.gethostname(), "architecture": platform.machine(), "system": platform.platform()},
    }


def terminate_process(process: subprocess.Popen[Any] | None, timeout: float = 5.0) -> None:
    if process is None or process.poll() is not None:
        return
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except (ProcessLookupError, PermissionError):
        process.terminate()
    try:
        process.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except (ProcessLookupError, PermissionError):
            process.kill()
        process.wait(timeout=timeout)


def wait_for_health(url: str, process: subprocess.Popen[Any], timeout: float = 30.0) -> float:
    started = time.monotonic()
    while time.monotonic() - started < timeout:
        if process.poll() is not None:
            raise RuntimeError(f"gateway exited before health check (code {process.returncode})")
        try:
            with urllib.request.urlopen(url, timeout=1) as response:
                if 200 <= response.status < 300:
                    return (time.monotonic() - started) * 1000
        except (OSError, urllib.error.URLError):
            time.sleep(0.25)
    raise TimeoutError(f"gateway health check timed out: {urllib.parse.urlsplit(url).path}")


def proof_url(gateway_url: str, scenario: str) -> str:
    with urllib.request.urlopen(gateway_url.rstrip("/") + "/proof/paths", timeout=10) as response:
        paths = json.loads(response.read())
    key = "hls_path" if scenario in {"direct-hls", "remux"} else "mp4_path"
    path = paths.get(key)
    if not isinstance(path, str) or not path.startswith("/"):
        raise RuntimeError(f"gateway did not expose a usable {key}")
    return gateway_url.rstrip("/") + path


def fetch_bytes(url: str, limit: int = 1024 * 1024) -> bytes:
    with urllib.request.urlopen(url, timeout=10) as response:
        return response.read(limit)


def playlist_children(body: bytes, base_url: str) -> list[str]:
    """Resolve variant, segment, and URI-attribute children from an HLS playlist."""
    text = body.decode("utf-8", errors="replace")
    children: list[str] = []
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        if stripped.startswith("#"):
            match = re.search(r'URI="([^"]+)"', stripped)
            if not match:
                continue
            candidate = match.group(1)
        else:
            candidate = stripped
        resolved = urllib.parse.urljoin(base_url, candidate)
        if resolved not in children:
            children.append(resolved)
    return children


def hls_media_children(url: str, max_requests: int = 4) -> list[str]:
    """Fetch a playlist and its first variant, returning actual child media URLs."""
    first_body = fetch_bytes(url, 512 * 1024)
    first_children = playlist_children(first_body, url)
    if not first_children:
        raise RuntimeError("HLS playlist has no child variant or media URI")
    if b"#EXT-X-STREAM-INF" not in first_body:
        return first_children[:max_requests]
    variant_body = fetch_bytes(first_children[0], 512 * 1024)
    second_children = playlist_children(variant_body, first_children[0])
    if not second_children:
        # A media playlist may itself be the first response.
        second_children = first_children
    return second_children[:max_requests]


def traffic_loop(
    url: str,
    stop: threading.Event,
    hls: bool = False,
    stats: dict[str, Any] | None = None,
    errors: list[str] | None = None,
) -> None:
    while not stop.is_set():
        try:
            if hls:
                children = hls_media_children(url)
                if stats is not None:
                    stats["playlist_cycles"] = stats.get("playlist_cycles", 0) + 1
                    stats["child_requests"] = stats.get("child_requests", 0) + len(children)
                for child in children:
                    if stop.is_set():
                        break
                    fetch_bytes(child, 512 * 1024)
            else:
                fetch_bytes(url)
                if stats is not None:
                    stats["requests"] = stats.get("requests", 0) + 1
            stop.wait(0.25)
        except (OSError, RuntimeError, urllib.error.URLError) as exc:
            if errors is not None and len(errors) < 16:
                errors.append(f"{type(exc).__name__}: {exc}")
            stop.wait(0.5)


def build_ffmpeg_command(executable: str, scenario: str, media_url: str, output_file: Path, duration: float) -> list[str]:
    if scenario not in {"remux", "transcode-boundary"}:
        raise ValueError("FFmpeg command requires remux or transcode-boundary scenario")
    input_args = ["-stream_loop", "-1", "-re", "-i", media_url] if scenario == "remux" else ["-i", media_url]
    codec_args = ["-c", "copy"] if scenario == "remux" else ["-c:v", "libx264", "-c:a", "aac", "-preset", "veryfast"]
    return [executable, "-hide_banner", "-loglevel", "warning", "-y", *input_args, *codec_args, "-t", str(duration), str(output_file)]


def build_chromium_command(executable: str, target: str, user_data_dir: Path) -> list[str]:
    return [
        executable,
        "--headless=new",
        "--disable-gpu",
        "--disable-dev-shm-usage",
        "--no-first-run",
        "--remote-debugging-port=0",
        "--user-data-dir",
        str(user_data_dir),
        target,
    ]


def build_candidate(candidate_dir: Path) -> None:
    subprocess.run(
        ["cargo", "build", "--manifest-path", str(candidate_dir / "Cargo.toml"), "--bin", "r001-server"],
        check=True,
    )


def git_head(directory: Path) -> str:
    return subprocess.check_output(["git", "-C", str(directory), "rev-parse", "HEAD"], text=True).strip().lower()


def run_scenario(args: argparse.Namespace) -> int:
    STOP_EVENT.clear()
    validate_sha(args.candidate_sha, "candidate_sha")
    validate_sha(args.harness_sha, "harness_sha")
    scenario = validate_scenario(args.scenario)
    requested_duration = resolve_duration(args.checkpoint_profile, args.duration_seconds)
    # The transcode boundary is intentionally short. Its collector window is
    # the same bounded window as the actual FFmpeg workload, not post-exit idle.
    duration = measurement_duration(scenario, requested_duration)
    output = Path(args.output_dir)
    output.mkdir(parents=True, exist_ok=True)
    capabilities = preflight()
    write_json(output / "preflight.json", capabilities)
    if scenario in {"remux", "transcode-boundary"} and not capabilities["capabilities"].get("ffmpeg", {}).get("available", False):
        run = {
            "schema_version": SCHEMA_VERSION,
            "candidate_sha": args.candidate_sha,
            "harness_sha": args.harness_sha,
            "scenario": scenario,
            "status": "BLOCKED",
            "blocked_reason": "ffmpeg is unavailable; target workflow does not install packages",
        }
        write_json(output / "run.json", run)
        return 2
    if scenario == "chromium-baseline" and not capabilities["capabilities"]["chromium"]["available"]:
        run = {
            "schema_version": SCHEMA_VERSION,
            "candidate_sha": args.candidate_sha,
            "harness_sha": args.harness_sha,
            "scenario": scenario,
            "status": "BLOCKED",
            "blocked_reason": "Chromium is unavailable; target workflow does not install packages",
        }
        write_json(output / "run.json", run)
        return 2

    gateway: subprocess.Popen[Any] | None = None
    workload: subprocess.Popen[Any] | None = None
    traffic_stop = threading.Event()
    traffic: threading.Thread | None = None
    traffic_stats: dict[str, Any] = {}
    traffic_errors: list[str] = []
    log_handle = (output / "process.log").open("w", encoding="utf-8")
    startup_latency: float | None = None
    metadata = {
        "candidate_sha": args.candidate_sha.lower(),
        "harness_sha": args.harness_sha.lower(),
        "scenario": scenario,
        "source_type": args.source_type,
        "media_bitrate_bps": args.media_bitrate_bps,
        "startup_latency_ms": None,
        "host": {"hostname": socket.gethostname(), "architecture": platform.machine(), "system": platform.platform()},
        "preflight_status": capabilities["status"],
        "checkpoint_profile": args.checkpoint_profile,
        "requested_duration_seconds": requested_duration,
        "effective_duration_seconds": duration,
        "workload_duration_seconds": duration,
    }
    try:
        if args.gateway_command:
            gateway = subprocess.Popen(
                args.gateway_command,
                cwd=args.candidate_dir or None,
                stdout=log_handle,
                stderr=subprocess.STDOUT,
                start_new_session=True,
            )
            startup_latency = wait_for_health(args.gateway_url.rstrip("/") + "/healthz", gateway)
            metadata["startup_latency_ms"] = startup_latency
        pids = [process.pid for process in (gateway,) if process is not None]
        media_url = args.media_url or proof_url(args.gateway_url, scenario) if args.gateway_url else args.media_url
        if scenario in {"direct-http", "direct-hls", "direct-4k"}:
            if not media_url:
                raise ValueError("direct scenarios require gateway_url or media_url")
            traffic = threading.Thread(
                target=traffic_loop,
                args=(media_url, traffic_stop, scenario == "direct-hls", traffic_stats, traffic_errors),
                daemon=True,
            )
            traffic.start()
        elif scenario in {"remux", "transcode-boundary"}:
            if not media_url:
                raise ValueError("FFmpeg scenarios require gateway_url or media_url")
            ffmpeg = capabilities["capabilities"]["ffmpeg"]["path"]
            output_file = output / ("remux.ts" if scenario == "remux" else "transcode.mp4")
            command = build_ffmpeg_command(ffmpeg or "ffmpeg", scenario, media_url, output_file, duration)
            workload = subprocess.Popen(
                command,
                stdout=log_handle,
                stderr=subprocess.STDOUT,
                start_new_session=True,
            )
            pids.append(workload.pid)
        elif scenario == "chromium-baseline":
            target = args.chromium_url or args.gateway_url
            if not target:
                raise ValueError("Chromium baseline requires chromium_url or gateway_url")
            chromium = capabilities["capabilities"]["chromium"]["path"]
            user_data_dir = Path(tempfile.mkdtemp(prefix="chromium-", dir=output))
            metadata["workload_mode"] = "live-chromium-page-load"
            workload = subprocess.Popen(
                build_chromium_command(chromium or "chromium", target, user_data_dir),
                stdout=log_handle,
                stderr=subprocess.STDOUT,
                start_new_session=True,
            )
            pids.append(workload.pid)
            time.sleep(1)
            if workload.poll() is not None:
                run = dict(metadata)
                run.update(
                    {
                        "schema_version": SCHEMA_VERSION,
                        "status": "BLOCKED",
                        "blocked_reason": "Chromium exited before the requested bounded live-process window",
                        "started_at": now_utc(),
                        "ended_at": now_utc(),
                        "duration_seconds": duration,
                        "checkpoint_seconds": [],
                    }
                )
                write_json(output / "run.json", run)
                return 2
        metadata["startup_latency_ms"] = startup_latency
        checkpoint_values = checkpoint_schedule(args.checkpoint_profile) if args.checkpoint_profile else []
        if scenario == "transcode-boundary":
            checkpoint_values = [value for value in checkpoint_values if value <= duration]
        run = collect_metrics(
            output,
            pids,
            duration,
            args.interval_seconds,
            checkpoint_values,
            metadata,
        )
        workload_exited_before_window = workload is not None and workload.poll() is not None
        workload_returncode = workload.returncode if workload else None
        traffic_stop.set()
        if traffic:
            traffic.join(timeout=2)
        terminate_process(workload)
        terminate_process(gateway)
        run["process_results"] = {
            "gateway": {"pid": gateway.pid, "returncode": gateway.returncode, "controlled_stop": True} if gateway else None,
            "workload": {
                "pid": workload.pid,
                "returncode": workload_returncode,
                "exited_before_window": workload_exited_before_window,
                "controlled_stop": True,
            }
            if workload
            else None,
        }
        run["traffic"] = {"stats": traffic_stats, "errors": traffic_errors}
        if scenario == "direct-hls" and not traffic_stats.get("child_requests"):
            run["status"] = "FAILED"
            run["error"] = "direct-hls produced no rewritten child/media requests"
        if scenario == "chromium-baseline" and workload_exited_before_window:
            run["status"] = "BLOCKED"
            run["blocked_reason"] = "Chromium exited before the requested bounded live-process window"
        elif scenario == "remux" and workload_exited_before_window:
            run["status"] = "FAILED"
            run["error"] = "remux workload exited before the requested continuous measurement window"
        write_json(output / "run.json", run)
        summarize(output)
        return 1 if run.get("status") == "FAILED" else 2 if run.get("status") == "BLOCKED" else 0
    except Exception as exc:
        write_json(output / "error.json", {"schema_version": SCHEMA_VERSION, "error": f"{type(exc).__name__}: {exc}"})
        return 1
    finally:
        traffic_stop.set()
        if traffic:
            traffic.join(timeout=2)
        terminate_process(workload)
        terminate_process(gateway)
        log_handle.close()
        chromium_dir = locals().get("user_data_dir")
        if chromium_dir:
            shutil.rmtree(chromium_dir, ignore_errors=True)


def validate_target(args: argparse.Namespace) -> int:
    validate_sha(args.candidate_sha, "candidate_sha")
    validate_sha(args.harness_sha, "harness_sha")
    validate_scenario(args.scenario)
    checkpoint_seconds(args.checkpoint_profile)
    candidate = Path(args.candidate_dir).resolve()
    trusted = Path(args.trusted_dir).resolve()
    if candidate == trusted or trusted in candidate.parents or candidate in trusted.parents:
        raise ValueError("trusted harness and candidate checkouts must be separate")
    if not (candidate / "Cargo.toml").is_file():
        raise ValueError("candidate checkout is missing Cargo.toml")
    if not (trusted / "scripts/r003/harness.py").is_file():
        raise ValueError("trusted checkout is missing the R003 harness")
    if git_head(candidate) != args.candidate_sha.lower():
        raise ValueError("candidate checkout HEAD does not match candidate_sha")
    if git_head(trusted) != args.harness_sha.lower():
        raise ValueError("trusted harness checkout HEAD does not match harness_sha")
    if os.geteuid() == 0:
        raise ValueError("target job must run as the dedicated low-privilege account")
    if str(candidate).startswith("/var/lib/web-media-gateway"):
        raise ValueError("candidate checkout overlaps production Gateway state")
    print(json.dumps({"candidate_sha": args.candidate_sha.lower(), "harness_sha": args.harness_sha.lower(), "status": "VALID"}))
    return 0


def validate_workflow(path: Path) -> list[str]:
    text = path.read_text(encoding="utf-8")
    failures: list[str] = []
    required = (
        "workflow_dispatch:",
        "candidate_sha:",
        "scenario:",
        "checkpoint_profile:",
        "runs-on: [self-hosted, linux, ARM64, ubuntu-arm64, target-device]",
        "permissions:\n  contents: read",
        "concurrency:",
        "actions/checkout@v4",
        "trusted-harness",
        "candidate",
        "validate-target",
        "--candidate-sha",
    )
    for marker in required:
        if marker not in text:
            failures.append(f"missing workflow marker: {marker}")
    if "pull_request:" in text or "push:" in text:
        failures.append("target workflow must not have pull_request or push triggers")
    if "sudo" in text.lower():
        failures.append("target workflow must not install or elevate privileges")
    if "${{ inputs." in text and "env:" not in text:
        failures.append("inputs must be passed through environment, not shell interpolation")
    return failures


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("preflight")
    summary = sub.add_parser("summarize")
    summary.add_argument("--output-dir", required=True)
    build = sub.add_parser("build-candidate")
    build.add_argument("--candidate-dir", required=True, type=Path)
    workflow = sub.add_parser("validate-workflow")
    workflow.add_argument("--path", required=True, type=Path)
    target = sub.add_parser("validate-target")
    target.add_argument("--candidate-sha", required=True)
    target.add_argument("--harness-sha", required=True)
    target.add_argument("--scenario", required=True)
    target.add_argument("--checkpoint-profile", required=True)
    target.add_argument("--candidate-dir", required=True)
    target.add_argument("--trusted-dir", required=True)
    scenario = sub.add_parser("run-scenario")
    scenario.add_argument("--candidate-sha", required=True)
    scenario.add_argument("--harness-sha", required=True)
    scenario.add_argument("--scenario", required=True)
    scenario.add_argument("--checkpoint-profile")
    scenario.add_argument("--duration-seconds", type=float)
    scenario.add_argument("--interval-seconds", type=float, default=5.0)
    scenario.add_argument("--output-dir", required=True)
    scenario.add_argument("--candidate-dir")
    scenario.add_argument("--gateway-url")
    scenario.add_argument("--gateway-command", nargs="+")
    scenario.add_argument("--media-url")
    scenario.add_argument("--chromium-url")
    scenario.add_argument("--source-type", default="unknown")
    scenario.add_argument("--media-bitrate-bps", type=int)
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        if args.command == "preflight":
            print(json.dumps(preflight(), indent=2, sort_keys=True))
            return 0
        if args.command == "summarize":
            summarize(Path(args.output_dir))
            return 0
        if args.command == "build-candidate":
            build_candidate(args.candidate_dir)
            return 0
        if args.command == "validate-workflow":
            failures = validate_workflow(args.path)
            if failures:
                for failure in failures:
                    print(failure, file=sys.stderr)
                return 1
            print("workflow validation: PASS")
            return 0
        if args.command == "validate-target":
            return validate_target(args)
        if args.command == "run-scenario":
            return run_scenario(args)
    except (ValueError, OSError, RuntimeError, TimeoutError) as exc:
        print(f"{type(exc).__name__}: {exc}", file=sys.stderr)
        return 1
    return 1


if __name__ == "__main__":
    sys.exit(main())
