#!/usr/bin/env bash
# Run the polyglot tracks' mounted peers for real, and check the pages against what happens
# (ledger #2612): books/ikigai/src/polyglot/python.md and typescript.md, and the Python peer
# of books/ikigai/src/explore/beyond-rules.md (ledger #2693).
#
# Their binding and family steps cannot be REPL transcripts: each starts a Python or Deno peer
# on a socket and mounts it with `ikigai --mount`. Until this ran, nothing replayed them, and
# the TypeScript track's rotted unseen: its peer no longer started, its pipe changed nothing,
# and its verdicts were an older cli's. So this does what a reader does, from the pages' text:
#
#   * clones ikigai-python and ikigai-deno at `main` into a scratch directory (never your own
#     checkouts) and copies this repository's `examples/` beside them, the layout the pages
#     assume (the listings import `../../../ikigai-deno` by path). A copy, not a symlink: Deno
#     resolves a module from its real path, so through a symlink the import finds nothing;
#   * installs that ikigai-python clone, with its one dependency (`blake3`), into a scratch
#     virtualenv whose `python3` comes first on PATH, as the Python page's `pip install` does;
#   * walks each page in order: a `<!-- demo: start -->` block's lines are run in the copy, a
#     line ending in `&` in the background (its output kept in a log), every other line to
#     completion, so the page's own wait for its socket is what the next block relies on; a
#     `<!-- demo: run -->` block's `$ ` commands are run and what they print is compared with
#     the lines under them, verdict lines included: on these pages the verdict is the claim;
#   * stops everything it started.
#
# Durations (`0ms`) are normalized on both sides; a line that is exactly `…` means "any lines".
# `ikigai` is whatever is first on PATH: `scripts/test-transcripts.sh` runs this with the
# book's pinned cli (`books/ikigai/ikigai-cli.version`) there.
#
# It needs python3 (with venv), deno, ikigai, git and the network (two clones, PyPI, JSR).
# Under CI (`CI` set) a missing tool fails the run; by hand it is a SKIP, said as one, and
# exits 0. The pages' sockets are fixed `/tmp` paths; one that answers already is a peer
# you are running, and the run refuses rather than replace it.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

missing=()
for tool in python3 deno ikigai git; do
    command -v "$tool" > /dev/null 2>&1 || missing+=("$tool")
done
if [ "${#missing[@]}" -gt 0 ]; then
    if [ -n "${CI:-}" ]; then
        echo "polyglot-tracks: FAILED — not installed: ${missing[*]}" >&2
        exit 1
    fi
    echo "polyglot-tracks: SKIPPED — not installed: ${missing[*]}"
    echo "  (the polyglot tracks' peers need all of: python3 deno ikigai git)"
    exit 0
fi

exec python3 - "$PWD" <<'PY'
import os, re, signal, socket, subprocess, sys, time, difflib
from pathlib import Path

REPO = Path(sys.argv[1])
PAGES = [REPO / "books/ikigai/src/polyglot/python.md",
         REPO / "books/ikigai/src/polyglot/typescript.md",
         REPO / "books/ikigai/src/explore/beyond-rules.md"]
DEMO = Path("/tmp/ikbook-polyglot")
WORK = DEMO / "work"
TUTORIAL = WORK / "ikigai-tutorial"
SIBLINGS = {
    "ikigai-python": os.environ.get("IKIGAI_PYTHON_URL", "https://github.com/ikigai-rs/ikigai-python"),
    "ikigai-deno": os.environ.get("IKIGAI_DENO_URL", "https://github.com/ikigai-rs/ikigai-deno"),
}
SOCKET = re.compile(r"/tmp/[\w.-]+\.sock")

def answering(path):
    with socket.socket(socket.AF_UNIX) as s:
        s.settimeout(0.5)
        try:
            s.connect(path)
            return True
        except OSError:
            return False

texts = {page: page.read_text() for page in PAGES}
busy = sorted({s for text in texts.values() for s in SOCKET.findall(text) if answering(s)})
if busy:
    sys.exit(f"polyglot-tracks: {', '.join(busy)} already answers; stop that peer and run again")

subprocess.run(["rm", "-rf", str(DEMO)], check=True)
WORK.mkdir(parents=True)
(DEMO / "logs").mkdir()
for name, url in SIBLINGS.items():
    subprocess.run(["git", "clone", "--quiet", "--depth", "1", url, str(WORK / name)], check=True)
    head = subprocess.run(["git", "-C", str(WORK / name), "rev-parse", "--short", "HEAD"],
                          capture_output=True, text=True, check=True).stdout.strip()
    print(f"· {name} main is {head}")

# The working tree's examples, tracked or new (not ignored), so a change not yet committed is
# the one checked.
listed = subprocess.run(["git", "-C", str(REPO), "ls-files", "-z", "--cached", "--others",
                         "--exclude-standard", "examples"],
                        capture_output=True, check=True).stdout.split(b"\0")
for rel in filter(None, (p.decode() for p in listed)):
    src, dst = REPO / rel, TUTORIAL / rel
    if src.is_file():
        dst.parent.mkdir(parents=True, exist_ok=True)
        dst.write_bytes(src.read_bytes())

venv = DEMO / "venv"
subprocess.run([sys.executable, "-m", "venv", str(venv)], check=True)
subprocess.run([str(venv / "bin/pip"), "install", "--quiet", str(WORK / "ikigai-python")], check=True)
env = dict(os.environ, PATH=f"{venv / 'bin'}:{os.environ['PATH']}")
for tool in ("python3 --version", "deno --version", "ikigai --version"):
    out = subprocess.run(["bash", "-c", tool], env=env, capture_output=True, text=True).stdout
    print(f"· {out.splitlines()[0] if out else tool + ': ?'}")

DIRECTIVE = re.compile(r"<!-- demo: ([a-z -]+?) -->\n```\w*\n(.*?)\n```", re.S)
failures = []
started = []

def normalize(out):
    return re.sub(r"\b\d+ms\b", "Nms", out)

def matches(got, want):
    """`want`'s lines, with `…` meaning any lines, against `got`'s."""
    got = [l.rstrip() for l in got.rstrip("\n").split("\n")]
    segments, current = [], []
    for line in want:
        if line == "…":
            segments.append(current)
            current = []
        else:
            current.append(line.rstrip())
    segments.append(current)
    if len(segments) == 1:
        return got == segments[0]
    at = 0
    for i, segment in enumerate(segments):
        if not segment:
            continue
        if i == 0:
            if got[: len(segment)] != segment:
                return False
            at = len(segment)
            continue
        found = next((j for j in range(at, len(got) - len(segment) + 1)
                      if got[j : j + len(segment)] == segment), None)
        if found is None or (i == len(segments) - 1 and found + len(segment) != len(got)):
            return False
        at = found + len(segment)
    return True

# A page that declares `<!-- transcripts: workspace … -->` (as "When a rule is not enough"
# does: its reader works in `~/quakes`, not in a clone) runs in a directory of its own, seeded
# with those repository files under their own names, as `book-transcripts` seeds it; every
# other page runs in the copy of the repository.
WORKSPACE = re.compile(r"<!-- transcripts: workspace ([^>]*?) -->")

def page_dir(page):
    paths = [p for decl in WORKSPACE.findall(texts[page]) for p in decl.split()]
    if not paths:
        return TUTORIAL
    where = DEMO / "pages" / page.stem
    where.mkdir(parents=True)
    for rel in paths:
        (where / Path(rel).name).write_bytes((REPO / rel).read_bytes())
    return where

cwd = TUTORIAL

def shell(command, **kw):
    return subprocess.run(["bash", "-c", command], cwd=cwd, env=env, **kw)

try:
    for page in PAGES:
        print(f"── {page.relative_to(REPO)}")
        cwd = page_dir(page)
        for kind, block in DIRECTIVE.findall(texts[page]):
            if kind == "start":
                for line in block.splitlines():
                    line = line.split("  #")[0].strip()
                    if not line:
                        continue
                    if line.endswith("&"):
                        log = open(DEMO / "logs" / f"{len(started)}.log", "w")
                        started.append(subprocess.Popen(
                            ["bash", "-c", line[:-1]], cwd=cwd, env=env,
                            stdout=log, stderr=subprocess.STDOUT, start_new_session=True))
                    else:
                        # The page's own wait, given two minutes; a peer that exits meanwhile
                        # will never answer, so that ends it at once, with the peer's log.
                        step = subprocess.Popen(["bash", "-c", line], cwd=cwd, env=env,
                                                start_new_session=True)
                        deadline = time.time() + 120
                        while step.poll() is None and time.time() < deadline:
                            dead = [p for p in started if p.poll() is not None]
                            if dead:
                                break
                            time.sleep(0.2)
                        if step.poll() != 0:
                            if step.poll() is None:
                                os.killpg(step.pid, signal.SIGTERM)
                            failures.append(line)
                            print(f"✗ did not finish: {line}")
                            for n, process in enumerate(started):
                                if process.poll() is not None:
                                    print(f"  a peer exited ({process.returncode}); its log:")
                                    print((DEMO / "logs" / f"{n}.log").read_text().rstrip())
                            raise SystemExit(1)
                print(f"· started: {sum(1 for l in block.splitlines() if l.strip().endswith('&'))} process(es)")
                continue
            if kind != "run":
                sys.exit(f"polyglot-tracks: {page.name}: unknown directive `demo: {kind}`")
            commands, want, lines = [], [], block.splitlines()
            i = 0
            while i < len(lines):
                line = lines[i]
                if line.startswith("$ "):
                    command = line[2:]
                    while command.endswith("\\"):
                        i += 1
                        command = command[:-1] + " " + lines[i].strip()
                    commands.append(command)
                else:
                    want.append(line)
                i += 1
            got = ""
            for command in commands:
                # One pipe for both streams, so the verdicts (stderr) land where the reader
                # sees them, between the answers (stdout).
                done = shell(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                             text=True, timeout=120)
                out = done.stdout
                got += out if out.endswith("\n") or not out else out + "\n"
            got = normalize(got)
            want = [normalize(l) for l in want]
            first = commands[0] if commands else "?"
            label = f"{first[:70]}{'…' if len(first) > 70 else ''}"
            if matches(got, want):
                print(f"✓ {label}")
            else:
                failures.append(label)
                print(f"✗ {label}")
                sys.stdout.writelines(difflib.unified_diff(
                    [l + "\n" for l in want], got.splitlines(True), "the page", "a real run"))
finally:
    for process in started:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass

if failures:
    print(f"\npolyglot-tracks: {len(failures)} difference(s) between the pages and a real run")
    sys.exit(1)
print("\npolyglot-tracks: the pages match a real run")
PY
