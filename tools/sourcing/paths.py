"""Paths as committed provenance records them."""

from __future__ import annotations

from pathlib import Path


def repo_relative(path: Path, root: Path) -> str:
    """Return ``path`` relative to the repository, as POSIX text.

    Provenance is committed and diffed between checkouts, so it must never
    carry an absolute path (and with it a maintainer's account name).  A file
    outside the repository is recorded by name only; its hash identifies it.
    """
    try:
        return Path(path).resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return "<outside repository>/" + Path(path).name
