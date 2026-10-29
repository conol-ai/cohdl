"""Offline tests for the shared sourcing HTTP client.

All HTTP responses come from an http.server bound to 127.0.0.1.  A socket
guard makes every non-loopback connection fail, including accidental DNS or
proxy use.  Rate and backoff waits use an injected clock and never really sleep.
"""

from datetime import datetime, timedelta, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import socket
import sys
import tempfile
import threading
import unittest
from unittest import mock


TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))

from sourcing.http import (  # noqa: E402
    USER_AGENT,
    FetchClient,
    PolicyError,
    RequestLimitError,
    RobotsError,
    RunStoppedError,
)


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
    def __init__(self, responses=None, robots="User-agent: *\nAllow: /\n", extra_headers=None):
        self.responses = responses or {}
        self.robots = robots
        self.extra_headers = extra_headers or {}
        self.requests = []
        self.headers = []
        outer = self

        class Handler(BaseHTTPRequestHandler):
            def do_GET(self):
                outer.requests.append(self.path)
                outer.headers.append(self.headers)
                entry = outer.responses.get(
                    self.path, (200, outer.robots if self.path == "/robots.txt" else "ok")
                )
                status, body, headers = (
                    entry if len(entry) == 3 else (entry[0], entry[1], {})
                )
                payload = body.encode("utf-8")
                self.send_response(status)
                for name, value in headers.items():
                    self.send_header(name, value)
                self.send_header("Content-Length", str(len(payload)))
                self.send_header("Connection", "close")
                self.end_headers()
                self.wfile.write(payload)

            def log_message(self, format, *args):
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

    def url(self, path="/part"):
        return f"http://127.0.0.1:{self.server.server_port}{path}"


class FetchClientTests(unittest.TestCase):
    def setUp(self):
        # Fail closed before client construction as well as during requests.
        self.real_connect = socket.socket.connect
        real_connect = self.real_connect

        def guarded_connect(sock, address):
            host = address[0] if isinstance(address, tuple) else address
            if host != "127.0.0.1":
                raise AssertionError(f"non-loopback connection attempted: {address!r}")
            return real_connect(sock, address)

        self.socket_guard = mock.patch.object(socket.socket, "connect", new=guarded_connect)
        self.socket_guard.start()
        self.addCleanup(self.socket_guard.stop)
        self.tempdir = tempfile.TemporaryDirectory()
        self.addCleanup(self.tempdir.cleanup)
        self.clock = FakeClock()

    def client(self, server, **kwargs):
        kwargs.setdefault("snapshot_clock", lambda: datetime(2026, 10, 29, tzinfo=timezone.utc))
        return FetchClient(
            Path(self.tempdir.name) / "cache",
            allowed_hosts=["127.0.0.1"],
            clock=self.clock,
            sleeper=self.clock.sleep,
            **kwargs,
        )

    def test_robots_disk_cache_expires_after_24_hours(self):
        moment = datetime(2026, 10, 29, tzinfo=timezone.utc)
        with LocalServer() as server:
            first = self.client(server, snapshot_clock=lambda: moment)
            first.get(server.url("/one"))
            second = self.client(server, snapshot_clock=lambda: moment + timedelta(hours=23))
            second.get(server.url("/two"))
            self.assertEqual(server.requests.count("/robots.txt"), 1)
            third = self.client(server, snapshot_clock=lambda: moment + timedelta(hours=25))
            third.get(server.url("/three"))
            self.assertEqual(server.requests.count("/robots.txt"), 2)

    def test_expired_robots_unavailable_does_not_use_stale_copy(self):
        moment = datetime(2026, 10, 29, tzinfo=timezone.utc)
        with LocalServer() as server:
            self.client(server, snapshot_clock=lambda: moment).get(server.url("/one"))
            server.responses["/robots.txt"] = (503, "unavailable")
            stale = self.client(server, snapshot_clock=lambda: moment + timedelta(hours=25))
            with self.assertRaises(RobotsError):
                stale.get(server.url("/two"))
            self.assertNotIn("/two", server.requests)

    def test_rate_limit_uses_injected_clock_and_fixed_user_agent(self):
        with LocalServer() as server:
            client = self.client(server)
            client.get(server.url("/one"))
            client.get(server.url("/two"))
            self.assertEqual(self.clock.sleeps, [2.0, 2.0])
            self.assertEqual(server.requests, ["/robots.txt", "/one", "/two"])
            self.assertTrue(all(h["User-Agent"] == USER_AGENT for h in server.headers))
            self.assertEqual(client.request_count, 3)

    def test_url_cache_hit_sends_no_request_and_has_dated_snapshot(self):
        with LocalServer() as server:
            client = self.client(server)
            url = server.url("/part?number=C1#ignored-fragment")
            first = client.get(url)
            requests = list(server.requests)
            second = self.client(server).get(url)
            self.assertFalse(first.from_cache)
            self.assertTrue(second.from_cache)
            self.assertEqual(first.body, second.body)
            self.assertEqual(second.snapshot_date, "2026-10-29T00:00:00Z")
            self.assertEqual(server.requests, requests)
            metadata = list((Path(self.tempdir.name) / "cache").glob("*.json"))
            self.assertEqual(len(metadata), 1)
            self.assertEqual(
                json.loads(metadata[0].read_text())["snapshot_date"],
                "2026-10-29T00:00:00Z",
            )

    def test_first_403_backs_off_and_second_consecutive_403_stops_run(self):
        responses = {"/one": (403, "no"), "/two": (403, "no"), "/three": (200, "unused")}
        with LocalServer(responses) as server:
            client = self.client(server, backoff=30)
            with self.assertRaisesRegex(Exception, "HTTP 403"):
                client.get(server.url("/one"))
            with self.assertRaises(RunStoppedError):
                client.get(server.url("/two"))
            count = len(server.requests)
            with self.assertRaises(RunStoppedError):
                client.get(server.url("/three"))
            self.assertEqual(self.clock.sleeps, [2.0, 30.0])
            self.assertEqual(len(server.requests), count)

    def test_two_consecutive_429_stop_and_success_resets_count(self):
        responses = {
            "/one": (429, "wait"),
            "/ok": (200, "yes"),
            "/two": (403, "no"),
            "/three": (429, "no"),
            "/four": (200, "unused"),
        }
        with LocalServer(responses) as server:
            client = self.client(server)
            with self.assertRaisesRegex(Exception, "HTTP 429"):
                client.get(server.url("/one"))
            self.assertEqual(client.get(server.url("/ok")).body, b"yes")
            with self.assertRaisesRegex(Exception, "HTTP 403"):
                client.get(server.url("/two"))
            with self.assertRaises(RunStoppedError):
                client.get(server.url("/three"))
            with self.assertRaises(RunStoppedError):
                client.get(server.url("/four"))
            self.assertNotIn("/four", server.requests)

    def test_robots_disallow_and_api_are_refused_before_target_request(self):
        robots = "User-agent: *\nDisallow: /private\n"
        with LocalServer(robots=robots) as server:
            client = self.client(server)
            with self.assertRaises(RobotsError):
                client.get(server.url("/private/item"))
            self.assertNotIn("/private/item", server.requests)
            before = list(server.requests)
            for path in ("/api/catalog", "/v1/API/items", "/%61pi/items"):
                with self.assertRaises(RobotsError):
                    client.get(server.url(path))
            self.assertEqual(server.requests, before)

    def test_crawl_delay_overrides_two_second_interval(self):
        robots = "User-agent: *\nCrawl-delay: 7\nAllow: /\n"
        with LocalServer(robots=robots) as server:
            client = self.client(server)
            client.get(server.url("/one"))
            client.get(server.url("/two"))
            self.assertEqual(self.clock.sleeps, [7.0, 7.0])

    def test_every_easyeda_host_is_refused_without_network(self):
        for host in ("easyeda.com", "www.easyeda.com", "easyeda.com."):
            with self.subTest(host=host), self.assertRaises(PolicyError):
                self.client(None).get(f"https://{host}/part")
        with self.assertRaises(PolicyError):
            FetchClient(self.tempdir.name, allowed_hosts=["easyeda.com"])

    def test_request_cap_includes_robots_and_no_request_over_cap(self):
        with LocalServer() as server:
            client = self.client(server, max_requests=2)
            client.get(server.url("/one"))
            before = list(server.requests)
            with self.assertRaises(RequestLimitError):
                client.get(server.url("/two"))
            self.assertEqual(server.requests, before)
            self.assertEqual(client.request_count, 2)

    def test_unlisted_host_is_refused_without_network(self):
        with self.assertRaises(PolicyError):
            self.client(None).get("https://example.com/part")

    def test_redirect_targets_are_never_requested(self):
        # The start URL is fetched; the forbidden redirect target never is.
        easyeda = "https://easyeda.com/part"
        offsite = "http://example.com/evil"
        responses = {
            "/start-api": (302, "moved", {"Location": "/api/secret"}),
            "/start-private": (302, "moved", {"Location": "/private/x"}),
            "/start-offsite": (302, "moved", {"Location": offsite}),
            "/start-easyeda": (302, "moved", {"Location": easyeda}),
        }
        robots = "User-agent: *\nDisallow: /private\n"
        with LocalServer(responses, robots=robots) as server:
            client = self.client(server)
            expected = [
                ("/start-api", RobotsError),
                ("/start-private", RobotsError),
                ("/start-offsite", PolicyError),
                ("/start-easyeda", PolicyError),
            ]
            for path, error in expected:
                with self.subTest(path=path), self.assertRaises(error):
                    client.get(server.url(path))
            self.assertEqual(client.request_count, 5)
            # Every forbidden target arrived with zero requests of its own.
            for forbidden in ("/api/secret", "/private/x", "/evil", easyeda):
                self.assertNotIn(forbidden, server.requests)
            self.assertEqual(
                [r for r in server.requests if r.startswith("/start")],
                ["/start-api", "/start-private", "/start-offsite", "/start-easyeda"],
            )

    def test_each_redirect_hop_is_rate_limited_and_budgeted(self):
        responses = {
            "/start": (302, "moved", {"Location": "/middle"}),
            "/middle": (302, "moved", {"Location": "/end"}),
            "/end": (200, "done"),
        }
        with LocalServer(responses) as server:
            client = self.client(server, max_requests=4)
            response = client.get(server.url("/start"))
            self.assertEqual(response.body, b"done")
            self.assertEqual(server.requests, ["/robots.txt", "/start", "/middle", "/end"])
            self.assertEqual(client.request_count, 4)
            self.assertEqual(self.clock.sleeps, [2.0, 2.0, 2.0])
            # The chain consumed the whole budget: the next fetch is refused.
            with self.assertRaises(RequestLimitError):
                client.get(server.url("/other"))
            self.assertNotIn("/other", server.requests)

    def test_redirect_loop_stops_at_hop_limit(self):
        responses = {"/loop": (302, "again", {"Location": "/loop"})}
        with LocalServer(responses) as server:
            client = self.client(server)
            with self.assertRaisesRegex(Exception, "too many redirects"):
                client.get(server.url("/loop"))
            self.assertEqual(server.requests.count("/loop"), 6)

    def test_unparseable_retry_after_falls_back_to_default_backoff(self):
        responses = {"/one": (429, "wait", {"Retry-After": "next tuesday"})}
        with LocalServer(responses) as server:
            client = self.client(server, backoff=45)
            with self.assertRaisesRegex(Exception, "HTTP 429"):
                client.get(server.url("/one"))
            self.assertEqual(self.clock.sleeps[-1], 45.0)

    def test_oversized_retry_after_stops_the_run(self):
        responses = {"/one": (429, "wait", {"Retry-After": "86400"})}
        with LocalServer(responses) as server:
            client = self.client(server)
            with self.assertRaisesRegex(RunStoppedError, "300"):
                client.get(server.url("/one"))
            self.assertEqual(self.clock.sleeps, [2.0])
            count = len(server.requests)
            with self.assertRaises(RunStoppedError):
                client.get(server.url("/two"))
            self.assertEqual(len(server.requests), count)


if __name__ == "__main__":
    unittest.main()
