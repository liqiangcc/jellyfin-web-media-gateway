#!/usr/bin/env python3
"""Small on-demand AF_UNIX-to-AF_UNIX relay for the accepted Android route."""

from __future__ import annotations

import argparse
import os
import signal
import socket
import threading


MAX_SOCKET_PATH = 255


def _unix_address(value: str) -> str:
    if value.startswith("@"):
        return "\0" + value[1:]
    if not value.startswith("/") or len(value) > MAX_SOCKET_PATH:
        raise ValueError("socket must be an absolute bounded AF_UNIX path or @abstract name")
    return value


class Relay:
    def __init__(self, listen_path: str, upstream_name: str):
        if not listen_path.startswith("/") or len(listen_path) > MAX_SOCKET_PATH:
            raise ValueError("listen socket must be an absolute bounded path")
        if not upstream_name.startswith("@") or len(upstream_name) > MAX_SOCKET_PATH:
            raise ValueError("upstream must be an abstract socket name")
        self.listen_path = listen_path
        self.upstream = _unix_address(upstream_name)
        self.listener: socket.socket | None = None
        self.stop_event = threading.Event()
        self.threads: list[threading.Thread] = []

    @staticmethod
    def _copy(source: socket.socket, destination: socket.socket) -> None:
        try:
            while True:
                chunk = source.recv(65536)
                if not chunk:
                    break
                destination.sendall(chunk)
        except OSError:
            pass
        finally:
            try:
                destination.shutdown(socket.SHUT_WR)
            except OSError:
                pass

    def _serve(self, client: socket.socket) -> None:
        upstream = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        try:
            upstream.connect(self.upstream)
            left = threading.Thread(target=self._copy, args=(client, upstream), daemon=True)
            right = threading.Thread(target=self._copy, args=(upstream, client), daemon=True)
            left.start()
            right.start()
            left.join()
            right.join()
        except OSError:
            pass
        finally:
            client.close()
            upstream.close()

    def stop(self, *_: object) -> None:
        self.stop_event.set()
        if self.listener is not None:
            self.listener.close()

    def run(self) -> None:
        listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        listener.bind(self.listen_path)
        os.chmod(self.listen_path, 0o600)
        listener.listen(8)
        self.listener = listener
        signal.signal(signal.SIGTERM, self.stop)
        signal.signal(signal.SIGINT, self.stop)
        try:
            while not self.stop_event.is_set():
                try:
                    client, _ = listener.accept()
                except OSError:
                    break
                thread = threading.Thread(target=self._serve, args=(client,), daemon=True)
                self.threads.append(thread)
                thread.start()
        finally:
            listener.close()
            try:
                os.unlink(self.listen_path)
            except FileNotFoundError:
                pass


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="local-only AF_UNIX CDP relay")
    parser.add_argument("--listen", required=True, help="shared filesystem AF_UNIX path")
    parser.add_argument("--upstream", default="@chrome_devtools_remote", help="exact Beta abstract socket")
    args = parser.parse_args(argv)
    try:
        Relay(args.listen, args.upstream).run()
    except (OSError, ValueError) as exc:
        parser.error(str(exc))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
