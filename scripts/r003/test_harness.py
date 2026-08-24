import json
import http.server
import os
import signal
import socketserver
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))

from harness import (  # noqa: E402
    ProcReader,
    build_chromium_command,
    build_ffmpeg_command,
    checkpoint_schedule,
    collect_metrics,
    enrich_sample,
    hls_media_children,
    measurement_duration,
    playlist_children,
    resolve_duration,
    summarize,
    traffic_loop,
    validate_scenario,
    validate_workflow,
)


class HarnessTests(unittest.TestCase):
    def fixture_proc(self, root: Path) -> None:
        (root / "123").mkdir(parents=True)
        (root / "123" / "stat").write_text(
            "123 (gateway) S 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15\n", encoding="utf-8"
        )
        (root / "123" / "comm").write_text("gateway\n", encoding="utf-8")
        (root / "123" / "status").write_text("VmRSS:\t4 kB\nvoluntary_ctxt_switches:\t7\n", encoding="utf-8")
        (root / "stat").write_text("cpu  100 0 0 900 0 0 0 0 0 0\n", encoding="utf-8")
        (root / "loadavg").write_text("0.50 0.20 0.10 1/10 123\n", encoding="utf-8")
        (root / "net").mkdir()
        (root / "net" / "dev").write_text("Inter-| Receive | Transmit\n eth0: 100 0 0 0 0 0 0 0 200 0\n", encoding="utf-8")
        (root / "net" / "route").write_text("Iface Destination Gateway Flags\neth0 00000000 00000000 0003\n", encoding="utf-8")

    def test_proc_fixture_and_enrichment(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.fixture_proc(root)
            reader = ProcReader(root, root / "sys")
            first = reader.sample([123])
            (root / "123" / "stat").write_text(
                "123 (gateway) S 1 2 3 4 5 6 7 8 9 10 21 22 23 24 25\n", encoding="utf-8"
            )
            current = reader.sample([123])
            sample = enrich_sample(current, {**first, "elapsed_seconds": 1}, 2, reader)
            self.assertTrue(sample["processes"]["123"]["available"])
            self.assertEqual(sample["processes"]["123"]["rss_bytes"], 4096)
            self.assertIsNotNone(sample["network_throughput"]["rx_bytes_per_second"])
            self.assertEqual(sample["network_throughput"]["interface"], "eth0")

    def test_collects_raw_samples_and_checkpoint(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            metadata = {"scenario": "idle", "candidate_sha": "a" * 40, "harness_sha": "b" * 40}
            run = collect_metrics(output, [os.getpid()], 0.08, 0.02, [], metadata)
            self.assertEqual(run["status"], "COMPLETED")
            self.assertGreaterEqual(run["sample_count"], 2)
            self.assertTrue((output / "samples.jsonl").is_file())
            summary = summarize(output)
            self.assertEqual(summary["schema_version"], "r003.metrics.v1")
            self.assertTrue((output / "summary.md").is_file())

    def test_failure_cleanup_terminates_process_group(self) -> None:
        process = subprocess.Popen(["sleep", "60"], start_new_session=True)
        try:
            from harness import terminate_process

            terminate_process(process, timeout=1)
            self.assertIsNotNone(process.poll())
        finally:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGKILL)

    def test_scenario_and_workflow_contract(self) -> None:
        self.assertEqual(validate_scenario("remux"), "remux")
        self.assertEqual(validate_scenario("direct-4k"), "direct-4k")
        with self.assertRaises(ValueError):
            validate_scenario("arbitrary-shell")
        self.assertEqual(resolve_duration("5", None), 300)
        self.assertEqual(measurement_duration("transcode-boundary", 300), 30)
        self.assertEqual(measurement_duration("remux", 3600), 3600)
        self.assertEqual(checkpoint_schedule("5"), [300])
        self.assertEqual(checkpoint_schedule("30"), [300, 1800])
        self.assertEqual(checkpoint_schedule("60"), [300, 1800, 3600])
        with self.assertRaises(ValueError):
            resolve_duration("5", 1)
        failures = validate_workflow(ROOT / ".github/workflows/r003-target-resource.yml")
        self.assertEqual(failures, [])

    def test_hls_child_resolution_and_workload_commands(self) -> None:
        master = b"#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000\nvariant.m3u8\n"
        variant = b"#EXTM3U\n#EXTINF:1,\nseg-0.ts\n#EXTINF:1,\nseg-1.ts\n"
        self.assertEqual(playlist_children(master, "http://gateway/master.m3u8"), ["http://gateway/variant.m3u8"])
        self.assertEqual(
            playlist_children(variant, "http://gateway/variant.m3u8"),
            ["http://gateway/seg-0.ts", "http://gateway/seg-1.ts"],
        )
        remux = build_ffmpeg_command("ffmpeg", "remux", "http://gateway/media.m3u8", Path("out.ts"), 300)
        self.assertIn("-stream_loop", remux)
        self.assertIn("-re", remux)
        self.assertEqual(remux[remux.index("-t") + 1], "300")
        transcode = build_ffmpeg_command("ffmpeg", "transcode-boundary", "http://gateway/media.mp4", Path("out.mp4"), 30)
        self.assertNotIn("-stream_loop", transcode)
        self.assertEqual(transcode[transcode.index("-t") + 1], "30")
        chromium = build_chromium_command("chromium", "http://gateway/", Path("profile"))
        self.assertNotIn("--dump-dom", chromium)
        self.assertIn("--remote-debugging-port=0", chromium)

    def test_hls_child_requests_use_local_fixture(self) -> None:
        requests: list[str] = []

        class Handler(http.server.BaseHTTPRequestHandler):
            def do_GET(self) -> None:  # noqa: N802
                requests.append(self.path)
                bodies = {
                    "/master.m3u8": b"#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000\nvariant.m3u8\n",
                    "/variant.m3u8": b"#EXTM3U\n#EXTINF:1,\nseg-0.ts\n#EXTINF:1,\nseg-1.ts\n",
                    "/seg-0.ts": b"segment-0",
                    "/seg-1.ts": b"segment-1",
                }
                body = bodies.get(self.path, b"")
                self.send_response(200 if body else 404)
                self.end_headers()
                self.wfile.write(body)

            def log_message(self, _format: str, *_args: object) -> None:
                return

        server = socketserver.ThreadingTCPServer(("127.0.0.1", 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            base = f"http://127.0.0.1:{server.server_address[1]}"
            children = hls_media_children(f"{base}/master.m3u8")
            self.assertEqual(children, [f"{base}/seg-0.ts", f"{base}/seg-1.ts"])
            stop = threading.Event()
            stats: dict[str, object] = {}
            errors: list[str] = []
            worker = threading.Thread(
                target=traffic_loop,
                args=(f"{base}/master.m3u8", stop, True, stats, errors),
                daemon=True,
            )
            worker.start()
            time.sleep(0.25)
            stop.set()
            worker.join(timeout=2)
            self.assertGreaterEqual(stats.get("child_requests", 0), 2)
            self.assertEqual(errors, [])
            self.assertIn("/seg-0.ts", requests)
            self.assertIn("/seg-1.ts", requests)
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=2)

    def test_summary_preserves_slope_inputs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            (output / "run.json").write_text(
                json.dumps({"schema_version": "r003.metrics.v1", "scenario": "idle", "status": "COMPLETED", "checkpoints_emitted": 0}),
                encoding="utf-8",
            )
            rows = [
                {"elapsed_seconds": 0, "system_cpu_percent": 10, "thermal": {"celsius": {"tz0": 40}}, "processes": {"1": {"rss_bytes": 100}}},
                {"elapsed_seconds": 1, "system_cpu_percent": 20, "thermal": {"celsius": {"tz0": 42}}, "processes": {"1": {"rss_bytes": 120}}},
            ]
            (output / "samples.jsonl").write_text("\n".join(json.dumps(row) for row in rows) + "\n", encoding="utf-8")
            summary = summarize(output)
            self.assertEqual(summary["metrics"]["process_rss_bytes"]["1"]["slope_bytes_per_second"], 20)


if __name__ == "__main__":
    unittest.main()
