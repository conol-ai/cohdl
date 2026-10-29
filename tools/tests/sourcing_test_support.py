"""Loopback-only HTTP fixture shared by sourcing tool tests."""

from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import socket
import sys
import threading
from unittest import mock

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))

from sourcing.http import FetchClient  # noqa: E402


class FakeClock:
    def __init__(self):
        self.now = 1000.0
        self.sleeps = []

    def __call__(self):
        return self.now

    def sleep(self, seconds):
        self.sleeps.append(seconds)
        self.now += seconds


class LocalServer:
    def __init__(self, responses=None):
        self.responses = responses or {}
        self.requests = []
        outer = self

        class Handler(BaseHTTPRequestHandler):
            def do_GET(self):
                outer.requests.append(self.path)
                entry = outer.responses.get(self.path)
                if entry is None and self.path == "/robots.txt":
                    entry = (200, b"User-agent: *\nAllow: /\n")
                if entry is None:
                    entry = (404, b"missing")
                status, body = entry[:2]
                if isinstance(body, str):
                    body = body.encode("utf-8")
                self.send_response(status)
                self.send_header("Content-Length", str(len(body)))
                self.send_header("Connection", "close")
                self.end_headers()
                self.wfile.write(body)

            def log_message(self, *args):
                pass

        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)

    def __enter__(self):
        self.thread.start()
        return self

    def __exit__(self, *args):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join()

    def url(self, path=""):
        return f"http://127.0.0.1:{self.server.server_port}{path}"


def install_socket_guard(testcase):
    real_connect = socket.socket.connect

    def guarded(sock, address):
        host = address[0] if isinstance(address, tuple) else address
        if host != "127.0.0.1":
            raise AssertionError(f"non-loopback connection attempted: {address!r}")
        return real_connect(sock, address)

    guard = mock.patch.object(socket.socket, "connect", new=guarded)
    guard.start()
    testcase.addCleanup(guard.stop)


def make_client(cache_dir, max_requests=100):
    clock = FakeClock()
    return FetchClient(cache_dir, allowed_hosts=["127.0.0.1"], max_requests=max_requests,
                       clock=clock, sleeper=clock.sleep,
                       snapshot_clock=lambda: datetime(2026, 10, 30, tzinfo=timezone.utc))


def page(c, mpn, mfr="Acme", category="Basic", stock=10):
    return (f'<div data-field="c">{c}</div><div data-field="mpn">{mpn}</div>'
            f'<div data-field="mfr">{mfr}</div><div data-field="package">0603</div>'
            f'<div data-field="category">{category}</div><div data-field="stock">{stock}</div>'
            '<a data-field="datasheet" href="https://example.invalid/datasheet.pdf">PDF</a>')
