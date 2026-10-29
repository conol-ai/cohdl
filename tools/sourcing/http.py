"""Small, conservative HTTP client for approved supplier sourcing jobs.

Network access is opt-in: every ``FetchClient`` needs an explicit, non-empty
``allowed_hosts`` list.  Proxy variables in the environment are ignored.
Requests use a fixed, identifying User-Agent, are limited per run and per host,
and are refused for EasyEDA and paths disallowed by robots.txt.  In particular,
``/api`` is refused before a request is sent.  Two consecutive HTTP 403/429
responses permanently stop the client; successful responses reset that count.
No error response is cached.  Retry backoff and the 2-second minimum host
interval (or a longer robots.txt Crawl-delay) are injectable for offline tests.

The cache stores bytes under a SHA-256 digest of the normalized URL and writes
an ISO-8601 UTC snapshot timestamp beside each entry.  Cache files and raw pages
are transient working data, never repository inputs or distributable artifacts.
The contract for callers is to extract and save only factual fields: C-number,
MPN, brand, package name, class, stock, datasheet URL and product-page path.
Do not persist response bodies, HTML, markup, descriptions or other raw pages.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
import email.utils
import hashlib
import json
import math
from pathlib import Path
import socket
import time
from typing import Callable, Mapping, Sequence
import urllib.error
import urllib.parse
import urllib.request
import urllib.robotparser


USER_AGENT = "cohdl-library-bot/0.1 (+https://github.com/conol-ai/cohdl)"
DEFAULT_MIN_INTERVAL = 2.0
DEFAULT_BACKOFF = 60.0
MAX_REDIRECTS = 5
MAX_RETRY_AFTER = 300.0
_REDIRECT_STATUSES = frozenset({301, 302, 303, 307, 308})


class FetchError(RuntimeError):
    """Base error for a refused or failed fetch."""


class PolicyError(FetchError):
    """The URL is not permitted by the explicit fetching policy."""


class RobotsError(PolicyError):
    """robots.txt disallows the URL, or its policy could not be established."""


class RequestLimitError(FetchError):
    """The per-run request budget is exhausted."""


class RunStoppedError(FetchError):
    """Two consecutive 403/429 responses have stopped this client."""


@dataclass(frozen=True)
class Response:
    """A successful network or cached response."""

    url: str
    body: bytes
    snapshot_date: str
    from_cache: bool


@dataclass(frozen=True)
class RequestRecord:
    """Observable request metadata; never includes response content."""

    url: str
    user_agent: str
    snapshot_date: str


class FetchClient:
    """URL-cache an explicitly allowlisted set of HTTP(S) resources.

    ``clock`` and ``sleeper`` use seconds, like ``time.monotonic`` and
    ``time.sleep``.  ``snapshot_clock`` returns a datetime and exists so tests
    can make snapshot names deterministic.  The default maximum is deliberately
    small; callers must choose a finite budget appropriate to each run.
    """

    def __init__(
        self,
        cache_dir: str | Path,
        *,
        allowed_hosts: Sequence[str],
        max_requests: int = 100,
        timeout: float = 30.0,
        backoff: float = DEFAULT_BACKOFF,
        clock: Callable[[], float] = time.monotonic,
        sleeper: Callable[[float], None] = time.sleep,
        snapshot_clock: Callable[[], datetime] = lambda: datetime.now(timezone.utc),
    ) -> None:
        if max_requests < 0:
            raise ValueError("max_requests must be non-negative")
        if timeout <= 0 or backoff < 0:
            raise ValueError("timeout must be positive and backoff non-negative")
        hosts = {_canonical_host(host) for host in allowed_hosts}
        if not hosts or any(_is_easyeda(host) for host in hosts):
            raise PolicyError("allowed_hosts must be non-empty and contain no EasyEDA host")
        self.cache_dir = Path(cache_dir)
        self.cache_dir.mkdir(parents=True, exist_ok=True)
        self.allowed_hosts = frozenset(hosts)
        self.max_requests = max_requests
        self.timeout = timeout
        self.backoff = backoff
        self.clock = clock
        self.sleeper = sleeper
        self.snapshot_clock = snapshot_clock
        self.request_count = 0
        self.request_log: list[RequestRecord] = []
        self._last_request: dict[str, float] = {}
        self._intervals: dict[str, float] = {}
        self._robots: dict[str, urllib.robotparser.RobotFileParser] = {}
        self._consecutive_refusals = 0
        self._stopped = False
        self._opener = urllib.request.build_opener(_NoProxyHandler(), _NoRedirectHandler())

    def get(self, url: str) -> Response:
        """Return a cached response or fetch ``url`` under all run policies.

        Redirects are followed manually, one hop at a time.  Every hop passes
        the host allowlist, the EasyEDA and ``/api`` refusals, robots.txt, the
        per-host interval and the request cap *before* its request is sent.
        """
        normalized = _normalize_url(url)
        hops = 0
        while True:
            parts = urllib.parse.urlsplit(normalized)
            host = _canonical_host(parts.hostname or "")
            self._check_host(host)
            self._check_not_api(parts.path)

            cached = self._read_cache(normalized)
            if cached is not None:
                return cached
            if self._stopped:
                raise RunStoppedError("run stopped after two consecutive 403/429 responses")

            robots = self._robots_for(parts, host)
            if not robots.can_fetch(USER_AGENT, normalized):
                raise RobotsError(f"robots.txt disallows {normalized}")
            body, final_url = self._fetch(normalized, host)
            if final_url is None:
                snapshot_date = _snapshot_date(self.snapshot_clock())
                self._write_cache(normalized, body, snapshot_date)
                self.request_log.append(RequestRecord(normalized, USER_AGENT, snapshot_date))
                return Response(normalized, body, snapshot_date, False)
            if hops >= MAX_REDIRECTS:
                raise FetchError(f"too many redirects for {url} (limit {MAX_REDIRECTS})")
            hops += 1
            normalized = _normalize_url(final_url)

    def _fetch(self, url: str, host: str) -> tuple[bytes, str | None]:
        """Send one request; return ``(body, redirect_target)``.

        The opener never follows redirects; a 3xx surfaces here as an
        ``HTTPError`` carrying Location.  A 3xx without Location is a failure.
        A redirect hop is not a success, so it does not reset the 403/429
        streak; only a final non-redirect response does.
        """
        self._wait_for_host(host)
        self._consume_budget()
        self._last_request[host] = self.clock()
        request = urllib.request.Request(
            url,
            headers={"User-Agent": USER_AGENT, "Accept-Encoding": "identity"},
            method="GET",
        )
        try:
            with self._opener.open(request, timeout=self.timeout) as response:
                body = response.read()
        except urllib.error.HTTPError as error:
            if error.code in (403, 429):
                self._refusal(url, error)
            if error.code in _REDIRECT_STATUSES:
                location = error.headers.get("Location") if error.headers else None
                if location:
                    return b"", urllib.parse.urljoin(url, location)
            raise FetchError(f"HTTP {error.code} for {url}") from error
        except (urllib.error.URLError, TimeoutError, socket.timeout) as error:
            self._consecutive_refusals = 0
            raise FetchError(f"request failed for {url}: {error}") from error
        self._consecutive_refusals = 0
        return body, None

    def _check_host(self, host: str) -> None:
        if _is_easyeda(host):
            raise PolicyError("EasyEDA hosts are always refused")
        if host not in self.allowed_hosts:
            raise PolicyError(f"host is not explicitly allowed: {host}")

    @staticmethod
    def _check_not_api(path: str) -> None:
        decoded = urllib.parse.unquote(path)
        if any(part.lower() == "api" for part in decoded.split("/")):
            raise RobotsError("/api paths are always refused")

    def _robots_for(
        self, parts: urllib.parse.SplitResult, host: str
    ) -> urllib.robotparser.RobotFileParser:
        origin = f"{parts.scheme}://{parts.netloc}"
        cached = self._robots.get(origin)
        if cached is not None:
            return cached
        robots_url = f"{origin}/robots.txt"
        robots_cache = self._robots_cache_path(robots_url)
        try:
            cached_response = robots_cache.read_bytes()
        except OSError:
            cached_response = None
        if cached_response is not None:
            parser = self._cached_robots_parser(robots_url, cached_response)
            self._robots[origin] = parser
            self._set_crawl_delay(host, parser)
            return parser
        if self._stopped:
            raise RunStoppedError("run stopped; robots.txt will not be fetched")
        self._wait_for_host(host)
        self._consume_budget()
        self._last_request[host] = self.clock()
        robots_url = f"{origin}/robots.txt"
        request = urllib.request.Request(
            robots_url,
            headers={"User-Agent": USER_AGENT, "Accept-Encoding": "identity"},
            method="GET",
        )
        try:
            with self._opener.open(request, timeout=self.timeout) as response:
                final_parts = urllib.parse.urlsplit(_normalize_url(response.geturl()))
                if _canonical_host(final_parts.hostname or "") != host:
                    raise RobotsError("robots.txt redirect changed host")
                text = response.read().decode("utf-8", errors="replace")
        except urllib.error.HTTPError as error:
            if error.code in (403, 429):
                self._refusal(robots_url, error)
            if error.code in (401, 403):
                raise RobotsError("robots.txt is access-controlled; refusing site") from error
            if 400 <= error.code < 500:
                text = ""
            else:
                self._consecutive_refusals = 0
                raise RobotsError("robots.txt unavailable; refusing site") from error
        except (urllib.error.URLError, TimeoutError, socket.timeout) as error:
            self._consecutive_refusals = 0
            raise RobotsError("robots.txt unavailable; refusing site") from error

        parser = urllib.robotparser.RobotFileParser(robots_url)
        parser.parse(text.splitlines())
        self._robots[origin] = parser
        self._consecutive_refusals = 0
        self.request_log.append(RequestRecord(robots_url, USER_AGENT, _snapshot_date(self.snapshot_clock())))
        crawl_delay = parser.crawl_delay(USER_AGENT)
        if crawl_delay is None:
            crawl_delay = parser.crawl_delay("*")
        self._intervals[host] = max(DEFAULT_MIN_INTERVAL, crawl_delay or 0)
        robots_cache = self._robots_cache_path(robots_url)
        robots_cache.parent.mkdir(parents=True, exist_ok=True)
        robots_cache.write_bytes(text.encode("utf-8"))
        return parser

    def _robots_cache_path(self, url: str) -> Path:
        return self.cache_dir / ".robots" / (hashlib.sha256(url.encode("utf-8")).hexdigest() + ".txt")

    def _cached_robots_parser(self, url: str, body: bytes) -> urllib.robotparser.RobotFileParser:
        parser = urllib.robotparser.RobotFileParser(url)
        parser.parse(body.decode("utf-8", errors="replace").splitlines())
        return parser

    def _set_crawl_delay(self, host: str, parser: urllib.robotparser.RobotFileParser) -> None:
        crawl_delay = parser.crawl_delay(USER_AGENT)
        if crawl_delay is None:
            crawl_delay = parser.crawl_delay("*")
        self._intervals[host] = max(DEFAULT_MIN_INTERVAL, crawl_delay or 0)

    def _wait_for_host(self, host: str) -> None:
        previous = self._last_request.get(host)
        if previous is not None:
            remaining = self._intervals.get(host, DEFAULT_MIN_INTERVAL) - (self.clock() - previous)
            if remaining > 0:
                self.sleeper(remaining)

    def _consume_budget(self) -> None:
        if self.request_count >= self.max_requests:
            raise RequestLimitError(f"request cap reached: {self.max_requests}")
        self.request_count += 1

    def _refusal(self, url: str, error: urllib.error.HTTPError) -> None:
        self._consecutive_refusals += 1
        if self._consecutive_refusals >= 2:
            self._stopped = True
            raise RunStoppedError("two consecutive 403/429 responses; run stopped") from error
        retry_after = error.headers.get("Retry-After") if error.headers else None
        delay = _retry_delay(retry_after, self.backoff)
        if delay > MAX_RETRY_AFTER:
            self._stopped = True
            raise RunStoppedError(
                f"Retry-After {delay:g} s exceeds the {MAX_RETRY_AFTER:g} s cap; run stopped"
            ) from error
        self.sleeper(delay)

    def is_cached(self, url: str) -> bool:
        """Return whether a normalized URL has a readable cached response."""
        return self._read_cache(_normalize_url(url)) is not None

    def _cache_paths(self, url: str) -> tuple[Path, Path]:
        digest = hashlib.sha256(url.encode("utf-8")).hexdigest()
        return self.cache_dir / f"{digest}.body", self.cache_dir / f"{digest}.json"

    def _read_cache(self, url: str) -> Response | None:
        body_path, metadata_path = self._cache_paths(url)
        try:
            metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
            if metadata["url"] != url:
                return None
            body = body_path.read_bytes()
            if metadata["sha256"] != hashlib.sha256(body).hexdigest():
                return None
            return Response(url, body, metadata["snapshot_date"], True)
        except (FileNotFoundError, KeyError, ValueError, TypeError, json.JSONDecodeError):
            return None

    def _write_cache(self, url: str, body: bytes, snapshot_date: str) -> None:
        body_path, metadata_path = self._cache_paths(url)
        metadata = {
            "url": url,
            "snapshot_date": snapshot_date,
            "sha256": hashlib.sha256(body).hexdigest(),
        }
        body_path.write_bytes(body)
        metadata_path.write_text(json.dumps(metadata, sort_keys=True), encoding="utf-8")


class _NoProxyHandler(urllib.request.ProxyHandler):
    """Ignore HTTP(S)_PROXY and platform proxy auto-configuration."""

    def __init__(self) -> None:
        super().__init__({})

    def getproxies(self) -> Mapping[str, str]:
        return {}


class _NoRedirectHandler(urllib.request.HTTPRedirectHandler):
    """Never follow a redirect automatically; let 3xx surface as HTTPError."""

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def _canonical_host(host: str) -> str:
    value = host.strip().lower().rstrip(".")
    if "://" in value:
        value = urllib.parse.urlsplit(value).hostname or ""
    return value


def _is_easyeda(host: str) -> bool:
    host = _canonical_host(host)
    return host == "easyeda.com" or host.endswith(".easyeda.com")


def _normalize_url(url: str) -> str:
    parts = urllib.parse.urlsplit(url)
    if parts.scheme not in ("http", "https") or not parts.hostname:
        raise PolicyError("only absolute HTTP(S) URLs are supported")
    if parts.username is not None or parts.password is not None:
        raise PolicyError("URLs with credentials are refused")
    try:
        parts.port
    except ValueError as error:
        raise PolicyError("invalid URL port") from error
    host = _canonical_host(parts.hostname)
    netloc = f"[{host}]" if ":" in host else host
    if parts.port is not None and not (
        (parts.scheme == "http" and parts.port == 80)
        or (parts.scheme == "https" and parts.port == 443)
    ):
        netloc += f":{parts.port}"
    path = parts.path or "/"
    return urllib.parse.urlunsplit((parts.scheme, netloc, path, parts.query, ""))


def _snapshot_date(value: datetime) -> str:
    if value.tzinfo is None:
        raise ValueError("snapshot datetime must include a timezone")
    return value.astimezone(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")


def _retry_delay(value: str | None, default: float) -> float:
    """Parse a Retry-After value; fall back to ``default`` when unparseable.

    Numeric seconds are used as-is; HTTP-date values are resolved against the
    wall clock.  A value that cannot be parsed, or is not finite, falls back to
    the default backoff instead of raising anything.
    """
    if value:
        try:
            seconds = float(value)
        except ValueError:
            try:
                parsed = email.utils.parsedate_to_datetime(value)
            except (TypeError, ValueError):
                return default
            if parsed.tzinfo is None:
                parsed = parsed.replace(tzinfo=timezone.utc)
            seconds = parsed.timestamp() - time.time()
        if not math.isfinite(seconds):
            return default
        return max(0.0, seconds)
    return default
