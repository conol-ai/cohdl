#!/usr/bin/env python3
"""The MVP generate → check → repair harness (a script, not a product UI).

Loop (docs/design/09-mvp-definition.md, "Demo scenario"):
  1. Prompt an LLM with the language reference for .cohdl source.
  2. Run `cohdl build` (type checker + residual DRC + emitters).
  3. On failure, feed the compiler diagnostics back VERBATIM and regenerate.
  4. Repeat until clean or the attempt cap (default 5) is hit.

Writes a full markdown transcript of every attempt — the transcript showing at
least one genuine type-checker catch + repair is the MVP's proof artifact.

Usage:
  python3 harness/repair_loop.py                # runs the canonical demo spec
  python3 harness/repair_loop.py --spec "..."   # custom natural-language spec
  python3 harness/repair_loop.py --max-attempts 5 --model claude-opus-4-8

Backends (--backend, default auto):
  api         the Anthropic SDK (`pip install anthropic` + ANTHROPIC_API_KEY
              or an `ant auth login` profile)
  claude-cli  `claude -p` print mode (uses your existing Claude Code login)
"""

from __future__ import annotations

import argparse
import datetime
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
DEFAULT_SPEC = (
    "An ESP32-S3-based sensor node: USB-C power/data, one MEMS microphone, "
    "one status LED, a 3.3V LDO regulator, standard decoupling."
)

SYSTEM_PROMPT_TEMPLATE = """\
You are an expert electronics engineer writing CoHDL, a typed hardware
description language for PCB schematics. You will be given a natural-language
board specification. Respond with a complete .cohdl source file implementing
it.

Rules:
- Output exactly one fenced code block marked ```cohdl containing the full
  source file, and nothing else of substance.
- The file must contain exactly one `design` declaration.
- Use ONLY the constructs and pinned library items in the reference below.
- Every instance must be a concrete part (instantiated by its qualified name)
  so the BOM resolves.
- If you receive compiler diagnostics, fix exactly what they report and
  return the corrected complete file.

{reference}
"""


def build_compiler() -> pathlib.Path:
    subprocess.run(
        ["cargo", "build", "--quiet"], cwd=REPO_ROOT, check=True
    )
    return REPO_ROOT / "target" / "debug" / "cohdl"


def extract_cohdl(text: str) -> str | None:
    blocks = re.findall(r"```cohdl\n(.*?)```", text, re.DOTALL)
    if blocks:
        return blocks[-1]
    # Fall back to any fenced block.
    blocks = re.findall(r"```\w*\n(.*?)```", text, re.DOTALL)
    return blocks[-1] if blocks else None


SCHEMA_VERSION = 1  # RFC-010: check before parsing further.


def run_build(compiler: pathlib.Path, project_dir: pathlib.Path) -> dict:
    """Run `cohdl build --json` and return the parsed RFC-010 document.

    Retires text-scraping entirely (RFC-010): the harness now consumes the
    structured diagnostics contract, not the human-readable render. A verdict
    of "pass" means the design built; `diagnostics` is the flat list to feed
    back; `build` (on pass) names the emitted artifacts.
    """
    proc = subprocess.run(
        [str(compiler), "build", str(project_dir),  "--json"],
        capture_output=True,
        text=True,
    )
    try:
        doc = json.loads(proc.stdout)
    except json.JSONDecodeError:
        # Exit code 2: a pre-pipeline invocation failure (E000), which is never
        # part of the diagnostics array — surface stderr as a synthetic verdict.
        return {
            "schema_version": SCHEMA_VERSION,
            "verdict": "fail",
            "diagnostics": [],
            "invocation_error": proc.stderr.strip() or "cohdl invocation failed",
        }
    got = doc.get("schema_version")
    if got != SCHEMA_VERSION:
        raise SystemExit(
            f"cohdl --json schema_version {got!r} != expected {SCHEMA_VERSION} — "
            "update the harness before parsing"
        )
    return doc


def format_in_place(compiler: pathlib.Path, path: pathlib.Path, fallback: str) -> str:
    """Run `cohdl fmt` on `path` (RFC-009). Returns the canonical text on
    success, or `fallback` unchanged if the source does not parse (fmt refuses
    to touch non-parsing source)."""
    subprocess.run(
        [str(compiler), "fmt", str(path)],
        capture_output=True,
        text=True,
    )
    try:
        return path.read_text()
    except OSError:
        return fallback


def format_diagnostics(doc: dict) -> str:
    """Render RFC-010 diagnostics as readable text — from the structured data,
    never by regex-scraping the compiler's own text output."""
    if err := doc.get("invocation_error"):
        return f"cohdl could not run: {err}"
    lines = []
    for d in doc["diagnostics"]:
        p = d["primary"]
        loc = f'{p["file"]}:{p["start_line"]}:{p["start_col"]}'
        head = f'{d["severity"]}[{d["code"]}] at {loc}: {d["message"]}'
        lines.append(head)
        if p.get("message"):
            lines.append(f'    {loc}: {p["message"]}')
        for s in d.get("secondary", []):
            sloc = f'{s["file"]}:{s["start_line"]}:{s["start_col"]}'
            lines.append(f'    note: {sloc}: {s["message"]}')
        for h in d.get("help", []):
            lines.append(f"    help: {h}")
    return "\n".join(lines) if lines else "(no diagnostics)"


class ApiBackend:
    """The Anthropic Messages API via the official SDK."""

    def __init__(self, model: str, system_prompt: str):
        import anthropic

        self.client = anthropic.Anthropic()
        self.model = model
        self.system_prompt = system_prompt

    def generate(self, messages: list[dict]) -> str | None:
        response = self.client.messages.create(
            model=self.model,
            max_tokens=16000,
            thinking={"type": "adaptive"},
            system=[{
                "type": "text",
                "text": self.system_prompt,
                "cache_control": {"type": "ephemeral"},
            }],
            messages=messages,
        )
        if response.stop_reason == "refusal":
            return None
        return "".join(b.text for b in response.content if b.type == "text")


class ClaudeCliBackend:
    """`claude -p` print mode — reuses the local Claude Code login.

    Print mode is stateless, so each call sends the full conversation
    rendered into one prompt.
    """

    def __init__(self, model: str, system_prompt: str):
        self.model = model
        self.system_prompt = system_prompt

    def generate(self, messages: list[dict]) -> str | None:
        parts = []
        for m in messages:
            speaker = "USER" if m["role"] == "user" else "YOUR PREVIOUS REPLY"
            parts.append(f"[{speaker}]\n{m['content']}")
        prompt = "\n\n".join(parts)
        proc = subprocess.run(
            [
                "claude", "-p",
                "--model", self.model,
                "--system-prompt", self.system_prompt,
                "--tools", "",
            ],
            input=prompt,
            capture_output=True,
            text=True,
            timeout=1200,
        )
        if proc.returncode != 0:
            print(f"claude -p failed: {proc.stderr.strip()}", file=sys.stderr)
            return None
        return proc.stdout


def pick_backend(name: str, model: str, system_prompt: str):
    if name == "api":
        return ApiBackend(model, system_prompt)
    if name == "claude-cli":
        return ClaudeCliBackend(model, system_prompt)
    # auto: prefer the SDK when it's importable and credentialed.
    try:
        import anthropic  # noqa: F401

        if os.environ.get("ANTHROPIC_API_KEY") or os.environ.get("ANTHROPIC_AUTH_TOKEN"):
            return ApiBackend(model, system_prompt)
    except ImportError:
        pass
    if shutil.which("claude"):
        return ClaudeCliBackend(model, system_prompt)
    print(
        "error: no LLM backend available — pip install anthropic + set "
        "ANTHROPIC_API_KEY, or install Claude Code",
        file=sys.stderr,
    )
    sys.exit(2)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--spec", default=DEFAULT_SPEC, help="natural-language board spec")
    parser.add_argument("--max-attempts", type=int, default=5)
    parser.add_argument("--model", default="claude-opus-4-8")
    parser.add_argument("--backend", choices=["auto", "api", "claude-cli"], default="auto")
    parser.add_argument(
        "--lean-reference",
        action="store_true",
        help="prompt with the language reference only — strip the electrical "
        "design-notes section (the MVP-faithful mode: note 10 + snippets, no hand-holding)",
    )
    parser.add_argument(
        "--run-dir",
        default=None,
        help="output directory (default harness/runs/<timestamp>)",
    )
    args = parser.parse_args()

    reference = (REPO_ROOT / "harness" / "prompt" / "language-reference.md").read_text()
    if args.lean_reference:
        reference = reference.split("## Electrical design notes")[0].rstrip() + "\n"
    system_prompt = SYSTEM_PROMPT_TEMPLATE.format(reference=reference)
    backend = pick_backend(args.backend, args.model, system_prompt)

    stamp = datetime.datetime.now().strftime("%Y%m%d-%H%M%S")
    run_dir = pathlib.Path(args.run_dir) if args.run_dir else REPO_ROOT / "harness" / "runs" / stamp
    run_dir.mkdir(parents=True, exist_ok=True)

    compiler = build_compiler()

    transcript: list[str] = [
        "# CoHDL generate → check → repair transcript",
        "",
        f"- Date: {datetime.datetime.now().isoformat(timespec='seconds')}",
        f"- Model: {args.model}",
        f"- Attempt cap: {args.max_attempts}",
        "",
        "## Natural-language specification",
        "",
        f"> {args.spec}",
        "",
    ]

    messages = [
        {
            "role": "user",
            "content": (
                "Write the complete .cohdl source for this board:\n\n"
                f"{args.spec}"
            ),
        }
    ]

    success = False
    caught_errors = 0
    for attempt in range(1, args.max_attempts + 1):
        print(f"--- attempt {attempt}/{args.max_attempts}: generating…", flush=True)
        reply_text = backend.generate(messages)
        if reply_text is None:
            print("generation failed; aborting", file=sys.stderr)
            transcript.append(f"## Attempt {attempt}\n\nGeneration failed (refusal or backend error).\n")
            break
        source = extract_cohdl(reply_text)

        transcript.append(f"## Attempt {attempt}")
        transcript.append("")
        if source is None:
            transcript.append("The model returned no code block. Full reply:")
            transcript.append("")
            transcript.append(reply_text)
            transcript.append("")
            messages.append({"role": "assistant", "content": reply_text})
            messages.append({
                "role": "user",
                "content": "You returned no ```cohdl code block. Return the complete source file in one.",
            })
            continue

        attempt_dir = run_dir / f"attempt_{attempt}"
        (attempt_dir / "src").mkdir(parents=True, exist_ok=True)
        main_file = attempt_dir / "src" / "main.cohdl"
        main_file.write_text(source)
        (attempt_dir / "cohdl.toml").write_text(
            '[package]\n'
            'name = "sensor-node"\n'
            'version = "0.1.0"\n'
            '\n'
            '[dependencies]\n'
            '"@espressif/esp32" = "0.2.0"\n'
            'esd = "0.1.0"\n'
            'ldo = "0.1.0"\n'
            'led = "0.1.0"\n'
            'mic = "0.1.0"\n'
            'passive = "0.2.0"\n'
            'std = "0.3.0"\n'
            'usb = "0.1.0"\n'
        )

        # RFC-009: normalize generated source through `cohdl fmt` before we
        # display/diff it, so a diagnostic-driven repair shows as a small diff
        # rather than a wall of incidental whitespace. fmt is a no-op on source
        # that does not parse (it is a serializer, not a repair tool).
        source = format_in_place(compiler, main_file, fallback=source)

        doc = run_build(compiler, attempt_dir)
        ok = doc["verdict"] == "pass"
        compiler_output = format_diagnostics(doc)
        transcript.append("### Generated source")
        transcript.append("")
        transcript.append("```cohdl")
        transcript.append(source.rstrip("\n"))
        transcript.append("```")
        transcript.append("")
        transcript.append("### Compiler verdict (RFC-010 `--json`)")
        transcript.append("")
        transcript.append("```text")
        transcript.append(compiler_output if compiler_output else "(no output)")
        transcript.append("```")
        transcript.append("")

        if ok:
            print(f"attempt {attempt}: CLEAN — netlist + BOM emitted")
            build = doc.get("build", {})
            netlist = build.get("netlist", str(attempt_dir / "out" / "sensor-node.net"))
            bom = build.get("bom", str(attempt_dir / "out" / "sensor-node-bom.csv"))
            transcript.append(
                f"**Attempt {attempt} is clean** — the design parses, resolves, "
                "type-checks, passes residual DRC, and emitted a KiCad netlist + BOM."
            )
            transcript.append("")
            transcript.append(f"- Netlist: `{netlist}`")
            transcript.append(f"- BOM: `{bom}`")
            transcript.append("")
            success = True
            break

        # Count type-checker/DRC catches for the proof requirement — straight
        # from the structured diagnostics list, no text-scraping.
        n_errors = len(doc["diagnostics"]) or (1 if doc.get("invocation_error") else 0)
        caught_errors += n_errors
        print(f"attempt {attempt}: {n_errors} diagnostics — feeding back verbatim")

        messages.append({"role": "assistant", "content": reply_text})
        messages.append({
            "role": "user",
            "content": (
                "The CoHDL compiler rejected this source. Diagnostics, verbatim:\n\n"
                f"```\n{compiler_output}\n```\n\n"
                "Fix exactly what is reported and return the corrected complete file."
            ),
        })

    transcript.append("## Result")
    transcript.append("")
    if success:
        transcript.append(
            f"Landed on a clean design. The compiler caught and reported "
            f"{caught_errors} diagnostics across the failed attempts; every one "
            "was fed back verbatim and repaired by the model."
        )
    else:
        transcript.append(
            f"Did NOT land within {args.max_attempts} attempts "
            f"({caught_errors} diagnostics caught)."
        )
    transcript.append("")

    transcript_path = run_dir / "transcript.md"
    transcript_path.write_text("\n".join(transcript))
    print(f"transcript: {transcript_path}")
    return 0 if success else 1


if __name__ == "__main__":
    sys.exit(main())
