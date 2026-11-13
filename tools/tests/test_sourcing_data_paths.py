"""tools/sourcing_data is committed evidence: no file in it may carry a path from someone's machine.

A record that names `/Users/<name>/...`, `/home/<name>/...`, a temp directory (`/private/...`, macOS
`$TMPDIR` under `/var/folders/...`, an agent's Linux scratch directory `/tmp/claude-<uid>/...`) or a
Windows home seen from WSL (`/mnt/c/Users/...`) tells a reader nothing they can open, and it leaks
the layout of the machine that wrote it. Records name repository-relative paths instead.
"""
import io
import re
import tempfile
import unittest
import zipfile
from pathlib import Path

DATA = Path(__file__).resolve().parents[1] / "sourcing_data"
# A path root at the start of a path. A URL such as https://example.com/home/x is not one: its
# `/home/` follows a host or path character. `/mnt/<drive>/` is its own root because the drive
# letter before `/Users/` would otherwise read as such a character; `/tmp/` alone is not listed (a
# record may tell its reader to work in a scratch directory), only the scratch form that names a
# user id and that user's directories.
LOCAL_PATH = re.compile(rb"(?<![\w.~-])/(?:(?:Users|home|private|var/folders|mnt/[A-Za-z])/|tmp/claude-)")


def local_paths(data):
    """(offset, excerpt) for each local path in `data`."""
    return [(m.start(), data[m.start():m.start() + 60].split(b"\n")[0].decode("utf-8", "replace"))
            for m in LOCAL_PATH.finditer(data)]


def scan(root):
    """Every file under `root`, read as bytes, so no encoding or file type is skipped: text records,
    image metadata, and a zip's stored names. A zip's members are also inflated and scanned."""
    found, files = [], 0
    for path in sorted(p for p in root.rglob("*") if p.is_file()):
        files += 1
        data = path.read_bytes()
        rel = path.relative_to(root).as_posix()
        found += [f"{rel}@{offset}: {text}" for offset, text in local_paths(data)]
        if zipfile.is_zipfile(io.BytesIO(data)):
            with zipfile.ZipFile(io.BytesIO(data)) as archive:
                for info in archive.infolist():
                    member = info.filename.encode() + b"\n" + archive.read(info)
                    found += [f"{rel}!{info.filename}@{offset}: {text}" for offset, text in local_paths(member)]
    return found, files


class SourcingDataPathTests(unittest.TestCase):
    def test_no_file_under_sourcing_data_contains_a_local_path(self):
        found, files = scan(DATA)
        self.assertGreater(files, 100, "tools/sourcing_data was not scanned")
        self.assertEqual(found, [], "local paths in tools/sourcing_data; use repository-relative paths")

    def test_the_pattern_finds_paths_and_leaves_urls(self):
        for text in ('"cwd": "/Users/someone/repo"', "out=/home/ci/build", "see /private/tmp/x/audit.cohdl",
                     "file:///Users/someone/a.pdf", "/home/a at the start",
                     # Linux agent scratch, macOS $TMPDIR (bare and under /private), WSL: one hit each.
                     '"run": "/tmp/claude-1000/-home-someone-repo/scratch/run"', "out=/var/folders/zz/abc123/T/x.bmp",
                     "/private/var/folders/zz/abc123/T/x.bmp", "cd /mnt/c/Users/someone/repo",
                     "/mnt/d/work/repo/board.json"):
            with self.subTest(text=text):
                self.assertEqual(len(local_paths(text.encode())), 1)
        for text in ("https://example.com/home/page", "https://example.com/Users/list", "docs/private/notes.md",
                     "tools/home/x", "-Users-someone-repo", "~/home/x",
                     "https://example.com/tmp/claude-1/x", "https://example.com/var/folders/a",
                     "https://example.com/mnt/c/Users/list", "https://example.com/a/tmp/b", "T=/tmp/scratch",
                     "docs/var/folders/x", "/var/log/syslog", "/mnt/data/x"):
            with self.subTest(text=text):
                self.assertEqual(local_paths(text.encode()), [])

    def test_a_path_inside_a_zip_member_is_found(self):
        with tempfile.TemporaryDirectory() as td:
            with zipfile.ZipFile(Path(td) / "a.zip", "w", zipfile.ZIP_DEFLATED) as archive:
                archive.writestr("job.gbr", "G04 written to /Users/someone/board/job.gbr*\n" * 50)
            (Path(td) / "ok.json").write_text('{"record": "tools/sourcing_data/x/README.md"}')
            found, files = scan(Path(td))
        self.assertEqual(files, 2)
        self.assertEqual(len(found), 50)
        self.assertTrue(all(f.startswith("a.zip!job.gbr@") for f in found))


if __name__ == "__main__":
    unittest.main()
