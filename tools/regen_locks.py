#!/usr/bin/env python3
"""Regenerate every tracked cohdl.lock outside tests/ from the tree's own lib/, offline.

Usage: python3 -I tools/regen_locks.py [WORKTREE] [--label LABEL]

Run it after any change to a package's content (sources, README, docs), because a lib package's
lock is part of its content hash and every example pins those hashes. Packages are installed
leaf-first from a fresh COHDL_HOME holding only this tree's lib/ (COHDL_REGISTRY points at a dead
port, so nothing is fetched), and the whole pass repeats until it is a fixed point. The
regenerated locks are left in the working tree; the report compares each with the lock that was
there before, and checks that every (name, version) pin has one hash across all locks.
Needs target/debug/cohdl in WORKTREE.
"""
import argparse, os, re, shutil, subprocess, sys, tempfile, json

ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
ap.add_argument("worktree", nargs="?", default=".")
ap.add_argument("--label", default="relock")
_a = ap.parse_args()
W, L = os.path.abspath(_a.worktree), _a.label
BIN = os.path.join(W, "target/debug/cohdl")


def manifest(d):
    t = open(os.path.join(d, "cohdl.toml")).read()
    name = re.search(r'^name = "(.*)"$', t, re.M).group(1)
    ver = re.search(r'^version = "(.*)"$', t, re.M)
    deps = {}
    m = re.search(r"^\[dependencies\]\n(.*?)(?=^\[|\Z)", t, re.M | re.S)
    if m:
        for line in m.group(1).splitlines():
            mm = re.match(r'\s*"?([^"=\s]+)"?\s*=\s*"([^"]+)"', line)
            if mm:
                deps[mm.group(1)] = mm.group(2)
    return name, ver.group(1) if ver else None, deps


def lib_packages():
    out = {}
    lib = os.path.join(W, "lib")
    for e in sorted(os.listdir(lib)):
        p = os.path.join(lib, e)
        if not os.path.isdir(p):
            continue
        subs = [p] if not e.startswith("@") else [os.path.join(p, s) for s in sorted(os.listdir(p))]
        for s in subs:
            if os.path.isfile(os.path.join(s, "cohdl.toml")):
                n, v, d = manifest(s)
                out[n] = (s, v, d)
    return out


def fresh_home(libs):
    h = tempfile.mkdtemp(prefix=f"cohdl-home-{L}-lock.")
    for n, (p, v, _) in libs.items():
        dst = os.path.join(h, "registry", n, v)
        shutil.copytree(p, dst)
    return h


def run(args, home):
    env = dict(os.environ, COHDL_HOME=home, COHDL_REGISTRY="http://127.0.0.1:9")
    r = subprocess.run([BIN] + args, cwd=W, env=env, capture_output=True, text=True)
    return r.returncode, (r.stdout + r.stderr)


def parse_lock(text):
    return {m.group(1): (m.group(2), m.group(3)) for m in re.finditer(
        r'\[\[package\]\]\nname = "(.*)"\nversion = "(.*)"\nhash = "(.*)"', text)}


libs = lib_packages()
tracked = subprocess.run(["git", "ls-files", "*cohdl.lock"], cwd=W, capture_output=True, text=True).stdout.split()
projects = [os.path.dirname(t) for t in tracked if not t.startswith("tests/")]
skipped = [t for t in tracked if t.startswith("tests/")]
for ex in sorted(os.listdir(os.path.join(W, "examples"))):
    if f"examples/{ex}" not in projects and os.path.isfile(os.path.join(W, "examples", ex, "cohdl.toml")):
        projects.append(f"examples/{ex}")

# topological order of lib packages
order, seen = [], set()
def visit(n):
    if n in seen or n not in libs:
        return
    seen.add(n)
    for d in libs[n][2]:
        visit(d)
    order.append(n)
for n in sorted(libs):
    visit(n)
libdirs = [os.path.relpath(libs[n][0], W) for n in order]
ordered = [d for d in libdirs if d in projects] + [p for p in projects if p not in libdirs]
nomanifest = [p for p in ordered if not os.path.isfile(os.path.join(W, p, "cohdl.toml"))]
ordered = [p for p in ordered if p not in nomanifest]
print(f"[{L}] lock without a manifest (snapshot, cannot be regenerated; left as is): {nomanifest}")

merged = {p: open(os.path.join(W, p, "cohdl.lock")).read() for p in ordered if os.path.isfile(os.path.join(W, p, "cohdl.lock"))}

# Phase 0: check with merged locks (E1103 etc.)
print(f"[{L}] == phase 0: cohdl check with the merged locks")
h = fresh_home(libs)
targets = [p for p in ordered if p.startswith("examples/")] + libdirs
bad = 0
for p in targets:
    rc, out = run(["check", p], h)
    errs = sorted(set(re.findall(r"E1[12]\d\d", out)))
    if rc != 0 or errs:
        bad += 1
        print(f"[{L}] check {p}: exit {rc} codes {errs}")
        for line in out.splitlines():
            if re.search(r"E1[12]\d\d|locked|error", line):
                print("      " + line[:400])
print(f"[{L}] phase 0: {len(targets)} checked ({len(libdirs)} lib packages + examples), {bad} with errors/E11xx")
shutil.rmtree(h)

# Phase 1..n: regenerate to a fixed point
for it in range(1, 6):
    before = {p: (open(os.path.join(W, p, "cohdl.lock")).read() if os.path.isfile(os.path.join(W, p, "cohdl.lock")) else None) for p in ordered}
    fails = []
    for p in ordered:
        lp = os.path.join(W, p, "cohdl.lock")
        if os.path.exists(lp):
            os.remove(lp)
        h = fresh_home(lib_packages())
        rc, out = run(["install", p], h)
        shutil.rmtree(h)
        if rc != 0 or not os.path.isfile(lp):
            fails.append(p)
            print(f"[{L}] install {p}: exit {rc} lock={'yes' if os.path.isfile(lp) else 'NO'} :: {out.strip()[-600:]}")
            if not os.path.isfile(lp) and before[p] is not None:
                open(lp, "w").write(before[p])  # restore
    after = {p: (open(os.path.join(W, p, "cohdl.lock")).read() if os.path.isfile(os.path.join(W, p, "cohdl.lock")) else None) for p in ordered}
    changed = [p for p in ordered if before[p] != after[p]]
    print(f"[{L}] pass {it}: {len(ordered)} projects installed, {len(fails)} failed, {len(changed)} locks changed in this pass")
    if it > 1 and not changed:
        break

# Report
print(f"[{L}] == per-lock result (regenerated vs merged)")
same = 0
for p in ordered:
    lp = os.path.join(W, p, "cohdl.lock")
    new = open(lp).read() if os.path.isfile(lp) else None
    old = merged.get(p)
    if new == old:
        same += 1
        if p.startswith("examples/") or not p.startswith("lib/"):
            print(f"[{L}] IDENTICAL {p}/cohdl.lock")
        continue
    print(f"[{L}] DIFFERS   {p}/cohdl.lock")
    a, b = parse_lock(old or ""), parse_lock(new or "")
    for n in sorted(set(a) | set(b)):
        if a.get(n) != b.get(n):
            print(f"      {n}: merged {a.get(n)} -> regenerated {b.get(n)}")
print(f"[{L}] {same}/{len(ordered)} locks byte-identical (identical lib locks not listed individually)")
if skipped:
    print(f"[{L}] not regenerated (under tests/): {skipped}")

# Consistency: one hash per (name, version) across all regenerated locks; versions match in-tree lib
by = {}
for p in ordered:
    lp = os.path.join(W, p, "cohdl.lock")
    if os.path.isfile(lp):
        for n, (v, hsh) in parse_lock(open(lp).read()).items():
            by.setdefault((n, v), {}).setdefault(hsh, []).append(p)
multi = {k: v for k, v in by.items() if len(v) > 1}
libs = lib_packages()
off = sorted({(n, v) for (n, v) in by if n in libs and libs[n][1] != v})
print(f"[{L}] regenerated locks: {len(by)} distinct (name,version) pins; with >1 hash: {len(multi)}")
for k, v in multi.items():
    print(f"      {k}: {json.dumps(v)}")
print(f"[{L}] pins naming a version other than the in-tree lib version: {off}")
for (n, v) in off:
    print(f"      {n} {v} (in-tree {libs[n][1]}) pinned by {[p for hs in by[(n, v)].values() for p in hs]}")
# merged-lock consistency
bym = {}
for p, t in merged.items():
    for n, (v, hsh) in parse_lock(t).items():
        bym.setdefault((n, v), {}).setdefault(hsh, []).append(p)
multim = {k: v for k, v in bym.items() if len(v) > 1}
print(f"[{L}] merged locks: {len(bym)} distinct pins; with >1 hash: {len(multim)}")
for k, v in multim.items():
    print(f"      {k}: {json.dumps(v)}")
print(f"[{L}] git status: {subprocess.run(['git','status','--short'],cwd=W,capture_output=True,text=True).stdout.strip()}")
